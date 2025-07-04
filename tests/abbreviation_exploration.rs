// Abbreviation Detection Strategy Exploration
// Test harness for comparing different abbreviation detection approaches

use std::time::Instant;
use regex_automata::meta::Regex;
use rs_sft_sentences::sentence_detector::{SentenceDetector, SentenceDetectorDFA};

// Title abbreviations that cause false sentence boundaries when followed by proper nouns
// These are the first part of 2-segment identifiers like "Dr. Smith", "Mr. Johnson"
const TITLE_FALSE_POSITIVES: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr."
];

// All abbreviations that should not cause sentence splits
const ABBREVIATIONS: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
    "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
    "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
    "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
];

#[cfg(test)]
mod abbreviation_tests {
    use super::*;

    // Test scenarios covering different abbreviation types and contexts
    
    #[test]
    fn test_narrative_context_scenarios() {
        let scenarios = [
            // Title abbreviations
            ("Dr. Smith examined the patient. The results were clear.", vec!["Dr. Smith examined the patient.", "The results were clear."]),
            ("Mr. and Mrs. Johnson arrived. They were late.", vec!["Mr. and Mrs. Johnson arrived.", "They were late."]),
            ("Prof. Williams teaches at the university. Students love her classes.", vec!["Prof. Williams teaches at the university.", "Students love her classes."]),
            
            // Geographic abbreviations
            ("The U.S.A. declared independence. It was 1776.", vec!["The U.S.A. declared independence.", "It was 1776."]),
            ("He traveled to N.Y.C. last week. The trip was exhausting.", vec!["He traveled to N.Y.C. last week.", "The trip was exhausting."]),
            ("The company moved to L.A. in 2020. Business has grown since.", vec!["The company moved to L.A. in 2020.", "Business has grown since."]),
            
            // Measurement abbreviations
            ("The box measures 5 ft. by 3 ft. It weighs 10 lbs.", vec!["The box measures 5 ft. by 3 ft.", "It weighs 10 lbs."]),
            ("Temperature rose to 98.6°F. The patient felt better.", vec!["Temperature rose to 98.6°F.", "The patient felt better."]),
            ("Distance is 2.5 mi. from here. We can walk it.", vec!["Distance is 2.5 mi. from here.", "We can walk it."]),
        ];

        println!("=== Testing Dictionary Post-Processing Strategy ===");
        for (input, expected) in scenarios {
            println!("\nInput: {}", input);
            println!("Expected: {:?}", expected);
            
            // Test current production DFA (baseline)
            let detector = SentenceDetectorDFA::new().unwrap();
            let production_result = detector.detect_sentences(input).unwrap();
            let production_sentences: Vec<String> = production_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Production DFA: {:?}", production_sentences);
            
            // Test dictionary post-processing strategy
            let dictionary_result = detect_sentences_dictionary_full(input).unwrap();
            let dictionary_sentences: Vec<String> = dictionary_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Dictionary Strategy: {:?}", dictionary_sentences);
            
            // Quality comparison
            let production_correct = production_sentences == expected;
            let dictionary_correct = dictionary_sentences == expected;
            println!("Production correct: {}, Dictionary correct: {}", production_correct, dictionary_correct);
            
            if dictionary_correct && !production_correct {
                println!("✅ IMPROVEMENT: Dictionary strategy fixes abbreviation splitting");
            } else if !dictionary_correct && production_correct {
                println!("❌ REGRESSION: Dictionary strategy breaks correct behavior");
            } else if dictionary_correct && production_correct {
                println!("➖ NO CHANGE: Both strategies work correctly");
            } else {
                println!("❌ BOTH WRONG: Neither strategy handles this case correctly");
            }
        }
    }

