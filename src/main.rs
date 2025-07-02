use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

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
    println!("rs-sft-sentences v{} - Project setup complete", env!("CARGO_PKG_VERSION"));
    
    Ok(())
}