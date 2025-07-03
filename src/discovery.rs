use anyhow::Result;
use futures::stream::{Stream, StreamExt};
use glob::glob;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Configuration for file discovery behavior
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DiscoveryConfig {
    /// Whether to fail fast on first error or continue processing
    pub fail_fast: bool,
}


/// Result of file discovery validation
#[derive(Debug)]
pub struct FileValidation {
    pub path: PathBuf,
    pub is_valid_utf8: bool,
    pub error: Option<String>,
}

/// Discovers all files matching the pattern `**/*-0.txt` recursively under the given root directory.
/// Returns an async stream of validated file paths.
///
/// # Arguments
/// * `root_dir` - Root directory to search recursively
/// * `config` - Discovery configuration (fail_fast behavior)
///
/// # Returns
/// Stream of `FileValidation` results containing file paths and validation status
pub fn discover_files(
    root_dir: impl AsRef<Path>,
    config: DiscoveryConfig,
) -> impl Stream<Item = Result<FileValidation>> {
    let root_path = root_dir.as_ref().to_path_buf();
    
    // WHY: using async_stream would be cleaner but adds dependency; 
    // futures::stream provides sufficient async iteration capabilities
    futures::stream::unfold(
        DiscoveryState::new(root_path, config),
        |mut state| async move {
            state.next_file().await.map(|result| (result, state))
        }
    )
}

/// Internal state for file discovery iteration
struct DiscoveryState {
    root_dir: PathBuf,
    config: DiscoveryConfig,
    glob_iter: Option<glob::Paths>,
}

impl DiscoveryState {
    fn new(root_dir: PathBuf, config: DiscoveryConfig) -> Self {
        Self {
            root_dir,
            config,
            glob_iter: None,
        }
    }

    async fn next_file(&mut self) -> Option<Result<FileValidation>> {
        // Initialize glob iterator on first call
        if self.glob_iter.is_none() {
            let pattern = format!("{}/**/*-0.txt", self.root_dir.display());
            debug!("Starting file discovery with pattern: {}", pattern);
            
            match glob(&pattern) {
                Ok(paths) => {
                    self.glob_iter = Some(paths);
                    info!("File discovery initialized for root: {}", self.root_dir.display());
                }
                Err(e) => {
                    return Some(Err(anyhow::anyhow!("Failed to create glob pattern: {}", e)));
                }
            }
        }

        // Process next file from glob iterator
        if let Some(ref mut glob_iter) = self.glob_iter {
            match glob_iter.next() {
                Some(glob_result) => {
                    match glob_result {
                        Ok(path) => {
                            debug!("Found file: {}", path.display());
                            Some(self.validate_file(path).await)
                        }
                        Err(e) => {
                            let error_msg = format!("Glob iteration error: {e}");
                            warn!("{}", error_msg);
                            
                            if self.config.fail_fast {
                                Some(Err(anyhow::anyhow!(error_msg)))
                            } else {
                                // Continue to next file on non-fatal glob errors
                                Box::pin(self.next_file()).await
                            }
                        }
                    }
                }
                None => {
                    info!("File discovery completed");
                    None
                }
            }
        } else {
            None
        }
    }

    async fn validate_file(&self, path: PathBuf) -> Result<FileValidation> {
        debug!("Validating file: {}", path.display());
        
        // Check if file is accessible
        match fs::metadata(&path).await {
            Ok(metadata) => {
                if !metadata.is_file() {
                    let error = format!("Path is not a file: {}", path.display());
                    warn!("{}", error);
                    return Ok(FileValidation {
                        path,
                        is_valid_utf8: false,
                        error: Some(error),
                    });
                }
            }
            Err(e) => {
                let error = format!("Cannot access file {}: {}", path.display(), e);
                warn!("{}", error);
                
                if self.config.fail_fast {
                    return Err(anyhow::anyhow!(error));
                } else {
                    return Ok(FileValidation {
                        path,
                        is_valid_utf8: false,
                        error: Some(error),
                    });
                }
            }
        }

        // Validate UTF-8 encoding by reading a sample
        let is_valid_utf8 = match self.check_utf8_encoding(&path).await {
            Ok(valid) => valid,
            Err(e) => {
                let error = format!("UTF-8 validation failed for {}: {}", path.display(), e);
                warn!("{}", error);
                
                if self.config.fail_fast {
                    return Err(anyhow::anyhow!(error));
                } else {
                    return Ok(FileValidation {
                        path,
                        is_valid_utf8: false,
                        error: Some(error),
                    });
                }
            }
        };

        Ok(FileValidation {
            path,
            is_valid_utf8,
            error: None,
        })
    }

