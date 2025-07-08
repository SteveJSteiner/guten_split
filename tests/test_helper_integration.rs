// Test to ensure test-helpers feature functions are actually used  
// WHY: Uses public API directly to test incremental functionality

#[cfg(feature = "test-helpers")]
#[test]
fn test_public_api_integration() {
    use seams::incremental::{create_complete_aux_file, generate_aux_file_path};
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let source_file = temp_dir.path().join("test-0.txt");
    
    // Create a test file
    fs::write(&source_file, "Sample content.").expect("Failed to write test file");
    
    // Test aux file operations using public API directly
    let aux_content = "0\tSample content.\t(1,1,1,15)\n";
    let aux_path = create_complete_aux_file(&source_file, aux_content).expect("Failed to create aux file");
    
    assert!(aux_path.exists());
    
    let read_content = std::fs::read_to_string(generate_aux_file_path(&source_file)).expect("Failed to read aux file");
    assert_eq!(read_content, aux_content);
}