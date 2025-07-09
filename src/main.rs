use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::info;

mod discovery;
mod sentence_detector;
mod incremental;
mod parallel_processing;
mod restart_log;

use crate::incremental::create_complete_aux_file;
use crate::parallel_processing::FileStats;
use crate::restart_log::{RestartLog, should_process_file};
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
    restart_log: &mut RestartLog,
) -> Result<(u64, u64, u64, u64, u64, Vec<FileStats>, std::time::Duration)> {
    let pipeline_start = std::time::Instant::now();
    
    // Always perform fresh discovery - parallel discovery is fast enough
    let discovery_config = discovery::DiscoveryConfig {
        fail_fast: args.fail_fast,
    };
    
    // Perform overlapped discovery and processing
    info!("Starting overlapped file discovery and processing in: {}", args.root_dir.display());
    
    let fail_fast = args.fail_fast;
    let quiet = args.quiet;
    let mut discovery_stream = Box::pin(discovery::discover_files_parallel(&args.root_dir, discovery_config));
    let mut discovered_files = Vec::new();
    let mut valid_files = Vec::new();
    let mut invalid_files = Vec::new();
    let mut processing_queue = VecDeque::new();
    let mut processing_results = Vec::new();
    
    // WHY: Use bounded concurrency to prevent resource exhaustion
    let max_concurrent = (num_cpus::get() / 2).max(1);
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
                    let path = file_validation.path.clone();
                    let overwrite_all = args.overwrite_all;
                    
                    // Check if file should be processed using restart log
                    let should_process = should_process_file(&path, restart_log, overwrite_all).await.unwrap_or(true);
                    
                    if should_process {
                        let task = tokio::spawn(async move {
                            let _permit = semaphore_clone.acquire().await.unwrap();
                            process_single_file_restart(
                                &path,
                                &detector_clone,
                                overwrite_all,
                                quiet,
                            ).await
                        });
                        
                        processing_queue.push_back(task);
                    } else {
                        // File is already processed, create a skip result
                        let skip_result = Ok((0u64, 0u64, 0u64, true, FileStats {
                            path: path.to_string_lossy().to_string(),
                            chars_processed: 0,
                            sentences_detected: 0,
                            processing_time_ms: 0,
                            sentence_detection_time_ms: 0,
                            chars_per_sec: 0.0,
                            status: "skipped".to_string(),
                            error: None,
                        }));
                        processing_results.push(skip_result);
                    }
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
    
    // Process results and update restart log
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
                    // Mark file as completed in restart log
                    let path = std::path::PathBuf::from(&stats.path);
                    restart_log.mark_completed(&path);
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
                    sentence_detection_time_ms: 0,
                    chars_per_sec: 0.0,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                };
                file_stats.push(failed_stats);
            }
        }
    }
    
    // Save restart log
    if let Err(e) = restart_log.save(&args.root_dir).await {
        info!("Warning: Failed to save restart log: {}", e);
    } else {
        info!("Saved restart log to {}", args.root_dir.join(".seams_restart.json").display());
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
    
    if !quiet {
        println!("seams v{} - Overlapped discovery and processing complete", env!("CARGO_PKG_VERSION"));
        println!("Found {} files matching pattern *-0.txt", discovered_files.len());
        println!("Valid files: {}, Files with issues: {}", valid_files.len(), invalid_files.len());
        println!("Restart log: {} files tracked as completed", restart_log.completed_count());
    }
    
    // WHY: Demonstrate public API usage for external developers (minimal example)
    if std::env::var("SEAMS_DEBUG_API").is_ok() && !valid_files.is_empty() {
        let example_path = &valid_files[0].path;
        let demo_content = "0\tExample usage of public API.\t(1,1,1,27)\n";
        if create_complete_aux_file(example_path, demo_content).is_ok() {
            info!("Created demo aux file using public API for {}", example_path.display());
        }
    }
    
    if !quiet && !valid_files.is_empty() {
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
            println!("  Overall throughput: {throughput_chars_per_sec:.0} chars/sec ({throughput_mb_per_sec:.2} MB/s)");
            
            // WHY: Show sentence detection throughput for just the detection algorithm
            let total_sentence_detection_time_ms: u64 = file_stats.iter()
                .map(|fs| fs.sentence_detection_time_ms)
                .sum();
            
            if total_sentence_detection_time_ms > 0 && total_sentences > 0 {
                let sentence_detection_time_sec = total_sentence_detection_time_ms as f64 / 1000.0;
                let sentence_detection_throughput_chars_per_sec = total_bytes as f64 / sentence_detection_time_sec;
                let sentence_detection_throughput_mb_per_sec = sentence_detection_throughput_chars_per_sec / 1_000_000.0;
                println!("  Sentence detection throughput: {sentence_detection_throughput_chars_per_sec:.0} chars/sec ({sentence_detection_throughput_mb_per_sec:.2} MB/s)");
                println!("  Sentence detection time: {sentence_detection_time_sec:.2}s of {:.2}s total ({:.1}%)", 
                         processing_duration.as_secs_f64(), 
                         (sentence_detection_time_sec / processing_duration.as_secs_f64()) * 100.0);
            }
        }
    }
        
    info!("Overlapped pipeline completed: {} processed, {} skipped, {} failed, {} sentences detected", 
          processed_files, skipped_files, failed_files, total_sentences);
    
    Ok((total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration))
}

