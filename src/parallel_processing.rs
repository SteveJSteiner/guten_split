// WHY: Parallel processing functionality for benchmarking and external use
// Extracted from main.rs to enable benchmark access while maintaining functionality

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use tracing::info;
use memmap2::MmapOptions;

use crate::incremental::{generate_aux_file_path, cache_exists, read_cache_async, aux_file_exists};

/// Comprehensive file metadata for both discovery and processing cache
/// WHY: Unified structure reduces duplication and simplifies cache management
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// Last modification timestamp (seconds since epoch)
    pub modified: u64,
    /// Whether file passed UTF-8 validation
    pub is_valid_utf8: bool,
    /// Processing completion timestamp (None if not yet processed)
    pub completed_at: Option<u64>,
}

/// Unified cache for file discovery and processing state
/// WHY: Single source of truth for all file state, eliminating redundancy
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProcessingCache {
    /// Map from source file path to comprehensive metadata
    pub files: HashMap<String, FileMetadata>,
    /// Timestamp when file discovery was last performed
    pub discovery_timestamp: Option<u64>,
}

impl ProcessingCache {
    /// Load cache from file, returns empty cache if file doesn't exist or is corrupted
    /// WHY: Fail-safe approach - missing/corrupted cache just means reprocessing everything
    pub async fn load(root_dir: &Path) -> Self {
        if !cache_exists(root_dir) {
            return Self::default();
        }
        
        match read_cache_async(root_dir).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(cache) => cache,
                    Err(e) => {
                        info!("Cache file corrupted, starting fresh: {}", e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                info!("Could not read cache file, starting fresh: {}", e);
                Self::default()
            }
        }
    }
    
    /// Save cache to file
    /// WHY: Persist completion state for future runs
    pub async fn save(&self, cache_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(cache_path, content).await?;
        Ok(())
    }
    
