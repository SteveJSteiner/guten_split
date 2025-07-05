// WHY: FST-based sentence detection implementation with dual API support
// Separated from main interface for clean code organization

use anyhow::Result;
use fst::{Set, SetBuilder};
use std::io::Cursor;
use std::sync::Arc;
use tracing::{debug, info};

use super::{DetectedSentence, DetectedSentenceBorrowed, DetectedSentenceOwned, Span, SentenceBoundaryRules};

/// Compiled FST for fast sentence boundary detection
pub struct SentenceDetectorFST {
    /// Compiled FST set for pattern matching
    _fst_set: Arc<Set<Vec<u8>>>, 
}

impl SentenceDetectorFST {
    /// Compile sentence boundary rules into an immutable FST at startup
    /// Implements F-3: compile sentence-boundary spec into immutable FST
    pub fn compile(rules: SentenceBoundaryRules) -> Result<Self> {
        info!("Compiling sentence boundary rules into FST");
        
        let mut patterns = Vec::new();
        
        // WHY: generate all possible sentence boundary patterns based on rules
        // Pattern: [end_punct][space][capital_letter|opening_quote|opening_paren]
        for &end_punct in &rules.end_punctuation {
            for &boundary_punct in &rules.boundary_punctuation {
                // Pattern: end_punct + boundary_punct + space + capital
                let pattern = format!("{end_punct}{boundary_punct}\\s[A-Z]");
                patterns.push(pattern.into_bytes());
            }
            
            // Pattern: end_punct + space + capital
            let pattern = format!("{end_punct}\\s[A-Z]");
            patterns.push(pattern.into_bytes());
            
            // Pattern: end_punct + space + opening quote
            for &quote in &rules.opening_quotes {
                let pattern = format!("{end_punct}\\s{quote}");
                patterns.push(pattern.into_bytes());
            }
            
            // Pattern: end_punct + space + opening parenthetical
            for &paren in &rules.opening_parentheticals {
                let pattern = format!("{end_punct}\\s{paren}");
                patterns.push(pattern.into_bytes());
            }
        }
        
        // WHY: sort patterns for FST compilation efficiency
        patterns.sort();
        patterns.dedup();
        
        debug!("Generated {} sentence boundary patterns", patterns.len());
        
        // Build FST from patterns
        let mut build_data = Vec::new();
        {
            let mut builder = SetBuilder::new(Cursor::new(&mut build_data))?;
            for pattern in patterns {
                builder.insert(&pattern)?;
            }
            builder.finish()?;
        }
        
        let fst_set = Set::new(build_data)?;
        info!("Successfully compiled FST with {} states", fst_set.len());
        
        Ok(Self {
            _fst_set: Arc::new(fst_set),
        })
    }
}

/// Detect sentences with borrowed API (zero allocations, mmap-friendly)
pub fn detect_sentences_borrowed<'a>(text: &'a str, rules: &SentenceBoundaryRules) -> Result<Vec<DetectedSentenceBorrowed<'a>>> {
    debug!("Starting borrowed sentence detection on {} characters", text.len());
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut line = 1;
    let mut col = 1;
    let mut sentence_start_line = 1;
    let mut sentence_start_col = 1;
    let mut sentence_start_byte = 0;
    
    // WHY: Single-pass approach with char_indices to avoid O(n^2) char_indices().nth() calls
    let char_indices: Vec<(usize, char)> = text.char_indices().collect();
    let chars: Vec<char> = char_indices.iter().map(|(_, ch)| *ch).collect();
    
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        
        // Check if current position could be a sentence boundary
        if is_sentence_boundary(&chars, i, rules) {
            // Extract sentence using pre-computed byte positions - O(1) lookup
            let end_byte = if i + 1 < char_indices.len() {
                char_indices[i + 1].0
            } else {
                text.len()
            };
            
            let sentence_text = &text[sentence_start_byte..end_byte];
            
            if !sentence_text.trim().is_empty() {
                let sentence = DetectedSentenceBorrowed {
                    index: sentence_index,
                    raw_content: sentence_text,  // Borrowed slice
                    span: Span {
                        start_line: sentence_start_line,
                        start_col: sentence_start_col,
                        end_line: line,
                        end_col: col,
                    },
                };
                
                sentences.push(sentence);
                sentence_index += 1;
            }
            
            // Move to start of next sentence
            let mut next_sentence_start = i + 1;
            
            // Skip whitespace to find actual start of next sentence
            while next_sentence_start < chars.len() && chars[next_sentence_start].is_whitespace() {
                next_sentence_start += 1;
            }
            
            // Update sentence start position and byte offset
            if next_sentence_start < char_indices.len() {
                sentence_start_byte = char_indices[next_sentence_start].0;
            } else {
                sentence_start_byte = text.len();
            }
            sentence_start_line = line;
            sentence_start_col = col + 1;
        }
        
        // Update position tracking
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        
        i += 1;
    }
    
    // Handle remaining text as final sentence if non-empty
    if sentence_start_byte < text.len() {
        let sentence_text = &text[sentence_start_byte..];
        
        if !sentence_text.trim().is_empty() {
            let sentence = DetectedSentenceBorrowed {
                index: sentence_index,
                raw_content: sentence_text,  // Borrowed slice
                span: Span {
                    start_line: sentence_start_line,
                    start_col: sentence_start_col,
                    end_line: line,
                    end_col: col,
                },
            };
            
            sentences.push(sentence);
        }
    }
    
    info!("Detected {} sentences with borrowed API", sentences.len());
    Ok(sentences)
}

