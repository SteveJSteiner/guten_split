use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::info;
use num_cpus::get as num_cpus_get;

mod discovery;
mod sentence_detector;
mod incremental;
mod parallel_processing;

use crate::incremental::{generate_cache_path, aux_file_exists, create_complete_aux_file, read_cache};
use crate::parallel_processing::{ProcessingCache, FileStats, process_files_parallel, should_process_file};
use futures::stream::StreamExt;
use futures::future::join_all;
use std::sync::Arc;
use std::collections::VecDeque;

// Types and implementations moved to parallel_processing module

/// Aggregate run statistics
/// WHY: Provides overall metrics for the entire run per PRD F-8 requirements
#[derive(Serialize, Deserialize, Debug)]
struct RunStats {
    /// Timestamp when run started
    run_start: String,
    /// Total processing time in milliseconds
    total_processing_time_ms: u64,
    /// Total characters processed across all files
    total_chars_processed: u64,
    /// Total sentences detected across all files
    total_sentences_detected: u64,
    /// Overall throughput in characters per second
    overall_chars_per_sec: f64,
    /// Number of files successfully processed
    files_processed: u64,
    /// Number of files skipped (already complete)
    files_skipped: u64,
    /// Number of files that failed processing
    files_failed: u64,
    /// Per-file statistics
    file_stats: Vec<FileStats>,
}