    #[test]
    fn test_dialog_analysis() {
        // Analyze the failing patterns to understand if 5/5 is achievable
        let problem_cases = [
            (
                "He said, 'Dr. Smith will see you.' She nodded.",
                "Issue: Abbreviation 'Dr.' inside dialog - should not split sentence"
            ),
            (
                "John asked, 'Is Mr. Johnson here?' 'Yes,' came the reply.",
                "Issue: Abbreviation 'Mr.' inside dialog + consecutive quotes"
            ),
            (
                "She whispered, 'Prof. Davis is strict.' The class fell silent.",
                "Issue: Abbreviation 'Prof.' inside dialog"
            ),
        ];

        println!("=== Dialog Pattern Analysis ===");
        for (input, issue) in problem_cases {
            println!("\nCase: {}", input);
            println!("Problem: {}", issue);
            
            // Test if we can detect dialog context to avoid splitting on abbreviations
            let has_dialog_context = input.contains("'") && (input.contains(" said") || input.contains(" asked") || input.contains(" whispered"));
            println!("Has dialog context: {}", has_dialog_context);
            
            // Show where current patterns are splitting
            let manual_detector = SentenceDetector::with_default_rules().unwrap();
            let manual_result = manual_detector.detect_sentences(input).unwrap();
            let manual_sentences: Vec<String> = manual_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Manual splits at: {:?}", manual_sentences);
        }
    }

    #[test]
    fn test_dialog_context_scenarios() {
        let scenarios = [
            // Dialog with abbreviations
            ("He said, 'Dr. Smith will see you.' She nodded.", vec!["He said, 'Dr. Smith will see you.'", "She nodded."]),
            ("'The U.S.A. is large,' he noted. 'Indeed,' she replied.", vec!["'The U.S.A. is large,' he noted.", "'Indeed,' she replied."]),
            ("John asked, 'Is Mr. Johnson here?' 'Yes,' came the reply.", vec!["John asked, 'Is Mr. Johnson here?'", "'Yes,' came the reply."]),
            
            // Complex dialog scenarios
            ("She whispered, 'Prof. Davis is strict.' The class fell silent.", vec!["She whispered, 'Prof. Davis is strict.'", "The class fell silent."]),
            ("'Meet me at 5 p.m. sharp,' he said. The appointment was set.", vec!["'Meet me at 5 p.m. sharp,' he said.", "The appointment was set."]),
        ];

        println!("\n=== Testing Dialog Support Comparison ===");
        for (input, expected) in scenarios {
            println!("\nInput: {}", input);
            println!("Expected: {:?}", expected);
            
            // Test manual detector (full feature support baseline)
            let manual_detector = SentenceDetector::with_default_rules().unwrap();
            let manual_result = manual_detector.detect_sentences(input).unwrap();
            let manual_sentences: Vec<String> = manual_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Manual Detector: {:?}", manual_sentences);
            
            // Test current DFA (simple pattern only)
            let dfa_detector = SentenceDetectorDFA::new().unwrap();
            let dfa_result = dfa_detector.detect_sentences(input).unwrap();
            let dfa_sentences: Vec<String> = dfa_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Current DFA: {:?}", dfa_sentences);
            
            // Test enhanced dictionary strategy (with dialog support)
            let enhanced_result = detect_sentences_dictionary_enhanced(input).unwrap();
            let enhanced_sentences: Vec<String> = enhanced_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Enhanced Dictionary: {:?}", enhanced_sentences);
            
            // Test forward-probing strategy 
            let forward_probe_result = detect_sentences_dictionary_forward_probe(input).unwrap();
            let forward_probe_sentences: Vec<String> = forward_probe_result.iter()
                .map(|s| s.normalized_content.trim().to_string())
                .collect();
            println!("Forward Probe: {:?}", forward_probe_sentences);
            
            // Quality comparison
            let manual_correct = manual_sentences == expected;
            let dfa_correct = dfa_sentences == expected;
            let enhanced_correct = enhanced_sentences == expected;
            let forward_probe_correct = forward_probe_sentences == expected;
            
            println!("Manual: {}, DFA: {}, Enhanced: {}, Forward Probe: {}", 
                    manual_correct, dfa_correct, enhanced_correct, forward_probe_correct);
        }
    }

    #[test]
    fn test_edge_cases() {
        let scenarios = [
            // Multiple abbreviations
            ("A. B. Smith vs. Dr. C. Johnson went to U.S.A.", vec!["A. B. Smith vs. Dr. C. Johnson went to U.S.A."]),
            ("The meeting was at 3 p.m. in Rm. 205. Everyone attended.", vec!["The meeting was at 3 p.m. in Rm. 205.", "Everyone attended."]),
            
            // Mixed abbreviations and sentences
            ("Visit www.example.com for info. Contact Dr. Smith.", vec!["Visit www.example.com for info.", "Contact Dr. Smith."]),
            ("Price: $19.99 ea. Buy now and save!", vec!["Price: $19.99 ea.", "Buy now and save!"]),
            
            // Ambiguous cases
            ("The study was conducted by Smith et al. Results varied.", vec!["The study was conducted by Smith et al.", "Results varied."]),
            ("Temperature was 72 deg. F. outside today.", vec!["Temperature was 72 deg. F. outside today."]),
        ];

        for (input, _expected) in scenarios {
            println!("Testing edge case: {}", input);
            // We'll implement detection functions here
        }
    }