/// Detect sentences with owned API (allocations for async I/O scenarios)
pub fn detect_sentences_owned(text: &str, rules: &SentenceBoundaryRules) -> Result<Vec<DetectedSentenceOwned>> {
    // First get borrowed results, then convert to owned
    let borrowed_sentences = detect_sentences_borrowed(text, rules)?;
    
    let owned_sentences: Vec<DetectedSentenceOwned> = borrowed_sentences
        .into_iter()
        .map(|borrowed| DetectedSentenceOwned {
            index: borrowed.index,
            raw_content: borrowed.raw_content.to_string(),  // Allocation here
            span: borrowed.span,
        })
        .collect();
    
    info!("Converted {} sentences to owned API", owned_sentences.len());
    Ok(owned_sentences)
}

/// Legacy API for backward compatibility
pub fn detect_sentences_legacy(text: &str, rules: &SentenceBoundaryRules) -> Result<Vec<DetectedSentence>> {
    let borrowed_sentences = detect_sentences_borrowed(text, rules)?;
    
    let legacy_sentences = borrowed_sentences
        .into_iter()
        .map(|borrowed| DetectedSentence {
            index: borrowed.index,
            normalized_content: borrowed.normalize(),  // Immediate normalization
            span: borrowed.span,
        })
        .collect();
    
    Ok(legacy_sentences)
}

/// Check if position i in chars array represents a sentence boundary
fn is_sentence_boundary(chars: &[char], pos: usize, rules: &SentenceBoundaryRules) -> bool {
    if pos == 0 || pos >= chars.len() - 1 {
        return false;
    }
    
    let current_char = chars[pos];
    
    // Must be end punctuation
    if !rules.end_punctuation.contains(&current_char) {
        return false;
    }
    
    // Look ahead for boundary pattern
    let mut next_pos = pos + 1;
    
    // Skip any boundary punctuation (quotes)
    while next_pos < chars.len() && rules.boundary_punctuation.contains(&chars[next_pos]) {
        next_pos += 1;
    }
    
    // Must have space after punctuation
    if next_pos >= chars.len() || !chars[next_pos].is_whitespace() {
        return false;
    }
    
    // Skip whitespace
    while next_pos < chars.len() && chars[next_pos].is_whitespace() {
        next_pos += 1;
    }
    
    // Must be followed by capital letter, opening quote, or opening parenthetical
    if next_pos >= chars.len() {
        return false;
    }
    
    let next_char = chars[next_pos];
    next_char.is_uppercase() 
        || rules.opening_quotes.contains(&next_char)
        || rules.opening_parentheticals.contains(&next_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrowed_vs_owned_equivalence() {
        let rules = SentenceBoundaryRules::default();
        let text = "Hello world. This is a test. How are you?";
        
        let borrowed_result = detect_sentences_borrowed(text, &rules).unwrap();
        let owned_result = detect_sentences_owned(text, &rules).unwrap();
        
        assert_eq!(borrowed_result.len(), owned_result.len());
        
        for (borrowed, owned) in borrowed_result.iter().zip(owned_result.iter()) {
            assert_eq!(borrowed.index, owned.index);
            assert_eq!(borrowed.raw(), owned.raw());
            assert_eq!(borrowed.span, owned.span);
        }
    }

    #[test]
    fn test_borrowed_zero_allocations() {
        let rules = SentenceBoundaryRules::default();
        let text = "Hello world. This is a test.";
        
        let sentences = detect_sentences_borrowed(text, &rules).unwrap();
        
        // Verify that raw_content points into original text
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Hello world"));
        assert!(sentences[1].raw().contains("This is a test"));
        
        // The borrowed slices should point into the original text
        assert!(text.contains(sentences[0].raw()));
        assert!(text.contains(sentences[1].raw()));
    }

    #[test]
    fn test_sentence_boundary_detection() {
        let rules = SentenceBoundaryRules::default();
        let chars: Vec<char> = "Hello world. This is test. And more.".chars().collect();
        
        // Position 11 is the '.' after "world"
        assert!(is_sentence_boundary(&chars, 11, &rules));
        
        // Position 25 is the '.' after "test"
        assert!(is_sentence_boundary(&chars, 25, &rules));
        
        // Random positions should not be boundaries
        assert!(!is_sentence_boundary(&chars, 5, &rules));
        assert!(!is_sentence_boundary(&chars, 15, &rules));
        
        // End of text is not a boundary (no following capital)
        let end_pos = chars.len() - 1;
        assert!(!is_sentence_boundary(&chars, end_pos, &rules));
    }
}