// Gutenberg Sentence Generation Utility
// Processes all files in Gutenberg mirror with both Enhanced Dictionary and Dialog State Machine strategies
// Creates .norm_sents and .norm_sm_sents files next to each original file for comparison

use anyhow::Result;
use seams::discovery::{collect_discovered_files, DiscoveryConfig};
use std::path::PathBuf;
use tokio::fs;
use regex_automata::meta::Regex;

// Import dialog state machine module for comparison
use seams::sentence_detector::dialog_detector::*;

// Enhanced Dictionary Strategy - Copy from tests for sentence generation
fn detect_sentences_dictionary_enhanced(text: &str) -> Result<Vec<String>> {
    // Title abbreviations that cause false sentence boundaries when followed by proper nouns
    let title_false_positives = &[
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr."
    ];
    
    // Use separate patterns to handle each boundary type correctly
    let basic_pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    let dialog_end_pattern = Regex::new(r#"[.!?]['"\u{201D}\u{2019}]\s+[A-Z]"#).unwrap();
    let quote_start_pattern = Regex::new(r#"[.!?]\s+['"\u{201C}\u{2018}]"#).unwrap();
    let paren_start_pattern = Regex::new(r"[.!?]\s+[({\[]").unwrap();
    let dialog_to_quote_pattern = Regex::new(r#"[.!?]['"\u{201D}\u{2019}]\s+['"\u{201C}\u{2018}]"#).unwrap();
    
    // Collect all potential boundaries with their types
    let mut boundaries = Vec::new();
    
    // Find all boundary types
    for mat in basic_pattern.find_iter(text) {
        boundaries.push((mat.start(), mat.end(), "basic"));
    }
    for mat in dialog_end_pattern.find_iter(text) {
        boundaries.push((mat.start(), mat.end(), "dialog_end"));
    }
    for mat in quote_start_pattern.find_iter(text) {
        boundaries.push((mat.start(), mat.end(), "quote_start"));
    }
    for mat in paren_start_pattern.find_iter(text) {
        boundaries.push((mat.start(), mat.end(), "paren_start"));
    }
    for mat in dialog_to_quote_pattern.find_iter(text) {
        boundaries.push((mat.start(), mat.end(), "dialog_to_quote"));
    }
    
    // Sort by position and remove duplicates
    boundaries.sort_by_key(|&(start, _, _)| start);
    boundaries.dedup_by_key(|&mut (start, _, _)| start);
    
    let mut sentences = Vec::new();
    let mut last_start = 0;
    
    for (boundary_start, _boundary_end, boundary_type) in boundaries {
        // Find the sentence end position based on boundary type
        let potential_end = match boundary_type {
            "dialog_end" | "dialog_to_quote" => {
                find_dialog_end_position(text, BytePos::new(boundary_start)).0
            },
            _ => find_punctuation_end(text, BytePos::new(boundary_start)).0,
        };
        let preceding_text = &text[last_start..potential_end];
        
        // Check for title + proper noun false positives
        let is_title_false_positive = {
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                let clean_word = last_word.trim_matches(|c: char| 
                    c == '\'' || c == '"' || c == '\u{201C}' || c == '\u{201D}' || 
                    c == '\u{2018}' || c == '\u{2019}');
                title_false_positives.contains(&clean_word)
            } else {
                false
            }
        };
        
        if !is_title_false_positive {
            let sentence_text = &text[last_start..potential_end];
            sentences.push(sentence_text.trim().to_string());
            
            // Find start of next sentence based on boundary type
            last_start = match boundary_type {
                "dialog_end" | "dialog_to_quote" => {
                    find_next_sentence_start(text, BytePos::new(potential_end)).0
                },
                "quote_start" | "paren_start" => {
                    find_quote_or_paren_start(text, BytePos::new(boundary_start + 1)).0
                },
                _ => find_next_sentence_start(text, BytePos::new(boundary_start + 1)).0,
            };
        }
    }
    
    // Add final sentence if there's remaining text
    if last_start < text.len() {
        let final_text = &text[last_start..];
        if !final_text.trim().is_empty() {
            sentences.push(final_text.trim().to_string());
        }
    }
    
    Ok(sentences)
}

// Dialog State Machine Strategy - Uses the optimized Phase 1 implementation
fn detect_sentences_dialog_state_machine(text: &str) -> Result<Vec<String>> {
    let detector = SentenceDetectorDialog::new()
        .map_err(|e| anyhow::anyhow!("Failed to create sentence detector: {}", e))?;
    
    let sentences = detector.detect_sentences_borrowed(text)
        .map_err(|e| anyhow::anyhow!("Failed to detect sentences: {}", e))?;
    
    // Convert DetectedSentenceBorrowed to strings
    let sentence_strings: Vec<String> = sentences
        .into_iter()
        .map(|s| s.normalize())
        .collect();
    
    Ok(sentence_strings)
}

// Helper functions (copied from test implementation)
fn find_punctuation_end(text: &str, boundary_start: BytePos) -> BytePos {
    let mut char_indices = text[boundary_start.0..].char_indices();
    
    if let Some((_, _ch)) = char_indices.next() {
        if let Some((next_byte_offset, _)) = char_indices.next() {
            boundary_start.advance(next_byte_offset)
        } else {
            BytePos::new(text.len())
        }
    } else {
        BytePos::new(text.len())
    }
}

fn find_dialog_end_position(text: &str, boundary_start: BytePos) -> BytePos {
    let mut char_indices = text[boundary_start.0..].char_indices();
    
    // Skip the punctuation mark
    if let Some((_, _punctuation)) = char_indices.next() {
        // Skip the quote character
        if let Some((_, _quote)) = char_indices.next() {
            // Return position after the quote
            if let Some((next_byte_offset, _)) = char_indices.next() {
                boundary_start.advance(next_byte_offset)
            } else {
                BytePos::new(text.len())
            }
        } else {
            boundary_start.advance(1)
        }
    } else {
        BytePos::new(text.len())
    }
}

fn find_next_sentence_start(text: &str, start_pos: BytePos) -> BytePos {
    let char_indices = text[start_pos.0..].char_indices();
    
    for (byte_offset, ch) in char_indices {
        if !ch.is_whitespace() {
            return start_pos.advance(byte_offset);
        }
    }
    BytePos::new(text.len())
}

fn find_quote_or_paren_start(text: &str, start_pos: BytePos) -> BytePos {
    let char_indices = text[start_pos.0..].char_indices();
    
    for (byte_offset, ch) in char_indices {
        if !ch.is_whitespace() {
            return start_pos.advance(byte_offset);
        }
    }
    BytePos::new(text.len())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Generating sentence outputs for Gutenberg texts with both strategies...");
    println!("   📚 Enhanced Dictionary (.norm_sents)");
    println!("   🤖 Dialog State Machine (.norm_sm_sents)");
    
    // Get Gutenberg mirror directory
    let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{home}/gutenberg_texts")
        });
    let root_dir = PathBuf::from(mirror_dir);
    
    if !root_dir.exists() {
        eprintln!("❌ Gutenberg mirror directory {root_dir:?} does not exist");
        eprintln!("   Set GUTENBERG_MIRROR_DIR environment variable or ensure ~/gutenberg_texts exists");
        return Ok(());
    }
    
    println!("📂 Scanning directory: {root_dir:?}");
    
    // Discover all files
    let discovery_config = DiscoveryConfig::default();
    let discovered_files = collect_discovered_files(&root_dir, discovery_config).await?;
    
    // Filter to valid UTF-8 files only
    let valid_files: Vec<_> = discovered_files
        .iter()
        .filter(|f| f.error.is_none())
        .collect();
    
    println!("📊 Found {} valid UTF-8 files to process", valid_files.len());
    
    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;
    
    for file_info in &valid_files {
        let file_path = &file_info.path;
        let base_extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("txt");
        let sentences_path = file_path.with_extension(format!("{base_extension}.norm_sents"));
        let sm_sentences_path = file_path.with_extension(format!("{base_extension}.norm_sm_sents"));
        
        // Check if both files already exist
        let dict_exists = sentences_path.exists();
        let sm_exists = sm_sentences_path.exists();
        
        if dict_exists && sm_exists {
            skipped += 1;
            if processed % 50 == 0 {
                println!("⏭️  Skipping {} (both sentence files exist)", file_path.display());
            }
            continue;
        }
        
        match process_file_both_strategies(file_path, &sentences_path, &sm_sentences_path, dict_exists, sm_exists).await {
            Ok((dict_count, sm_count)) => {
                processed += 1;
                if processed % 10 == 0 {
                    println!("✅ Processed {} files... Latest: {} (dict: {}, sm: {} sentences)", 
                        processed, file_path.file_name().unwrap().to_string_lossy(), dict_count, sm_count);
                }
            }
            Err(e) => {
                errors += 1;
                eprintln!("❌ Error processing {}: {}", file_path.display(), e);
            }
        }
    }
    
    println!("\n🎉 Generation complete!");
    println!("   📄 Processed: {processed} files");
    println!("   ⏭️  Skipped: {skipped} files (already had both sentence files)");
    println!("   ❌ Errors: {errors} files");
    
    if processed > 0 {
        println!("\n💡 Sentence files created with two strategies:");
        println!("   📚 .norm_sents = Enhanced Dictionary strategy");
        println!("   🤖 .norm_sm_sents = Dialog State Machine strategy");
        println!("   Compare files to analyze dialog coalescing differences");
        
        // Show some examples
        if let Some(first_file) = valid_files.first() {
            let base_ext = first_file.path.extension().and_then(|s| s.to_str()).unwrap_or("txt");
            let dict_path = first_file.path.with_extension(format!("{base_ext}.norm_sents"));
            let sm_path = first_file.path.with_extension(format!("{base_ext}.norm_sm_sents"));
            
            if dict_path.exists() {
                println!("   📝 Example dictionary file: {}", dict_path.display());
            }
            if sm_path.exists() {
                println!("   📝 Example state machine file: {}", sm_path.display());
            }
        }
    }
    
    Ok(())
}

