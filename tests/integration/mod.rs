// Integration test utilities and common code
// WHY: Centralized utilities avoid duplication across integration tests

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use seams::incremental::{generate_aux_file_path, generate_cache_path};


/// Test fixture helper for creating temporary directories with Gutenberg-style files
pub struct TestFixture {
    _temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let root_path = temp_dir.path().to_path_buf();
        
        Self {
            _temp_dir: temp_dir,
            root_path,
        }
    }
    
    /// Create a Gutenberg-style text file with given content
    pub fn create_gutenberg_file<P: AsRef<Path>>(&self, relative_path: P, content: &str) -> PathBuf {
        let file_path = self.root_path.join(relative_path);
        
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }
    
    /// Generate auxiliary file path matching main implementation
    pub fn generate_aux_file_path<P: AsRef<Path>>(&self, source_path: P) -> PathBuf {
        generate_aux_file_path(source_path.as_ref())
    }
    
    
    /// Create a partial aux file (without trailing newline) for testing
    #[allow(dead_code)]
    pub fn create_partial_aux_file<P: AsRef<Path>>(&self, source_path: P, content: &str) -> PathBuf {
        let aux_path = self.generate_aux_file_path(source_path);
        // Write content without trailing newline to simulate partial file
        fs::write(&aux_path, content.trim_end_matches('\n')).expect("Failed to write partial aux file");
        aux_path
    }
    
    /// Generate cache file path for the test fixture root
    pub fn cache_path(&self) -> PathBuf {
        generate_cache_path(&self.root_path)
    }
    
    /// Create a cache file with specific content for testing
    #[allow(dead_code)]
    pub fn create_cache(&self, content: &str) -> PathBuf {
        let cache_path = self.cache_path();
        fs::write(&cache_path, content).expect("Failed to write cache file");
        cache_path
    }
}