    #[test]
    fn test_performance_baseline() {
        let text = super::generate_large_test_text();
        
        // Test actual production SentenceDetectorDFA
        let detector = SentenceDetectorDFA::new().unwrap();
        let start = Instant::now();
        let sentences = detector.detect_sentences(&text).unwrap();
        let duration = start.elapsed();
        
        println!("Production SentenceDetectorDFA: {} sentences in {:?}", sentences.len(), duration);
        println!("Text length: {} bytes", text.len());
        
        if duration.as_millis() > 0 {
            let throughput = (text.len() as f64) / (duration.as_secs_f64() * 1024.0 * 1024.0);
            println!("Production throughput: {:.1} MiB/s", throughput);
        }
        
        // Also test simple regex for comparison
        let start = Instant::now();
        let simple_pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
        let matches: Vec<_> = simple_pattern.find_iter(&text).collect();
        let simple_duration = start.elapsed();
        
        println!("Simple regex: {} matches in {:?}", matches.len(), simple_duration);
        if simple_duration.as_millis() > 0 {
            let simple_throughput = (text.len() as f64) / (simple_duration.as_secs_f64() * 1024.0 * 1024.0);
            println!("Simple regex throughput: {:.1} MiB/s", simple_throughput);
        }
    }
}

// Performance test data  
fn generate_large_test_text() -> String {
    let base_text = "Dr. Smith examined Mrs. Johnson carefully. The U.S.A. has many doctors. Prof. Williams studied at N.Y.C. University. The patient weighed 150 lbs. and was 5 ft. 8 in. tall. ";
    base_text.repeat(1000) // Create larger text for performance testing
}

// Strategy 1: Simple Pattern Count (for performance comparison)
// WHY: Count raw pattern matches without sentence processing overhead
// Trade-off: Not a realistic abbreviation solution, just for baseline measurement
fn detect_sentences_lookbehind(text: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // Use simple meta regex for now - focus on getting performance baseline
    let pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    let match_count = pattern.find_iter(text).count();
    Ok(match_count)
}

// Strategy 2: Dictionary Post-Processing Approach
// WHY: Two-phase approach allows dynamic abbreviation checking  
// Trade-off: More complex, potential performance impact from double-pass
fn detect_sentences_dictionary(text: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg."
    ];
    
    // Phase 1: Use simple pattern matching
    let pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    
    // Phase 2: Filter out abbreviations 
    let mut sentence_count = 0;
    let mut last_end = 0;
    
    for mat in pattern.find_iter(text) {
        let potential_end = mat.start() + 1;
        let preceding_text = &text[last_end..potential_end];
        
        // Check if this ends with a known abbreviation
        let is_abbreviation = abbreviations.iter().any(|abbrev| {
            preceding_text.trim_end().ends_with(abbrev)
        });
        
        // Only check for title + proper noun false positives
        let is_title_false_positive = {
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                TITLE_FALSE_POSITIVES.contains(last_word)
            } else {
                false
            }
        };
        
        if !is_title_false_positive {
            sentence_count += 1;
            last_end = mat.end() - 1;
        }
    }
    
    if last_end < text.len() {
        sentence_count += 1; // Final sentence
    }
    
    Ok(sentence_count)
}

