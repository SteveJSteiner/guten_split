use seams::{discovery, reader, sentence_detector};

#[path = "integration/fixtures/mod.rs"]
mod fixtures;
use fixtures::*;

#[path = "integration/mod.rs"]
mod test_utils;
use test_utils::{TestFixture, assert_golden_file};

/// Test complete pipeline with simple single-line text
#[tokio::test]
async fn test_pipeline_simple_text() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_gutenberg_file("simple-0.txt", SIMPLE_TEXT);
    
    // Test discovery
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], file_path);
    
    // Test file reading
    let content = reader::read_file_async(&file_path).await
        .expect("File reading should succeed");
    assert_eq!(content, SIMPLE_TEXT);
    
    // Test sentence detection
    let detector = sentence_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection should succeed");
    
    // Format output according to PRD spec
    let output = format_sentences_output(&sentences);
    
    // Golden-file validation
    assert_golden_file(&output, SIMPLE_EXPECTED, "Simple text pipeline");
}

/// Test pipeline with challenging punctuation using SIMPLE rules
/// WHY: This tests current implementation limitations - simple rules don't handle abbreviations
#[tokio::test]
async fn test_pipeline_punctuation_simple_rules() {
    let fixture = TestFixture::new();
    let _file_path = fixture.create_gutenberg_file("punct-0.txt", PUNCTUATION_TEXT);
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    let content = reader::read_file_async(&files[0]).await
        .expect("File reading should succeed");
    let detector = sentence_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    let sentences = detector.detect_sentences(&content)
        .expect("Sentence detection should succeed");
    
    let output = format_sentences_output(&sentences);
    assert_golden_file(&output, PUNCTUATION_SIMPLE_EXPECTED, "Punctuation text pipeline (simple rules)");
}

/// Test multiple file discovery and processing
#[tokio::test]
async fn test_pipeline_multiple_files() {
    let fixture = TestFixture::new();
    
    // Create multiple test files
    fixture.create_gutenberg_file("book1/chapter1-0.txt", SIMPLE_TEXT);
    fixture.create_gutenberg_file("book2/chapter1-0.txt", MINIMAL_TEXT);
    
    let files = discovery::find_gutenberg_files(&fixture.root_path).await
        .expect("Discovery should succeed");
    
    assert_eq!(files.len(), 2);
    
    // Process all files
    let detector = sentence_detector::SentenceDetectorDialog::new()
        .expect("Detector creation should succeed");
    
    for file_path in files {
        let content = reader::read_file_async(&file_path).await
            .expect("File reading should succeed");
        let sentences = detector.detect_sentences(&content)
            .expect("Sentence detection should succeed");
        
        // Ensure each file produces some sentences
        assert!(!sentences.is_empty(), 
            "File {:?} should produce sentences", file_path);
    }
}

/// Helper function to format sentences according to PRD spec
/// Format: index<TAB>sentence<TAB>(start_line,start_col,end_line,end_col)
fn format_sentences_output(sentences: &[sentence_detector::DetectedSentence]) -> String {
    sentences.iter()
        .enumerate()
        .map(|(i, sentence)| {
            format!("{}\t{}\t({},{},{},{})",
                i,
                sentence.normalized_content,
                sentence.span.start_line,
                sentence.span.start_col,
                sentence.span.end_line,
                sentence.span.end_col
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}