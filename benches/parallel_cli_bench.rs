use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use seams::discovery;
use seams::incremental::generate_cache_path;
use seams::parallel_processing::ProcessingCache;
use std::path::PathBuf;
use std::time::Instant;
use num_cpus::get as num_cpus_get;

#[derive(Debug, Clone)]
struct ParallelResult {
    concurrency: usize,
    files_processed: u64,
    files_skipped: u64,
    files_failed: u64,
    total_chars: u64,
    total_sentences: u64,
    duration: std::time::Duration,
    throughput_chars_per_sec: f64,
    throughput_mb_per_sec: f64,
}

impl ParallelResult {
    fn new(
        concurrency: usize,
        files_processed: u64,
        files_skipped: u64,
        files_failed: u64,
        total_chars: u64,
        total_sentences: u64,
        duration: std::time::Duration,
    ) -> Self {
        let throughput_chars_per_sec = if duration.as_secs_f64() > 0.0 {
            total_chars as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        let throughput_mb_per_sec = throughput_chars_per_sec / 1_000_000.0;
        
        Self {
            concurrency,
            files_processed,
            files_skipped,
            files_failed,
            total_chars,
            total_sentences,
            duration,
            throughput_chars_per_sec,
            throughput_mb_per_sec,
        }
    }
}

fn get_sample_files_for_parallel_test() -> Vec<PathBuf> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{home}/gutenberg_texts")
            });
        let root_dir = PathBuf::from(mirror_dir);

        if !root_dir.exists() {
            eprintln!("Gutenberg mirror directory {root_dir:?} does not exist");
            return Vec::new();
        }

        let discovery_config = discovery::DiscoveryConfig::default();
        let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
            .await
            .unwrap_or_else(|_| Vec::new());

        // Take up to 10 files for parallel testing - enough to show scaling but not too slow
        let valid_files: Vec<PathBuf> = discovered_files
            .iter()
            .filter(|f| f.is_valid_utf8 && f.error.is_none())
            .take(10)
            .map(|f| f.path.clone())
            .collect();

        if valid_files.is_empty() {
            eprintln!("No valid files found for parallel CLI benchmark");
        } else {
            println!("Found {} valid files for parallel CLI processing", valid_files.len());
        }

        valid_files
    })
}

async fn process_files_parallel_benchmark(
    files: &[PathBuf],
    concurrency: usize,
) -> Result<ParallelResult, Box<dyn std::error::Error + Send + Sync>> {
    // WHY: Convert to references as expected by process_files_parallel
    let file_refs: Vec<&PathBuf> = files.iter().collect();
    
    // WHY: Create fresh cache for each benchmark run to ensure consistent test conditions
    let cache = ProcessingCache::default();
    
    // WHY: Use overwrite_all=true to force processing and measure actual performance
    let overwrite_all = true;
    let overwrite_use_cached_locations = false;
    let fail_fast = false;
    
    let start = Instant::now();
    
    // WHY: Use the actual process_files_parallel function from parallel_processing module
    let (total_sentences, total_bytes, processed_files, skipped_files, failed_files, _file_stats) = 
        seams::parallel_processing::process_files_parallel(
            &file_refs,
            cache,
            overwrite_all,
            overwrite_use_cached_locations,
            fail_fast,
            concurrency,
        ).await?;
    
    let duration = start.elapsed();
    
    Ok(ParallelResult::new(
        concurrency,
        processed_files,
        skipped_files,
        failed_files,
        total_bytes,
        total_sentences,
        duration,
    ))
}

