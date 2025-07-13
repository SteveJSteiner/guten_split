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
    
    aux_path.set_file_name(format!("{file_stem}_seams2.txt"));
    aux_path
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
        format!("{content}\n") 
    };
    fs::write(&aux_path, content_with_newline)?;
    Ok(aux_path)
}