/// Process files using overlapped discovery and processing pipeline
/// WHY: Overlaps file discovery with processing to reduce total pipeline time
async fn process_with_overlapped_pipeline(
    args: &Args,
    cache: &mut ProcessingCache,
) -> Result<(u64, u64, u64, u64, u64, Vec<FileStats>, std::time::Duration)> {
    let pipeline_start = std::time::Instant::now();
    
    // Check if we can use cached discovery results
    let discovery_config = discovery::DiscoveryConfig {
        fail_fast: args.fail_fast,
    };
    
    let use_cached_discovery = !args.overwrite_all && !args.overwrite_use_cached_locations;
    
    if use_cached_discovery {
        if let Some(cached_files) = cache.get_cached_discovered_files(&args.root_dir).await? {
            info!("Using cached file discovery results ({} files)", cached_files.len());
            return process_discovered_files(args, cache, cached_files, pipeline_start).await;
        }
    } else if args.overwrite_use_cached_locations {
        if let Some(cached_files) = cache.get_cached_discovered_files(&args.root_dir).await? {
            info!("Using cached file discovery results with overwrite mode ({} files)", cached_files.len());
            return process_discovered_files(args, cache, cached_files, pipeline_start).await;
        }
    }
    
    // Perform overlapped discovery and processing
    info!("Starting overlapped file discovery and processing in: {}", args.root_dir.display());
    
    let fail_fast = args.fail_fast;
    let mut discovery_stream = Box::pin(discovery::discover_files_parallel(&args.root_dir, discovery_config));
    let mut discovered_files = Vec::new();
    let mut valid_files = Vec::new();
    let mut invalid_files = Vec::new();
    let mut processing_queue = VecDeque::new();
    let mut processing_results = Vec::new();
    
    // WHY: Use bounded concurrency to prevent resource exhaustion
    let max_concurrent = num_cpus_get().min(8);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    let detector = Arc::new(
        crate::sentence_detector::dialog_detector::SentenceDetectorDialog::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize dialog sentence detector: {}", e))?
    );
    
    // Process files as they're discovered
    while let Some(discovery_result) = discovery_stream.next().await {
        match discovery_result {
            Ok(file_validation) => {
                discovered_files.push(file_validation.clone());
                
                if file_validation.is_valid_utf8 && file_validation.error.is_none() {
                    valid_files.push(file_validation.clone());
                    
                    // Start processing this file immediately
                    let semaphore_clone = semaphore.clone();
                    let detector_clone = detector.clone();
                    let cache_clone = cache.clone();
                    let path = file_validation.path.clone();
                    let overwrite_all = args.overwrite_all;
                    let overwrite_use_cached_locations = args.overwrite_use_cached_locations;
                    
                    let task = tokio::spawn(async move {
                        let _permit = semaphore_clone.acquire().await.unwrap();
                        process_single_file(
                            &path,
                            &detector_clone,
                            &cache_clone,
                            overwrite_all,
                            overwrite_use_cached_locations,
                        ).await
                    });
                    
                    processing_queue.push_back(task);
                } else {
                    invalid_files.push(file_validation);
                }
            }
            Err(e) => {
                if fail_fast {
                    return Err(e);
                }
                info!("Discovery error (continuing): {}", e);
            }
        }
        
        // Check for completed processing tasks
        while let Some(task) = processing_queue.front() {
            if task.is_finished() {
                let task = processing_queue.pop_front().unwrap();
                match task.await {
                    Ok(result) => processing_results.push(result),
                    Err(e) => {
                        info!("Processing task error: {}", e);
                        if fail_fast {
                            return Err(anyhow::anyhow!("Processing task failed: {}", e));
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
    
    // Wait for remaining processing tasks to complete
    let remaining_tasks: Vec<_> = processing_queue.into_iter().collect();
    let remaining_results = join_all(remaining_tasks).await;
    
    for result in remaining_results {
        match result {
            Ok(processing_result) => processing_results.push(processing_result),
            Err(e) => {
                info!("Remaining processing task error: {}", e);
                if fail_fast {
                    return Err(anyhow::anyhow!("Processing task failed: {}", e));
                }
            }
        }
    }
    
    // Update cache with discovery results
    cache.update_discovery_cache(&args.root_dir, &discovered_files).await?;
    
    // Process results and update cache
    let mut total_sentences = 0u64;
    let mut total_bytes = 0u64;
    let mut processed_files = 0u64;
    let mut skipped_files = 0u64;
    let mut failed_files = 0u64;
    let mut file_stats = Vec::new();
    
    for result in processing_results {
        match result {
            Ok((sentences, bytes, files, skipped, stats)) => {
                total_sentences += sentences;
                total_bytes += bytes;
                if skipped {
                    skipped_files += 1;
                } else {
                    processed_files += files;
                    // Mark file as completed in cache
                    let path = std::path::PathBuf::from(&stats.path);
                    cache.mark_completed(&path);
                }
                file_stats.push(stats);
            }
            Err(e) => {
                failed_files += 1;
                info!("Processing error: {}", e);
                
                let failed_stats = FileStats {
                    path: "unknown".to_string(),
                    chars_processed: 0,
                    sentences_detected: 0,
                    processing_time_ms: 0,
                    chars_per_sec: 0.0,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                };
                file_stats.push(failed_stats);
            }
        }
    }
    
    // Save cache
    let cache_path = generate_cache_path(&args.root_dir);
    if let Err(e) = cache.save(&cache_path).await {
        info!("Warning: Failed to save processing cache: {}", e);
    } else {
        info!("Saved processing cache to {}", cache_path.display());
    }
    
    let processing_duration = pipeline_start.elapsed();
    
    // Log and display results
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
    
    println!("seams v{} - Overlapped discovery and processing complete", env!("CARGO_PKG_VERSION"));
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
    
    if !valid_files.is_empty() {
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
        
        info!("Overlapped pipeline completed: {} processed, {} skipped, {} failed, {} sentences detected", 
              processed_files, skipped_files, failed_files, total_sentences);
    }
    
    Ok((total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration))
}

/// Process a single file - extracted from parallel_processing for use in overlapped pipeline
async fn process_single_file(
    path: &std::path::Path,
    detector: &crate::sentence_detector::dialog_detector::SentenceDetectorDialog,
    cache: &ProcessingCache,
    overwrite_all: bool,
    overwrite_use_cached_locations: bool,
) -> Result<(u64, u64, u64, bool, FileStats)> {
    let start_time = std::time::Instant::now();
    
    // Check if file should be processed
    let should_process = should_process_file(path, cache, overwrite_all, overwrite_use_cached_locations).await
        .unwrap_or(true);
    
    if !should_process {
        let processing_time = start_time.elapsed();
        let file_stats = FileStats {
            path: path.to_string_lossy().to_string(),
            chars_processed: 0,
            sentences_detected: 0,
            processing_time_ms: processing_time.as_millis() as u64,
            chars_per_sec: 0.0,
            status: "skipped".to_string(),
            error: None,
        };
        return Ok((0u64, 0u64, 0u64, true, file_stats));
    }
    
    // Process the file
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
    let content = std::str::from_utf8(&mmap)
        .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in file: {}", path.display()))?;
    
    let sentences = detector.detect_sentences_borrowed(content)?;
    let sentence_count = sentences.len() as u64;
    let byte_count = content.len() as u64;
    
    // Generate auxiliary file
    let aux_path = crate::incremental::generate_aux_file_path(path);
    crate::parallel_processing::write_auxiliary_file_borrowed(&aux_path, &sentences, detector).await?;
    
    let processing_time = start_time.elapsed();
    let processing_time_ms = processing_time.as_millis() as u64;
    let chars_per_sec = if processing_time.as_secs_f64() > 0.0 {
        byte_count as f64 / processing_time.as_secs_f64()
    } else {
        0.0
    };
    
    let file_stats = FileStats {
        path: path.to_string_lossy().to_string(),
        chars_processed: byte_count,
        sentences_detected: sentence_count,
        processing_time_ms,
        chars_per_sec,
        status: "success".to_string(),
        error: None,
    };
    
    info!("Processed {}: {} sentences, {} bytes", path.display(), sentence_count, byte_count);
    
    Ok((sentence_count, byte_count, 1u64, false, file_stats))
}

/// Process already-discovered files (fallback for cached discovery)
async fn process_discovered_files(
    args: &Args,
    cache: &mut ProcessingCache,
    discovered_files: Vec<discovery::FileValidation>,
    pipeline_start: std::time::Instant,
) -> Result<(u64, u64, u64, u64, u64, Vec<FileStats>, std::time::Duration)> {
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
    
    // Process valid files
    let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, _processing_duration) = 
        if !valid_files.is_empty() {
            info!("Starting async file reading for {} valid files", valid_files.len());
            
            let cache_path = generate_cache_path(&args.root_dir);
            info!("Using processing cache from {}", cache_path.display());
            
            // WHY: Log cache status for debugging (sync API usage)
            if let Ok(cache_content) = read_cache(&args.root_dir) {
                let cache_size = cache_content.len();
                info!("Cache file size: {} bytes", cache_size);
            }
            
            // WHY: Use bounded concurrency to prevent resource exhaustion
            let max_concurrent = num_cpus_get().min(8);
            info!("Processing files with concurrency limit: {}", max_concurrent);
            
            let valid_paths: Vec<_> = valid_files.iter().map(|f| &f.path).collect();
            
            // WHY: Use existing parallel processing for cached discovery
            let start_time = std::time::Instant::now();
            let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats) = 
                process_files_parallel(&valid_paths, cache.clone(), args.overwrite_all, args.overwrite_use_cached_locations, args.fail_fast, max_concurrent).await?;
            let processing_duration = start_time.elapsed();
            
            // WHY: Update cache for successfully processed files
            if processed_files > 0 {
                for path in &valid_paths {
                    if aux_file_exists(path) {
                        let should_process = should_process_file(path, cache, args.overwrite_all, args.overwrite_use_cached_locations).await.unwrap_or(false);
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
            
            // WHY: Show performance metrics
            if total_bytes > 0 && processing_duration.as_secs_f64() > 0.0 {
                let throughput_chars_per_sec = total_bytes as f64 / processing_duration.as_secs_f64();
                let throughput_mb_per_sec = throughput_chars_per_sec / 1_000_000.0;
                println!("  Throughput: {throughput_chars_per_sec:.0} chars/sec ({throughput_mb_per_sec:.2} MB/s)");
            }
            
            info!("Parallel processing completed: {} processed, {} skipped, {} failed, {} sentences detected", 
                  processed_files, skipped_files, failed_files, total_sentences);
                  
            // WHY: Save cache after processing
            let cache_path = generate_cache_path(&args.root_dir);
            if let Err(e) = cache.save(&cache_path).await {
                info!("Warning: Failed to save processing cache: {}", e);
            } else {
                info!("Saved processing cache to {}", cache_path.display());
            }
            
            (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration)
        } else {
            // WHY: No valid files found, return empty stats
            (0, 0, 0, 0, 0, Vec::new(), std::time::Duration::from_millis(0))
        };
    
    let total_duration = pipeline_start.elapsed();
    Ok((total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, total_duration))
}

// Functions moved to parallel_processing module

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
    
    // WHY: Load processing cache early to check if discovery cache is valid
    let _cache_path = generate_cache_path(&args.root_dir);
    let mut cache = ProcessingCache::load(&args.root_dir).await;
    
    // WHY: Use overlapped discovery and processing pipeline for optimal performance
    let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration) = 
        process_with_overlapped_pipeline(&args, &mut cache).await?;
    
    // WHY: Generate run statistics per PRD F-8 requirement
    let overall_chars_per_sec = if processing_duration.as_secs_f64() > 0.0 {
        total_bytes as f64 / processing_duration.as_secs_f64()
    } else {
        0.0
    };
    
    let run_stats = RunStats {
        run_start: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs().to_string(),
        total_processing_time_ms: processing_duration.as_millis() as u64,
        total_chars_processed: total_bytes,
        total_sentences_detected: total_sentences,
        overall_chars_per_sec,
        files_processed: processed_files,
        files_skipped: skipped_files,
        files_failed: failed_files,
        file_stats,
    };
    
    // WHY: Write stats to JSON file as specified by --stats-out flag
    match serde_json::to_string_pretty(&run_stats) {
        Ok(json_content) => {
            match tokio::fs::write(&args.stats_out, json_content).await {
                Ok(()) => {
                    info!("Stats written to {}", args.stats_out.display());
                    println!("Stats written to {}", args.stats_out.display());
                }
                Err(e) => {
                    info!("Warning: Failed to write stats file: {e}");
                    println!("Warning: Failed to write stats file: {e}");
                }
            }
        }
        Err(e) => {
            info!("Warning: Failed to serialize stats: {e}");
            println!("Warning: Failed to serialize stats: {e}");
        }
    }
    
    Ok(())
}