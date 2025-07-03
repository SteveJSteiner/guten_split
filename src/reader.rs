use anyhow::Result;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, info, warn};

/// Configuration for file reading behavior
#[derive(Debug, Clone)]
pub struct ReaderConfig {
    /// Whether to fail fast on first error or continue processing
    pub fail_fast: bool,
    /// Buffer size for async reading (default: 8KB)
    pub buffer_size: usize,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            buffer_size: 8192, // WHY: 8KB is optimal for most filesystems and network storage
        }
    }
}

/// Statistics for file reading operations
#[derive(Debug, Clone)]
pub struct ReadStats {
    pub file_path: String,
    pub lines_read: u64,
    pub bytes_read: u64,
    pub duration_ms: u64,
    pub read_error: Option<String>,
}

/// Async file reader that streams file contents line-by-line
pub struct AsyncFileReader {
    config: ReaderConfig,
}

impl AsyncFileReader {
    pub fn new(config: ReaderConfig) -> Self {
        Self { config }
    }

    /// Read file contents line-by-line with async buffered I/O
    /// Returns an iterator of lines and final read statistics
    pub async fn read_file_lines<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(Vec<String>, ReadStats)> {
        let path = file_path.as_ref();
        let start_time = std::time::Instant::now();
        
        debug!("Starting async read of file: {}", path.display());
        
        // WHY: early validation prevents partial processing and provides clear error context
        let file = match File::open(path).await {
            Ok(file) => file,
            Err(e) => {
                let error_msg = format!("Failed to open file {}: {}", path.display(), e);
                warn!("{}", error_msg);
                
                let stats = ReadStats {
                    file_path: path.display().to_string(),
                    lines_read: 0,
                    bytes_read: 0,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    read_error: Some(error_msg.clone()),
                };
                
                if self.config.fail_fast {
                    return Err(anyhow::anyhow!(error_msg));
                } else {
                    return Ok((Vec::new(), stats));
                }
            }
        };

        // WHY: BufReader with custom buffer size reduces syscalls and improves throughput
        let reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut lines = reader.lines();
        let mut result_lines = Vec::new();
        let mut line_count = 0u64;
        let mut byte_count = 0u64;

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    byte_count += line.len() as u64 + 1; // +1 for newline
                    line_count += 1;
                    result_lines.push(line);
                }
                Ok(None) => {
                    // End of file reached
                    break;
                }
                Err(e) => {
                    let error_msg = format!("UTF-8 decoding error in {} at line {}: {}", 
                                          path.display(), line_count + 1, e);
                    warn!("{}", error_msg);
                    
                    let stats = ReadStats {
                        file_path: path.display().to_string(),
                        lines_read: line_count,
                        bytes_read: byte_count,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        read_error: Some(error_msg.clone()),
                    };
                    
                    if self.config.fail_fast {
                        return Err(anyhow::anyhow!(error_msg));
                    } else {
                        // Return partial results with error information
                        return Ok((result_lines, stats));
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        let stats = ReadStats {
            file_path: path.display().to_string(),
            lines_read: line_count,
            bytes_read: byte_count,
            duration_ms: duration.as_millis() as u64,
            read_error: None,
        };

        info!(
            "Successfully read {}: {} lines, {} bytes in {}ms ({:.2} MB/s)",
            path.display(),
            line_count,
            byte_count,
            stats.duration_ms,
            if stats.duration_ms > 0 {
                (byte_count as f64 / 1_000_000.0) / (stats.duration_ms as f64 / 1000.0)
            } else {
                0.0
            }
        );

        debug!("Completed async read of file: {}", path.display());
        Ok((result_lines, stats))
    }

    /// Process multiple files concurrently with async I/O
    pub async fn read_files_batch<P: AsRef<Path>>(
        &self,
        file_paths: &[P],
    ) -> Result<Vec<(Vec<String>, ReadStats)>> {
        info!("Starting batch read of {} files", file_paths.len());
        
        let mut results = Vec::new();
        
        // WHY: sequential processing ensures memory usage stays bounded for large file sets
        // and provides better error reporting per file
        for file_path in file_paths {
            match self.read_file_lines(file_path).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    if self.config.fail_fast {
                        return Err(e);
                    } else {
                        warn!("Failed to read file {}: {}", file_path.as_ref().display(), e);
                        // Continue with next file
                        let stats = ReadStats {
                            file_path: file_path.as_ref().display().to_string(),
                            lines_read: 0,
                            bytes_read: 0,
                            duration_ms: 0,
                            read_error: Some(e.to_string()),
                        };
                        results.push((Vec::new(), stats));
                    }
                }
            }
        }
        
