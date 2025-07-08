use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use tracing::info;
use memmap2::MmapOptions;
use num_cpus::get as num_cpus_get;

mod discovery;
mod sentence_detector;
mod incremental;

use crate::incremental::{generate_aux_file_path, generate_cache_path, cache_exists, read_cache_async, aux_file_exists, create_complete_aux_file, read_cache};

/// Comprehensive file metadata for both discovery and processing cache
/// WHY: Unified structure reduces duplication and simplifies cache management
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileMetadata {
    /// File size in bytes
    size: u64,
    /// Last modification timestamp (seconds since epoch)
    modified: u64,
    /// Whether file passed UTF-8 validation
    is_valid_utf8: bool,
    /// Processing completion timestamp (None if not yet processed)
    completed_at: Option<u64>,
}

/// Unified cache for file discovery and processing state
/// WHY: Single source of truth for all file state, eliminating redundancy
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct ProcessingCache {
    /// Map from source file path to comprehensive metadata
    files: HashMap<String, FileMetadata>,
    /// Timestamp when file discovery was last performed
    discovery_timestamp: Option<u64>,
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
        
        if let Some(file_meta) = self.files.get(&source_path_str) {
            if let Some(completion_timestamp) = file_meta.completed_at {
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
        
        // Update existing entry or create new one
        if let Some(file_meta) = self.files.get_mut(&source_path_str) {
            file_meta.completed_at = Some(now);
        } else {
            // This shouldn't happen if discovery was done properly, but handle gracefully
            info!("Marking completion for undiscovered file: {}", source_path.display());
            self.files.insert(source_path_str, FileMetadata {
                size: 0, // Will be updated on next discovery
                modified: 0,
                is_valid_utf8: true, // Assume valid if we processed it
                completed_at: Some(now),
            });
        }
    }
    
    /// Check if file discovery cache is valid for given root directory
    /// WHY: Implements cache invalidation logic based on file-level verification
    async fn is_discovery_cache_valid(&self, _root_dir: &Path) -> Result<bool> {
        // Check if we have discovery metadata at all
        if self.discovery_timestamp.is_none() || self.files.is_empty() {
            return Ok(false);
        }
        
        // For performance, we validate cached files during get_cached_discovered_files
        // This method only checks if we have any cached discovery data
        Ok(true)
    }
    
    /// Get cached discovered files if cache is valid
    /// WHY: Provides fast restart by avoiding directory traversal when possible
    async fn get_cached_discovered_files(&self, root_dir: &Path) -> Result<Option<Vec<discovery::FileValidation>>> {
        if !self.is_discovery_cache_valid(root_dir).await? {
            return Ok(None);
        }
        
        let mut files = Vec::new();
        for (path_str, file_meta) in &self.files {
            let path = PathBuf::from(path_str);
            
            // Verify file still exists and hasn't changed
            if let Ok(current_metadata) = tokio::fs::metadata(&path).await {
                let current_modified = current_metadata.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                let current_size = current_metadata.len();
                
                if current_modified <= file_meta.modified && current_size == file_meta.size {
                    files.push(discovery::FileValidation {
                        path,
                        is_valid_utf8: file_meta.is_valid_utf8,
                        error: None,
                    });
                } else {
                    // File changed, cache is invalid
                    return Ok(None);
                }
            } else {
                // File no longer exists, cache is invalid
                return Ok(None);
            }
        }
        
        Ok(Some(files))
    }
    
    /// Update discovery cache with new file discovery results
    /// WHY: Stores discovery results for future fast restarts
    async fn update_discovery_cache(&mut self, _root_dir: &Path, files: &[discovery::FileValidation]) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        
        // Clear existing discovery cache but preserve completion status
        let mut completed_files = HashMap::new();
        for (path, file_meta) in &self.files {
            if let Some(completed_at) = file_meta.completed_at {
                completed_files.insert(path.clone(), completed_at);
            }
        }
        
        // Clear and rebuild file cache
        self.files.clear();
        
        // Add new discovery results
        for file in files {
            if let Ok(metadata) = tokio::fs::metadata(&file.path).await {
                let modified = metadata.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                
                let path_str = file.path.to_string_lossy().to_string();
                self.files.insert(
                    path_str.clone(),
                    FileMetadata {
                        size: metadata.len(),
                        modified,
                        is_valid_utf8: file.is_valid_utf8,
                        completed_at: completed_files.get(&path_str).copied(),
                    }
                );
            }
        }
        
        self.discovery_timestamp = Some(now);
        
        Ok(())
    }
}