fn validate_parallel_scaling(results: &[ParallelResult]) {
    println!("=== Parallel Scaling Validation ===");
    
    if results.len() < 2 {
        println!("Need at least 2 results to validate scaling");
        return;
    }
    
    // WHY: Sort by concurrency to ensure proper ordering
    let mut sorted_results = results.to_vec();
    sorted_results.sort_by_key(|r| r.concurrency);
    
    let baseline = &sorted_results[0];
    println!("Baseline ({}x): {:.0} chars/sec ({:.2} MB/s)", 
             baseline.concurrency, baseline.throughput_chars_per_sec, baseline.throughput_mb_per_sec);
    
    for result in &sorted_results[1..] {
        let speedup = result.throughput_chars_per_sec / baseline.throughput_chars_per_sec;
        let efficiency = speedup / (result.concurrency as f64 / baseline.concurrency as f64);
        
        println!("{}x concurrency: {:.0} chars/sec ({:.2} MB/s) | Speedup: {:.2}x | Efficiency: {:.1}%",
                 result.concurrency, result.throughput_chars_per_sec, result.throughput_mb_per_sec,
                 speedup, efficiency * 100.0);
    }
    
    // WHY: Validate that parallel processing actually improves performance
    let max_throughput = sorted_results.iter().map(|r| r.throughput_chars_per_sec).fold(0.0, f64::max);
    let improvement_ratio = max_throughput / baseline.throughput_chars_per_sec;
    
    println!("\n=== Performance Claims Validation ===");
    if improvement_ratio > 1.2 {
        println!("✅ PARALLEL CLAIM VALIDATED: {:.1}x throughput improvement observed", improvement_ratio);
    } else {
        println!("⚠️  PARALLEL CLAIM QUESTIONABLE: Only {:.1}x improvement (expected >1.2x)", improvement_ratio);
    }
    
    // WHY: Validate high-performance claim (>10 MB/s per PRD)
    let max_mb_per_sec = sorted_results.iter().map(|r| r.throughput_mb_per_sec).fold(0.0, f64::max);
    if max_mb_per_sec >= 10.0 {
        println!("✅ HIGH-PERFORMANCE CLAIM VALIDATED: {:.2} MB/s achieved (≥10 MB/s)", max_mb_per_sec);
    } else {
        println!("⚠️  HIGH-PERFORMANCE CLAIM NOT MET: {:.2} MB/s achieved (<10 MB/s)", max_mb_per_sec);
    }
}

fn bench_parallel_cli_processing(c: &mut Criterion) {
    // WHY: Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();
    
    let files = get_sample_files_for_parallel_test();
    
    if files.is_empty() {
        eprintln!("No files available for parallel CLI benchmark, skipping");
        return;
    }
    
    println!("=== Parallel CLI Benchmark Setup ===");
    println!("Processing {} files with different concurrency levels", files.len());
    
    // WHY: Calculate total characters for throughput measurement
    let rt = tokio::runtime::Runtime::new().unwrap();
    let total_chars: u64 = rt.block_on(async {
        let mut total = 0u64;
        for file in &files {
            if let Ok(metadata) = tokio::fs::metadata(file).await {
                total += metadata.len();
            }
        }
        total
    });
    
    println!("Total input size: {} chars ({:.2} MB)", total_chars, total_chars as f64 / 1_000_000.0);
    
    // WHY: Test different concurrency levels to demonstrate scaling
    let max_cores = num_cpus_get().min(16); // Cap at 16 to avoid excessive test times
    let concurrency_levels = [1, 2, 4, 8, max_cores].into_iter()
        .filter(|&c| c <= max_cores)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    
    println!("Testing concurrency levels: {:?}", concurrency_levels);
    
    let mut group = c.benchmark_group("parallel_cli_processing");
    group.throughput(Throughput::Bytes(total_chars));
    
    // WHY: Collect results for validation
    let mut validation_results = Vec::new();
    
    for &concurrency in &concurrency_levels {
        // WHY: Run a validation test first to collect results
        let validation_result = rt.block_on(async {
            process_files_parallel_benchmark(&files, concurrency).await
        });
        
        if let Ok(result) = validation_result {
            validation_results.push(result);
        }
        
        // WHY: Benchmark the actual function
        group.bench_with_input(
            BenchmarkId::new("concurrency", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let result = process_files_parallel_benchmark(black_box(&files), concurrency).await;
                    let _ = black_box(result);
                });
            },
        );
    }
    
    group.finish();
    
    // WHY: Validate performance claims after benchmarking
    validate_parallel_scaling(&validation_results);
    
    // WHY: Use all ParallelResult fields to prevent dead code warnings
    if !validation_results.is_empty() {
        let total_duration: std::time::Duration = validation_results.iter().map(|r| r.duration).sum();
        let total_files_processed: u64 = validation_results.iter().map(|r| r.files_processed).sum();
        let total_files_skipped: u64 = validation_results.iter().map(|r| r.files_skipped).sum();
        let total_files_failed: u64 = validation_results.iter().map(|r| r.files_failed).sum();
        let total_chars: u64 = validation_results.iter().map(|r| r.total_chars).sum();
        let total_sentences: u64 = validation_results.iter().map(|r| r.total_sentences).sum();
        println!("Summary: {} processed, {} skipped, {} failed files", total_files_processed, total_files_skipped, total_files_failed);
        println!("Total: {} chars, {} sentences in {:.3}s", total_chars, total_sentences, total_duration.as_secs_f64());
    }
}

