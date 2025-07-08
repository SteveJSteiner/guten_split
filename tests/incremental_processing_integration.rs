// Integration tests for incremental processing behavior (F-9)
// WHY: Validates that aux file existence and completeness checking works as expected

#![cfg(feature = "test-helpers")]

use std::process::Command;
use seams::incremental::{aux_file_exists, create_complete_aux_file, generate_aux_file_path};

mod integration;
use integration::TestFixture;

/// Test that complete aux files are skipped on second run
#[tokio::test]
async fn test_skip_complete_aux_files() {
    let fixture = TestFixture::new();
    
    // Create a test file
    let source_content = "This is a test sentence. This is another sentence.";
    let source_path = fixture.create_gutenberg_file("test-0.txt", source_content);
    
    // First run - should process the file
    let output1 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run first command");
    
    assert!(output1.status.success(), "First run failed: {}", String::from_utf8_lossy(&output1.stderr));
    
    // Verify aux file and restart log were created using public API
    
    assert!(aux_file_exists(&source_path), "Aux file should exist after first run");
    let restart_log_path = fixture.root_path.join(".seams_restart.json");
    assert!(restart_log_path.exists(), "Restart log should exist after first run");
    
    let aux_content = std::fs::read_to_string(generate_aux_file_path(&source_path)).expect("Should be able to read aux file");
    let restart_log_content = std::fs::read_to_string(&restart_log_path).expect("Should be able to read restart log");
    
    // Second run - should skip the file
    let output2 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run second command");
    
    assert!(output2.status.success(), "Second run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Skipped (complete aux files): 1 files"), 
           "Second run should report 1 skipped file, stdout: {stdout2}");
    
    // Verify aux file is unchanged and restart log still exists
    let aux_content_after = std::fs::read_to_string(generate_aux_file_path(&source_path)).expect("Should be able to read aux file after second run");
    assert_eq!(aux_content, aux_content_after, "Aux file content should be unchanged");
    
    let restart_log_content_after = std::fs::read_to_string(&restart_log_path).expect("Restart log should still exist after second run");
    assert_eq!(restart_log_content, restart_log_content_after, "Restart log content should be unchanged");
}

/// Test that aux files without cache entries are processed
#[tokio::test] 
async fn test_process_aux_files_missing_from_cache() {
    let fixture = TestFixture::new();
    
    // Create a test file
    let source_content = "This is a test sentence. This is another sentence.";
    let source_path = fixture.create_gutenberg_file("test-0.txt", source_content);
    
    // Create an aux file but no cache entry (simulates partial or external creation)
    
    let aux_content = "0\t32\tThis is a test sentence.\n";
    create_complete_aux_file(&source_path, aux_content).expect("Failed to create aux file");
    
    // Verify aux file exists but no restart log
    assert!(aux_file_exists(&source_path), "Aux file should exist");
    let restart_log_path = fixture.root_path.join(".seams_restart.json");
    assert!(!restart_log_path.exists(), "Restart log should not exist initially");
    
    // Run processing - should process file since it's not in restart log
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::process::Command::new("cargo")
            .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
            .output()
    )
    .await
    .expect("Command timed out after 30 seconds")
    .expect("Failed to run command");
    
    assert!(output.status.success(), "Run failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully processed: 1 files"), 
           "Should report 1 processed file, stdout: {stdout}");
    
    // Verify restart log was created and aux file was regenerated
    assert!(restart_log_path.exists(), "Restart log should exist after processing");
    let final_aux_content = std::fs::read_to_string(generate_aux_file_path(&source_path)).expect("Should be able to read aux file");
    let lines: Vec<&str> = final_aux_content.lines().collect();
    
    // Should have multiple sentences detected
    assert!(lines.len() >= 2, "Complete aux file should have multiple sentences");
    assert!(final_aux_content.ends_with('\n'), "Complete aux file should end with newline");
}

/// Test that --overwrite_all flag forces reprocessing
#[tokio::test]
async fn test_overwrite_all_flag() {
    let fixture = TestFixture::new();
    
    // Create a test file
    let source_content = "This is a test sentence. This is another sentence.";
    let source_path = fixture.create_gutenberg_file("test-0.txt", source_content);
    
    // First run - should process the file
    let output1 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run first command");
    
    assert!(output1.status.success(), "First run failed: {}", String::from_utf8_lossy(&output1.stderr));
    
    // Verify aux file was created
    assert!(aux_file_exists(&source_path), "Aux file should exist after first run");
    
    // Second run with --overwrite-all - should process the file again
    let output2 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", "--overwrite-all", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run second command");
    
    assert!(output2.status.success(), "Second run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Successfully processed: 1 files"), 
           "Second run with --overwrite-all should report 1 processed file, stdout: {stdout2}");
    assert!(!stdout2.contains("Skipped"), 
           "Second run with --overwrite-all should not report any skipped files, stdout: {stdout2}");
}

