use criterion::{black_box, criterion_group, criterion_main, Criterion};
use seams::{discovery, reader};
use std::path::PathBuf;
use std::time::Instant;

fn bench_gutenberg_discovery(c: &mut Criterion) {
    // WHY: quick discovery benchmark to catch obvious regressions in file finding
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gutenberg_discovery");
    group.sample_size(10); // WHY: just 10 samples for speed
    
    group.bench_function("file_discovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
                    .expect("GUTENBERG_MIRROR_DIR environment variable must be set to your local Project Gutenberg mirror");
                let root_dir = PathBuf::from(mirror_dir);
                
                if !root_dir.exists() {
                    return (0u64, 0u64);
                }
                
                let start = Instant::now();
                let discovery_config = discovery::DiscoveryConfig::default();
                let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
                    .await
                    .unwrap_or_else(|_| Vec::new());
                
                let discovery_ms = start.elapsed().as_millis() as u64;
                let file_count = discovered_files.len() as u64;
                
                black_box((file_count, discovery_ms))
            })
        });
    });
    group.finish();
}

fn bench_gutenberg_reading(c: &mut Criterion) {
    // WHY: quick reading benchmark using pre-discovered files to catch read regressions
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Pre-discover files once
    let discovered_files = rt.block_on(async {
        let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
            .expect("GUTENBERG_MIRROR_DIR environment variable must be set to your local Project Gutenberg mirror");
        let root_dir = PathBuf::from(mirror_dir);
        
        if !root_dir.exists() {
            return Vec::new();
        }
        
        let discovery_config = discovery::DiscoveryConfig::default();
        discovery::collect_discovered_files(&root_dir, discovery_config)
            .await
            .unwrap_or_else(|_| Vec::new())
    });
    
    if discovered_files.is_empty() {
        eprintln!("No files found for reading benchmark");
        return;
    }
    
    let valid_files: Vec<_> = discovered_files
        .iter()
        .filter(|f| f.is_valid_utf8 && f.error.is_none())
        .take(5) // WHY: just first 5 files for speed
        .collect();
    
    if valid_files.is_empty() {
        eprintln!("No valid files found for reading benchmark");
        return;
    }
    
    let mut group = c.benchmark_group("gutenberg_reading");
    group.sample_size(10); // WHY: just 10 samples for speed
    
    group.bench_function("file_reading", |b| {
        b.iter(|| {
            rt.block_on(async {
                let start = Instant::now();
                let reader_config = reader::ReaderConfig::default();
                let file_reader = reader::AsyncFileReader::new(reader_config);
                
                let valid_paths: Vec<_> = valid_files.iter().map(|f| &f.path).collect();
                let read_results = file_reader.read_files_batch(&valid_paths).await.unwrap();
                
                let mut total_lines = 0u64;
                let mut total_bytes = 0u64;
                
                for (_lines, stats) in read_results {
                    if stats.read_error.is_none() {
                        total_lines += stats.lines_read;
                        total_bytes += stats.bytes_read;
                    }
                }
                
                let read_ms = start.elapsed().as_millis() as u64;
                
                black_box((total_lines, total_bytes, read_ms))
            })
        });
    });
    group.finish();
}

fn reader_benchmarks(c: &mut Criterion) {
    bench_gutenberg_discovery(c);
    bench_gutenberg_reading(c);
}

criterion_group!(benches, reader_benchmarks);
criterion_main!(benches);