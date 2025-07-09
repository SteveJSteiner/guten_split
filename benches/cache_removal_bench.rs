use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use seams::discovery::{self, DiscoveryConfig};
use std::path::PathBuf;
use std::time::Instant;

/// Benchmark result for discovery performance comparison
#[derive(Debug, Clone)]
struct DiscoveryBenchmarkResult {
    strategy: String,
    files_discovered: usize,
    discovery_time: std::time::Duration,
    files_per_sec: f64,
}

impl DiscoveryBenchmarkResult {
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

async fn benchmark_fresh_parallel_discovery(root_dir: &PathBuf) -> Result<DiscoveryBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    let config = DiscoveryConfig::default();
    let start = Instant::now();
    
    let discovered_files = discovery::collect_discovered_files_parallel(root_dir, config).await?;
    
    let duration = start.elapsed();
    Ok(DiscoveryBenchmarkResult::new(
        "fresh_parallel_discovery".to_string(),
        discovered_files.len(),
        duration,
    ))
}

async fn benchmark_fresh_serial_discovery(root_dir: &PathBuf) -> Result<DiscoveryBenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
    let config = DiscoveryConfig::default();
    let start = Instant::now();
    
    let discovered_files = discovery::collect_discovered_files(root_dir, config).await?;
    
    let duration = start.elapsed();
    Ok(DiscoveryBenchmarkResult::new(
        "fresh_serial_discovery".to_string(),
        discovered_files.len(),
        duration,
    ))
}


fn validate_discovery_performance(results: &[DiscoveryBenchmarkResult]) {
    println!("=== Discovery Performance Validation ===");
    
    let fresh_parallel = results.iter().find(|r| r.strategy == "fresh_parallel_discovery");
    let fresh_serial = results.iter().find(|r| r.strategy == "fresh_serial_discovery");
    
    if let (Some(fresh_parallel), Some(fresh_serial)) = (fresh_parallel, fresh_serial) {
        println!("Parallel Discovery: {} files in {:.3}s ({:.1} files/sec)", 
                 fresh_parallel.files_discovered, fresh_parallel.discovery_time.as_secs_f64(), fresh_parallel.files_per_sec);
        
        println!("Serial Discovery: {} files in {:.3}s ({:.1} files/sec)", 
                 fresh_serial.files_discovered, fresh_serial.discovery_time.as_secs_f64(), fresh_serial.files_per_sec);
        
        let parallel_vs_serial_speedup = fresh_parallel.files_per_sec / fresh_serial.files_per_sec;
        
        println!("\n=== Performance Analysis ===");
        println!("Parallel vs Serial Discovery: {:.1}x speedup", parallel_vs_serial_speedup);
        
        // Validate performance claims
        if parallel_vs_serial_speedup > 1.5 {
            println!("✅ PARALLEL DISCOVERY BENEFICIAL: {:.1}x faster than serial", parallel_vs_serial_speedup);
        } else {
            println!("⚠️  PARALLEL DISCOVERY MARGINAL: Only {:.1}x faster than serial", parallel_vs_serial_speedup);
        }
        
        // Check discovery speed
        if fresh_parallel.discovery_time.as_secs_f64() < 1.0 {
            println!("✅ FAST DISCOVERY: Parallel discovery completes in <1s");
        } else if fresh_parallel.discovery_time.as_secs_f64() < 5.0 {
            println!("✅ MODERATE DISCOVERY: Parallel discovery completes in <5s");
        } else {
            println!("⚠️  SLOW DISCOVERY: Parallel discovery takes >5s");
        }
    }
}

fn bench_discovery_comparison(c: &mut Criterion) {
    dotenvy::dotenv().ok();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let root_dir = get_test_directory();
    
    if !root_dir.exists() {
        eprintln!("Test directory {root_dir:?} does not exist, skipping discovery benchmark");
        return;
    }
    
    println!("=== Discovery Performance Benchmark ===");
    println!("Testing parallel vs serial discovery performance");
    println!("Root directory: {}", root_dir.display());
    
    let mut group = c.benchmark_group("discovery_comparison");
    
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
    
    // Calculate total bytes for throughput
    let total_files = validation_results.first().map(|r| r.files_discovered).unwrap_or(0);
    group.throughput(Throughput::Elements(total_files as u64));
    
    // Benchmark fresh parallel discovery
    group.bench_function("parallel_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let result = benchmark_fresh_parallel_discovery(black_box(&root_dir)).await;
            let _ = black_box(result);
        });
    });
    
    // Benchmark fresh serial discovery
    group.bench_function("serial_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let result = benchmark_fresh_serial_discovery(black_box(&root_dir)).await;
            let _ = black_box(result);
        });
    });
    
    group.finish();
    
    // Validate discovery performance
    validate_discovery_performance(&validation_results);
    
    // Use all fields to prevent dead code warnings
    if !validation_results.is_empty() {
        let total_discovery_time: std::time::Duration = validation_results.iter().map(|r| r.discovery_time).sum();
        let total_files_discovered: usize = validation_results.iter().map(|r| r.files_discovered).sum();
        println!("Summary: {} files discovered in {:.3}s across all strategies", 
                 total_files_discovered, total_discovery_time.as_secs_f64());
    }
}

criterion_group!(benches, bench_discovery_comparison);
criterion_main!(benches);