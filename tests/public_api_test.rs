// Comprehensive tests for public incremental API functions
// WHY: Public API functions must be tested to ensure they work correctly for external users

use seams::incremental::{
    generate_aux_file_path, aux_file_exists, create_complete_aux_file,
    generate_cache_path, cache_exists, read_cache, read_cache_async
};
use tempfile::TempDir;

#[test]
fn test_aux_file_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let source_file = temp_dir.path().join("book-0.txt");
    std::fs::write(&source_file, "Sample book content").expect("Failed to write source file");
    
    // Test aux file path generation
    let aux_path = generate_aux_file_path(&source_file);
    assert!(aux_path.to_string_lossy().ends_with("book-0_seams.txt"));
    
    // Initially aux file should not exist
    assert!(!aux_file_exists(&source_file), "Aux file should not exist initially");
    
    // Create aux file with content
    let aux_content = "0\tSample sentence.\t(1,1,1,16)\n1\tAnother sentence.\t(1,18,1,34)\n";
    let created_path = create_complete_aux_file(&source_file, aux_content)
        .expect("Failed to create aux file");
    assert_eq!(created_path, aux_path, "Created path should match generated path");
    
    // Now aux file should exist
    assert!(aux_file_exists(&source_file), "Aux file should exist after creation");
    
    // Read aux file content
    let read_content = std::fs::read_to_string(&aux_path)
        .expect("Failed to read aux file");
    assert_eq!(read_content, aux_content, "Read content should match written content");
    
    // Test content with trailing newline handling
    let content_without_newline = "0\tTest.\t(1,1,1,5)";
    create_complete_aux_file(&source_file, content_without_newline)
        .expect("Failed to create aux file without newline");
    let read_content_with_newline = std::fs::read_to_string(&aux_path)
        .expect("Failed to read aux file");
    assert!(read_content_with_newline.ends_with('\n'), "Content should have trailing newline");
    assert_eq!(read_content_with_newline, format!("{content_without_newline}\n"));
}

#[test]
fn test_cache_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test cache path generation
    let cache_path = generate_cache_path(temp_dir.path());
    assert!(cache_path.to_string_lossy().ends_with(".seams_cache.json"));
    
    // Initially cache should not exist
    assert!(!cache_exists(temp_dir.path()), "Cache should not exist initially");
    
    // Attempt to read non-existent cache should fail
    let read_result = read_cache(temp_dir.path());
    assert!(read_result.is_err(), "Reading non-existent cache should fail");
    
    // Create cache file manually
    let cache_content = r#"{"completed_files":{"test-0.txt":1234567890}}"#;
    std::fs::write(&cache_path, cache_content).expect("Failed to write cache file");
    
    // Now cache should exist
    assert!(cache_exists(temp_dir.path()), "Cache should exist after creation");
    
    // Read cache content
    let read_content = read_cache(temp_dir.path())
        .expect("Failed to read cache");
    assert_eq!(read_content, cache_content, "Read cache content should match written content");
}

#[tokio::test]
async fn test_async_cache_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test async cache reading with non-existent file
    let read_result = read_cache_async(temp_dir.path()).await;
    assert!(read_result.is_err(), "Reading non-existent cache should fail");
    
    // Create cache file and test async reading
    let cache_content = r#"{"completed_files":{"async-test-0.txt":9876543210}}"#;
    let cache_path = generate_cache_path(temp_dir.path());
    tokio::fs::write(&cache_path, cache_content).await
        .expect("Failed to write cache file");
    
    // Read cache content asynchronously
    let read_content = read_cache_async(temp_dir.path()).await
        .expect("Failed to read cache asynchronously");
    assert_eq!(read_content, cache_content, "Async read content should match written content");
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let non_existent_source = temp_dir.path().join("does-not-exist-0.txt");
    
    // Reading non-existent aux file should fail gracefully
    let aux_path = generate_aux_file_path(&non_existent_source);
    let read_result = std::fs::read_to_string(&aux_path);
    assert!(read_result.is_err(), "Reading non-existent aux file should fail");
    
    // aux_file_exists should return false for non-existent files
    assert!(!aux_file_exists(&non_existent_source), "Non-existent aux file should return false");
    
    // Creating aux file for non-existent source should still work (creates the aux file)
    let aux_content = "0\tTest sentence.\t(1,1,1,14)\n";
    let create_result = create_complete_aux_file(&non_existent_source, aux_content);
    assert!(create_result.is_ok(), "Creating aux file should succeed even if source doesn't exist");
    
    // Now the aux file should exist and be readable
    assert!(aux_file_exists(&non_existent_source), "Aux file should exist after creation");
    let read_content = std::fs::read_to_string(&aux_path)
        .expect("Should be able to read created aux file");
    assert_eq!(read_content, aux_content);
}