use anyhow::Result;
use futures::stream::{Stream, StreamExt};
use glob::glob;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::mpsc;
use futures::stream;
use ignore::{WalkBuilder, WalkState};

/// Configuration for file discovery behavior
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DiscoveryConfig {
    /// Whether to fail fast on first error or continue processing
    pub fail_fast: bool,
}


/// Result of file discovery validation
#[derive(Debug, Clone)]
pub struct FileValidation {
    pub path: PathBuf,
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

/// Parallel directory traversal using walkdir for improved performance
/// WHY: walkdir can be parallelized while glob is inherently sequential
pub fn discover_files_parallel(
    root_dir: impl AsRef<Path>,
    config: DiscoveryConfig,
) -> impl Stream<Item = Result<FileValidation>> {
    let root_path = root_dir.as_ref().to_path_buf();
    let config = Arc::new(config);
    
    // Create a channel for sending discovered files
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Spawn a task to perform parallel directory traversal
    tokio::spawn(async move {
        info!("Starting directory traversal in: {}", root_path.display());
        let traversal_start = std::time::Instant::now();
        
        // WHY: Use ignore::WalkBuilder (from ripgrep) for optimized deep directory traversal
        // Stream files as they're discovered for true overlapped processing
        let walker = WalkBuilder::new(&root_path)
            .threads((num_cpus::get() / 2).max(1)) // Use no more than half of available CPU cores
            .follow_links(false) // Don't follow symlinks
            .hidden(false) // Don't skip hidden files/dirs (some Gutenberg files might be in hidden dirs)
            .ignore(false) // Don't read .gitignore files
            .git_ignore(false) // Don't read .gitignore files
            .build_parallel();
        
        // Use a channel to stream results from parallel walker
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let tx_clone = tx.clone();
        
        // Spawn walker in a separate thread to avoid blocking
        std::thread::spawn(move || {
            walker.run(|| {
                let result_tx = result_tx.clone();
                Box::new(move |result| {
                    if let Ok(entry) = result {
                        if entry.file_type().is_some_and(|ft| ft.is_file()) {
                            if let Some(file_name) = entry.file_name().to_str() {
                                if file_name.ends_with("-0.txt") {
                                    debug!("Found matching file: {}", entry.path().display());
                                    let _ = result_tx.send(entry.path().to_path_buf());
                                }
                            }
                        }
                    }
                    WalkState::Continue
                })
            });
            // Close the channel when done
            drop(result_tx);
        });
        
        // Stream discovered files immediately to processing pipeline
        let mut file_count = 0;
        while let Ok(path) = result_rx.recv() {
            file_count += 1;
            
            // Create validation result for this file
            let validation_result = validate_file_standalone(&path, &config).await;
            match validation_result {
                Ok(validation) => {
                    if tx_clone.send(Ok(validation)).is_err() {
                        debug!("Receiver dropped, stopping discovery");
                        break;
                    }
                }
                Err(e) => {
                    if config.fail_fast {
                        if tx_clone.send(Err(e)).is_err() {
                            debug!("Receiver dropped, stopping discovery");
                        }
                        break;
                    } else {
                        warn!("File validation error (continuing): {}", e);
                    }
                }
            }
        }
        
        let traversal_time = traversal_start.elapsed();
        info!("Discovery and validation completed in {:.2}ms, streamed {} files", traversal_time.as_millis(), file_count);
        
        // Close the channel to signal completion
        drop(tx_clone);
    });
    
    // Convert channel receiver to stream
    stream::unfold(rx, |mut receiver| async move {
        receiver.recv().await.map(|result| (result, receiver))
    })
}

/// Standalone file validation function for parallel processing
async fn validate_file_standalone(
    path: &Path,
    config: &DiscoveryConfig,
) -> Result<FileValidation> {
    // Check if file is accessible
    match fs::metadata(path).await {
        Ok(metadata) => {
            if !metadata.is_file() {
                let error = format!("Path is not a file: {}", path.display());
                warn!("{}", error);
                return Ok(FileValidation {
                    path: path.to_path_buf(),
                    error: Some(error),
                });
            }
        }
        Err(e) => {
            let error = format!("Cannot access file {}: {}", path.display(), e);
            warn!("{}", error);
            
            if config.fail_fast {
                return Err(anyhow::anyhow!(error));
            } else {
                return Ok(FileValidation {
                    path: path.to_path_buf(),
                    error: Some(error),
                });
            }
        }
    }

    // Skip pre-validation - UTF-8 validation will happen naturally during processing
    Ok(FileValidation {
        path: path.to_path_buf(),
        error: None,
    })
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
                            error: Some(error),
                    });
                }
            }
        }

        // Skip pre-validation - UTF-8 validation will happen naturally during processing
        Ok(FileValidation {
            path,
            error: None,
        })
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
    let valid_count = files.iter().filter(|f| f.error.is_none()).count();
    let invalid_count = files.len() - valid_count;
    
    if invalid_count > 0 {
        warn!("Found {} files with validation issues", invalid_count);
    }
    
    info!("File discovery summary: {} valid, {} invalid", valid_count, invalid_count);
    
    Ok(files)
}

