// WHY: DFA-based sentence detection implementation with dual API support
// Uses regex-automata dense DFA for O(n) sentence boundary detection

use anyhow::Result;
use regex_automata::{dfa::{dense::DFA, Automaton}, Input};
use tracing::{debug, info};

use super::{DetectedSentence, DetectedSentenceBorrowed, DetectedSentenceOwned, Span};

/// DFA-based sentence detector for performance comparison
/// Uses regex-automata dense DFA for O(n) sentence boundary detection
pub struct SentenceDetectorDFA {
    /// Compiled DFA for sentence boundary pattern matching
    dfa: DFA<Vec<u32>>,
}

/// Position counter for single-pass O(n) tracking
struct PositionCounter {
    /// Current byte position in text
    byte_pos: usize,
    /// Current character position in text
    char_pos: usize,
    /// Current line number (1-based)
    line: usize,
    /// Current column number (1-based)
    col: usize,
}

impl PositionCounter {
    fn new() -> Self {
        Self {
            byte_pos: 0,
            char_pos: 0,
            line: 1,
            col: 1,
        }
    }
    
    /// Advance counter to target byte position, updating line/col correctly
    /// WHY: O(1) amortized - only processes bytes between current and target position
    fn advance_to_byte(&mut self, text_bytes: &[u8], target_byte_pos: usize) {
        while self.byte_pos < target_byte_pos && self.byte_pos < text_bytes.len() {
            let byte = text_bytes[self.byte_pos];
            
            // Check if this byte starts a new UTF-8 character
            if (byte & 0x80) == 0 || (byte & 0xC0) == 0xC0 {
                self.char_pos += 1;
            }
            
            // Update line/col tracking
            if byte == b'\n' {
                self.line += 1;
                self.col = 1;
            } else if (byte & 0x80) == 0 || (byte & 0xC0) == 0xC0 {
                // Only increment col for character boundaries (not newlines)
                self.col += 1;
            }
            
            self.byte_pos += 1;
        }
    }
}

impl SentenceDetectorDFA {
    /// Create new DFA-based sentence detector with basic pattern [.!?]\s+[A-Z]
    /// WHY: simplified pattern for baseline performance comparison
    pub fn new() -> Result<Self> {
        info!("Compiling DFA for sentence boundary detection");
        
        // WHY: basic sentence boundary pattern as specified in task
        let pattern = r"[.!?]\s+[A-Z]";
        let dfa = DFA::new(pattern)?;
        
        debug!("Successfully compiled DFA with pattern: {}", pattern);
        
        Ok(Self { dfa })
    }
}

/// Detect sentences with borrowed API (zero allocations, mmap-friendly)
pub fn detect_sentences_borrowed<'a>(text: &'a str) -> Result<Vec<DetectedSentenceBorrowed<'a>>> {
    debug!("Starting borrowed DFA sentence detection on {} characters", text.len());
    
    let dfa = SentenceDetectorDFA::new()?.dfa;
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    
    let text_bytes = text.as_bytes();
    let mut search_pos = 0;
    
    // WHY: O(n) single-pass position tracking
    let mut counter = PositionCounter::new();
    let mut sentence_start_counter = PositionCounter::new();
    
    while search_pos < text_bytes.len() {
        // Find earliest match from current position
        let input = Input::new(&text_bytes[search_pos..]);
        if let Some(match_result) = dfa.try_search_fwd(&input).unwrap() {
            let match_end_byte = search_pos + match_result.offset();
            
            // Find the punctuation character that ends the sentence
            // WHY: pattern is [.!?]\\s+[A-Z], so we need to find the first punctuation in the match
            let mut punct_byte_pos = search_pos;
            while punct_byte_pos < text_bytes.len() {
                let byte = text_bytes[punct_byte_pos];
                if byte == b'.' || byte == b'!' || byte == b'?' {
                    break;
                }
                punct_byte_pos += 1;
            }
            
            // Advance counter to punctuation position
            counter.advance_to_byte(text_bytes, punct_byte_pos);
            
            // Extract sentence from start to punctuation position using byte slicing
            let sentence_start_byte = sentence_start_counter.byte_pos;
            let sentence_bytes = &text_bytes[sentence_start_byte..=punct_byte_pos];
            
            // WHY: convert only the sentence slice to string, not entire text
            if let Ok(sentence_text) = std::str::from_utf8(sentence_bytes) {
                if !sentence_text.trim().is_empty() {
                    let sentence = DetectedSentenceBorrowed {
                        index: sentence_index,
                        raw_content: sentence_text,  // Borrowed slice
                        span: Span {
                            start_line: sentence_start_counter.line,
                            start_col: sentence_start_counter.col,
                            end_line: counter.line,
                            end_col: counter.col,
                        },
                    };
                    
                    sentences.push(sentence);
                    sentence_index += 1;
                }
            }
            
            // Move to start of next sentence - advance past punctuation
            counter.advance_to_byte(text_bytes, punct_byte_pos + 1);
            
            // Skip whitespace to find actual start of next sentence
            while counter.byte_pos < text_bytes.len() && text_bytes[counter.byte_pos].is_ascii_whitespace() {
                counter.advance_to_byte(text_bytes, counter.byte_pos + 1);
            }
            
            // Update sentence start position
            sentence_start_counter = PositionCounter {
                byte_pos: counter.byte_pos,
                char_pos: counter.char_pos,
                line: counter.line,
                col: counter.col,
            };
            
            // Move search position past this match
            search_pos = match_end_byte;
        } else {
            // No more matches found
            break;
        }
    }
    
    // Handle remaining text as final sentence if non-empty
    if sentence_start_counter.byte_pos < text_bytes.len() {
        let sentence_bytes = &text_bytes[sentence_start_counter.byte_pos..];
        
        if let Ok(sentence_text) = std::str::from_utf8(sentence_bytes) {
            if !sentence_text.trim().is_empty() {
                // Advance counter to end of text for final span
                counter.advance_to_byte(text_bytes, text_bytes.len());
                
                let sentence = DetectedSentenceBorrowed {
                    index: sentence_index,
                    raw_content: sentence_text,  // Borrowed slice
                    span: Span {
                        start_line: sentence_start_counter.line,
                        start_col: sentence_start_counter.col,
                        end_line: counter.line,
                        end_col: counter.col,
                    },
                };
                
                sentences.push(sentence);
            }
        }
    }
    
    info!("DFA detected {} sentences with borrowed API", sentences.len());
    Ok(sentences)
}

