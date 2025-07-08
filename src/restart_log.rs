use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Simple restart log that tracks successfully processed files
/// WHY: Provides restartability without complex discovery cache management
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RestartLog {
    /// Set of successfully processed file paths
    completed_files: HashSet<String>,
    /// Timestamp of last update
    last_updated: u64,
}

impl RestartLog {
    /// Load restart log from file, returns empty log if file doesn't exist
    /// WHY: Graceful fallback ensures the system works even without prior log
    pub async fn load(root_dir: &Path) -> Self {
        let log_path = Self::get_log_path(root_dir);
        
        match fs::read_to_string(&log_path).await {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }
    
    /// Save restart log to file
    /// WHY: Persist completion state for future runs
    pub async fn save(&self, root_dir: &Path) -> Result<()> {
        let log_path = Self::get_log_path(root_dir);
        let content = serde_json::to_string_pretty(self)?;
        
        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&log_path, content).await?;
        Ok(())
    }
    
    /// Check if a file has been successfully processed
    /// WHY: Core restart logic - skip files that are already complete
    pub fn is_completed(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy().to_string();
        self.completed_files.contains(&path_str)
    }
    
    /// Mark a file as successfully processed
    /// WHY: Record completion for future restart capability
    pub fn mark_completed(&mut self, file_path: &Path) {
        let path_str = file_path.to_string_lossy().to_string();
        self.completed_files.insert(path_str);
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Get all completed files
    /// WHY: Useful for debugging and status reporting
    pub fn get_completed_files(&self) -> Vec<PathBuf> {
        self.completed_files.iter()
            .map(PathBuf::from)
            .collect()
    }
    
    /// Get count of completed files
    /// WHY: Quick status check without allocating full paths
    pub fn completed_count(&self) -> usize {
        self.completed_files.len()
    }
    
    /// Clear all completed files
    /// WHY: Allows full reprocessing when needed
    pub fn clear(&mut self) {
        self.completed_files.clear();
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Get the path to the restart log file
    /// WHY: Centralized path management
    fn get_log_path(root_dir: &Path) -> PathBuf {
        root_dir.join(".seams_restart.json")
    }
    
    /// Batch append multiple completed files
    /// WHY: Efficient for processing multiple files at once
    pub async fn append_completed_files(&mut self, files: &[&Path]) -> Result<()> {
        for file in files {
            self.mark_completed(file);
        }
        Ok(())
    }
    
    /// Verify completed files still exist and have aux files
    /// WHY: Validate restart log integrity and detect incomplete processing
    pub async fn verify_completed_files(&mut self) -> Result<Vec<PathBuf>> {
        let mut invalid_files = Vec::new();
        let mut valid_files = HashSet::new();
        
        for file_path_str in &self.completed_files {
            let file_path = PathBuf::from(file_path_str);
            
            // Check if source file exists
            if !file_path.exists() {
                invalid_files.push(file_path.clone());
                continue;
            }
            
            // Check if aux file exists
            let aux_path = crate::incremental::generate_aux_file_path(&file_path);
            if !aux_path.exists() {
                invalid_files.push(file_path.clone());
                continue;
            }
            
            valid_files.insert(file_path_str.clone());
        }
        
        // Update completed files to only include valid ones
        self.completed_files = valid_files;
        
        Ok(invalid_files)
    }
    
    /// Get restart log statistics
    /// WHY: Provides useful information for debugging and monitoring
    pub fn get_stats(&self) -> RestartStats {
        RestartStats {
            completed_files: self.completed_files.len(),
            last_updated: self.last_updated,
        }
    }
}

/// Statistics about the restart log
#[derive(Debug, Clone)]
pub struct RestartStats {
    pub completed_files: usize,
    pub last_updated: u64,
}

/// Check if a file should be processed based on restart log and overwrite flags
/// WHY: Centralized logic for restart decisions
pub async fn should_process_file(
    file_path: &Path,
    restart_log: &RestartLog,
    overwrite_all: bool,
    overwrite_use_cached_locations: bool,
) -> Result<bool> {
    // If overwrite_all is true, always process
    if overwrite_all {
        return Ok(true);
    }
    
    // If overwrite_use_cached_locations is true, process even if completed
    if overwrite_use_cached_locations {
        return Ok(true);
    }
    
    // Check if file is already completed
    if restart_log.is_completed(file_path) {
        // Verify aux file still exists
        let aux_path = crate::incremental::generate_aux_file_path(file_path);
        if aux_path.exists() {
            return Ok(false); // Skip processing
        }
    }
    
    // Process the file
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    
    #[tokio::test]
    async fn test_restart_log_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();
        
        // Create a new restart log
        let mut log = RestartLog::default();
        assert_eq!(log.completed_count(), 0);
        
        // Mark some files as completed
        let file1 = root_path.join("test1-0.txt");
        let file2 = root_path.join("test2-0.txt");
        
        log.mark_completed(&file1);
        log.mark_completed(&file2);
        
        assert_eq!(log.completed_count(), 2);
        assert!(log.is_completed(&file1));
        assert!(log.is_completed(&file2));
        
        // Save and reload
        log.save(root_path).await.unwrap();
        let loaded_log = RestartLog::load(root_path).await;
        
        assert_eq!(loaded_log.completed_count(), 2);
        assert!(loaded_log.is_completed(&file1));
        assert!(loaded_log.is_completed(&file2));
    }
    
    #[tokio::test]
    async fn test_restart_log_verification() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();
        
        // Create test files
        let file1 = root_path.join("test1-0.txt");
        let file2 = root_path.join("test2-0.txt");
        
        fs::write(&file1, "content1").await.unwrap();
        fs::write(&file2, "content2").await.unwrap();
        
        // Create aux files
        let aux1 = crate::incremental::generate_aux_file_path(&file1);
        let aux2 = crate::incremental::generate_aux_file_path(&file2);
        
        fs::write(&aux1, "aux1").await.unwrap();
        fs::write(&aux2, "aux2").await.unwrap();
        
        // Create restart log
        let mut log = RestartLog::default();
        log.mark_completed(&file1);
        log.mark_completed(&file2);
        
        // Verify - should pass
        let invalid = log.verify_completed_files().await.unwrap();
        assert_eq!(invalid.len(), 0);
        assert_eq!(log.completed_count(), 2);
        
        // Remove one aux file
        fs::remove_file(&aux1).await.unwrap();
        
        // Verify - should find invalid file
        let invalid = log.verify_completed_files().await.unwrap();
        assert_eq!(invalid.len(), 1);
        assert_eq!(log.completed_count(), 1);
        assert!(log.is_completed(&file2));
        assert!(!log.is_completed(&file1));
    }
    
