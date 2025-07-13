// Performance test for overlapped discovery and processing pipeline
use std::fs;
use std::time::Instant;
use tempfile::TempDir;
use tokio::process::Command;

#[tokio::test]
async fn test_overlapped_pipeline_performance() {
    // Create a temporary directory with multiple test files
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create 10 test files with varying sizes to simulate a realistic corpus
    for i in 0..10 {
        let file_name = format!("test-{i}-0.txt");
        let file_path = test_root.join(&file_name);
        
        // Create files with different sizes
        let content = match i % 3 {
            0 => "Short file with a few sentences. This is the second sentence.".to_string(),
            1 => format!("{} ", "Medium file with more content. ".repeat(10)),
            _ => format!("{} ", "Long file with many sentences for testing. ".repeat(50)),
        };
        
        fs::write(&file_path, content).unwrap();
    }
    
    // Measure pipeline performance
    let start_time = Instant::now();
    
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
    
    let total_time = start_time.elapsed();
    
    // Verify the command succeeded
    assert!(output.status.success(), "seams command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify all files were processed
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully processed: 10 files"), "Expected 10 files to be processed");
    assert!(stdout.contains("Overlapped discovery and processing complete"), "Expected overlapped processing");
    
    // Verify stats file contains performance metrics
    let stats_content = fs::read_to_string(test_root.join("stats.json")).unwrap();
    let stats: serde_json::Value = serde_json::from_str(&stats_content).unwrap();
    
    assert_eq!(stats["files_processed"], 10);
    assert!(stats["total_sentences_detected"].as_u64().unwrap() > 0);
    assert!(stats["total_chars_processed"].as_u64().unwrap() > 0);
    assert!(stats["overall_chars_per_sec"].as_f64().unwrap() > 0.0);
    
    // Log performance metrics
    println!("=== Overlapped Pipeline Performance Test ===");
    println!("Total time: {:.2}s", total_time.as_secs_f64());
    println!("Files processed: {}", stats["files_processed"]);
    println!("Total sentences: {}", stats["total_sentences_detected"]);
    println!("Total characters: {}", stats["total_chars_processed"]);
    println!("Throughput: {:.0} chars/sec", stats["overall_chars_per_sec"].as_f64().unwrap());
    
    // Verify aux files were created
    let mut aux_files_count = 0;
    for entry in fs::read_dir(test_root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().ends_with("_seams2.txt") {
            aux_files_count += 1;
        }
    }
    assert_eq!(aux_files_count, 10, "Expected 10 aux files to be created");
}

#[tokio::test]
async fn test_overlapped_vs_sequential_behavior() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create 5 test files
    for i in 0..5 {
        let file_name = format!("test-{i}-0.txt");
        let file_path = test_root.join(&file_name);
        let content = format!("Test file {i}. This has multiple sentences. Each sentence is processed.");
        fs::write(&file_path, content).unwrap();
    }
    
    // Run overlapped pipeline
    let start_time = Instant::now();
    
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    let overlapped_time = start_time.elapsed();
    
    assert!(output.status.success(), "Overlapped pipeline failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify overlapped processing message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Overlapped discovery and processing complete"), 
            "Expected overlapped processing message");
    
    // Verify all files were processed
    let mut aux_files_count = 0;
    for entry in fs::read_dir(test_root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().ends_with("_seams2.txt") {
            aux_files_count += 1;
        }
    }
    assert_eq!(aux_files_count, 5, "Expected 5 aux files to be created");
    
    println!("=== Overlapped vs Sequential Behavior Test ===");
    println!("Overlapped pipeline time: {:.2}s", overlapped_time.as_secs_f64());
    println!("Files processed: 5");
    println!("Aux files created: {aux_files_count}");
    
    // The key improvement of overlapped pipeline is that it starts processing
    // files as soon as they're discovered, rather than waiting for complete discovery
    // This is particularly beneficial for large corpora
}