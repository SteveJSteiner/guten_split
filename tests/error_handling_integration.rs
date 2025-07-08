// use std::path::PathBuf;
use seams::{discovery, reader, sentence_detector};

#[path = "integration/mod.rs"]
mod test_utils;
use test_utils::{TestFixture};

/// Test pipeline behavior with malformed UTF-8 files
#[tokio::test]
async fn test_pipeline_invalid_utf8() {
    let fixture = TestFixture::new();
    
    // Create file with invalid UTF-8 bytes
    let invalid_path = fixture.root_path.join("invalid-0.txt");
    std::fs::write(&invalid_path, [0xFF, 0xFE, 0xFD]).expect("Failed to write invalid UTF-8 file");
    
    // Discovery should exclude invalid UTF-8 files
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    
    // Should find no valid files
    assert_eq!(files.len(), 0, "Invalid UTF-8 files should be excluded");
}

/// Test pipeline with permission denied scenarios
#[tokio::test]
async fn test_pipeline_permission_denied() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_gutenberg_file("restricted-0.txt", "Test content.");
    
    // Remove read permissions (Unix-specific test)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&file_path, perms).unwrap();
        
        // Discovery should handle permission errors gracefully
        let files = discovery::find_gutenberg_files(&fixture.root_path).await
            .expect("Discovery should handle permission errors");
        
        // Should find no accessible files
        assert_eq!(files.len(), 0, "Inaccessible files should be excluded");
        
        // Restore permissions for cleanup
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&file_path, perms).unwrap();
    }
}

/// Test pipeline with empty files
#[tokio::test]
async fn test_pipeline_empty_files() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_gutenberg_file("empty-0.txt", "");
    
    // Discovery should find the file
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    assert_eq!(files.len(), 1);
    
    // Reader should handle empty file
    let content = reader::read_file_async(&file_path).await
        .expect("Reading empty file should succeed");
    assert_eq!(content, "");
    
    // Sentence detector should handle empty content
    let detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection on empty content should succeed");
    
    assert_eq!(sentences.len(), 0, "Empty file should produce no sentences");
}

/// Test pipeline with files containing only whitespace
#[tokio::test]
async fn test_pipeline_whitespace_only() {
    let fixture = TestFixture::new();
    let whitespace_content = "   \n\t  \r\n   ";
    let file_path = fixture.create_gutenberg_file("whitespace-0.txt", whitespace_content);
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    assert_eq!(files.len(), 1);
    
    let content = reader::read_file_async(&file_path).await
        .expect("Reading whitespace file should succeed");
    
    let detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection on whitespace should succeed");
    
    assert_eq!(sentences.len(), 0, "Whitespace-only file should produce no sentences");
}

/// Test pipeline with files containing only punctuation
#[tokio::test]
async fn test_pipeline_punctuation_only() {
    let fixture = TestFixture::new();
    let punct_content = "...!?!...";
    let file_path = fixture.create_gutenberg_file("punct-0.txt", punct_content);
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    assert_eq!(files.len(), 1);
    
    let content = reader::read_file_async(&file_path).await
        .expect("Reading punctuation file should succeed");
    
    let detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection on punctuation should succeed");
    
    // Should handle gracefully - might produce sentences depending on FST rules
    assert!(sentences.len() <= 3, "Punctuation-only should produce few or no sentences");
}

/// Test pipeline with non-matching filename patterns
#[tokio::test]
async fn test_pipeline_non_matching_files() {
    let fixture = TestFixture::new();
    
    // Create files that shouldn't match the pattern
    fixture.create_gutenberg_file("book-1.txt", "This should not match.");
    fixture.create_gutenberg_file("book.txt", "This should not match either.");
    fixture.create_gutenberg_file("readme.md", "Not a Gutenberg file.");
    
    // Create one matching file
    fixture.create_gutenberg_file("valid-0.txt", "This should match.");
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    
    // Should only find the matching file
    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().unwrap().to_str().unwrap() == "valid-0.txt");
}

/// Test pipeline with nested directory structures
#[tokio::test]
async fn test_pipeline_nested_directories() {
    let fixture = TestFixture::new();
    
    // Create files in nested structure
    fixture.create_gutenberg_file("level1/book-0.txt", "First level.");
    fixture.create_gutenberg_file("level1/level2/book-0.txt", "Second level.");
    fixture.create_gutenberg_file("level1/level2/level3/deep-0.txt", "Deep nesting.");
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    
    assert_eq!(files.len(), 3, "Should find files in all nested directories");
    
    // Verify each file can be processed
    let detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    
    for file_path in files {
        let content = reader::read_file_async(&file_path).await
            .expect("File reading should succeed");
        let sentences = detector.detect_sentences(&content)
            .expect("Sentence detection should succeed");
        
        assert_eq!(sentences.len(), 1, "Each test file should have one sentence");
    }
}

/// Test pipeline with very long file paths
#[tokio::test]
async fn test_pipeline_long_paths() {
    let fixture = TestFixture::new();
    
    // Create a deeply nested structure with long names
    let long_path = "very_long_directory_name/another_very_long_directory_name/yet_another_long_name/final_directory";
    let file_path_str = format!("{long_path}/extremely_long_filename_that_tests_path_limits-0.txt");
    
    fixture.create_gutenberg_file(&file_path_str, "Content in deeply nested file.");
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should handle long paths");
    assert_eq!(files.len(), 1);
    
    let content = reader::read_file_async(&files[0]).await
        .expect("Should read file with long path");
    
    let detector = sentence_detector::dialog_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection should succeed");
    
    assert_eq!(sentences.len(), 1);
    assert_eq!(sentences[0].normalized_content, "Content in deeply nested file.");
}