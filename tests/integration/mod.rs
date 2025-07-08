// Integration test utilities and common code
// WHY: Centralized utilities avoid duplication across integration tests

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use seams::incremental::{generate_aux_file_path, aux_file_exists, read_aux_file, create_complete_aux_file, generate_cache_path, cache_exists, read_cache};

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
    
    /// Check if aux file exists for given source file
    /// WHY: Test helper for core incremental processing functionality (F-7, F-9)
    pub fn aux_file_exists<P: AsRef<Path>>(&self, source_path: P) -> bool {
        aux_file_exists(source_path)
    }
    
    /// Read aux file content for given source file
    /// WHY: Test helper for core incremental processing functionality (F-7, F-9)
    pub fn read_aux_file<P: AsRef<Path>>(&self, source_path: P) -> Result<String, std::io::Error> {
        read_aux_file(source_path)
    }
    
    /// Create a partial aux file (without trailing newline) for testing
    #[allow(dead_code)]
    pub fn create_partial_aux_file<P: AsRef<Path>>(&self, source_path: P, content: &str) -> PathBuf {
        let aux_path = self.generate_aux_file_path(source_path);
        // Write content without trailing newline to simulate partial file
        fs::write(&aux_path, content.trim_end_matches('\n')).expect("Failed to write partial aux file");
        aux_path
    }
    
    /// Create a complete aux file (with trailing newline) for testing
    /// WHY: Test helper for core incremental processing functionality (F-7, F-9)
    pub fn create_complete_aux_file<P: AsRef<Path>>(&self, source_path: P, content: &str) -> PathBuf {
        create_complete_aux_file(source_path, content).expect("Failed to write complete aux file")
    }
    
    /// Generate cache file path for the test fixture root
    pub fn cache_path(&self) -> PathBuf {
        generate_cache_path(&self.root_path)
    }
    
    /// Check if cache file exists
    pub fn cache_exists(&self) -> bool {
        cache_exists(&self.root_path)
    }
    
    /// Read cache file content
    pub fn read_cache(&self) -> Result<String, std::io::Error> {
        read_cache(&self.root_path)
    }
    
    /// Create a cache file with specific content for testing
    #[allow(dead_code)]
    pub fn create_cache(&self, content: &str) -> PathBuf {
        let cache_path = self.cache_path();
        fs::write(&cache_path, content).expect("Failed to write cache file");
        cache_path
    }
}

