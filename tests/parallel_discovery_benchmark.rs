// Benchmark test for parallel vs serial discovery performance
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

#[tokio::test]
async fn benchmark_parallel_vs_serial_discovery() {
    // Create a temporary directory with many test files
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create subdirectories to simulate realistic project structure
    let subdirs = ["books", "classics", "poetry", "novels", "essays"];
    for subdir in &subdirs {
        fs::create_dir_all(test_root.join(subdir)).unwrap();
    }
    
    // Create 50 test files across different subdirectories
    for i in 0..50 {
        let subdir = &subdirs[i % subdirs.len()];
        let file_name = format!("test-{i}-0.txt");
        let file_path = test_root.join(subdir).join(&file_name);
        
        // Vary content size to simulate realistic files
        let content = match i % 3 {
            0 => "Short content with a sentence.".to_string(),
            1 => format!("{} ", "Medium content with multiple sentences. ".repeat(20)),
            _ => format!("{} ", "Long content with many sentences for testing purposes. ".repeat(100)),
        };
        
        fs::write(&file_path, content).unwrap();
    }
    
    let config = seams::discovery::DiscoveryConfig::default();
    
    // Benchmark serial discovery
    let start_time = Instant::now();
    let serial_files = seams::discovery::collect_discovered_files(test_root, config.clone()).await.unwrap();
    let serial_time = start_time.elapsed();
    
    // Benchmark parallel discovery
    let start_time = Instant::now();
    let parallel_files = seams::discovery::collect_discovered_files_parallel(test_root, config).await.unwrap();
    let parallel_time = start_time.elapsed();
    
    // Verify same results
    assert_eq!(serial_files.len(), parallel_files.len());
    assert_eq!(serial_files.len(), 50);
    
    // Log performance comparison
    println!("=== Directory Discovery Performance Benchmark ===");
    println!("Files discovered: {}", serial_files.len());
    println!("Serial discovery time: {:.2}ms", serial_time.as_millis());
    println!("Parallel discovery time: {:.2}ms", parallel_time.as_millis());
    
    if parallel_time < serial_time {
        let improvement = serial_time.as_nanos() as f64 / parallel_time.as_nanos() as f64;
        println!("Parallel discovery is {improvement:.2}x faster");
    } else {
        println!("Serial discovery was faster (overhead on small test set)");
    }
    
    // For large file sets, parallel should be faster or comparable
    // For small file sets, serial might be faster due to overhead
    println!("Directory structure: {} subdirectories", subdirs.len());
    
    // Verify all files are valid
    let valid_serial = serial_files.iter().filter(|f| f.error.is_none()).count();
    let valid_parallel = parallel_files.iter().filter(|f| f.error.is_none()).count();
    
    assert_eq!(valid_serial, valid_parallel);
    assert_eq!(valid_serial, 50);
}

#[tokio::test]
async fn benchmark_parallel_discovery_large_directory() {
    // Create a large directory structure to better demonstrate parallel benefits
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create deep directory structure
    let depth = 5;
    let files_per_dir = 10;
    
    fn create_recursive_structure(base: &std::path::Path, depth: i32, files_per_dir: i32) -> i32 {
        let mut file_count = 0;
        
        if depth > 0 {
            // Create files in current directory
            for i in 0..files_per_dir {
                let file_name = format!("file-{i}-0.txt");
                let file_path = base.join(&file_name);
                fs::write(&file_path, format!("Content for file {i} at depth {depth}")).unwrap();
                file_count += 1;
            }
            
            // Create subdirectories and recurse
            for i in 0..3 {
                let subdir = base.join(format!("subdir-{i}"));
                fs::create_dir_all(&subdir).unwrap();
                file_count += create_recursive_structure(&subdir, depth - 1, files_per_dir);
            }
        }
        
        file_count
    }
    
    let total_files = create_recursive_structure(test_root, depth, files_per_dir);
    
    let config = seams::discovery::DiscoveryConfig::default();
    
    // Benchmark parallel discovery on large structure
    let start_time = Instant::now();
    let parallel_files = seams::discovery::collect_discovered_files_parallel(test_root, config.clone()).await.unwrap();
    let parallel_time = start_time.elapsed();
    
    // Benchmark serial discovery on large structure
    let start_time = Instant::now();
    let serial_files = seams::discovery::collect_discovered_files(test_root, config).await.unwrap();
    let serial_time = start_time.elapsed();
    
    // Verify same results
    assert_eq!(serial_files.len(), parallel_files.len());
    assert_eq!(serial_files.len(), total_files as usize);
    
    // Log performance comparison
    println!("=== Large Directory Discovery Performance Benchmark ===");
    println!("Files discovered: {}", parallel_files.len());
    println!("Directory depth: {depth}");
    println!("Serial discovery time: {:.2}ms", serial_time.as_millis());
    println!("Parallel discovery time: {:.2}ms", parallel_time.as_millis());
    
    if parallel_time < serial_time {
        let improvement = serial_time.as_nanos() as f64 / parallel_time.as_nanos() as f64;
        println!("Parallel discovery is {improvement:.2}x faster");
    } else {
        let overhead = parallel_time.as_nanos() as f64 / serial_time.as_nanos() as f64;
        println!("Serial discovery was {overhead:.2}x faster");
    }
}

#[tokio::test]
async fn test_parallel_discovery_with_overlapped_pipeline() {
    // Test that parallel discovery works correctly with the overlapped pipeline
    let temp_dir = TempDir::new().unwrap();
    let test_root = temp_dir.path();
    
    // Create test files
    for i in 0..10 {
        let file_name = format!("test-{i}-0.txt");
        let file_path = test_root.join(&file_name);
        let content = format!("Test file {i} content. This has sentences.");
        fs::write(&file_path, content).unwrap();
    }
    
    // Run the full overlapped pipeline with parallel discovery
    let start_time = Instant::now();
    
    let output = tokio::process::Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("seams")
        .arg("--")
        .arg(test_root.to_str().unwrap())
        .output()
        .await
        .expect("Failed to run seams binary");
    
    let total_time = start_time.elapsed();
    
    // Verify success
    assert!(output.status.success(), "Overlapped pipeline with parallel discovery failed: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    // Verify all files were processed
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully processed: 10 files"));
    assert!(stdout.contains("Overlapped discovery and processing complete"));
    
    // Verify aux files were created
    let mut aux_files_count = 0;
    for entry in fs::read_dir(test_root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().ends_with("_seams.txt") {
            aux_files_count += 1;
        }
    }
    assert_eq!(aux_files_count, 10);
    
    println!("=== Overlapped Pipeline with Parallel Discovery ===");
    println!("Total time: {:.2}ms", total_time.as_millis());
    println!("Files processed: 10");
    println!("Aux files created: {aux_files_count}");
}