use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use seams::restart_log::{RestartLog, should_process_file};
use seams::incremental::generate_aux_file_path;

/// Integration test for restart functionality
#[tokio::test]
async fn test_restart_functionality_integration() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create test files
    let test_files = [
        "test1-0.txt",
        "test2-0.txt", 
        "test3-0.txt",
    ];
    
    for file_name in &test_files {
        let file_path = root_path.join(file_name);
        let content = format!("Test content for {}. This is a sentence. Another sentence here.", file_name);
        fs::write(&file_path, content).await.unwrap();
    }
    
    // Test 1: Initial run with empty restart log
    let mut restart_log = RestartLog::load(root_path).await;
    assert_eq!(restart_log.completed_count(), 0);
    
    // All files should be processed initially
    for file_name in &test_files {
        let file_path = root_path.join(file_name);
        let should_process = should_process_file(&file_path, &restart_log, false, false).await.unwrap();
        assert!(should_process, "File {} should be processed initially", file_name);
    }
    
    // Test 2: Simulate processing files by creating aux files and marking completed
    for file_name in &test_files {
        let file_path = root_path.join(file_name);
        let aux_path = generate_aux_file_path(&file_path);
        
        // Create aux file
        let aux_content = "0\tTest sentence.\t(1,1,1,14)\n1\tAnother sentence here.\t(1,15,1,37)\n";
        fs::write(&aux_path, aux_content).await.unwrap();
        
        // Mark as completed
        restart_log.mark_completed(&file_path);
    }
    
    // Save restart log
    restart_log.save(root_path).await.unwrap();
    
    // Test 3: Restart behavior - files should be skipped
    let restart_log = RestartLog::load(root_path).await;
    assert_eq!(restart_log.completed_count(), 3);
    
    for file_name in &test_files {
        let file_path = root_path.join(file_name);
        let should_process = should_process_file(&file_path, &restart_log, false, false).await.unwrap();
        assert!(!should_process, "File {} should be skipped on restart", file_name);
    }
    
    // Test 4: Overwrite flags behavior
    for file_name in &test_files {
        let file_path = root_path.join(file_name);
        
        // With overwrite_all=true, should process
        let should_process = should_process_file(&file_path, &restart_log, true, false).await.unwrap();
        assert!(should_process, "File {} should be processed with overwrite_all=true", file_name);
        
        // With overwrite_use_cached_locations=true, should process  
        let should_process = should_process_file(&file_path, &restart_log, false, true).await.unwrap();
        assert!(should_process, "File {} should be processed with overwrite_use_cached_locations=true", file_name);
    }
    
    // Test 5: Missing aux file should trigger reprocessing
    let file_path = root_path.join("test1-0.txt");
    let aux_path = generate_aux_file_path(&file_path);
    fs::remove_file(&aux_path).await.unwrap();
    
    let should_process = should_process_file(&file_path, &restart_log, false, false).await.unwrap();
    assert!(should_process, "File should be reprocessed if aux file is missing");
    
    // Test 6: Restart log persistence
    let restart_log_path = root_path.join(".seams_restart.json");
    assert!(restart_log_path.exists(), "Restart log file should exist");
    
    let log_content = fs::read_to_string(&restart_log_path).await.unwrap();
    assert!(log_content.contains("test1-0.txt"), "Restart log should contain test1-0.txt");
    assert!(log_content.contains("test2-0.txt"), "Restart log should contain test2-0.txt");
    assert!(log_content.contains("test3-0.txt"), "Restart log should contain test3-0.txt");
}

/// Test restart log verification functionality
#[tokio::test]
async fn test_restart_log_verification() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create test files
    let file1 = root_path.join("file1-0.txt");
    let file2 = root_path.join("file2-0.txt");
    let file3 = root_path.join("file3-0.txt");
    
    fs::write(&file1, "content1").await.unwrap();
    fs::write(&file2, "content2").await.unwrap();
    fs::write(&file3, "content3").await.unwrap();
    
    // Create aux files
    let aux1 = generate_aux_file_path(&file1);
    let aux2 = generate_aux_file_path(&file2);
    let aux3 = generate_aux_file_path(&file3);
    
    fs::write(&aux1, "aux1").await.unwrap();
    fs::write(&aux2, "aux2").await.unwrap();
    fs::write(&aux3, "aux3").await.unwrap();
    
    // Create restart log with all files marked as completed
    let mut restart_log = RestartLog::load(root_path).await;
    restart_log.mark_completed(&file1);
    restart_log.mark_completed(&file2);
    restart_log.mark_completed(&file3);
    
    // Verify all files are valid
    let invalid_files = restart_log.verify_completed_files().await.unwrap();
    assert_eq!(invalid_files.len(), 0, "All files should be valid initially");
    assert_eq!(restart_log.completed_count(), 3, "All files should remain in restart log");
    
    // Remove one aux file and one source file
    fs::remove_file(&aux1).await.unwrap();
    fs::remove_file(&file2).await.unwrap();
    
    // Verify should find invalid files
    let invalid_files = restart_log.verify_completed_files().await.unwrap();
    assert_eq!(invalid_files.len(), 2, "Should find 2 invalid files");
    assert_eq!(restart_log.completed_count(), 1, "Only 1 file should remain in restart log");
    
    // Remaining file should still be valid
    assert!(restart_log.is_completed(&file3), "file3 should still be completed");
    assert!(!restart_log.is_completed(&file1), "file1 should no longer be completed");
    assert!(!restart_log.is_completed(&file2), "file2 should no longer be completed");
}

