use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that --stats-out flag creates valid JSON file with correct structure
#[tokio::test]
async fn test_stats_output_json_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root_path = temp_dir.path();
    
    // Create test file
    let test_file = root_path.join("test-0.txt");
    fs::write(&test_file, "This is a test sentence. Here is another one.").expect("Failed to write test file");
    
    // Run seams with stats output
    let stats_file = root_path.join("test_stats.json");
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "seams", "--"])
        .arg(root_path.as_os_str())
        .arg("--stats-out")
        .arg(stats_file.as_os_str())
        .arg("--overwrite-all")
        .output()
        .expect("Failed to run seams");
    
    assert!(output.status.success(), "seams command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Read and validate JSON structure
    let json_content = fs::read_to_string(&stats_file)
        .expect("Failed to read stats file");
    
    let stats: Value = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON");
    
    // Validate top-level structure
    assert!(stats.is_object(), "Stats should be a JSON object");
    
    let obj = stats.as_object().unwrap();
    assert!(obj.contains_key("run_start"), "Missing run_start field");
    assert!(obj.contains_key("total_processing_time_ms"), "Missing total_processing_time_ms field");
    assert!(obj.contains_key("total_chars_processed"), "Missing total_chars_processed field");
    assert!(obj.contains_key("total_sentences_detected"), "Missing total_sentences_detected field");
    assert!(obj.contains_key("overall_chars_per_sec"), "Missing overall_chars_per_sec field");
    assert!(obj.contains_key("files_processed"), "Missing files_processed field");
    assert!(obj.contains_key("files_skipped"), "Missing files_skipped field");
    assert!(obj.contains_key("files_failed"), "Missing files_failed field");
    assert!(obj.contains_key("file_stats"), "Missing file_stats field");
    
    // Validate file_stats array
    let file_stats = obj["file_stats"].as_array().expect("file_stats should be an array");
    assert_eq!(file_stats.len(), 1, "Should have stats for 1 file");
    
    let file_stat = &file_stats[0];
    assert!(file_stat.is_object(), "File stat should be an object");
    
    let file_obj = file_stat.as_object().unwrap();
    assert!(file_obj.contains_key("path"), "Missing path field");
    assert!(file_obj.contains_key("chars_processed"), "Missing chars_processed field");
    assert!(file_obj.contains_key("sentences_detected"), "Missing sentences_detected field");
    assert!(file_obj.contains_key("processing_time_ms"), "Missing processing_time_ms field");
    assert!(file_obj.contains_key("chars_per_sec"), "Missing chars_per_sec field");
    assert!(file_obj.contains_key("status"), "Missing status field");
    assert!(file_obj.contains_key("error"), "Missing error field");
    
    // Validate field values
    assert_eq!(file_obj["status"].as_str().unwrap(), "success", "Status should be success");
    assert!(file_obj["error"].is_null(), "Error should be null for successful processing");
    assert!(file_obj["chars_processed"].as_u64().unwrap() > 0, "Should have processed some chars");
    assert!(file_obj["sentences_detected"].as_u64().unwrap() > 0, "Should have detected some sentences");
    assert!(file_obj["chars_per_sec"].as_f64().unwrap() > 0.0, "Should have positive throughput");
    
    // Validate aggregate values
    assert_eq!(obj["files_processed"].as_u64().unwrap(), 1, "Should have processed 1 file");
    assert_eq!(obj["files_skipped"].as_u64().unwrap(), 0, "Should have skipped 0 files");
    assert_eq!(obj["files_failed"].as_u64().unwrap(), 0, "Should have failed 0 files");
    assert!(obj["total_chars_processed"].as_u64().unwrap() > 0, "Should have processed some chars");
    assert!(obj["total_sentences_detected"].as_u64().unwrap() > 0, "Should have detected some sentences");
    assert!(obj["overall_chars_per_sec"].as_f64().unwrap() > 0.0, "Should have positive overall throughput");
}

