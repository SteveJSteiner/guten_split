// WHY: Public utilities for incremental processing functionality (F-7, F-9)
// Provides helpers for aux file management and cache operations used by CLI and tests

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

/// Generate auxiliary file path from source file path
/// WHY: Follows PRD F-7 specification for aux file naming
pub fn generate_aux_file_path(source_path: &Path) -> PathBuf {
    let mut aux_path = source_path.to_path_buf();
    let file_stem = aux_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    aux_path.set_file_name(format!("{file_stem}_seams.txt"));
    aux_path
}

/// Check if auxiliary file exists for given source file
/// WHY: Core utility for incremental processing - check if work is already done
pub fn aux_file_exists<P: AsRef<Path>>(source_path: P) -> bool {
    let aux_path = generate_aux_file_path(source_path.as_ref());
    aux_path.exists()
}

/// Read auxiliary file content for given source file
/// WHY: Core utility for accessing previously processed sentence data
/// 
/// # Example
/// ```no_run
/// use seams::incremental::read_aux_file;
/// let content = read_aux_file("path/to/book-0.txt").expect("Failed to read aux file");
/// ```
pub fn read_aux_file<P: AsRef<Path>>(source_path: P) -> Result<String, io::Error> {
    let aux_path = generate_aux_file_path(source_path.as_ref());
    fs::read_to_string(aux_path)
}

/// Create a complete auxiliary file (with trailing newline) for given source
/// WHY: Core utility for writing properly formatted aux files per F-7 spec
/// 
/// # Example
/// ```no_run
/// use seams::incremental::create_complete_aux_file;
/// let content = "0\tSample sentence.\t(1,1,1,16)\n";
/// create_complete_aux_file("path/to/book-0.txt", content).expect("Failed to create aux file");
/// ```
pub fn create_complete_aux_file<P: AsRef<Path>>(source_path: P, content: &str) -> Result<PathBuf, io::Error> {
    let aux_path = generate_aux_file_path(source_path.as_ref());
    // Ensure content ends with newline per F-7 specification
    let content_with_newline = if content.ends_with('\n') { 
        content.to_string() 
    } else { 
        format!("{}\n", content) 
    };
    fs::write(&aux_path, content_with_newline)?;
    Ok(aux_path)
}

/// Generate cache file path for given root directory
/// WHY: Standard cache location for incremental processing state
pub fn generate_cache_path<P: AsRef<Path>>(root_dir: P) -> PathBuf {
    root_dir.as_ref().join(".seams_cache.json")
}

/// Check if cache file exists in given directory
/// WHY: Core utility for incremental processing - check if cache is available
pub fn cache_exists<P: AsRef<Path>>(root_dir: P) -> bool {
    generate_cache_path(root_dir).exists()
}

/// Read cache file content from given directory
/// WHY: Core utility for loading incremental processing state
/// 
/// # Example
/// ```no_run
/// use seams::incremental::read_cache;
/// let cache_content = read_cache("/path/to/gutenberg/root").expect("Failed to read cache");
/// ```
pub fn read_cache<P: AsRef<Path>>(root_dir: P) -> Result<String, io::Error> {
    let cache_path = generate_cache_path(root_dir);
    fs::read_to_string(cache_path)
}

/// Read cache file content from given directory (async version)
/// WHY: Async utility for loading incremental processing state without blocking
pub async fn read_cache_async<P: AsRef<Path>>(root_dir: P) -> Result<String, io::Error> {
    let cache_path = generate_cache_path(root_dir);
    tokio::fs::read_to_string(cache_path).await
}