    async fn check_utf8_encoding(&self, path: &Path) -> Result<bool> {
        // WHY: Reading first 4KB is sufficient to detect encoding issues in most cases
        // while avoiding loading entire large files into memory during discovery
        const SAMPLE_SIZE: usize = 4096;
        
        match fs::read(path).await {
            Ok(bytes) => {
                let sample = if bytes.len() > SAMPLE_SIZE {
                    &bytes[..SAMPLE_SIZE]
                } else {
                    &bytes
                };
                
                match std::str::from_utf8(sample) {
                    Ok(_) => {
                        debug!("UTF-8 validation passed for: {}", path.display());
                        Ok(true)
                    }
                    Err(_) => {
                        debug!("UTF-8 validation failed for: {}", path.display());
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to read file for UTF-8 validation: {}", e))
            }
        }
    }
}

/// Collect all discovered files into a Vec for easier processing
pub async fn collect_discovered_files(
    root_dir: impl AsRef<Path>,
    config: DiscoveryConfig,
) -> Result<Vec<FileValidation>> {
    let mut files = Vec::new();
    let mut stream = Box::pin(discover_files(root_dir, config));
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(validation) => {
                files.push(validation);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    
    info!("Discovered {} files total", files.len());
    let valid_count = files.iter().filter(|f| f.is_valid_utf8 && f.error.is_none()).count();
    let invalid_count = files.len() - valid_count;
    
    if invalid_count > 0 {
        warn!("Found {} files with validation issues", invalid_count);
    }
    
    info!("File discovery summary: {} valid, {} invalid", valid_count, invalid_count);
    
    Ok(files)
}

/// Convenience function to find all valid Gutenberg files (only paths, not validation details)
/// WHY: Simplifies common use case for integration tests and external callers
#[cfg_attr(test, allow(dead_code))]
pub async fn find_gutenberg_files<P: AsRef<Path>>(root_dir: P) -> Result<Vec<PathBuf>> {
    let config = DiscoveryConfig::default();
    let validations = collect_discovered_files(root_dir, config).await?;
    
    // Return only valid files
    let valid_files: Vec<PathBuf> = validations
        .into_iter()
        .filter(|v| v.is_valid_utf8 && v.error.is_none())
        .map(|v| v.path)
        .collect();
    
    Ok(valid_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<PathBuf> {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    #[tokio::test]
    async fn test_discover_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        let files = collect_discovered_files(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_files_matching_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        // Create test files - some matching, some not
        create_test_file(temp_dir.path(), "book-0.txt", "Valid UTF-8 content").await.unwrap();
        create_test_file(temp_dir.path(), "subdir/another-0.txt", "More content").await.unwrap();
        create_test_file(temp_dir.path(), "book-1.txt", "Should not match").await.unwrap();
        create_test_file(temp_dir.path(), "not-gutenberg.txt", "Also should not match").await.unwrap();
        
        let files = collect_discovered_files(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let valid_files: Vec<_> = files.iter().filter(|f| f.is_valid_utf8 && f.error.is_none()).collect();
        assert_eq!(valid_files.len(), 2);
        
        let file_names: Vec<String> = files.iter()
            .map(|f| f.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(file_names.contains(&"book-0.txt".to_string()));
        assert!(file_names.contains(&"another-0.txt".to_string()));
    }

    #[tokio::test]
    async fn test_utf8_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        // Create valid UTF-8 file
        create_test_file(temp_dir.path(), "valid-0.txt", "Hello, 世界!").await.unwrap();
        
        // Create invalid UTF-8 file
        let invalid_path = temp_dir.path().join("invalid-0.txt");
        std::fs::write(&invalid_path, &[0xFF, 0xFE, 0xFD]).unwrap();
        
        let files = collect_discovered_files(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let valid_file = files.iter().find(|f| f.path.file_name().unwrap() == "valid-0.txt").unwrap();
        assert!(valid_file.is_valid_utf8);
        assert!(valid_file.error.is_none());
        
        let invalid_file = files.iter().find(|f| f.path.file_name().unwrap() == "invalid-0.txt").unwrap();
        assert!(!invalid_file.is_valid_utf8);
    }

    #[tokio::test]
    async fn test_fail_fast_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig { fail_fast: true };
        
        // Create a file and then remove read permissions to simulate permission error
        let file_path = create_test_file(temp_dir.path(), "restricted-0.txt", "content").await.unwrap();
        
        // Remove read permissions (Unix-specific test)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0000);
            std::fs::set_permissions(&file_path, perms).unwrap();
            
            let result = collect_discovered_files(temp_dir.path(), config).await;
            
            // Should fail fast on permission error
            assert!(result.is_err());
        }
    }
}