async fn process_file_both_strategies(
    input_path: &PathBuf, 
    dict_output_path: &PathBuf, 
    sm_output_path: &PathBuf,
    dict_exists: bool,
    sm_exists: bool
) -> Result<(usize, usize)> {
    // Read the file once
    let content = fs::read_to_string(input_path).await?;
    
    let mut dict_count = 0;
    let mut sm_count = 0;
    
    // Process with Enhanced Dictionary strategy if needed
    if !dict_exists {
        let sentences = detect_sentences_dictionary_enhanced(&content)
            .map_err(|e| anyhow::anyhow!("Failed to detect sentences with dictionary: {}", e))?;
        
        let mut output = String::new();
        for (index, sentence) in sentences.iter().enumerate() {
            let normalized = sentence.replace('\n', " ").replace('\r', "");
            output.push_str(&format!("{:<5} {:<5} {}\n", index + 1, normalized.len(), normalized));
        }
        
        fs::write(dict_output_path, output).await?;
        dict_count = sentences.len();
    }
    
    // Process with Dialog State Machine strategy if needed
    if !sm_exists {
        let sentences = detect_sentences_dialog_state_machine(&content)
            .map_err(|e| anyhow::anyhow!("Failed to detect sentences with state machine: {}", e))?;
        
        let mut output = String::new();
        for (index, sentence) in sentences.iter().enumerate() {
            let normalized = sentence.replace('\n', " ").replace('\r', "");
            output.push_str(&format!("{:<5} {:<5} {}\n", index + 1, normalized.len(), normalized));
        }
        
        fs::write(sm_output_path, output).await?;
        sm_count = sentences.len();
    }
    
    Ok((dict_count, sm_count))
}