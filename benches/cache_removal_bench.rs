use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use seams::discovery::{self, DiscoveryConfig};
use seams::parallel_processing::ProcessingCache;
use std::path::PathBuf;
use std::time::Instant;

/// Benchmark result for cache performance comparison
#[derive(Debug, Clone)]
struct CacheBenchmarkResult {
    strategy: String,
    files_discovered: usize,
    discovery_time: std::time::Duration,
    files_per_sec: f64,
}

impl CacheBenchmarkResult {
    fn new(strategy: String, files_discovered: usize, discovery_time: std::time::Duration) -> Self {
        let files_per_sec = if discovery_time.as_secs_f64() > 0.0 {
            files_discovered as f64 / discovery_time.as_secs_f64()
        } else {
            0.0
        };
        
        Self {
            strategy,
            files_discovered,
            discovery_time,
            files_per_sec,
        }
    }
}

fn get_test_directory() -> PathBuf {
    let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{home}/gutenberg_texts")
        });
    PathBuf::from(mirror_dir)
}

async fn benchmark_fresh_parallel_discovery(root_dir: &PathBuf) -> Result<CacheBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    let config = DiscoveryConfig::default();
    let start = Instant::now();
    
    let discovered_files = discovery::collect_discovered_files_parallel(root_dir, config).await?;
    
    let duration = start.elapsed();
    Ok(CacheBenchmarkResult::new(
        "fresh_parallel_discovery".to_string(),
        discovered_files.len(),
        duration,
    ))
}

async fn benchmark_fresh_serial_discovery(root_dir: &PathBuf) -> Result<CacheBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    let config = DiscoveryConfig::default();
    let start = Instant::now();
    
    let discovered_files = discovery::collect_discovered_files(root_dir, config).await?;
    
    let duration = start.elapsed();
    Ok(CacheBenchmarkResult::new(
        "fresh_serial_discovery".to_string(),
        discovered_files.len(),
        duration,
    ))
}

async fn benchmark_cached_discovery(root_dir: &PathBuf) -> Result<CacheBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    let cache = ProcessingCache::load(root_dir).await;
    let start = Instant::now();
    
    let cached_files = cache.get_cached_discovered_files(root_dir).await?;
    
    let duration = start.elapsed();
    
    let files_count = cached_files.map(|f| f.len()).unwrap_or(0);
    Ok(CacheBenchmarkResult::new(
        "cached_discovery".to_string(),
        files_count,
        duration,
    ))
}

async fn populate_cache(root_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cache = ProcessingCache::load(root_dir).await;
    
    // Force discovery to populate cache
    let config = DiscoveryConfig::default();
    let discovered_files = discovery::collect_discovered_files_parallel(root_dir, config).await?;
    
    // Update cache with discovery results
    cache.update_discovery_cache(root_dir, &discovered_files).await?;
    
    // Save cache
    let cache_path = seams::incremental::generate_cache_path(root_dir);
    cache.save(&cache_path).await?;
    
    Ok(())
}

