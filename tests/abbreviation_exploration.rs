// Abbreviation Detection Strategy Exploration
// Test harness for comparing different abbreviation detection approaches

use std::time::Instant;
use regex_automata::meta::Regex;
use rs_sft_sentences::sentence_detector::{SentenceDetector, SentenceDetectorDFA};

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
            
            // Quality comparison
            let manual_correct = manual_sentences == expected;
            let dfa_correct = dfa_sentences == expected;
            let enhanced_correct = enhanced_sentences == expected;
            
            println!("Manual: {}, DFA: {}, Enhanced: {}", manual_correct, dfa_correct, enhanced_correct);
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
        
        if !is_abbreviation {
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
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
    ];
    
    // Phase 1: Find all potential sentence boundaries using simple pattern
    let pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    for mat in pattern.find_iter(text) {
        let potential_end = mat.start() + 1; // Position after the punctuation
        let preceding_text = &text[last_start..potential_end];
        
        // Phase 2: Check if this ends with a known abbreviation
        let is_abbreviation = abbreviations.iter().any(|abbrev| {
            preceding_text.trim_end().ends_with(abbrev)
        });
        
        if !is_abbreviation {
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

// Enhanced Dictionary Strategy - Full Feature Support
// WHY: Matches manual detector functionality while adding abbreviation filtering
// Trade-off: More complex pattern matching but equivalent feature set to manual detector
fn detect_sentences_dictionary_enhanced(text: &str) -> Result<Vec<rs_sft_sentences::DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
    ];
    
    // Enhanced patterns to match manual detector functionality
    // WHY: Support same dialog and parenthetical patterns as SentenceBoundaryRules::default()
    let enhanced_patterns = [
        // Basic: [.!?]\s+[A-Z]
        r"[.!?]\s+[A-Z]",
        // Dialog patterns: [.!?]['"\u{201D}\u{2019}]\s+[A-Z]
        r#"[.!?]['"\u{201D}\u{2019}]\s+[A-Z]"#,
        // Quote starts: [.!?]\s+['"\u{201C}\u{2018}]
        r#"[.!?]\s+['"\u{201C}\u{2018}]"#,
        // Parenthetical starts: [.!?]\s+[({\[]
        r"[.!?]\s+[({\[]",
    ];
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    // Combine all patterns into a single regex
    let combined_pattern = enhanced_patterns.join("|");
    let pattern = Regex::new(&combined_pattern).unwrap();
    
    for mat in pattern.find_iter(text) {
        // Find the punctuation position (always first character of the match)
        let potential_end = mat.start() + 1; // Position after the punctuation
        let preceding_text = &text[last_start..potential_end];
        
        // Phase 2: Check if this ends with a known abbreviation
        let is_abbreviation = abbreviations.iter().any(|abbrev| {
            preceding_text.trim_end().ends_with(abbrev)
        });
        
        if !is_abbreviation {
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
            
            // Find start of next sentence (skip whitespace and quotes/parens)
            let mut next_start = mat.start() + 1; // After punctuation
            while next_start < text.len() {
                let ch = text.chars().nth(next_start).unwrap_or('\0');
                if ch.is_whitespace() || ch == '"' || ch == '\'' || ch == '(' || ch == '[' || ch == '{' {
                    next_start += 1;
                } else {
                    break;
                }
            }
            last_start = next_start;
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

// Strategy 3: Context Analysis Approach
fn detect_sentences_context(text: &str) -> Vec<String> {
    // WHY: Analyzes patterns around periods to distinguish abbreviations from sentence ends
    // Trade-off: Most sophisticated but potentially slowest approach
    
    let mut sentences = Vec::new();
    let mut last_end = 0;
    let chars: Vec<char> = text.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        if matches!(ch, '.' | '!' | '?') {
            // Look ahead for space + capital letter
            if i + 2 < chars.len() && chars[i + 1].is_whitespace() && chars[i + 2].is_uppercase() {
                // Look behind for abbreviation indicators
                let is_abbreviation = if i >= 3 {
                    // Check for pattern: Letter + period + space + lowercase (like "Dr. smith")
                    let prev_chars = &chars[i.saturating_sub(3)..i];
                    prev_chars.len() >= 2 && 
                    prev_chars[prev_chars.len()-1].is_alphabetic() &&
                    prev_chars.iter().any(|&c| c.is_uppercase()) &&
                    prev_chars.len() <= 4 // Typical abbreviation length
                } else {
                    false
                };
                
                if !is_abbreviation {
                    let sentence_end = i + 1;
                    let sentence: String = chars[last_end..sentence_end].iter().collect();
                    sentences.push(sentence);
                    last_end = i + 2; // Skip the space
                }
            }
        }
    }
    
    if last_end < chars.len() {
        let final_sentence: String = chars[last_end..].iter().collect();
        sentences.push(final_sentence);
    }
    
    sentences
}

// Strategy 4: Multi-Pattern DFA Approach (placeholder)
fn detect_sentences_multipattern(text: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // WHY: Uses regex-automata with multiple patterns for different contexts
    // Trade-off: Most complex to implement but potentially highest performance
    
    // This would use regex_automata::multi::MultiBuilder with separate patterns
    // Pattern 0: Narrative sentence boundaries  
    // Pattern 1: Dialog sentence boundaries
    // For now, use simple fallback
    
    detect_sentences_lookbehind(text)
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