use seams::sentence_detector::dialog_detector::SentenceDetectorDialog;
use std::fs;

fn main() {
    let input = fs::read_to_string("test_input.txt").expect("Failed to read test file");
    
    let detector = SentenceDetectorDialog::new().expect("Failed to create detector");
    let sentences = detector.detect_sentences_borrowed(&input).expect("Failed to detect sentences");
    
    println!("Input text split into {} sentences:", sentences.len());
    println!();
    
    for (i, sentence) in sentences.iter().enumerate() {
        println!("Sentence {}: {:?}", i + 1, sentence.raw_content);
        println!("  Span: ({},{}) to ({},{})", 
                 sentence.span.start_line, sentence.span.start_col,
                 sentence.span.end_line, sentence.span.end_col);
        println!();
    }
}