// Compare sentence detection approaches on full Project Gutenberg files
use rs_sft_sentences::sentence_detector::{SentenceDetector, SentenceDetectorDFA};
use std::fs;

// Import dialog state machine
mod dialog_state_machine_exploration {
    include!("tests/dialog_state_machine_exploration.rs");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a single Project Gutenberg file for detailed comparison
    let file_path = "/Users/stevejs/gutenberg_texts/1/3/4/1342/1342-0.txt";
    let full_text = fs::read_to_string(file_path)?;
    
    println!("=== Analyzing: {} ===", file_path);
    println!("Total characters: {}", full_text.chars().count());
    println!("Total bytes: {}", full_text.len());
    
    // Run all three approaches on FULL file
    let manual_detector = SentenceDetector::with_default_rules()?;
    let dfa_detector = SentenceDetectorDFA::new()?;
    let dialog_machine = dialog_state_machine_exploration::DialogStateMachine::new()?;
    
    println!("\nDetecting sentences...");
    let manual_result = manual_detector.detect_sentences(&full_text)?;
    let dfa_result = dfa_detector.detect_sentences(&full_text)?;
    let dialog_result = dialog_machine.detect_sentences(&full_text)?;
    
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
    
    // Sample from middle content (skip headers)
    let manual_start = manual_result.len() / 4;
    let dfa_start = dfa_result.len() / 4;
    let dialog_start = dialog_result.len() / 4;
    
    println!("\n=== Sample Sentences from Middle Content ===");
    
    println!("\n--- Manual FST (starting at sentence {}) ---", manual_start + 1);
    for (i, sentence) in manual_result.iter().skip(manual_start).take(5).enumerate() {
        println!("{}. [{}] \"{}\"", 
                 manual_start + i + 1, 
                 sentence.normalized_content.chars().count(),
                 sentence.normalized_content
        );
        println!();
    }
    
    println!("\n--- DFA (starting at sentence {}) ---", dfa_start + 1);
    for (i, sentence) in dfa_result.iter().skip(dfa_start).take(5).enumerate() {
        println!("{}. [{}] \"{}\"", 
                 dfa_start + i + 1, 
                 sentence.normalized_content.chars().count(),
                 sentence.normalized_content
        );
        println!();
    }
    
    println!("\n--- Dialog State Machine (starting at sentence {}) ---", dialog_start + 1);
    for (i, sentence) in dialog_result.iter().skip(dialog_start).take(5).enumerate() {
        println!("{}. [{}] \"{}\"", 
                 dialog_start + i + 1, 
                 sentence.content.chars().count(),
                 sentence.content
        );
        println!();
    }
    
    Ok(())
}