fn bench_end_to_end_pipeline(c: &mut Criterion) {
    // WHY: Test complete pipeline including discovery, cache, and processing
    dotenvy::dotenv().ok();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{home}/gutenberg_texts")
        });
    let root_dir = PathBuf::from(mirror_dir);
    
    if !root_dir.exists() {
        eprintln!("Gutenberg mirror directory {root_dir:?} does not exist, skipping end-to-end test");
        return;
    }
    
    println!("=== End-to-End Pipeline Benchmark ===");
    println!("Testing complete discovery + cache + processing pipeline");
    println!("⚠️  Note: This benchmark shows sequential discovery → processing bottleneck");
    println!("📋 Task overlapped-discovery-processing_57 would address this by overlapping phases");
    
    let mut group = c.benchmark_group("end_to_end_pipeline");
    
    // WHY: Reduce sample count for end-to-end tests since they're slow
    // WHY: Sequential discovery → processing creates significant bottleneck
    group.sample_size(5);
    
    // WHY: Test with fresh cache (cold start)
    group.bench_function("cold_start", |b| {
        b.to_async(&rt).iter(|| async {
            let cache_path = generate_cache_path(&root_dir);
            let _ = tokio::fs::remove_file(&cache_path).await; // Clear cache
            
            let discovery_config = discovery::DiscoveryConfig::default();
            let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
                .await
                .unwrap_or_default();
            
            let valid_files: Vec<_> = discovered_files
                .iter()
                .filter(|f| f.is_valid_utf8 && f.error.is_none())
                .take(2) // Limit for reasonable test time
                .map(|f| &f.path)
                .collect();
            
            if !valid_files.is_empty() {
                let cache = ProcessingCache::default();
                let result = seams::parallel_processing::process_files_parallel(
                    &valid_files,
                    cache,
                    true, // overwrite_all
                    false,
                    false,
                    num_cpus_get().min(4),
                ).await;
                let _ = black_box(result);
            }
        });
    });
    
    // WHY: Test with warm cache (incremental run)
    group.bench_function("warm_cache", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = ProcessingCache::load(&root_dir).await;
            
            let discovery_config = discovery::DiscoveryConfig::default();
            let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
                .await
                .unwrap_or_default();
            
            let valid_files: Vec<_> = discovered_files
                .iter()
                .filter(|f| f.is_valid_utf8 && f.error.is_none())
                .take(2) // Limit for reasonable test time
                .map(|f| &f.path)
                .collect();
            
            if !valid_files.is_empty() {
                let result = seams::parallel_processing::process_files_parallel(
                    &valid_files,
                    cache,
                    false, // Don't overwrite - test incremental
                    false,
                    false,
                    num_cpus_get().min(4),
                ).await;
                let _ = black_box(result);
            }
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_parallel_cli_processing, bench_end_to_end_pipeline);
criterion_main!(benches);