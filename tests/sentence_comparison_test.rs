// Compare sentence detection approaches on full Project Gutenberg files
use rs_sft_sentences::sentence_detector::{SentenceDetector, SentenceDetectorDFA};
use std::fs;

// Import dialog state machine
mod dialog_state_machine_exploration {
    include!("dialog_state_machine_exploration.rs");
}

#[test]
fn compare_sentence_detection_approaches() {
    // Use a single Project Gutenberg file for detailed comparison
    let file_path = "/Users/stevejs/gutenberg_texts/1/3/4/1342/1342-0.txt";
    let full_text = fs::read_to_string(file_path).expect("Failed to read test file");
    
    println!("=== Analyzing: {} ===", file_path);
    println!("Total characters: {}", full_text.chars().count());
    println!("Total bytes: {}", full_text.len());
    
    // Run all three approaches on FULL file
    let manual_detector = SentenceDetector::with_default_rules().unwrap();
    let dfa_detector = SentenceDetectorDFA::new().unwrap();
    let dialog_machine = dialog_state_machine_exploration::DialogStateMachine::new().unwrap();
    
    println!("\nDetecting sentences...");
    let manual_result = manual_detector.detect_sentences(&full_text).unwrap();
    let dfa_result = dfa_detector.detect_sentences(&full_text).unwrap();
    let dialog_result = dialog_machine.detect_sentences(&full_text).unwrap();
    
    println!("\n=== Detection Results ===");
    println!("Manual FST: {} sentences", manual_result.len());
    println!("DFA: {} sentences", dfa_result.len());
    println!("Dialog State Machine: {} sentences", dialog_result.len());
    
    // Calculate averages
    let manual_avg = if manual_result.is_empty() { 0.0 } else {
        manual_result.iter().map(|s| s.normalized_content.chars().count()).sum::<usize>() as f64 / manual_result.len() as f64
    };
    let dfa_avg = if dfa_result.is_empty() { 0.0 } else {
        dfa_result.iter().map(|s| s.normalized_content.chars().count()).sum::<usize>() as f64 / dfa_result.len() as f64
    };
    let dialog_avg = if dialog_result.is_empty() { 0.0 } else {
        dialog_result.iter().map(|s| s.content.chars().count()).sum::<usize>() as f64 / dialog_result.len() as f64
    };
    
    println!("\nAverage sentence lengths:");
    println!("  Manual FST: {:.1} characters", manual_avg);
    println!("  DFA: {:.1} characters", dfa_avg);
    println!("  Dialog State Machine: {:.1} characters", dialog_avg);
    
    // Take a specific text segment and show how each method processes it
    let text_start = full_text.len() / 3;  // Start from middle third
    let text_end = (text_start + 5000).min(full_text.len());  // 5000 character sample
    let sample_text = &full_text[text_start..text_end];
    
    // Run each method on the SAME text sample
    let sample_manual = manual_detector.detect_sentences(sample_text).unwrap();
    let sample_dfa = dfa_detector.detect_sentences(sample_text).unwrap();
    let sample_dialog = dialog_machine.detect_sentences(sample_text).unwrap();
    
    // Write detailed comparison to file
    let mut output = String::new();
    output.push_str(&format!("=== Sentence Detection Comparison ===\n"));
    output.push_str(&format!("File: {}\n", file_path));
    output.push_str(&format!("Total characters: {}\n", full_text.chars().count()));
    output.push_str(&format!("Total bytes: {}\n\n", full_text.len()));
    
    output.push_str(&format!("Full File Detection Results:\n"));
    output.push_str(&format!("Manual FST: {} sentences (avg {:.1} chars)\n", manual_result.len(), manual_avg));
    output.push_str(&format!("DFA: {} sentences (avg {:.1} chars)\n", dfa_result.len(), dfa_avg));
    output.push_str(&format!("Dialog State Machine: {} sentences (avg {:.1} chars)\n\n", dialog_result.len(), dialog_avg));
    
    output.push_str(&format!("=== SAME TEXT SAMPLE COMPARISON ===\n"));
    output.push_str(&format!("Sample text (chars {}-{}, {} chars):\n", text_start, text_end, sample_text.len()));
    output.push_str(&format!("\"{}...\"\n\n", sample_text.chars().take(200).collect::<String>()));
    
    output.push_str(&format!("Sample Results:\n"));
    output.push_str(&format!("Manual FST: {} sentences\n", sample_manual.len()));
    output.push_str(&format!("DFA: {} sentences\n", sample_dfa.len()));
    output.push_str(&format!("Dialog State Machine: {} sentences\n\n", sample_dialog.len()));
    
    output.push_str(&format!("--- Manual FST Results ---\n"));
    for (i, sentence) in sample_manual.iter().enumerate() {
        output.push_str(&format!("{}. [{}] {}\n\n", 
                 i + 1, 
                 sentence.normalized_content.chars().count(),
                 sentence.normalized_content
        ));
    }
    
    output.push_str(&format!("--- DFA Results ---\n"));
    for (i, sentence) in sample_dfa.iter().enumerate() {
        output.push_str(&format!("{}. [{}] {}\n\n", 
                 i + 1, 
                 sentence.normalized_content.chars().count(),
                 sentence.normalized_content
        ));
    }
    
    output.push_str(&format!("--- Dialog State Machine Results ---\n"));
    for (i, sentence) in sample_dialog.iter().enumerate() {
        output.push_str(&format!("{}. [{}] {}\n\n", 
                 i + 1, 
                 sentence.content.chars().count(),
                 sentence.content
        ));
    }
    
    fs::write("sentence_detection_comparison.txt", output).expect("Failed to write comparison file");
    println!("Comparison written to sentence_detection_comparison.txt");
}