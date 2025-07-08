use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tracing::info;

mod discovery;
mod reader;
mod sentence_detector;
mod incremental;

use crate::incremental::{generate_aux_file_path, generate_cache_path, cache_exists, read_cache_async, aux_file_exists, read_aux_file, create_complete_aux_file, read_cache};

/// Cache for tracking completed auxiliary files
/// WHY: Provides robust incremental processing by tracking completion timestamps
#[derive(Serialize, Deserialize, Debug, Default)]
struct ProcessingCache {
    /// Map from source file path to completion timestamp (seconds since epoch)
    completed_files: HashMap<String, u64>,
}

impl ProcessingCache {
    /// Load cache from file, returns empty cache if file doesn't exist or is corrupted
    /// WHY: Fail-safe approach - missing/corrupted cache just means reprocessing everything
    async fn load(root_dir: &Path) -> Self {
        if !cache_exists(root_dir) {
            return Self::default();
        }
        
        match read_cache_async(root_dir).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(cache) => cache,
                    Err(e) => {
                        info!("Cache file corrupted, starting fresh: {}", e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                info!("Could not read cache file, starting fresh: {}", e);
                Self::default()
            }
        }
    }
    
    /// Save cache to file
    /// WHY: Persist completion state for future runs
    async fn save(&self, cache_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(cache_path, content).await?;
        Ok(())
    }
    
    /// Check if source file has been processed and aux file is still valid
    /// WHY: Core incremental logic - compare source modification time vs completion time
    async fn is_file_processed(&self, source_path: &Path) -> Result<bool> {
        let source_path_str = source_path.to_string_lossy().to_string();
        
        if let Some(&completion_timestamp) = self.completed_files.get(&source_path_str) {
            // Check if source file has been modified since completion
            let source_metadata = tokio::fs::metadata(source_path).await?;
            let source_modified = source_metadata.modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            
            // Also verify aux file still exists
            if !aux_file_exists(source_path) {
                info!("Aux file missing for {}, reprocessing", source_path.display());
                return Ok(false);
            }
            
            Ok(source_modified <= completion_timestamp)
        } else {
            Ok(false)
        }
    }
    
    /// Mark file as completed with current timestamp
    /// WHY: Record successful completion for future incremental runs
    fn mark_completed(&mut self, source_path: &Path) {
        let source_path_str = source_path.to_string_lossy().to_string();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.completed_files.insert(source_path_str, now);
    }
}


/// Determine if file should be processed based on cache and incremental rules
/// WHY: Implements core F-9 logic using robust timestamp-based cache
async fn should_process_file(source_path: &Path, cache: &ProcessingCache, overwrite_all: bool) -> Result<bool> {
    if overwrite_all {
        return Ok(true);
    }
    
    let is_processed = cache.is_file_processed(source_path).await?;
    
    if is_processed {
        info!("Skipping {} - already processed and up to date", source_path.display());
        return Ok(false);
    }
    
    let aux_path = generate_aux_file_path(source_path);
    if aux_path.exists() {
        info!("Processing {} - source newer than cache or aux file missing from cache", source_path.display());
    }
    
    Ok(true)
}