    #[tokio::test]
    async fn test_should_process_file_logic() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();
        
        let file1 = root_path.join("test1-0.txt");
        fs::write(&file1, "content1").await.unwrap();
        
        let aux1 = crate::incremental::generate_aux_file_path(&file1);
        fs::write(&aux1, "aux1").await.unwrap();
        
        let mut log = RestartLog::default();
        log.mark_completed(&file1);
        
        // Without overwrite flags - should skip completed file
        let should_process = should_process_file(&file1, &log, false, false).await.unwrap();
        assert!(!should_process);
        
        // With overwrite_all - should process
        let should_process = should_process_file(&file1, &log, true, false).await.unwrap();
        assert!(should_process);
        
        // With overwrite_use_cached_locations - should process
        let should_process = should_process_file(&file1, &log, false, true).await.unwrap();
        assert!(should_process);
        
        // Remove aux file - should process even without overwrite
        fs::remove_file(&aux1).await.unwrap();
        let should_process = should_process_file(&file1, &log, false, false).await.unwrap();
        assert!(should_process);
    }
    
    #[tokio::test]
    async fn test_restart_log_clear() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();
        
        let mut log = RestartLog::default();
        let file1 = root_path.join("test1-0.txt");
        
        log.mark_completed(&file1);
        assert_eq!(log.completed_count(), 1);
        
        log.clear();
        assert_eq!(log.completed_count(), 0);
        assert!(!log.is_completed(&file1));
    }
}