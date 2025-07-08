// Integration test for overlapped discovery and processing pipeline
use std::fs;
use tempfile::TempDir;
use tokio::process::Command;

#[tokio::test]
async fn test_overlapped_pipeline_functionality() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create test files with different sizes to test overlapped processing
    let small_file = test_root.join("small-0.txt");
    let medium_file = test_root.join("medium-0.txt");
    let large_file = test_root.join("large-0.txt");
    
    fs::write(&small_file, "This is a small test file. It contains a few sentences.").unwrap();
    fs::write(&medium_file, "This is a medium test file. It has more content than the small file. It contains multiple sentences to test the sentence detection. This ensures we have sufficient content for processing.").unwrap();
    fs::write(&large_file, format!("{} ", "This is a large test file with many sentences. ".repeat(100))).unwrap();
    
    // Run the seams binary with the test directory
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .arg("--stats-out")
        .arg(test_root.join("stats.json").to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    // Check that the command succeeded
    if !output.status.success() {
        eprintln!("seams command failed. stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        panic!("seams command failed");
    }
    
    // Verify stdout contains expected overlapped processing message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Overlapped discovery and processing complete") || stdout.contains("File discovery complete"), 
            "Expected overlapped processing message not found in stdout: {stdout}");

    // Verify aux files were created
    let aux_files = vec![
        test_root.join("small-0_seams.txt"),
        test_root.join("medium-0_seams.txt"),
        test_root.join("large-0_seams.txt"),
    ];
    
    for aux_file in &aux_files {
        assert!(aux_file.exists(), "Aux file not created: {}", aux_file.display());
        
        // Verify aux file has content
        let aux_content = fs::read_to_string(aux_file).unwrap();
        assert!(!aux_content.is_empty(), "Aux file is empty: {}", aux_file.display());
        
        // Verify aux file format (tab-separated values)
        let lines: Vec<&str> = aux_content.lines().collect();
        assert!(!lines.is_empty(), "Aux file has no lines: {}", aux_file.display());
        
        for line in lines {
            let parts: Vec<&str> = line.split('\t').collect();
            assert!(parts.len() >= 3, "Aux file line doesn't have expected format: {line}");
        }
    }
    
    // Verify stats file was created
    let stats_file = test_root.join("stats.json");
    assert!(stats_file.exists(), "Stats file not created");
    
    // Verify stats file has expected content
    let stats_content = fs::read_to_string(&stats_file).unwrap();
    let stats: serde_json::Value = serde_json::from_str(&stats_content).unwrap();
    
    assert_eq!(stats["files_processed"], 3, "Expected 3 files to be processed");
    assert!(stats["total_sentences_detected"].as_u64().unwrap() > 0, "Expected sentences to be detected");
    assert!(stats["total_chars_processed"].as_u64().unwrap() > 0, "Expected characters to be processed");
}

#[tokio::test]
async fn test_overlapped_pipeline_with_cache() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create a test file
    let test_file = test_root.join("test-0.txt");
    fs::write(&test_file, "This is a test file for cache testing. It has sentences.").unwrap();
    
    // Run seams first time
    let output1 = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    assert!(output1.status.success(), "First seams run failed: {}", String::from_utf8_lossy(&output1.stderr));
    
    // Verify aux file was created
    let aux_file = test_root.join("test-0_seams.txt");
    assert!(aux_file.exists(), "Aux file not created on first run");
    
    // Run seams second time (should use cache)
    let output2 = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    assert!(output2.status.success(), "Second seams run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    // Verify cache was used (should mention cached discovery or skipped files)
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("cached") || stdout2.contains("Skipped"), 
            "Expected cache usage message not found in stdout: {stdout2}");
}

#[tokio::test]
async fn test_overlapped_pipeline_error_handling() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create a valid test file
    let valid_file = test_root.join("valid-0.txt");
    fs::write(&valid_file, "This is a valid test file.").unwrap();
    
    // Create an invalid UTF-8 file
    let invalid_file = test_root.join("invalid-0.txt");
    fs::write(&invalid_file, [0xFF, 0xFE, 0xFD]).unwrap();
    
    // Run seams (should process valid file and report invalid file)
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    assert!(output.status.success(), "seams command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify valid file was processed
    let aux_file = test_root.join("valid-0_seams.txt");
    assert!(aux_file.exists(), "Aux file not created for valid file");
    
    // Verify stdout reports the issue with invalid file
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Files with issues"), "Expected error reporting message not found");
}