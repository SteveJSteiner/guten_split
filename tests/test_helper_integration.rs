// Test to ensure test-helpers feature functions are actually used  
// WHY: Uses public API directly to test incremental functionality

#[cfg(feature = "test-helpers")]
#[test]
fn test_public_api_integration() {
    use seams::incremental::{aux_file_exists, read_aux_file, create_complete_aux_file, cache_exists, read_cache};
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let source_file = temp_dir.path().join("test-0.txt");
    
    // Create a test file
    fs::write(&source_file, "Sample content.").expect("Failed to write test file");
    
    // Test aux file operations using public API directly
    assert!(!aux_file_exists(&source_file));
    
    let aux_content = "0\tSample content.\t(1,1,1,15)\n";
    let aux_path = create_complete_aux_file(&source_file, aux_content).expect("Failed to create aux file");
    
    assert!(aux_file_exists(&source_file));
    assert!(aux_path.exists());
    
    let read_content = read_aux_file(&source_file).expect("Failed to read aux file");
    assert_eq!(read_content, aux_content);
    
    // Test cache operations using public API directly
    assert!(!cache_exists(temp_dir.path()));
    
    let cache_content = r#"{"completed_files":{"test-0.txt":1234567890}}"#;
    let cache_path = temp_dir.path().join(".seams_cache.json");
    fs::write(&cache_path, cache_content).expect("Failed to write cache file");
    
    assert!(cache_exists(temp_dir.path()));
    
    let read_cache_content = read_cache(temp_dir.path()).expect("Failed to read cache");
    assert_eq!(read_cache_content, cache_content);
}