    /// Check if source file has been processed and aux file is still valid
    /// WHY: Core incremental logic - compare source modification time vs completion time
    pub async fn is_file_processed(&self, source_path: &Path) -> Result<bool> {
        let source_path_str = source_path.to_string_lossy().to_string();
        
        if let Some(file_meta) = self.files.get(&source_path_str) {
            if let Some(completion_timestamp) = file_meta.completed_at {
                // Check if source file has been modified since completion
                let source_metadata = tokio::fs::metadata(source_path).await?;
                let source_modified = source_metadata.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                
                // Also verify aux file still exists
                if !aux_file_exists(source_path) {
                    info!("Aux file missing for {}, reprocessing", source_path.display());
                    return Ok(false);
                }
                
                Ok(source_modified <= completion_timestamp)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    /// Mark file as completed with current timestamp
    /// WHY: Record successful completion for future incremental runs
    pub fn mark_completed(&mut self, source_path: &Path) {
        let source_path_str = source_path.to_string_lossy().to_string();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Update existing entry or create new one
        if let Some(file_meta) = self.files.get_mut(&source_path_str) {
            file_meta.completed_at = Some(now);
        } else {
            // This shouldn't happen if discovery was done properly, but handle gracefully
            info!("Marking completion for undiscovered file: {}", source_path.display());
            self.files.insert(source_path_str, FileMetadata {
                size: 0, // Will be updated on next discovery
                modified: 0,
                is_valid_utf8: true, // Assume valid if we processed it
                completed_at: Some(now),
            });
        }
    }
    
    /// Check if file discovery cache is valid for given root directory
    /// WHY: Implements cache invalidation logic based on file-level verification
    pub async fn is_discovery_cache_valid(&self, _root_dir: &Path) -> Result<bool> {
        // Check if we have discovery metadata at all
        if self.discovery_timestamp.is_none() || self.files.is_empty() {
            return Ok(false);
        }
        
        // For performance, we validate cached files during get_cached_discovered_files
        // This method only checks if we have any cached discovery data
        Ok(true)
    }
    
    /// Get cached discovered files if cache is valid
    /// WHY: Provides fast restart by avoiding directory traversal when possible
    pub async fn get_cached_discovered_files(&self, root_dir: &Path) -> Result<Option<Vec<crate::discovery::FileValidation>>> {
        if !self.is_discovery_cache_valid(root_dir).await? {
            return Ok(None);
        }
        
        let mut files = Vec::new();
        for (path_str, file_meta) in &self.files {
            let path = PathBuf::from(path_str);
            
            // Verify file still exists and hasn't changed
            if let Ok(current_metadata) = tokio::fs::metadata(&path).await {
                let current_modified = current_metadata.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                let current_size = current_metadata.len();
                
                if current_modified <= file_meta.modified && current_size == file_meta.size {
                    files.push(crate::discovery::FileValidation {
                        path,
                        is_valid_utf8: file_meta.is_valid_utf8,
                        error: None,
                    });
                } else {
                    // File changed, cache is invalid
                    return Ok(None);
                }
            } else {
                // File no longer exists, cache is invalid
                return Ok(None);
            }
        }
        
        Ok(Some(files))
    }
    
    /// Update discovery cache with new file discovery results
    /// WHY: Stores discovery results for future fast restarts
    pub async fn update_discovery_cache(&mut self, _root_dir: &Path, files: &[crate::discovery::FileValidation]) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        
        // Clear existing discovery cache but preserve completion status
        let mut completed_files = HashMap::new();
        for (path, file_meta) in &self.files {
            if let Some(completed_at) = file_meta.completed_at {
                completed_files.insert(path.clone(), completed_at);
            }
        }
        
        // Clear and rebuild file cache
        self.files.clear();
        
        // Add new discovery results
        for file in files {
            if let Ok(metadata) = tokio::fs::metadata(&file.path).await {
                let modified = metadata.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                
                let path_str = file.path.to_string_lossy().to_string();
                self.files.insert(
                    path_str.clone(),
                    FileMetadata {
                        size: metadata.len(),
                        modified,
                        is_valid_utf8: file.is_valid_utf8,
                        completed_at: completed_files.get(&path_str).copied(),
                    }
                );
            }
        }
        
        self.discovery_timestamp = Some(now);
        
        Ok(())
    }
}

/// Per-file processing statistics
/// WHY: Collects metrics for each file processed to meet PRD F-8 requirements
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStats {
    /// File path relative to root directory
    pub path: String,
    /// Number of characters processed
    pub chars_processed: u64,
    /// Number of sentences detected
    pub sentences_detected: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Throughput in characters per second
    pub chars_per_sec: f64,
    /// Processing status (success, skipped, failed)
    pub status: String,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Determine if file should be processed based on cache and incremental rules
/// WHY: Implements core F-9 logic using robust timestamp-based cache
pub async fn should_process_file(source_path: &Path, cache: &ProcessingCache, overwrite_all: bool, overwrite_use_cached_locations: bool) -> Result<bool> {
    if overwrite_all || overwrite_use_cached_locations {
        return Ok(true);
    }
    
    let is_processed = cache.is_file_processed(source_path).await?;
    
    if is_processed {
        info!("Skipping {} - already processed and up to date", source_path.display());
        return Ok(false);
    }
    
    let aux_path = generate_aux_file_path(source_path);
    if aux_path.exists() {
        info!("Processing {} - source newer than cache or aux file missing from cache", source_path.display());
    }
    
    Ok(true)
}

/// Write auxiliary file with borrowed sentence data in F-5 format
/// WHY: Zero-allocation async I/O optimized for mmap-based processing
pub async fn write_auxiliary_file_borrowed(
    aux_path: &Path,
    sentences: &[crate::sentence_detector::DetectedSentenceBorrowed<'_>],
    _detector: &crate::sentence_detector::dialog_detector::SentenceDetectorDialog,
) -> Result<()> {
    let file = tokio::fs::File::create(aux_path).await?;
    let mut writer = BufWriter::new(file);
    
    for sentence in sentences {
        // WHY: Call normalize() on-demand to maintain zero-allocation benefits
        let formatted_line = format!("{}\t{}\t({},{},{},{})", 
            sentence.index, 
            sentence.normalize(),
            sentence.span.start_line,
            sentence.span.start_col,
            sentence.span.end_line,
            sentence.span.end_col
        );
        writer.write_all(formatted_line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    
    writer.flush().await?;
    Ok(())
}

/// Process multiple files in parallel using memory-mapped I/O
/// WHY: Combines async orchestration with mmap for optimal performance and true parallelism
pub async fn process_files_parallel(
    file_paths: &[&PathBuf],
    cache: ProcessingCache,
    overwrite_all: bool,
    overwrite_use_cached_locations: bool,
    fail_fast: bool,
    max_concurrent: usize,
) -> Result<(u64, u64, u64, u64, u64, Vec<FileStats>)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let detector = Arc::new(
        crate::sentence_detector::dialog_detector::SentenceDetectorDialog::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize dialog sentence detector: {}", e))?
    );
    
    let tasks: Vec<_> = file_paths.iter().map(|&path| {
        let semaphore = semaphore.clone();
        let detector = detector.clone();
        let path = path.clone();
        let cache_clone = cache.clone();
        
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let start_time = Instant::now();
            
            // WHY: Check if file should be processed based on cache and incremental rules
            let should_process = should_process_file(&path, &cache_clone, overwrite_all, overwrite_use_cached_locations).await
                .unwrap_or(true); // Process on error to be safe
            
            if !should_process {
                let processing_time = start_time.elapsed();
                let file_stats = FileStats {
                    path: path.to_string_lossy().to_string(),
                    chars_processed: 0,
                    sentences_detected: 0,
                    processing_time_ms: processing_time.as_millis() as u64,
                    chars_per_sec: 0.0,
                    status: "skipped".to_string(),
                    error: None,
                };
                return Ok::<(u64, u64, u64, bool, FileStats), anyhow::Error>((0u64, 0u64, 0u64, true, file_stats)); // skipped
            }
            
            // WHY: Use mmap for efficient file access instead of loading into memory
            let file = std::fs::File::open(&path)?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            let content = std::str::from_utf8(&mmap)
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in file: {}", path.display()))?;
            
            // WHY: Use borrowed API for zero-allocation sentence detection
            let sentences = detector.detect_sentences_borrowed(content)?;
            let sentence_count = sentences.len() as u64;
            let byte_count = content.len() as u64;
            
            // WHY: Generate auxiliary file as per F-7 requirement
            let aux_path = generate_aux_file_path(&path);
            write_auxiliary_file_borrowed(&aux_path, &sentences, &detector).await?;
            
            let processing_time = start_time.elapsed();
            let processing_time_ms = processing_time.as_millis() as u64;
            let chars_per_sec = if processing_time.as_secs_f64() > 0.0 {
                byte_count as f64 / processing_time.as_secs_f64()
            } else {
                0.0
            };
            
            let file_stats = FileStats {
                path: path.to_string_lossy().to_string(),
                chars_processed: byte_count,
                sentences_detected: sentence_count,
                processing_time_ms,
                chars_per_sec,
                status: "success".to_string(),
                error: None,
            };
            
            info!("Processed {}: {} sentences, {} bytes", path.display(), sentence_count, byte_count);
            
            Ok::<(u64, u64, u64, bool, FileStats), anyhow::Error>((sentence_count, byte_count, 1u64, false, file_stats)) // sentences, bytes, files_processed, skipped, stats
        })
    }).collect();
    
    let mut total_sentences = 0u64;
    let mut total_bytes = 0u64;
    let mut processed_files = 0u64;
    let mut skipped_files = 0u64;
    let mut failed_files = 0u64;
    let mut file_stats = Vec::new();
    
    // WHY: Wait for all tasks and handle results
    let results = futures::future::join_all(tasks).await;
    
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok((sentences, bytes, files, skipped, stats))) => {
                total_sentences += sentences;
                total_bytes += bytes;
                if skipped {
                    skipped_files += 1;
                } else {
                    processed_files += files;
                }
                file_stats.push(stats);
            }
            Ok(Err(e)) => {
                info!("File processing error: {}", e);
                failed_files += 1;
                
                // WHY: Create failed FileStats entry for error tracking
                let failed_stats = FileStats {
                    path: if i < file_paths.len() {
                        file_paths[i].to_string_lossy().to_string()
                    } else {
                        "unknown".to_string()
                    },
                    chars_processed: 0,
                    sentences_detected: 0,
                    processing_time_ms: 0,
                    chars_per_sec: 0.0,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                };
                file_stats.push(failed_stats);
                
                if fail_fast {
                    return Err(e);
                }
            }
            Err(e) => {
                info!("Task execution error: {}", e);
                failed_files += 1;
                
                // WHY: Create failed FileStats entry for task execution error
                let failed_stats = FileStats {
                    path: if i < file_paths.len() {
                        file_paths[i].to_string_lossy().to_string()
                    } else {
                        "unknown".to_string()
                    },
                    chars_processed: 0,
                    sentences_detected: 0,
                    processing_time_ms: 0,
                    chars_per_sec: 0.0,
                    status: "failed".to_string(),
                    error: Some(format!("Task failed: {e}")),
                };
                file_stats.push(failed_stats);
                
                if fail_fast {
                    return Err(anyhow::anyhow!("Task failed: {}", e));
                }
            }
        }
    }
    
    Ok((total_sentences, total_bytes, processed_files, skipped_files, failed_files, file_stats))
}