/// Test full CLI integration with restart log
#[tokio::test]
async fn test_cli_restart_integration() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create test files
    for i in 1..=5 {
        let file_path = root_path.join(format!("test{}-0.txt", i));
        let content = format!("Test file {} content. This is a sentence. Another sentence.", i);
        fs::write(&file_path, content).await.unwrap();
    }
    
    // First run - simulate seams CLI
    let mut restart_log = RestartLog::load(root_path).await;
    assert_eq!(restart_log.completed_count(), 0);
    
    // Simulate processing files
    for i in 1..=5 {
        let file_path = root_path.join(format!("test{}-0.txt", i));
        let aux_path = generate_aux_file_path(&file_path);
        
        // Create aux file (simulating sentence processing)
        let aux_content = format!("0\tTest file {} content.\t(1,1,1,20)\n1\tThis is a sentence.\t(1,21,1,40)\n2\tAnother sentence.\t(1,41,1,58)\n", i);
        fs::write(&aux_path, aux_content).await.unwrap();
        
        // Mark as completed
        restart_log.mark_completed(&file_path);
    }
    
    // Save restart log
    restart_log.save(root_path).await.unwrap();
    
    // Second run - should skip all files
    let restart_log = RestartLog::load(root_path).await;
    assert_eq!(restart_log.completed_count(), 5);
    
    for i in 1..=5 {
        let file_path = root_path.join(format!("test{}-0.txt", i));
        let should_process = should_process_file(&file_path, &restart_log, false, false).await.unwrap();
        assert!(!should_process, "File {} should be skipped on second run", i);
    }
    
    // Modify one file and verify it gets reprocessed
    let file_path = root_path.join("test3-0.txt");
    let modified_content = "Modified content. This is different now.";
    fs::write(&file_path, modified_content).await.unwrap();
    
    // File should still be skipped (restart log doesn't check modification time)
    let should_process = should_process_file(&file_path, &restart_log, false, false).await.unwrap();
    assert!(!should_process, "File should still be skipped based on restart log");
    
    // But with overwrite flags it should be processed
    let should_process = should_process_file(&file_path, &restart_log, true, false).await.unwrap();
    assert!(should_process, "File should be processed with overwrite_all=true");
    
    // Check restart log file structure
    let restart_log_path = root_path.join(".seams_restart.json");
    let log_content = fs::read_to_string(&restart_log_path).await.unwrap();
    let log_data: serde_json::Value = serde_json::from_str(&log_content).unwrap();
    
    assert!(log_data["completed_files"].is_array(), "completed_files should be an array");
    assert!(log_data["last_updated"].is_number(), "last_updated should be a number");
    assert_eq!(log_data["completed_files"].as_array().unwrap().len(), 5, "Should have 5 completed files");
}

/// Test restart log with concurrent operations
#[tokio::test]
async fn test_restart_log_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create test files
    let file_paths: Vec<PathBuf> = (1..=10)
        .map(|i| {
            let file_path = root_path.join(format!("test{}-0.txt", i));
            file_path
        })
        .collect();
    
    // Create files concurrently
    let create_futures: Vec<_> = file_paths.iter().map(|path| {
        let content = format!("Content for {}", path.display());
        async move {
            fs::write(path, content).await.unwrap();
        }
    }).collect();
    
    futures::future::join_all(create_futures).await;
    
    // Create restart log and mark files as completed
    let mut restart_log = RestartLog::load(root_path).await;
    
    for path in &file_paths {
        let aux_path = generate_aux_file_path(path);
        fs::write(&aux_path, "aux content").await.unwrap();
        restart_log.mark_completed(path);
    }
    
    // Save and reload
    restart_log.save(root_path).await.unwrap();
    let loaded_log = RestartLog::load(root_path).await;
    
    assert_eq!(loaded_log.completed_count(), 10);
    
    // Check all files are marked as completed
    for path in &file_paths {
        assert!(loaded_log.is_completed(path), "File {} should be completed", path.display());
    }
    
    // Test bulk operations
    let mut bulk_log = RestartLog::load(root_path).await;
    let path_refs: Vec<_> = file_paths.iter().map(|p| p.as_path()).collect();
    bulk_log.append_completed_files(&path_refs).await.unwrap();
    
    assert_eq!(bulk_log.completed_count(), 10);
}