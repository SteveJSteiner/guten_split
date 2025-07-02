use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

mod discovery;
mod reader;

#[derive(Parser, Debug)]
#[command(name = "rs-sft-sentences")]
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
    
    info!("Starting rs-sft-sentences");
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
    
    println!("rs-sft-sentences v{} - File discovery complete", env!("CARGO_PKG_VERSION"));
    println!("Found {} files matching pattern *-0.txt", discovered_files.len());
    println!("Valid files: {}, Files with issues: {}", valid_files.len(), invalid_files.len());
    
    // Process valid files with async reader
    if !valid_files.is_empty() {
        info!("Starting async file reading for {} valid files", valid_files.len());
        
        let reader_config = reader::ReaderConfig {
            fail_fast: args.fail_fast,
            buffer_size: 8192,
        };
        let file_reader = reader::AsyncFileReader::new(reader_config);
        
        // WHY: process files sequentially to demonstrate async reading without overwhelming memory
        let valid_paths: Vec<_> = valid_files.iter().map(|f| &f.path).collect();
        let read_results = file_reader.read_files_batch(&valid_paths).await?;
        
        let mut total_lines = 0u64;
        let mut total_bytes = 0u64;
        let mut successful_reads = 0;
        let mut failed_reads = 0;
        
        for (_lines, stats) in read_results {
            total_lines += stats.lines_read;
            total_bytes += stats.bytes_read;
            
            if stats.read_error.is_some() {
                failed_reads += 1;
            } else {
                successful_reads += 1;
            }
            
            info!("Read {}: {} lines, {} bytes", stats.file_path, stats.lines_read, stats.bytes_read);
            
            if let Some(ref error) = stats.read_error {
                info!("Read error for {}: {}", stats.file_path, error);
            }
        }
        
        println!("File reading complete:");
        println!("  Successfully read: {successful_reads} files");
        if failed_reads > 0 {
            println!("  Failed to read: {failed_reads} files");
        }
        println!("  Total lines processed: {total_lines}");
        println!("  Total bytes processed: {total_bytes}");
        
        info!("Async file reading completed: {} successful, {} failed", successful_reads, failed_reads);
    }
    
    Ok(())
}