/// Determine if file should be processed based on cache and incremental rules
/// WHY: Implements core F-9 logic using robust timestamp-based cache
async fn should_process_file(source_path: &Path, cache: &ProcessingCache, overwrite_all: bool, overwrite_use_cached_locations: bool) -> Result<bool> {
    if overwrite_all || overwrite_use_cached_locations {
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


/// Write auxiliary file with borrowed sentence data in F-5 format
/// WHY: Zero-allocation async I/O optimized for mmap-based processing
async fn write_auxiliary_file_borrowed(
    aux_path: &Path,
    sentences: &[sentence_detector::DetectedSentenceBorrowed<'_>],
    _detector: &sentence_detector::dialog_detector::SentenceDetectorDialog,
) -> Result<()> {
    let file = tokio::fs::File::create(aux_path).await?;
    let mut writer = BufWriter::new(file);
    
    for sentence in sentences {
        // WHY: Call normalize() on-demand to maintain zero-allocation benefits
        let formatted_line = format!("{}\t{}\t({},{},{},{})", 
            sentence.index, 
            sentence.normalize(),
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

/// Process multiple files in parallel using memory-mapped I/O
/// WHY: Combines async orchestration with mmap for optimal performance and true parallelism
async fn process_files_parallel(
    file_paths: &[&PathBuf],
    cache: ProcessingCache,
    overwrite_all: bool,
    overwrite_use_cached_locations: bool,
    fail_fast: bool,
    max_concurrent: usize,
) -> Result<(u64, u64, u64, u64, u64)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let detector = Arc::new(
        sentence_detector::dialog_detector::SentenceDetectorDialog::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize dialog sentence detector: {}", e))?
    );
    
    let tasks: Vec<_> = file_paths.iter().map(|&path| {
        let semaphore = semaphore.clone();
        let detector = detector.clone();
        let path = path.clone();
        let cache_clone = cache.clone();
        
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            // WHY: Check if file should be processed based on cache and incremental rules
            let should_process = should_process_file(&path, &cache_clone, overwrite_all, overwrite_use_cached_locations).await
                .unwrap_or(true); // Process on error to be safe
            
            if !should_process {
                return Ok((0u64, 0u64, 0u64, true)); // skipped
            }
            
            // WHY: Use mmap for efficient file access instead of loading into memory
            let file = std::fs::File::open(&path)?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            let content = std::str::from_utf8(&mmap)
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in file: {}", path.display()))?;
            
            // WHY: Use borrowed API for zero-allocation sentence detection
            let sentences = detector.detect_sentences_borrowed(content)?;
            let sentence_count = sentences.len() as u64;
            let byte_count = content.len() as u64;
            
            // WHY: Generate auxiliary file as per F-7 requirement
            let aux_path = generate_aux_file_path(&path);
            write_auxiliary_file_borrowed(&aux_path, &sentences, &detector).await?;
            
            info!("Processed {}: {} sentences, {} bytes", path.display(), sentence_count, byte_count);
            
            Ok((sentence_count, byte_count, 1u64, false)) // sentences, bytes, files_processed, skipped
        })
    }).collect();
    
    let mut total_sentences = 0u64;
    let mut total_bytes = 0u64;
    let mut processed_files = 0u64;
    let mut skipped_files = 0u64;
    let mut failed_files = 0u64;
    
    // WHY: Wait for all tasks and handle results
    let results = futures::future::join_all(tasks).await;
    
    for result in results {
        match result {
            Ok(Ok((sentences, bytes, files, skipped))) => {
                total_sentences += sentences;
                total_bytes += bytes;
                if skipped {
                    skipped_files += 1;
                } else {
                    processed_files += files;
                }
            }
            Ok(Err(e)) => {
                info!("File processing error: {}", e);
                failed_files += 1;
                if fail_fast {
                    return Err(e);
                }
            }
            Err(e) => {
                info!("Task execution error: {}", e);
                failed_files += 1;
                if fail_fast {
                    return Err(anyhow::anyhow!("Task failed: {}", e));
                }
            }
        }
    }
    
    Ok((total_sentences, total_bytes, processed_files, skipped_files, failed_files))
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
    
    /// Overwrite aux files but use cached file discovery results
    #[arg(long)]
    overwrite_use_cached_locations: bool,
    
    /// Abort on first error
    #[arg(long)]
    fail_fast: bool,
    
    
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
    
    // WHY: Load processing cache early to check if discovery cache is valid
    let cache_path = generate_cache_path(&args.root_dir);
    let mut cache = ProcessingCache::load(&args.root_dir).await;
    
    // WHY: Use cached discovery results if available and valid, otherwise perform fresh discovery
    // Handle different overwrite modes appropriately
    let discovered_files = if !args.overwrite_all && !args.overwrite_use_cached_locations {
        if let Some(cached_files) = cache.get_cached_discovered_files(&args.root_dir).await? {
            info!("Using cached file discovery results ({} files)", cached_files.len());
            cached_files
        } else {
            info!("Cache invalid or missing, performing fresh file discovery");
            let fresh_files = discovery::collect_discovered_files(&args.root_dir, discovery_config).await?;
            
            // Update cache with fresh discovery results
            cache.update_discovery_cache(&args.root_dir, &fresh_files).await?;
            
            // Save cache immediately after discovery update
            cache.save(&cache_path).await?;
            
            fresh_files
        }
    } else if args.overwrite_use_cached_locations {
        // WHY: Use cached discovery but still allow overwriting aux files
        if let Some(cached_files) = cache.get_cached_discovered_files(&args.root_dir).await? {
            info!("Using cached file discovery results with overwrite mode ({} files)", cached_files.len());
            cached_files
        } else {
            info!("Cache invalid for overwrite mode, performing fresh file discovery");
            let fresh_files = discovery::collect_discovered_files(&args.root_dir, discovery_config).await?;
            
            // Update cache with fresh discovery results
            cache.update_discovery_cache(&args.root_dir, &fresh_files).await?;
            
            // Save cache immediately after discovery update
            cache.save(&cache_path).await?;
            
            fresh_files
        }
    } else {
        info!("Overwrite all flag specified, performing fresh file discovery");
        let fresh_files = discovery::collect_discovered_files(&args.root_dir, discovery_config).await?;
        
        // Update cache with fresh discovery results
        cache.update_discovery_cache(&args.root_dir, &fresh_files).await?;
        
        // Save cache immediately after discovery update
        cache.save(&cache_path).await?;
        
        fresh_files
    };
    
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
        if create_complete_aux_file(example_path, demo_content).is_ok() {
            info!("Created demo aux file using public API for {}", example_path.display());
        }
    }
    
    // Process valid files with async reader
    if !valid_files.is_empty() {
        info!("Starting async file reading for {} valid files", valid_files.len());
        
        // WHY: Processing cache was already loaded during discovery phase
        info!("Using processing cache from {}", cache_path.display());
        
        // WHY: Log cache status for debugging (sync API usage)
        if let Ok(cache_content) = read_cache(&args.root_dir) {
            let cache_size = cache_content.len();
            info!("Cache file size: {} bytes", cache_size);
        }
        
        // WHY: Use bounded concurrency to prevent resource exhaustion
        let max_concurrent = num_cpus_get().min(8); // Limit to 8 concurrent files max
        info!("Processing files with concurrency limit: {}", max_concurrent);
        
        let valid_paths: Vec<_> = valid_files.iter().map(|f| &f.path).collect();
        
        // WHY: Use new parallel mmap-based processing for optimal performance
        let start_time = std::time::Instant::now();
        let (total_sentences, total_bytes, processed_files, skipped_files, failed_files) = 
            process_files_parallel(&valid_paths, cache.clone(), args.overwrite_all, args.overwrite_use_cached_locations, args.fail_fast, max_concurrent).await?;
        let processing_duration = start_time.elapsed();
        
        // WHY: Update cache for successfully processed files (not skipped ones)
        if processed_files > 0 {
            for path in &valid_paths {
                if aux_file_exists(path) {
                    // Only mark as completed if the file was actually processed in this run
                    // Check if this file was newly created by checking if it should have been processed
                    let should_process = should_process_file(path, &cache, args.overwrite_all, args.overwrite_use_cached_locations).await.unwrap_or(false);
                    if should_process {
                        cache.mark_completed(path);
                    }
                }
            }
        }
        
        println!("File processing complete:");
        println!("  Successfully processed: {processed_files} files");
        if skipped_files > 0 {
            println!("  Skipped (complete aux files): {skipped_files} files");
        }
        if failed_files > 0 {
            println!("  Failed to process: {failed_files} files");
        }
        println!("  Total bytes processed: {total_bytes}");
        println!("  Total sentences detected: {total_sentences}");
        println!("  Total time spent: {:.2}s", processing_duration.as_secs_f64());
        
        // WHY: Show performance metrics - Total characters / Total time spent
        if total_bytes > 0 && processing_duration.as_secs_f64() > 0.0 {
            let throughput_chars_per_sec = total_bytes as f64 / processing_duration.as_secs_f64();
            let throughput_mb_per_sec = throughput_chars_per_sec / 1_000_000.0;
            println!("  Throughput: {throughput_chars_per_sec:.0} chars/sec ({throughput_mb_per_sec:.2} MB/s)");
        }
        
        info!("Parallel mmap processing completed: {} processed, {} skipped, {} failed, {} sentences detected", 
              processed_files, skipped_files, failed_files, total_sentences);
              
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