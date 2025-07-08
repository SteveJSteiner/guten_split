use seams::sentence_detector;
use tempfile::TempDir;
use std::fs;
use std::sync::OnceLock;

// WHY: Single shared detector instance reduces test overhead from multiple instantiations
static SHARED_DETECTOR: OnceLock<sentence_detector::dialog_detector::SentenceDetectorDialog> = OnceLock::new();

fn get_detector() -> &'static sentence_detector::dialog_detector::SentenceDetectorDialog {
    SHARED_DETECTOR.get_or_init(|| sentence_detector::dialog_detector::SentenceDetectorDialog::new().unwrap())
}

/// Test sentence detection pipeline on simple text
#[tokio::test]
async fn test_sentence_detection_simple() {
    let detector = get_detector();
    
    let text = "Hello world. This is a test. How are you?";
    let sentences = detector.detect_sentences_borrowed(text)
        .expect("Sentence detection should succeed");
    
    assert_eq!(sentences.len(), 3);
    assert_eq!(sentences[0].normalize(), "Hello world.");
    assert_eq!(sentences[1].normalize(), "This is a test.");
    assert_eq!(sentences[2].normalize(), "How are you?");
}

/// Test sentence detection with abbreviations and punctuation
#[tokio::test]
async fn test_sentence_detection_punctuation() {
    let detector = get_detector();
    
    let text = "Dr. Smith went to the U.S.A. yesterday. He said \"Hello there!\" to Mr. Jones.";
    let sentences = detector.detect_sentences_borrowed(text)
        .expect("Sentence detection should succeed");
    
    assert_eq!(sentences.len(), 2);
    assert!(sentences[0].normalize().contains("Dr. Smith"));
    assert!(sentences[0].normalize().contains("U.S.A."));
    assert!(sentences[1].normalize().contains("Hello there!"));
}

/// Test file discovery and processing with minimal setup
#[tokio::test]
async fn test_file_discovery_and_processing() {
    use seams::{discovery};
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root_path = temp_dir.path();
    
    // Create test files directly
    let file1_path = root_path.join("book1-0.txt");
    let file2_path = root_path.join("book2-0.txt");
    
    fs::write(&file1_path, "Hello world. This is a test.").expect("Failed to write file1");
    fs::write(&file2_path, "Another test. How are you?").expect("Failed to write file2");
    
    let files = discovery::find_gutenberg_files(root_path).await
        .expect("Discovery should succeed");
    
    assert_eq!(files.len(), 2);
    
    // Process all files with shared detector
    let detector = get_detector();
    
    for file_path in files {
        let content = std::fs::read_to_string(&file_path)
            .expect("File reading should succeed");
        let sentences = detector.detect_sentences_borrowed(&content)
            .expect("Sentence detection should succeed");
        
        assert!(!sentences.is_empty(), "File {file_path:?} should produce sentences");
    }
}