/// Collect all discovered files using parallel directory traversal
/// WHY: Significantly faster for large directory trees with many files
#[allow(dead_code)]
pub async fn collect_discovered_files_parallel(
    root_dir: impl AsRef<Path>,
    config: DiscoveryConfig,
) -> Result<Vec<FileValidation>> {
    let mut files = Vec::new();
    let mut stream = Box::pin(discover_files_parallel(root_dir, config));
    
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
    
    info!("Parallel discovery completed: {} files total", files.len());
    let valid_count = files.iter().filter(|f| f.error.is_none()).count();
    let invalid_count = files.len() - valid_count;
    
    if invalid_count > 0 {
        warn!("Found {} files with validation issues", invalid_count);
    }
    
    info!("Parallel file discovery summary: {} valid, {} invalid", valid_count, invalid_count);
    
    Ok(files)
}

/// Convenience function to find all valid Gutenberg files (only paths, not validation details)
/// WHY: Simplifies common use case for integration tests and external callers
#[allow(dead_code)]
pub async fn find_gutenberg_files<P: AsRef<Path>>(root_dir: P) -> Result<Vec<PathBuf>> {
    let config = DiscoveryConfig::default();
    let validations = collect_discovered_files(root_dir, config).await?;
    
    // Return only valid files
    let valid_files: Vec<PathBuf> = validations
        .into_iter()
        .filter(|v| v.error.is_none())
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
        
        let valid_files: Vec<_> = files.iter().filter(|f| f.error.is_none()).collect();
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
        std::fs::write(&invalid_path, [0xFF, 0xFE, 0xFD]).unwrap();
        
        let files = collect_discovered_files(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let valid_file = files.iter().find(|f| f.path.file_name().unwrap() == "valid-0.txt").unwrap();
        assert!(valid_file.error.is_none());
    }

    #[tokio::test]
    async fn test_fail_fast_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig { fail_fast: true };
        
        // Create a file that will be discovered successfully
        let _file_path = create_test_file(temp_dir.path(), "valid-0.txt", "content").await.unwrap();
        
        let result = collect_discovered_files(temp_dir.path(), config).await;
        
        // Discovery should succeed (fail_fast now applies to processing, not discovery)
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].error.is_none());
    }

    #[tokio::test]
    async fn test_parallel_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        // Create multiple test files
        create_test_file(temp_dir.path(), "book1-0.txt", "Valid UTF-8 content").await.unwrap();
        create_test_file(temp_dir.path(), "subdir/book2-0.txt", "More content").await.unwrap();
        create_test_file(temp_dir.path(), "book3-0.txt", "Third book").await.unwrap();
        create_test_file(temp_dir.path(), "book-1.txt", "Should not match").await.unwrap();
        
        // Test parallel discovery
        let files = collect_discovered_files_parallel(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 3);
        
        let valid_files: Vec<_> = files.iter().filter(|f| f.error.is_none()).collect();
        assert_eq!(valid_files.len(), 3);
        
        let file_names: Vec<String> = files.iter()
            .map(|f| f.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(file_names.contains(&"book1-0.txt".to_string()));
        assert!(file_names.contains(&"book2-0.txt".to_string()));
        assert!(file_names.contains(&"book3-0.txt".to_string()));
    }

    #[tokio::test]
    async fn test_parallel_vs_serial_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        // Create test files
        for i in 0..5 {
            let file_name = format!("test{i}-0.txt");
            create_test_file(temp_dir.path(), &file_name, "Test content").await.unwrap();
        }
        
        // Test serial discovery
        let serial_files = collect_discovered_files(temp_dir.path(), config.clone()).await.unwrap();
        
        // Test parallel discovery
        let parallel_files = collect_discovered_files_parallel(temp_dir.path(), config).await.unwrap();
        
        // Should find the same files
        assert_eq!(serial_files.len(), parallel_files.len());
        assert_eq!(serial_files.len(), 5);
        
        // Sort files by path for comparison
        let mut serial_paths: Vec<_> = serial_files.iter().map(|f| &f.path).collect();
        let mut parallel_paths: Vec<_> = parallel_files.iter().map(|f| &f.path).collect();
        
        serial_paths.sort();
        parallel_paths.sort();
        
        assert_eq!(serial_paths, parallel_paths);
    }

    #[tokio::test]
    async fn test_parallel_discovery_with_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        
        // Create valid file
        create_test_file(temp_dir.path(), "valid-0.txt", "Valid content").await.unwrap();
        
        // Create invalid UTF-8 file
        let invalid_path = temp_dir.path().join("invalid-0.txt");
        std::fs::write(&invalid_path, [0xFF, 0xFE, 0xFD]).unwrap();
        
        let files = collect_discovered_files_parallel(temp_dir.path(), config).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let valid_file = files.iter().find(|f| f.path.file_name().unwrap() == "valid-0.txt").unwrap();
        assert!(valid_file.error.is_none());
    }
}