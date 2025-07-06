use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::discovery;
use rs_sft_sentences::SentenceDetectorDialog;
use std::path::PathBuf;
use std::fs::File;
use memmap2::{MmapOptions, Mmap};
use std::time::Instant;

#[derive(Debug, Clone)]
struct FileResult {
    path: PathBuf,
    chars: usize,
    sentences: usize,
    duration: std::time::Duration,
    throughput: f64,
}

impl FileResult {
    fn new(path: PathBuf, chars: usize, sentences: usize, duration: std::time::Duration) -> Self {
        let throughput = if duration.as_secs_f64() > 0.0 {
            chars as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        Self {
            path,
            chars,
            sentences,
            duration,
            throughput,
        }
    }
}

fn get_sample_files() -> Vec<PathBuf> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/gutenberg_texts", home)
            });
        let root_dir = PathBuf::from(mirror_dir);

        if !root_dir.exists() {
            eprintln!("Gutenberg mirror directory {:?} does not exist", root_dir);
            return Vec::new();
        }

        let discovery_config = discovery::DiscoveryConfig::default();
        let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
            .await
            .unwrap_or_else(|_| Vec::new());

        // Take only 3 files for initial testing to avoid hangs
        let valid_files: Vec<PathBuf> = discovered_files
            .iter()
            .filter(|f| f.is_valid_utf8 && f.error.is_none())
            .take(3)
            .map(|f| f.path.clone())
            .collect();

        if valid_files.is_empty() {
            eprintln!("No valid files found for file-by-file benchmark");
        } else {
            println!("Found {} valid files for file-by-file processing", valid_files.len());
        }

        valid_files
    })
}

fn process_files_borrowed_mmap(files: &[PathBuf]) -> Result<Vec<FileResult>, Box<dyn std::error::Error>> {
    let detector = SentenceDetectorDialog::new()?;
    let mut results = Vec::new();
    
    for file_path in files {
        let file_handle = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file_handle)? };
        let content = std::str::from_utf8(&mmap)?;
        
        let start = Instant::now();
        let sentences = detector.detect_sentences_borrowed(content)?;
        let duration = start.elapsed();
        
        let result = FileResult::new(
            file_path.clone(),
            content.chars().count(),
            sentences.len(),
            duration,
        );
        
        results.push(result);
    }
    
    Ok(results)
}

fn process_files_owned_read(files: &[PathBuf]) -> Result<Vec<FileResult>, Box<dyn std::error::Error>> {
    let detector = SentenceDetectorDialog::new()?;
    let mut results = Vec::new();
    
    for file_path in files {
        let content = std::fs::read_to_string(file_path)?;
        
        let start = Instant::now();
        let sentences = detector.detect_sentences_owned(&content)?;
        let duration = start.elapsed();
        
        let result = FileResult::new(
            file_path.clone(),
            content.chars().count(),
            sentences.len(),
            duration,
        );
        
        results.push(result);
    }
    
    Ok(results)
}

fn calculate_stats(results: &[FileResult]) -> (f64, f64, f64, f64) {
    if results.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    
    let throughputs: Vec<f64> = results.iter().map(|r| r.throughput).collect();
    let min_throughput = throughputs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_throughput = throughputs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
    let total_chars = results.iter().map(|r| r.chars).sum::<usize>();
    
    (min_throughput, max_throughput, avg_throughput, total_chars as f64)
}

fn bench_file_by_file_processing(c: &mut Criterion) {
    let files = get_sample_files();
    
    if files.is_empty() {
        eprintln!("No files available for file-by-file benchmark, skipping");
        return;
    }
    
    println!("=== File-by-File Benchmark Setup ===");
    println!("Processing {} files individually", files.len());
    
    // Test both approaches to validate work equivalence
    println!("Validating work equivalence...");
    
    println!("Testing Dialog borrowed API (mmap)...");
    let borrowed_results = process_files_borrowed_mmap(&files).unwrap_or_else(|e| {
        eprintln!("Borrowed API failed: {}", e);
        Vec::new()
    });
    println!("Dialog borrowed API completed");
    
    println!("Testing Dialog owned API (read)...");
    let owned_results = process_files_owned_read(&files).unwrap_or_else(|e| {
        eprintln!("Owned API failed: {}", e);
        Vec::new()
    });
    println!("Dialog owned API completed");
    
    // Calculate and display statistics
    let (borrowed_min, borrowed_max, borrowed_avg, borrowed_total) = calculate_stats(&borrowed_results);
    let (owned_min, owned_max, owned_avg, owned_total) = calculate_stats(&owned_results);
    
    println!("=== Performance Results ===");
    println!("Dialog Borrowed (mmap): min={:.0} max={:.0} avg={:.0} chars/sec ({:.0} total chars)", 
             borrowed_min, borrowed_max, borrowed_avg, borrowed_total);
    println!("Dialog Owned (read): min={:.0} max={:.0} avg={:.0} chars/sec ({:.0} total chars)", 
             owned_min, owned_max, owned_avg, owned_total);
    
    // Check sentence count consistency
    let borrowed_sentences: usize = borrowed_results.iter().map(|r| r.sentences).sum();
    let owned_sentences: usize = owned_results.iter().map(|r| r.sentences).sum();
    
    println!("=== Sentence Count Validation ===");
    println!("Dialog Borrowed: {} sentences", borrowed_sentences);
    println!("Dialog Owned: {} sentences", owned_sentences);
    
    let total_chars = borrowed_total as u64;
    let mut group = c.benchmark_group("file_by_file_processing");
    group.throughput(Throughput::Elements(total_chars));
    
    // Benchmark borrowed API (mmap-based)
    group.bench_function("dialog_borrowed_mmap", |b| {
        b.iter(|| {
            let results = process_files_borrowed_mmap(black_box(&files)).unwrap();
            black_box(results);
        })
    });
    
    // Benchmark owned API (read-based)
    group.bench_function("dialog_owned_read", |b| {
        b.iter(|| {
            let results = process_files_owned_read(black_box(&files)).unwrap();
            black_box(results);
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_file_by_file_processing);
criterion_main!(benches);