/// Write sentences to auxiliary file in PRD F-7 format
/// WHY: Implements core requirement for auxiliary file generation
async fn write_auxiliary_file(
    aux_path: &Path,
    sentences: &[sentence_detector::DetectedSentence],
    _detector: &sentence_detector::dialog_detector::SentenceDetectorDialog,
) -> Result<()> {
    let file = File::create(aux_path).await?;
    let mut writer = BufWriter::new(file);
    
    for sentence in sentences {
        // Format manually since dialog detector uses different API
        let formatted_line = format!("{}\t{}\t({},{},{},{})", 
            sentence.index, 
            sentence.normalized_content,
            sentence.span.start_line,
            sentence.span.start_col,
            sentence.span.end_line,
            sentence.span.end_col
        );
        writer.write_all(formatted_line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    
    writer.flush().await?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "seams")]
#[command(about = "High-throughput sentence extractor for Project Gutenberg texts")]
#[command(version)]
struct Args {
    /// Root directory to scan for *-0.txt files
    root_dir: PathBuf,
    
    /// Overwrite even complete aux files
    #[arg(long)]
    overwrite_all: bool,
    
    /// Abort on first error
    #[arg(long)]
    fail_fast: bool,
    
    /// Use memory-mapped I/O instead of async buffered
    #[arg(long)]
    use_mmap: bool,
    
    /// Suppress console progress bars
    #[arg(long)]
    no_progress: bool,
    
    /// Stats output file path
    #[arg(long, default_value = "run_stats.json")]
    stats_out: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // WHY: structured JSON logging enables observability and debugging in production
    tracing_subscriber::fmt()
        .with_target(false)
        .json()
        .init();
    
    let args = Args::parse();
    
    info!("Starting seams");
    info!(?args, "Parsed CLI arguments");
    
    // WHY: validate root directory exists early to fail fast with clear error
    if !args.root_dir.exists() {
        anyhow::bail!("Root directory does not exist: {}", args.root_dir.display());
    }
    
    if !args.root_dir.is_dir() {
        anyhow::bail!("Root path is not a directory: {}", args.root_dir.display());
    }
    
    info!("Project setup validation completed successfully");
    
    // Discover and validate files
    let discovery_config = discovery::DiscoveryConfig {
        fail_fast: args.fail_fast,
    };
    
    info!("Starting file discovery in: {}", args.root_dir.display());
    let discovered_files = discovery::collect_discovered_files(&args.root_dir, discovery_config).await?;
    
    let valid_files: Vec<_> = discovered_files.iter()
        .filter(|f| f.is_valid_utf8 && f.error.is_none())
        .collect();
    
    let invalid_files: Vec<_> = discovered_files.iter()
        .filter(|f| !f.is_valid_utf8 || f.error.is_some())
        .collect();
    
    info!("File discovery completed: {} total files found", discovered_files.len());
    info!("Valid UTF-8 files: {}", valid_files.len());
    
    if !invalid_files.is_empty() {
        info!("Files with issues: {}", invalid_files.len());
        for file in &invalid_files {
            if let Some(ref error) = file.error {
                info!("Issue with {}: {}", file.path.display(), error);
            } else if !file.is_valid_utf8 {
                info!("UTF-8 validation failed: {}", file.path.display());
            }
        }
    }
    
    println!("seams v{} - File discovery complete", env!("CARGO_PKG_VERSION"));
    println!("Found {} files matching pattern *-0.txt", discovered_files.len());
    println!("Valid files: {}, Files with issues: {}", valid_files.len(), invalid_files.len());
    
    // WHY: Demonstrate public API usage for external developers (minimal example)
    if std::env::var("SEAMS_DEBUG_API").is_ok() && !valid_files.is_empty() {
        let example_path = &valid_files[0].path;
        let demo_content = "0\tExample usage of public API.\t(1,1,1,27)\n";
        if let Ok(_) = create_complete_aux_file(example_path, demo_content) {
            info!("Created demo aux file using public API for {}", example_path.display());
        }
    }
    
    // Process valid files with async reader
    if !valid_files.is_empty() {
        info!("Starting async file reading for {} valid files", valid_files.len());
        
        // WHY: Load processing cache for incremental processing
        let cache_path = generate_cache_path(&args.root_dir);
        let mut cache = ProcessingCache::load(&args.root_dir).await;
        info!("Loaded processing cache from {}", cache_path.display());
        
        // WHY: Log cache status for debugging (sync API usage)
        if let Ok(cache_content) = read_cache(&args.root_dir) {
            let cache_size = cache_content.len();
            info!("Cache file size: {} bytes", cache_size);
        }
        
        let reader_config = reader::ReaderConfig {
            fail_fast: args.fail_fast,
            buffer_size: 8192,
        };
        let file_reader = reader::AsyncFileReader::new(reader_config);
        
        // WHY: process files sequentially to demonstrate async reading without overwhelming memory
        let valid_paths: Vec<_> = valid_files.iter().map(|f| &f.path).collect();
        let read_results = file_reader.read_files_batch(&valid_paths).await?;
        
        // WHY: Use dialog detector with quote truncation fix
        info!("Initializing dialog-aware sentence detector");
        let sentence_detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize dialog sentence detector: {}", e))?;
        info!("Successfully initialized dialog sentence detector");
        
        let mut total_lines = 0u64;
        let mut total_bytes = 0u64;
        let mut total_sentences = 0u64;
        let mut successful_reads = 0;
        let mut failed_reads = 0;
        let mut skipped_files = 0;
        
        for (lines, stats) in read_results {
            total_lines += stats.lines_read;
            total_bytes += stats.bytes_read;
            
            if stats.read_error.is_some() {
                failed_reads += 1;
                if let Some(ref error) = stats.read_error {
                    info!("Read error for {}: {}", stats.file_path, error);
                }
            } else {
                successful_reads += 1;
                
                // WHY: check if file should be processed based on cache and incremental rules
                let source_path = Path::new(&stats.file_path);
                let should_process = match should_process_file(source_path, &cache, args.overwrite_all).await {
                    Ok(should_process) => should_process,
                    Err(e) => {
                        info!("Error checking if {} should be processed: {}", stats.file_path, e);
                        if args.fail_fast {
                            return Err(e);
                        }
                        true // Process on error to be safe
                    }
                };
                
                if !should_process {
                    skipped_files += 1;
                    continue;
                }
                
                // WHY: process sentences only for files that should be processed
                let file_content = lines.join("\n");
                
                match sentence_detector.detect_sentences(&file_content) {
                    Ok(sentences) => {
                        let sentence_count = sentences.len() as u64;
                        total_sentences += sentence_count;
                        
                        info!("Detected {} sentences in {}", sentence_count, stats.file_path);
                        
                        // WHY: generate auxiliary file as per F-7 requirement
                        // Note: For simple aux file creation, external users can use create_complete_aux_file()
                        let aux_path = generate_aux_file_path(source_path);
                        
                        match write_auxiliary_file(&aux_path, &sentences, &sentence_detector).await {
                            Ok(()) => {
                                info!("Successfully wrote auxiliary file: {}", aux_path.display());
                                
                                // WHY: validate aux file was written correctly using public API
                                if let Ok(aux_content) = read_aux_file(source_path) {
                                    let line_count = aux_content.lines().count();
                                    if line_count != sentence_count as usize {
                                        info!("Warning: aux file line count ({}) doesn't match sentence count ({})", 
                                              line_count, sentence_count);
                                    }
                                }
                                
                                // WHY: mark file as completed in cache after successful aux file write
                                cache.mark_completed(source_path);
                                
                                // WHY: demonstrate output format as per F-5 specification
                                if sentence_count > 0 && sentence_count <= 3 {
                                    info!("Sample sentences from {}:", stats.file_path);
                                    for sentence in sentences.iter().take(2) {
                                        let formatted = format!("{}\t{}\t({},{},{},{})", 
                                            sentence.index, 
                                            sentence.normalized_content,
                                            sentence.span.start_line,
                                            sentence.span.start_col,
                                            sentence.span.end_line,
                                            sentence.span.end_col
                                        );
                                        info!("  {}", formatted);
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Failed to write auxiliary file {}: {}", aux_path.display(), e);
                                if args.fail_fast {
                                    return Err(e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        info!("Failed to detect sentences in {}: {}", stats.file_path, e);
                        if args.fail_fast {
                            return Err(anyhow::anyhow!("Sentence detection failed: {}", e));
                        }
                    }
                }
            }
            
            info!("Read {}: {} lines, {} bytes", stats.file_path, stats.lines_read, stats.bytes_read);
        }
        
        println!("File processing complete:");
        println!("  Successfully processed: {} files", successful_reads - skipped_files);
        if skipped_files > 0 {
            println!("  Skipped (complete aux files): {skipped_files} files");
        }
        if failed_reads > 0 {
            println!("  Failed to read: {failed_reads} files");
        }
        println!("  Total lines processed: {total_lines}");
        println!("  Total bytes processed: {total_bytes}");
        println!("  Total sentences detected: {total_sentences}");
        
        info!("File processing completed: {} processed, {} skipped, {} failed, {} sentences detected", 
              successful_reads - skipped_files, skipped_files, failed_reads, total_sentences);
              
        // WHY: Save cache after processing to persist completion state for future runs
        if let Err(e) = cache.save(&cache_path).await {
            info!("Warning: Failed to save processing cache: {}", e);
            // Don't fail the entire process if cache save fails
        } else {
            info!("Saved processing cache to {}", cache_path.display());
        }
    }
    
    Ok(())
}