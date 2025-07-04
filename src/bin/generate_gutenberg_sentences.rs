// Gutenberg Sentence Generation Utility
// Processes all files in Gutenberg mirror with Enhanced Dictionary strategy
// Creates .sentences files next to each original file for manual inspection

use anyhow::Result;
use rs_sft_sentences::discovery::{collect_discovered_files, DiscoveryConfig};
use std::path::PathBuf;
use tokio::fs;
use regex_automata::meta::Regex;

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
                find_dialog_end_position(text, boundary_start)
            },
            _ => find_punctuation_end(text, boundary_start),
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
                    find_next_sentence_start(text, potential_end)
                },
                "quote_start" | "paren_start" => {
                    find_quote_or_paren_start(text, boundary_start + 1)
                },
                _ => find_next_sentence_start(text, boundary_start + 1),
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

// Helper functions (copied from test implementation)
fn find_punctuation_end(text: &str, boundary_start: usize) -> usize {
    let mut char_indices = text[boundary_start..].char_indices();
    
    if let Some((_, _ch)) = char_indices.next() {
        if let Some((next_byte_offset, _)) = char_indices.next() {
            boundary_start + next_byte_offset
        } else {
            text.len()
        }
    } else {
        text.len()
    }
}

fn find_dialog_end_position(text: &str, boundary_start: usize) -> usize {
    let mut char_indices = text[boundary_start..].char_indices();
    
    // Skip the punctuation mark
    if let Some((_, _punctuation)) = char_indices.next() {
        // Skip the quote character
        if let Some((_, _quote)) = char_indices.next() {
            // Return position after the quote
            if let Some((next_byte_offset, _)) = char_indices.next() {
                boundary_start + next_byte_offset
            } else {
                text.len()
            }
        } else {
            boundary_start + 1
        }
    } else {
        text.len()
    }
}

fn find_next_sentence_start(text: &str, start_pos: usize) -> usize {
    let mut char_indices = text[start_pos..].char_indices();
    
    while let Some((byte_offset, ch)) = char_indices.next() {
        if !ch.is_whitespace() {
            return start_pos + byte_offset;
        }
    }
    text.len()
}

fn find_quote_or_paren_start(text: &str, start_pos: usize) -> usize {
    let mut char_indices = text[start_pos..].char_indices();
    
    while let Some((byte_offset, ch)) = char_indices.next() {
        if !ch.is_whitespace() {
            return start_pos + byte_offset;
        }
    }
    text.len()
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Generating Enhanced Dictionary sentence outputs for Gutenberg texts...");
    
    // Get Gutenberg mirror directory
    let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/gutenberg_texts", home)
        });
    let root_dir = PathBuf::from(mirror_dir);
    
    if !root_dir.exists() {
        eprintln!("❌ Gutenberg mirror directory {:?} does not exist", root_dir);
        eprintln!("   Set GUTENBERG_MIRROR_DIR environment variable or ensure ~/gutenberg_texts exists");
        return Ok(());
    }
    
    println!("📂 Scanning directory: {:?}", root_dir);
    
    // Discover all files
    let discovery_config = DiscoveryConfig::default();
    let discovered_files = collect_discovered_files(&root_dir, discovery_config).await?;
    
    // Filter to valid UTF-8 files only
    let valid_files: Vec<_> = discovered_files
        .iter()
        .filter(|f| f.is_valid_utf8 && f.error.is_none())
        .collect();
    
    println!("📊 Found {} valid UTF-8 files to process", valid_files.len());
    
    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;
    
    for file_info in &valid_files {
        let file_path = &file_info.path;
        let sentences_path = file_path.with_extension(
            format!("{}.norm_sents", 
                file_path.extension().and_then(|s| s.to_str()).unwrap_or("txt"))
        );
        
        // Skip if sentences file already exists
        if sentences_path.exists() {
            skipped += 1;
            if processed % 50 == 0 {
                println!("⏭️  Skipping {} (sentences file exists)", file_path.display());
            }
            continue;
        } 
        
        match process_file(file_path, &sentences_path).await {
            Ok(sentence_count) => {
                processed += 1;
                if processed % 10 == 0 {
                    println!("✅ Processed {} files... Latest: {} ({} sentences)", 
                        processed, file_path.file_name().unwrap().to_string_lossy(), sentence_count);
                }
            }
            Err(e) => {
                errors += 1;
                eprintln!("❌ Error processing {}: {}", file_path.display(), e);
            }
        }
    }
    
    println!("\n🎉 Generation complete!");
    println!("   📄 Processed: {} files", processed);
    println!("   ⏭️  Skipped: {} files (already had .sentences)", skipped);
    println!("   ❌ Errors: {} files", errors);
    
    if processed > 0 {
        println!("\n💡 Sentence files created with .sentences extension next to original files");
        println!("   Use these for manual inspection of boundary types and quality assessment");
        
        // Show some examples
        if let Some(first_file) = valid_files.first() {
            let example_sentences_path = first_file.path.with_extension(
                format!("{}.sentences", 
                    first_file.path.extension().and_then(|s| s.to_str()).unwrap_or("txt"))
            );
            if example_sentences_path.exists() {
                println!("   📝 Example sentences file: {}", example_sentences_path.display());
            }
        }
    }
    
    Ok(())
}

async fn process_file(input_path: &PathBuf, output_path: &PathBuf) -> Result<usize> {
    // Read the file
    let content = fs::read_to_string(input_path).await?;
    
    // Apply Enhanced Dictionary sentence detection
    let sentences = detect_sentences_dictionary_enhanced(&content)
        .map_err(|e| anyhow::anyhow!("Failed to detect sentences: {}", e))?;
    
    // Format as left-justified columns: index, sentence length, and normalized sentence
    let mut output = String::new();
    for (index, sentence) in sentences.iter().enumerate() {
        // Normalize the sentence by removing newlines
        let normalized = sentence.replace('\n', " ").replace('\r', "");
        
        // Format with left-justified columns (5 chars for index, 5 chars for length)
        output.push_str(&format!("{:<5} {:<5} {}\n", index + 1, normalized.len(), normalized));
    }
    
    // Write sentences file
    fs::write(output_path, output).await?;
    
    Ok(sentences.len())
}