/// Test that deleted aux files are regenerated even if in cache  
#[tokio::test]
async fn test_deleted_aux_files_regenerated() {
    let fixture = TestFixture::new();
    
    // Create a test file
    let source_content = "This is a test sentence.";
    let source_path = fixture.create_gutenberg_file("test-0.txt", source_content);
    
    // First run to create aux file and cache
    let output1 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run first command");
    
    assert!(output1.status.success(), "First run failed: {}", String::from_utf8_lossy(&output1.stderr));
    assert!(aux_file_exists(&source_path), "Aux file should exist after first run");
    let restart_log_path = fixture.root_path.join(".seams_restart.json");
    assert!(restart_log_path.exists(), "Restart log should exist after first run");
    
    // Delete the aux file but keep the cache
    let aux_path = fixture.generate_aux_file_path(&source_path);
    std::fs::remove_file(&aux_path).expect("Failed to delete aux file");
    assert!(!aux_file_exists(&source_path), "Aux file should be deleted");
    
    // Second run - should detect missing aux file and regenerate it
    let output2 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run second command");
    
    assert!(output2.status.success(), "Second run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Successfully processed: 1 files"), 
           "Should report 1 processed file when aux file is missing, stdout: {stdout2}");
    
    // Verify aux file was regenerated
    assert!(aux_file_exists(&source_path), "Aux file should be regenerated");
    let aux_content = std::fs::read_to_string(generate_aux_file_path(&source_path)).expect("Should be able to read regenerated aux file");
    assert!(!aux_content.is_empty(), "Regenerated aux file should have content");
    assert!(aux_content.ends_with('\n'), "Regenerated aux file should end with newline");
}

/// Test multiple files with mixed incremental states using cache
#[tokio::test]
async fn test_mixed_incremental_states() {
    let fixture = TestFixture::new();
    
    // Create multiple test files
    let content1 = "First file sentence.";
    let content2 = "Second file sentence.";
    let content3 = "Third file sentence.";
    
    let path1 = fixture.create_gutenberg_file("file1-0.txt", content1);
    let path2 = fixture.create_gutenberg_file("file2-0.txt", content2);
    let path3 = fixture.create_gutenberg_file("file3-0.txt", content3);
    
    // First run - process all files to create cache
    let output1 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run first command");
    
    assert!(output1.status.success(), "First run failed: {}", String::from_utf8_lossy(&output1.stderr));
    
    // Verify all files were processed and cache was created
    assert!(aux_file_exists(&path1), "File1 aux should exist after first run");
    assert!(aux_file_exists(&path2), "File2 aux should exist after first run");
    assert!(aux_file_exists(&path3), "File3 aux should exist after first run");
    let restart_log_path = fixture.root_path.join(".seams_restart.json");
    assert!(restart_log_path.exists(), "Restart log should exist after first run");
    
    // Delete aux file for file3 to test regeneration
    let aux3_path = fixture.generate_aux_file_path(&path3);
    std::fs::remove_file(&aux3_path).expect("Failed to delete file3 aux");
    
    // Second run - should skip file1 and file2 (both in restart log with aux files), and regenerate file3 (missing aux)
    let output2 = Command::new("cargo")
        .args(["run", "--bin", "seams", "--", fixture.root_path.to_str().unwrap()])
        .output()
        .expect("Failed to run second command");
    
    assert!(output2.status.success(), "Second run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Successfully processed: 1 files"), 
           "Should report 1 processed file (file3), stdout: {stdout2}");
    assert!(stdout2.contains("Skipped (complete aux files): 2 files"), 
           "Should report 2 skipped files (file1 and file2), stdout: {stdout2}");
    
    // Verify all aux files exist 
    assert!(aux_file_exists(&path1), "File1 aux should still exist");
    assert!(aux_file_exists(&path2), "File2 aux should still exist");
    assert!(aux_file_exists(&path3), "File3 aux should be regenerated");
    
    let aux1 = std::fs::read_to_string(generate_aux_file_path(&path1)).expect("Should read file1 aux");
    let aux2 = std::fs::read_to_string(generate_aux_file_path(&path2)).expect("Should read file2 aux");
    let aux3 = std::fs::read_to_string(generate_aux_file_path(&path3)).expect("Should read file3 aux");
    
    assert!(aux1.ends_with('\n'), "File1 aux should end with newline");
    assert!(aux2.ends_with('\n'), "File2 aux should end with newline");
    assert!(aux3.ends_with('\n'), "File3 aux should end with newline");
    
    // File2 should contain the original content (not reprocessed)
    assert!(aux2.contains("Second file sentence"), "File2 aux should contain original content");
}