// Dictionary Post-Processing Strategy - Full Implementation
// WHY: Returns complete DetectedSentence objects for quality comparison with production DFA
// Trade-off: More memory allocation but enables proper sentence boundary analysis
fn detect_sentences_dictionary_full(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = ABBREVIATIONS;
    
    // Phase 1: Find all potential sentence boundaries using simple pattern
    let pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    for mat in pattern.find_iter(text) {
        let potential_end = mat.start() + 1; // Position after the punctuation
        let preceding_text = &text[last_start..potential_end];
        
        // Phase 2: Check if the punctuation is part of an abbreviation that should not be split
        // WHY: Only check the word immediately before the punctuation, not the entire text
        let is_abbreviation = {
            // Find the last word ending with the punctuation
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                abbreviations.iter().any(|&abbrev| *last_word == abbrev)
            } else {
                false
            }
        };
        
        // Only check for title + proper noun false positives
        let is_title_false_positive = {
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                TITLE_FALSE_POSITIVES.contains(last_word)
            } else {
                false
            }
        };
        
        if !is_title_false_positive {
            // This is a real sentence boundary
            let sentence_text = &text[last_start..potential_end];
            
            // Simple position calculation - just use 1-based line/col
            // WHY: Focus on correctness comparison rather than exact span calculation
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: sentence_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: potential_end + 1,
                },
            });
            sentence_index += 1;
            
            // Skip whitespace to start of next sentence
            let next_start = mat.end() - 1; // Position after the space
            last_start = text[next_start..].chars()
                .position(|c| !c.is_whitespace())
                .map(|pos| next_start + pos)
                .unwrap_or(text.len());
        }
    }
    
    // Add final sentence if there's remaining text
    if last_start < text.len() {
        let final_text = &text[last_start..];
        if !final_text.trim().is_empty() {
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: text.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Forward-Probing Dictionary Strategy - Proper Dialog Handling  
// WHY: Probes forward to find matching quote close, avoiding abbreviation splits inside dialog
fn detect_sentences_dictionary_forward_probe(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = ABBREVIATIONS;
    
    // Use all dialog patterns
    let basic_pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    let dialog_end_pattern = Regex::new(r#"[.!?]['"\u{201D}\u{2019}]\s+[A-Z]"#).unwrap();
    let quote_start_pattern = Regex::new(r#"[.!?]\s+['"\u{201C}\u{2018}]"#).unwrap();
    let paren_start_pattern = Regex::new(r"[.!?]\s+[({\[]").unwrap();
    
    let mut boundaries = Vec::new();
    
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
    
    boundaries.sort_by_key(|&(start, _, _)| start);
    boundaries.dedup_by_key(|&mut (start, _, _)| start);
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    for (boundary_start, _boundary_end, boundary_type) in boundaries {
        let potential_end = find_punctuation_end(text, boundary_start);
        let preceding_text = &text[last_start..potential_end];
        
        // Check if this ends with an abbreviation
        let is_abbreviation = abbreviations.iter().any(|abbrev| {
            preceding_text.trim_end().ends_with(abbrev)
        });
        
        // Only check for title + proper noun false positives
        let is_title_false_positive = {
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                TITLE_FALSE_POSITIVES.contains(last_word)
            } else {
                false
            }
        };
        
        if !is_title_false_positive {
            // For basic boundaries, check if we're splitting inside dialog
            let mut actual_end = potential_end;
            
            if boundary_type == "basic" {
                // Probe forward to see if this might be inside dialog
                if let Some(dialog_end) = find_matching_dialog_close(text, boundary_start) {
                    // We're inside dialog - extend to the dialog close
                    actual_end = dialog_end;
                }
            }
            
            let sentence_text = &text[last_start..actual_end];
            
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: sentence_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: actual_end + 1,
                },
            });
            sentence_index += 1;
            
            // Move past the actual sentence end
            last_start = find_next_sentence_start(text, actual_end);
        }
    }
    
    // Add final sentence if there's remaining text
    if last_start < text.len() {
        let final_text = &text[last_start..];
        if !final_text.trim().is_empty() {
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: text.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Helper function to find matching dialog close by probing forward
fn find_matching_dialog_close(text: &str, boundary_pos: usize) -> Option<usize> {
    // Look backwards to find if we're after an opening quote
    let before = &text[..boundary_pos];
    let mut quote_pos = None;
    let quote_chars = ['\'', '"', '\u{201C}', '\u{2018}'];
    
    // Find the most recent opening quote before the boundary
    for (i, ch) in before.char_indices().rev() {
        if quote_chars.contains(&ch) {
            quote_pos = Some(i);
            break;
        }
        // Stop if we hit sentence-ending punctuation (we've gone too far back)
        if matches!(ch, '.' | '!' | '?') {
            break;
        }
    }
    
    if let Some(open_pos) = quote_pos {
        let opening_char = text.chars().nth(text[..open_pos].chars().count()).unwrap();
        let closing_char = match opening_char {
            '\'' => '\'',
            '"' => '"', 
            '\u{201C}' => '\u{201D}', // " -> "
            '\u{2018}' => '\u{2019}', // ' -> '
            _ => return None,
        };
        
        // Probe forward from boundary to find matching close
        let after = &text[boundary_pos..];
        for (offset, ch) in after.char_indices() {
            if ch == closing_char {
                // Found matching close - look for sentence end after the quote
                let after_quote = &after[offset + ch.len_utf8()..];
                for (quote_offset, quote_ch) in after_quote.char_indices() {
                    if matches!(quote_ch, '.' | '!' | '?') {
                        return Some(boundary_pos + offset + ch.len_utf8() + quote_offset + quote_ch.len_utf8());
                    }
                    if !quote_ch.is_whitespace() {
                        break;
                    }
                }
                // No punctuation after quote, treat quote as sentence end
                return Some(boundary_pos + offset + ch.len_utf8());
            }
            // Stop if we hit another sentence boundary
            if matches!(ch, '.' | '!' | '?') && after.chars().nth(offset + 1).map_or(false, |c| c.is_whitespace()) {
                break;
            }
        }
    }
    
    None
}

// Enhanced Dictionary Strategy - Full Feature Support
// WHY: Matches manual detector functionality while adding abbreviation filtering
// Trade-off: More complex pattern matching but equivalent feature set to manual detector
fn detect_sentences_dictionary_enhanced(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = ABBREVIATIONS;
    
    // Use separate patterns to handle each boundary type correctly
    // WHY: Avoids UTF-8 character boundary issues by processing patterns individually
    let basic_pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    let dialog_end_pattern = Regex::new(r#"[.!?]['"\u{201D}\u{2019}]\s+[A-Z]"#).unwrap();
    let quote_start_pattern = Regex::new(r#"[.!?]\s+['"\u{201C}\u{2018}]"#).unwrap();
    let paren_start_pattern = Regex::new(r"[.!?]\s+[({\[]").unwrap();
    // Missing pattern: QUOTE_END + QUOTE_START (e.g., "?' '" in consecutive quotes)
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
    
    // Sort by position and remove duplicates (same boundary matched by multiple patterns)
    boundaries.sort_by_key(|&(start, _, _)| start);
    boundaries.dedup_by_key(|&mut (start, _, _)| start);
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    for (boundary_start, boundary_end, boundary_type) in boundaries {
        // Find the sentence end position based on boundary type
        let potential_end = match boundary_type {
            "dialog_end" | "dialog_to_quote" => {
                // For dialog_end pattern [.!?]['"\u{201D}\u{2019}]\s+[A-Z]
                // and dialog_to_quote pattern [.!?]['"\u{201D}\u{2019}]\s+['"\u{201C}\u{2018}]
                // we need to include the quote in the sentence
                find_dialog_end_position(text, boundary_start)
            },
            _ => find_punctuation_end(text, boundary_start),
        };
        let preceding_text = &text[last_start..potential_end];
        
        // Phase 2: Check for title + proper noun false positives
        let is_title_false_positive = {
            let words: Vec<&str> = preceding_text.split_whitespace().collect();
            if let Some(last_word) = words.last() {
                // Remove leading/trailing quotes and punctuation to get clean word
                let clean_word = last_word.trim_matches(|c: char| c == '\'' || c == '"' || c == '\u{201C}' || c == '\u{201D}' || c == '\u{2018}' || c == '\u{2019}');
                TITLE_FALSE_POSITIVES.contains(&clean_word)
            } else {
                false
            }
        };
        
        if !is_title_false_positive {
            // This is a real sentence boundary
            let sentence_text = &text[last_start..potential_end];
            
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: sentence_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: potential_end + 1,
                },
            });
            sentence_index += 1;
            
            // Find start of next sentence based on boundary type
            last_start = match boundary_type {
                "basic" => {
                    // Skip punctuation and whitespace to find capital letter
                    find_next_sentence_start(text, boundary_start + 1)
                },
                "dialog_end" | "dialog_to_quote" => {
                    // Skip punctuation, quote, and whitespace to find capital letter
                    // Use the position after the quote for proper sentence start
                    find_next_sentence_start(text, potential_end)
                },
                "quote_start" | "paren_start" => {
                    // Skip punctuation and whitespace, start includes the quote/paren
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
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: text.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Helper function to find next sentence start after punctuation and whitespace
fn find_next_sentence_start(text: &str, start_pos: usize) -> usize {
    let mut char_indices = text[start_pos..].char_indices();
    
    // Skip whitespace
    while let Some((byte_offset, ch)) = char_indices.next() {
        if !ch.is_whitespace() {
            return start_pos + byte_offset;
        }
    }
    text.len()
}

// Helper function to find start of quoted or parenthetical content
fn find_quote_or_paren_start(text: &str, start_pos: usize) -> usize {
    let mut char_indices = text[start_pos..].char_indices();
    
    // Skip whitespace to find the quote or parenthesis
    while let Some((byte_offset, ch)) = char_indices.next() {
        if !ch.is_whitespace() {
            return start_pos + byte_offset;
        }
    }
    text.len()
}

// Helper function to find punctuation end position using UTF-8 safe approach
fn find_punctuation_end(text: &str, boundary_start: usize) -> usize {
    let mut char_indices = text[boundary_start..].char_indices();
    
    // Find the first character (punctuation) and return position after it
    if let Some((_, ch)) = char_indices.next() {
        if let Some((next_byte_offset, _)) = char_indices.next() {
            boundary_start + next_byte_offset
        } else {
            // Last character in text
            text.len()
        }
    } else {
        // Edge case: boundary_start is at end of text
        text.len()
    }
}

// Helper function to find dialog end position (after punctuation AND quote)
fn find_dialog_end_position(text: &str, boundary_start: usize) -> usize {
    let mut char_indices = text[boundary_start..].char_indices();
    
    // Skip the punctuation mark
    if let Some((_, punctuation)) = char_indices.next() {
        // Skip the quote character
        if let Some((_, quote)) = char_indices.next() {
            // Return position after the quote
            if let Some((next_byte_offset, _)) = char_indices.next() {
                boundary_start + next_byte_offset
            } else {
                // Quote is at end of text
                text.len()
            }
        } else {
            // No quote found, fallback to after punctuation
            boundary_start + punctuation.len_utf8()
        }
    } else {
        // No punctuation found, edge case
        text.len()
    }
}

// Strategy 3: Context Analysis Approach - Full Implementation
// WHY: Deep context analysis around punctuation marks with sophisticated abbreviation detection
// Trade-off: Most complex logic but potentially highest accuracy on edge cases
#[cfg_attr(test, allow(dead_code))]
fn detect_sentences_context_full(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = ABBREVIATIONS;
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    let chars: Vec<char> = text.chars().collect();
    
    let mut i = 0;
    while i < chars.len() {
        if matches!(chars[i], '.' | '!' | '?') {
            // Look ahead for valid sentence boundary patterns
            if i + 2 < chars.len() && chars[i + 1].is_whitespace() {
                let next_char = chars[i + 2];
                let is_sentence_start = next_char.is_uppercase() || 
                                      matches!(next_char, '"' | '\'' | '(' | '[');
                
                if is_sentence_start {
                    // Advanced context analysis for abbreviations
                    let is_abbreviation = analyze_abbreviation_context(&chars, i, &abbreviations);
                    
                    if !is_abbreviation {
                        // Extract sentence
                        let sentence_text: String = chars[last_start..=i].iter().collect();
                        
                        sentences.push(rs_sft_sentences::DetectedSentence {
                            index: sentence_index,
                            normalized_content: sentence_text.trim().to_string(),
                            span: rs_sft_sentences::Span {
                                start_line: 1,
                                start_col: last_start + 1,
                                end_line: 1,
                                end_col: i + 2,
                            },
                        });
                        sentence_index += 1;
                        
                        // Skip to start of next sentence
                        last_start = i + 2;
                        while last_start < chars.len() && chars[last_start].is_whitespace() {
                            last_start += 1;
                        }
                        i = last_start.saturating_sub(1); // Will be incremented at end of loop
                    }
                }
            }
        }
        i += 1;
    }
    
    // Add final sentence
    if last_start < chars.len() {
        let final_text: String = chars[last_start..].iter().collect();
        if !final_text.trim().is_empty() {
            sentences.push(rs_sft_sentences::DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.trim().to_string(),
                span: rs_sft_sentences::Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: chars.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Advanced abbreviation detection using multiple context clues
#[cfg_attr(test, allow(dead_code))]
fn analyze_abbreviation_context(chars: &[char], punct_pos: usize, abbreviations: &[&str]) -> bool {
    // Convert to string for abbreviation matching
    let start_pos = if punct_pos >= 10 { punct_pos - 10 } else { 0 };
    let context: String = chars[start_pos..=punct_pos].iter().collect();
    
    // Direct abbreviation match
    if abbreviations.iter().any(|abbrev| context.ends_with(abbrev)) {
        return true;
    }
    
    // Pattern-based detection
    if punct_pos >= 3 {
        let before_punct = &chars[punct_pos.saturating_sub(3)..punct_pos];
        
        // Short word + period pattern (likely abbreviation)
        if before_punct.len() <= 3 && before_punct.iter().all(|&c| c.is_alphabetic()) {
            return true;
        }
        
        // Single letter + period pattern (initials)
        if before_punct.len() == 1 && before_punct[0].is_uppercase() {
            return true;
        }
        
        // Multiple periods pattern (like "U.S.A.")
        if punct_pos >= 4 && chars[punct_pos - 2] == '.' {
            return true;
        }
    }
    
    false
}

// Strategy 4: Multi-Pattern DFA Approach - High Performance Implementation
// WHY: Uses regex-automata multi-pattern DFA for maximum throughput
// Trade-off: Complex setup but potentially approaching 1GB/s performance
#[cfg_attr(test, allow(dead_code))]
fn detect_sentences_multipattern_full(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    // NOTE: multi module not available in this version of regex-automata
    // Using simplified approach for now - just call the dictionary method
    detect_sentences_dictionary_enhanced(text)
}


#[cfg(test)]
mod strategy_comparison {
    use super::*;
    use std::time::Instant;

    #[test]
    fn compare_all_strategies() {
        let test_cases = [
            "Dr. Smith examined the patient. The results were clear.",
            "He said, 'Dr. Smith will see you.' She nodded.", 
            "The U.S.A. declared independence. It was 1776.",
            "A. B. Smith vs. Dr. C. Johnson went to U.S.A.",
        ];

        for test_case in test_cases {
            println!("\nTesting: {}", test_case);
            
            let lookbehind_result = detect_sentences_lookbehind(test_case);
            println!("Lookbehind: {:?}", lookbehind_result);
            
            let dictionary_result = detect_sentences_dictionary(test_case);
            println!("Dictionary: {:?}", dictionary_result);
            
            // Context and multipattern strategies need more work to return &str properly
            println!("Context: [implementation in progress]");
            println!("Multipattern: [implementation in progress]");
        }
    }

    #[test]
    fn performance_comparison() {
        let large_text = super::generate_large_test_text();
        
        println!("Performance comparison on {} bytes:", large_text.len());
        
        // Test production DFA baseline
        let detector = SentenceDetectorDFA::new().unwrap();
        let start = Instant::now();
        let production_result = detector.detect_sentences(&large_text).unwrap();
        let production_duration = start.elapsed();
        
        // Test lookbehind approach
        let start = Instant::now();
        let lookbehind_result = super::detect_sentences_lookbehind(&large_text).unwrap();
        let lookbehind_duration = start.elapsed();
        
        // Test dictionary approach  
        let start = Instant::now();
        let dictionary_result = super::detect_sentences_dictionary(&large_text).unwrap();
        let dictionary_duration = start.elapsed();
        
        println!("Production DFA: {} sentences in {:?}", production_result.len(), production_duration);
        println!("Lookbehind DFA: {} matches in {:?}", lookbehind_result, lookbehind_duration);
        println!("Dictionary DFA: {} sentences in {:?}", dictionary_result, dictionary_duration);
        
        // Calculate throughput
        if lookbehind_duration.as_millis() > 0 {
            let throughput = (large_text.len() as f64) / (lookbehind_duration.as_secs_f64() * 1024.0 * 1024.0);
            println!("Lookbehind throughput: {:.1} MiB/s", throughput);
        }
        
        if dictionary_duration.as_millis() > 0 {
            let throughput = (large_text.len() as f64) / (dictionary_duration.as_secs_f64() * 1024.0 * 1024.0);
            println!("Dictionary throughput: {:.1} MiB/s", throughput);
        }
    }
}