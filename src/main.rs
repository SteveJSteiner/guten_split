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
    let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration) = 
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
            let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats) = 
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
            
            (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration)
        } else {
            // WHY: No valid files found, return empty stats
            (0, 0, 0, 0, 0, Vec::new(), std::time::Duration::from_millis(0))
        };
    
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