/// Test that stats output works with multiple files
#[tokio::test]
async fn test_stats_output_multiple_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root_path = temp_dir.path();
    
    // Create multiple test files
    let test_file1 = root_path.join("file1-0.txt");
    let test_file2 = root_path.join("file2-0.txt");
    fs::write(&test_file1, "First file content. Multiple sentences here.").expect("Failed to write test file 1");
    fs::write(&test_file2, "Second file content. Even more sentences.").expect("Failed to write test file 2");
    
    // Run seams with stats output
    let stats_file = root_path.join("multi_stats.json");
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "seams", "--"])
        .arg(root_path.as_os_str())
        .arg("--stats-out")
        .arg(stats_file.as_os_str())
        .arg("--overwrite-all")
        .output()
        .expect("Failed to run seams");
    
    assert!(output.status.success(), "seams command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Read and validate JSON structure
    let json_content = fs::read_to_string(&stats_file)
        .expect("Failed to read stats file");
    
    let stats: Value = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON");
    
    let obj = stats.as_object().unwrap();
    let file_stats = obj["file_stats"].as_array().expect("file_stats should be an array");
    
    assert_eq!(file_stats.len(), 2, "Should have stats for 2 files");
    assert_eq!(obj["files_processed"].as_u64().unwrap(), 2, "Should have processed 2 files");
    
    // Validate that total stats are sum of individual file stats
    let total_chars_expected: u64 = file_stats.iter()
        .map(|fs| fs["chars_processed"].as_u64().unwrap())
        .sum();
    let total_sentences_expected: u64 = file_stats.iter()
        .map(|fs| fs["sentences_detected"].as_u64().unwrap())
        .sum();
    
    assert_eq!(obj["total_chars_processed"].as_u64().unwrap(), total_chars_expected, "Total chars should match sum of individual files");
    assert_eq!(obj["total_sentences_detected"].as_u64().unwrap(), total_sentences_expected, "Total sentences should match sum of individual files");
}

/// Test that stats output works with default filename
#[tokio::test]
async fn test_stats_output_default_filename() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root_path = temp_dir.path();
    
    // Create test file
    let test_file = root_path.join("test-0.txt");
    fs::write(&test_file, "Default filename test sentence.").expect("Failed to write test file");
    
    // Run seams without --stats-out (should use default in current working directory)
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "seams", "--"])
        .arg(root_path.as_os_str())
        .arg("--overwrite-all")
        .current_dir(&cwd)
        .output()
        .expect("Failed to run seams");
    
    assert!(output.status.success(), "seams command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify default stats file was created in current working directory
    let default_stats_file = cwd.join("run_stats.json");
    assert!(default_stats_file.exists(), "Default run_stats.json should be created in current directory");
    
    // Validate it's valid JSON
    let json_content = fs::read_to_string(&default_stats_file)
        .expect("Failed to read default stats file");
    
    let _stats: Value = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON from default stats file");
}

/// Test that stats output shows skipped files correctly
#[tokio::test]
async fn test_stats_output_skipped_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root_path = temp_dir.path();
    
    // Create test file
    let test_file = root_path.join("test-0.txt");
    fs::write(&test_file, "This will be processed first.").expect("Failed to write test file");
    
    // Run seams first time to create aux file
    let stats_file1 = root_path.join("first_run_stats.json");
    let output1 = Command::new("cargo")
        .args(["run", "--release", "--bin", "seams", "--"])
        .arg(root_path.as_os_str())
        .arg("--stats-out")
        .arg(stats_file1.as_os_str())
        .output()
        .expect("Failed to run seams first time");
    
    assert!(output1.status.success(), "seams first run failed: {}", String::from_utf8_lossy(&output1.stderr));
    
    // Run seams second time - should skip the file
    let stats_file2 = root_path.join("second_run_stats.json");
    let output2 = Command::new("cargo")
        .args(["run", "--release", "--bin", "seams", "--"])
        .arg(root_path.as_os_str())
        .arg("--stats-out")
        .arg(stats_file2.as_os_str())
        .output()
        .expect("Failed to run seams second time");
    
    assert!(output2.status.success(), "seams second run failed: {}", String::from_utf8_lossy(&output2.stderr));
    
    // Read second run stats
    let json_content = fs::read_to_string(&stats_file2)
        .expect("Failed to read second run stats file");
    
    let stats: Value = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON from second run");
    
    let obj = stats.as_object().unwrap();
    
    // Verify file was skipped
    assert_eq!(obj["files_processed"].as_u64().unwrap(), 0, "Should have processed 0 files");
    assert_eq!(obj["files_skipped"].as_u64().unwrap(), 1, "Should have skipped 1 file");
    assert_eq!(obj["files_failed"].as_u64().unwrap(), 0, "Should have failed 0 files");
    
    let file_stats = obj["file_stats"].as_array().expect("file_stats should be an array");
    assert_eq!(file_stats.len(), 1, "Should have stats for 1 file");
    
    let file_stat = &file_stats[0];
    let file_obj = file_stat.as_object().unwrap();
    assert_eq!(file_obj["status"].as_str().unwrap(), "skipped", "Status should be skipped");
    assert_eq!(file_obj["chars_processed"].as_u64().unwrap(), 0, "Skipped file should have 0 chars processed");
    assert_eq!(file_obj["sentences_detected"].as_u64().unwrap(), 0, "Skipped file should have 0 sentences detected");
}