/// Process a single file - simplified version without cache dependencies
async fn process_single_file_restart(
    path: &std::path::Path,
    detector: &crate::sentence_detector::dialog_detector::SentenceDetectorDialog,
    _overwrite_all: bool,
    quiet: bool,
) -> Result<(u64, u64, u64, bool, FileStats)> {
    let start_time = std::time::Instant::now();
    
    // Process the file directly
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
    let content = std::str::from_utf8(&mmap)
        .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in file: {}", path.display()))?;

    // WHY: Measure sentence detection time separately from total processing time
    let sentence_detection_start = std::time::Instant::now();
    let sentences = detector.detect_sentences_borrowed(content)?;
    let sentence_detection_time = sentence_detection_start.elapsed();
    
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
        sentence_detection_time_ms: sentence_detection_time.as_millis() as u64,
        chars_per_sec,
        status: "success".to_string(),
        error: None,
    };
    
    //info!("Processed {}: {} sentences, {} bytes", path.display(), sentence_count, byte_count);
    let detection_ms = sentence_detection_time.as_millis();
    if !quiet {
        println!(
            "[Processed {}: {} sentences, {} bytes, detection {} ms",
            path.display(),
            sentence_count,
            byte_count,
            detection_ms
        );
    }
    Ok((sentence_count, byte_count, 1u64, false, file_stats))
}

// Cache-based discovery logic removed - using fresh parallel discovery only

// Functions moved to parallel_processing module

#[derive(Parser, Debug)]
#[command(name = "seams")]
#[command(about = "High-throughput sentence extractor for Project Gutenberg texts")]
#[command(long_about = "Seams is a high-performance CLI tool for extracting sentences from Project Gutenberg texts.\n\nIt recursively scans for *-0.txt files, detects sentence boundaries using a dialog-aware\nsentence detector, and outputs normalized sentences with span metadata to _seams.txt files.\n\nDesigned for narrative analysis pipelines with >50MB/s throughput.\n\nEXAMPLES:\n  seams ./gutenberg-mirror/            # Process all *-0.txt files\n  seams ./texts --overwrite-all        # Reprocess all files\n  seams ./texts --fail-fast            # Stop on first error\n  seams ./texts --no-progress          # Quiet mode for automation\n  seams ./texts --stats-out bench.json # Custom stats output")]
#[command(version)]
struct Args {
    /// Root directory to scan for *-0.txt files
    #[arg(value_name = "DIR", help = "Root directory to scan recursively for *-0.txt files")]
    root_dir: PathBuf,
    
    /// Overwrite even complete aux files
    #[arg(long, help = "Reprocess all files, even those with complete _seams.txt files")]
    overwrite_all: bool,
    
    
    /// Abort on first error
    #[arg(long, help = "Stop processing immediately on first I/O, UTF-8, or detection error")]
    fail_fast: bool,
    
    
    /// Suppress console progress bars
    #[arg(long, help = "Disable progress bars (useful for automation/CI)")]
    no_progress: bool,
    
    /// Quiet mode - minimal output for benchmarking
    #[arg(long, short = 'q', help = "Suppress all non-error output (implies --no-progress)")]
    quiet: bool,
    
    /// Stats output file path
    #[arg(long, default_value = "run_stats.json", value_name = "FILE", help = "Write performance statistics to JSON file")]
    stats_out: PathBuf,
    
    /// Clear the restart log before processing
    #[arg(long, help = "Clear the restart log and reprocess all files")]
    clear_restart_log: bool,
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
    
    // WHY: Load restart log to track completed files
    let mut restart_log = RestartLog::load(&args.root_dir).await;
    
    // WHY: Clear restart log if requested by user
    if args.clear_restart_log {
        let cleared_count = restart_log.completed_count();
        restart_log.clear();
        info!("Cleared {} entries from restart log", cleared_count);
        if !args.quiet {
            println!("Restart log cleared - will reprocess all files");
        }
    } else {
        // WHY: Verify restart log integrity and clean up stale entries
        let initial_count = restart_log.completed_count();
        if initial_count > 0 {
            info!("Loaded restart log with {} completed files", initial_count);
            let invalid_files = restart_log.verify_completed_files().await?;
            if !invalid_files.is_empty() {
                info!("Cleaned {} stale entries from restart log", invalid_files.len());
            }
        }
    }
    
    // WHY: Use overlapped discovery and processing pipeline for optimal performance
    let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats, processing_duration) = 
        process_with_overlapped_pipeline(&args, &mut restart_log).await?;
    
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
                    if !args.quiet {
                        println!("Stats written to {}", args.stats_out.display());
                    }
                }
                Err(e) => {
                    info!("Warning: Failed to write stats file: {e}");
                    if !args.quiet {
                        println!("Warning: Failed to write stats file: {e}");
                    }
                }
            }
        }
        Err(e) => {
            info!("Warning: Failed to serialize stats: {e}");
            if !args.quiet {
                println!("Warning: Failed to serialize stats: {e}");
            }
        }
    }
    
    Ok(())
}