        info!("Completed batch read of {} files", results.len());
        Ok(results)
    }
}

/// Convenience function for reading a single file with default configuration
/// WHY: Simplifies common use case for integration tests and external callers
#[cfg_attr(test, allow(dead_code))]
pub async fn read_file_async<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let reader = AsyncFileReader::new(ReaderConfig::default());
    let (lines, _stats) = reader.read_file_lines(file_path).await?;
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<std::path::PathBuf> {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    #[tokio::test]
    async fn test_read_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig::default();
        let reader = AsyncFileReader::new(config);
        
        let content = "Line 1\nLine 2\nLine 3";
        let file_path = create_test_file(temp_dir.path(), "test.txt", content).await.unwrap();
        
        let (lines, stats) = reader.read_file_lines(&file_path).await.unwrap();
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
        assert_eq!(stats.lines_read, 3);
        assert!(stats.bytes_read > 0);
        assert!(stats.read_error.is_none());
    }

    #[tokio::test]
    async fn test_read_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig::default();
        let reader = AsyncFileReader::new(config);
        
        let file_path = create_test_file(temp_dir.path(), "empty.txt", "").await.unwrap();
        
        let (lines, stats) = reader.read_file_lines(&file_path).await.unwrap();
        
        assert_eq!(lines.len(), 0);
        assert_eq!(stats.lines_read, 0);
        assert_eq!(stats.bytes_read, 0);
        assert!(stats.read_error.is_none());
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig { fail_fast: false, ..Default::default() };
        let reader = AsyncFileReader::new(config);
        
        let file_path = temp_dir.path().join("nonexistent.txt");
        
        let (lines, stats) = reader.read_file_lines(&file_path).await.unwrap();
        
        assert_eq!(lines.len(), 0);
        assert_eq!(stats.lines_read, 0);
        assert!(stats.read_error.is_some());
    }

    #[tokio::test]
    async fn test_read_nonexistent_file_fail_fast() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig { fail_fast: true, ..Default::default() };
        let reader = AsyncFileReader::new(config);
        
        let file_path = temp_dir.path().join("nonexistent.txt");
        
        let result = reader.read_file_lines(&file_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_utf8_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig::default();
        let reader = AsyncFileReader::new(config);
        
        // Create file with valid UTF-8 including Unicode
        let content = "Hello, 世界!\nThis is a test\nWith émojis 🦀";
        let file_path = create_test_file(temp_dir.path(), "unicode.txt", content).await.unwrap();
        
        let (lines, stats) = reader.read_file_lines(&file_path).await.unwrap();
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Hello, 世界!");
        assert_eq!(lines[2], "With émojis 🦀");
        assert!(stats.read_error.is_none());
    }

    #[tokio::test]
    async fn test_read_files_batch() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig::default();
        let reader = AsyncFileReader::new(config);
        
        let file1 = create_test_file(temp_dir.path(), "file1.txt", "Content 1\nLine 2").await.unwrap();
        let file2 = create_test_file(temp_dir.path(), "file2.txt", "Different content").await.unwrap();
        
        let file_paths = vec![&file1, &file2];
        let results = reader.read_files_batch(&file_paths).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.len(), 2); // file1 has 2 lines
        assert_eq!(results[1].0.len(), 1); // file2 has 1 line
        assert!(results[0].1.read_error.is_none());
        assert!(results[1].1.read_error.is_none());
    }

    #[tokio::test]
    async fn test_custom_buffer_size() {
        let temp_dir = TempDir::new().unwrap();
        let config = ReaderConfig { 
            fail_fast: false, 
            buffer_size: 1024 // smaller buffer for testing
        };
        let reader = AsyncFileReader::new(config);
        
        // Create file larger than buffer
        let content = "x".repeat(2048) + "\n" + &"y".repeat(2048);
        let file_path = create_test_file(temp_dir.path(), "large.txt", &content).await.unwrap();
        
        let (lines, stats) = reader.read_file_lines(&file_path).await.unwrap();
        
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 2048);
        assert_eq!(lines[1].len(), 2048);
        assert!(stats.read_error.is_none());
    }
}