fn validate_cache_performance(results: &[CacheBenchmarkResult]) {
    println!("=== Cache Performance Validation ===");
    
    let fresh_parallel = results.iter().find(|r| r.strategy == "fresh_parallel_discovery");
    let fresh_serial = results.iter().find(|r| r.strategy == "fresh_serial_discovery");
    let cached = results.iter().find(|r| r.strategy == "cached_discovery");
    
    if let (Some(fresh_parallel), Some(fresh_serial), Some(cached)) = (fresh_parallel, fresh_serial, cached) {
        println!("Fresh Parallel Discovery: {} files in {:.3}s ({:.1} files/sec)", 
                 fresh_parallel.files_discovered, fresh_parallel.discovery_time.as_secs_f64(), fresh_parallel.files_per_sec);
        
        println!("Fresh Serial Discovery: {} files in {:.3}s ({:.1} files/sec)", 
                 fresh_serial.files_discovered, fresh_serial.discovery_time.as_secs_f64(), fresh_serial.files_per_sec);
        
        println!("Cached Discovery: {} files in {:.3}s ({:.1} files/sec)", 
                 cached.files_discovered, cached.discovery_time.as_secs_f64(), cached.files_per_sec);
        
        let parallel_vs_serial_speedup = fresh_parallel.files_per_sec / fresh_serial.files_per_sec;
        let cache_vs_parallel_speedup = cached.files_per_sec / fresh_parallel.files_per_sec;
        
        println!("\n=== Performance Analysis ===");
        println!("Parallel vs Serial Discovery: {:.1}x speedup", parallel_vs_serial_speedup);
        println!("Cached vs Parallel Discovery: {:.1}x speedup", cache_vs_parallel_speedup);
        
        // Validate performance claims
        if parallel_vs_serial_speedup > 1.5 {
            println!("✅ PARALLEL DISCOVERY BENEFICIAL: {:.1}x faster than serial", parallel_vs_serial_speedup);
        } else {
            println!("⚠️  PARALLEL DISCOVERY MARGINAL: Only {:.1}x faster than serial", parallel_vs_serial_speedup);
        }
        
        if cache_vs_parallel_speedup > 10.0 {
            println!("✅ CACHE HIGHLY BENEFICIAL: {:.1}x faster than parallel discovery", cache_vs_parallel_speedup);
        } else if cache_vs_parallel_speedup > 2.0 {
            println!("✅ CACHE BENEFICIAL: {:.1}x faster than parallel discovery", cache_vs_parallel_speedup);
        } else {
            println!("⚠️  CACHE MARGINAL BENEFIT: Only {:.1}x faster than parallel discovery", cache_vs_parallel_speedup);
            println!("💡 RECOMMENDATION: Consider removing discovery cache complexity");
        }
        
        // Check if parallel discovery is fast enough to make cache unnecessary
        if fresh_parallel.discovery_time.as_secs_f64() < 1.0 {
            println!("💡 FAST DISCOVERY: Parallel discovery completes in <1s, cache may be unnecessary");
        } else if fresh_parallel.discovery_time.as_secs_f64() < 5.0 {
            println!("💡 MODERATE DISCOVERY: Parallel discovery completes in <5s, cache provides some benefit");
        } else {
            println!("💡 SLOW DISCOVERY: Parallel discovery takes >5s, cache provides significant benefit");
        }
    }
}

fn bench_cache_comparison(c: &mut Criterion) {
    dotenvy::dotenv().ok();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let root_dir = get_test_directory();
    
    if !root_dir.exists() {
        eprintln!("Test directory {root_dir:?} does not exist, skipping cache benchmark");
        return;
    }
    
    println!("=== Cache Performance Benchmark ===");
    println!("Testing discovery cache vs fresh parallel traversal");
    println!("Root directory: {}", root_dir.display());
    
    // Pre-populate cache for cached discovery test
    rt.block_on(async {
        if let Err(e) = populate_cache(&root_dir).await {
            eprintln!("Warning: Failed to populate cache: {e}");
        }
    });
    
    let mut group = c.benchmark_group("cache_comparison");
    
    // Collect results for validation
    let mut validation_results = Vec::new();
    
    // Benchmark fresh parallel discovery
    let fresh_parallel_result = rt.block_on(async {
        benchmark_fresh_parallel_discovery(&root_dir).await
    });
    if let Ok(result) = fresh_parallel_result {
        validation_results.push(result);
    }
    
    // Benchmark fresh serial discovery
    let fresh_serial_result = rt.block_on(async {
        benchmark_fresh_serial_discovery(&root_dir).await
    });
    if let Ok(result) = fresh_serial_result {
        validation_results.push(result);
    }
    
    // Benchmark cached discovery
    let cached_result = rt.block_on(async {
        benchmark_cached_discovery(&root_dir).await
    });
    if let Ok(result) = cached_result {
        validation_results.push(result);
    }
    
    // Calculate total bytes for throughput
    let total_files = validation_results.first().map(|r| r.files_discovered).unwrap_or(0);
    group.throughput(Throughput::Elements(total_files as u64));
    
    // Benchmark fresh parallel discovery
    group.bench_function("fresh_parallel_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let result = benchmark_fresh_parallel_discovery(black_box(&root_dir)).await;
            let _ = black_box(result);
        });
    });
    
    // Benchmark fresh serial discovery
    group.bench_function("fresh_serial_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let result = benchmark_fresh_serial_discovery(black_box(&root_dir)).await;
            let _ = black_box(result);
        });
    });
    
    // Benchmark cached discovery
    group.bench_function("cached_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let result = benchmark_cached_discovery(black_box(&root_dir)).await;
            let _ = black_box(result);
        });
    });
    
    group.finish();
    
    // Validate cache performance
    validate_cache_performance(&validation_results);
    
    // Use all fields to prevent dead code warnings
    if !validation_results.is_empty() {
        let total_discovery_time: std::time::Duration = validation_results.iter().map(|r| r.discovery_time).sum();
        let total_files_discovered: usize = validation_results.iter().map(|r| r.files_discovered).sum();
        println!("Summary: {} files discovered in {:.3}s across all strategies", 
                 total_files_discovered, total_discovery_time.as_secs_f64());
    }
}

criterion_group!(benches, bench_cache_comparison);
criterion_main!(benches);