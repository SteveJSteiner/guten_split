// Integration test utilities and common code
// WHY: Centralized utilities avoid duplication across integration tests

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;

/// Test fixture helper for creating temporary directories with Gutenberg-style files
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let root_path = temp_dir.path().to_path_buf();
        
        Self {
            temp_dir,
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
        let source_path = source_path.as_ref();
        let mut aux_path = source_path.to_path_buf();
        let file_stem = aux_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        aux_path.set_file_name(format!("{file_stem}_rs_sft_sentences.txt"));
        aux_path
    }
    
    /// Check if aux file exists for given source file
    pub fn aux_file_exists<P: AsRef<Path>>(&self, source_path: P) -> bool {
        let aux_path = self.generate_aux_file_path(source_path);
        aux_path.exists()
    }
    
    /// Read aux file content for given source file
    pub fn read_aux_file<P: AsRef<Path>>(&self, source_path: P) -> Result<String, std::io::Error> {
        let aux_path = self.generate_aux_file_path(source_path);
        fs::read_to_string(aux_path)
    }
    
    /// Create a partial aux file (without trailing newline) for testing
    pub fn create_partial_aux_file<P: AsRef<Path>>(&self, source_path: P, content: &str) -> PathBuf {
        let aux_path = self.generate_aux_file_path(source_path);
        // Write content without trailing newline to simulate partial file
        fs::write(&aux_path, content.trim_end_matches('\n')).expect("Failed to write partial aux file");
        aux_path
    }
    
    /// Create a complete aux file (with trailing newline) for testing
    pub fn create_complete_aux_file<P: AsRef<Path>>(&self, source_path: P, content: &str) -> PathBuf {
        let aux_path = self.generate_aux_file_path(source_path);
        // Ensure content ends with newline
        let content_with_newline = if content.ends_with('\n') { 
            content.to_string() 
        } else { 
            format!("{}\n", content) 
        };
        fs::write(&aux_path, content_with_newline).expect("Failed to write complete aux file");
        aux_path
    }
    
    /// Generate cache file path for the test fixture root
    pub fn cache_path(&self) -> PathBuf {
        self.root_path.join(".rs_sft_sentences_cache.json")
    }
    
    /// Check if cache file exists
    pub fn cache_exists(&self) -> bool {
        self.cache_path().exists()
    }
    
    /// Read cache file content
    pub fn read_cache(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(self.cache_path())
    }
    
    /// Create a cache file with specific content for testing
    pub fn create_cache(&self, content: &str) -> PathBuf {
        let cache_path = self.cache_path();
        fs::write(&cache_path, content).expect("Failed to write cache file");
        cache_path
    }
}

/// Compare two strings line by line, providing detailed diff on mismatch
pub fn assert_golden_file(actual: &str, expected: &str, context: &str) {
    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();
    
    if actual_lines.len() != expected_lines.len() {
        panic!(
            "{}: Line count mismatch. Expected {} lines, got {} lines",
            context, expected_lines.len(), actual_lines.len()
        );
    }
    
    for (i, (actual_line, expected_line)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
        if actual_line != expected_line {
            panic!(
                "{}: Line {} mismatch\nExpected: {}\nActual:   {}",
                context, i + 1, expected_line, actual_line
            );
        }
    }
}