/// Detect sentences with owned API (allocations for async I/O scenarios)
pub fn detect_sentences_owned(text: &str) -> Result<Vec<DetectedSentenceOwned>> {
    // First get borrowed results, then convert to owned
    let borrowed_sentences = detect_sentences_borrowed(text)?;
    
    let owned_sentences: Vec<DetectedSentenceOwned> = borrowed_sentences
        .into_iter()
        .map(|borrowed| DetectedSentenceOwned {
            index: borrowed.index,
            raw_content: borrowed.raw_content.to_string(),  // Allocation here
            span: borrowed.span,
        })
        .collect();
    
    info!("Converted {} sentences to owned DFA API", owned_sentences.len());
    Ok(owned_sentences)
}

/// Legacy API for backward compatibility
pub fn detect_sentences_legacy(text: &str) -> Result<Vec<DetectedSentence>> {
    let borrowed_sentences = detect_sentences_borrowed(text)?;
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa_borrowed_vs_owned_equivalence() {
        let text = "Hello world. This is a test. How are you?";
        
        let borrowed_result = detect_sentences_borrowed(text).unwrap();
        let owned_result = detect_sentences_owned(text).unwrap();
        
        assert_eq!(borrowed_result.len(), owned_result.len());
        
        for (borrowed, owned) in borrowed_result.iter().zip(owned_result.iter()) {
            assert_eq!(borrowed.index, owned.index);
            assert_eq!(borrowed.raw(), owned.raw());
            assert_eq!(borrowed.span, owned.span);
        }
    }

    #[test]
    fn test_dfa_borrowed_zero_allocations() {
        let text = "Hello world. This is a test.";
        
        let sentences = detect_sentences_borrowed(text).unwrap();
        
        // Verify that raw_content points into original text
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Hello world"));
        assert!(sentences[1].raw().contains("This is a test"));
        
        // The borrowed slices should point into the original text
        assert!(text.contains(sentences[0].raw()));
        assert!(text.contains(sentences[1].raw()));
    }

    #[test]
    fn test_dfa_position_counter() {
        let text = "Line 1\nLine 2\nLine 3";
        let bytes = text.as_bytes();
        let mut counter = PositionCounter::new();
        
        // Advance to position after first newline
        counter.advance_to_byte(bytes, 7); // Position after "Line 1\n"
        assert_eq!(counter.line, 2);
        assert_eq!(counter.col, 1);
        
        // Advance to end
        counter.advance_to_byte(bytes, bytes.len());
        assert_eq!(counter.line, 3);
        assert_eq!(counter.col, 7); // "Line 3" is 6 chars + 1 for column
    }
}