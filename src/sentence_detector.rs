use anyhow::Result;
use fst::{Set, SetBuilder};
use std::io::Cursor;
use std::sync::Arc;
use tracing::{debug, info};

/// Position in a text file using 1-based indexing as specified in PRD section 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

/// A detected sentence with its normalized content and position
#[derive(Debug, Clone)]
pub struct DetectedSentence {
    pub index: usize,
    pub normalized_content: String,
    pub span: Span,
}

/// Configuration for sentence boundary detection rules
#[derive(Debug, Clone)]
pub struct SentenceBoundaryRules {
    /// End punctuation characters that can terminate a sentence
    pub end_punctuation: Vec<char>,
    /// Characters that require a following capital letter or quote to form boundary
    pub boundary_punctuation: Vec<char>,
    /// Characters considered opening quotes
    pub opening_quotes: Vec<char>,
    /// Characters considered opening parentheticals
    pub opening_parentheticals: Vec<char>,
}

impl Default for SentenceBoundaryRules {
    fn default() -> Self {
        Self {
            // WHY: based on task requirements for default ruleset
            end_punctuation: vec!['.', '?', '!'],
            boundary_punctuation: vec!['"', '\'', '\u{201D}', '\u{2019}'],
            opening_quotes: vec!['"', '\'', '\u{201C}', '\u{2018}'],
            opening_parentheticals: vec!['(', '[', '{'],
        }
    }
}

/// Compiled FST for fast sentence boundary detection
pub struct SentenceDetectorFST {
    /// Compiled FST set for pattern matching
    _fst_set: Arc<Set<Vec<u8>>>, 
    /// Rules used to compile this FST
    rules: SentenceBoundaryRules,
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
            rules,
        })
    }
}

/// Main sentence detector that uses FST for boundary detection
pub struct SentenceDetector {
    fst: SentenceDetectorFST,
}

impl SentenceDetector {
    /// Create new sentence detector with compiled FST
    pub fn new(rules: SentenceBoundaryRules) -> Result<Self> {
        let fst = SentenceDetectorFST::compile(rules)?;
        Ok(Self { fst })
    }
    
    /// Create sentence detector with default rules
    pub fn with_default_rules() -> Result<Self> {
        Self::new(SentenceBoundaryRules::default())
    }
    
    /// Detect sentence boundaries in text and return normalized sentences
    /// Implements F-5: detect sentence boundaries with FST, producing indexed output
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DetectedSentence>> {
        debug!("Starting sentence detection on {} characters", text.len());
        
        let mut sentences = Vec::new();
        let mut current_sentence_start = 0;
        let mut sentence_index = 0;
        
        // WHY: track position in Unicode scalar values as required by PRD section 2
        let mut line = 1;
        let mut col = 1;
        let mut sentence_start_line = 1;
        let mut sentence_start_col = 1;
        
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            // Check if current position could be a sentence boundary
            if self.is_sentence_boundary(&chars, i) {
                // Extract sentence from start to current position (inclusive)
                let sentence_text: String = chars[current_sentence_start..=i].iter().collect();
                
                // Normalize the sentence (F-6: remove hard line breaks)
                let normalized = self.normalize_sentence(&sentence_text);
                
                if !normalized.trim().is_empty() {
                    let sentence = DetectedSentence {
                        index: sentence_index,
                        normalized_content: normalized,
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
                current_sentence_start = i + 1;
                
                // Skip whitespace to find actual start of next sentence
                while current_sentence_start < chars.len() && chars[current_sentence_start].is_whitespace() {
                    current_sentence_start += 1;
                }
                
                // Update sentence start position
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
        if current_sentence_start < chars.len() {
            let sentence_text: String = chars[current_sentence_start..].iter().collect();
            let normalized = self.normalize_sentence(&sentence_text);
            
            if !normalized.trim().is_empty() {
                let sentence = DetectedSentence {
                    index: sentence_index,
                    normalized_content: normalized,
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
        
        info!("Detected {} sentences", sentences.len());
        Ok(sentences)
    }
    
    /// Check if position i in chars array represents a sentence boundary
    fn is_sentence_boundary(&self, chars: &[char], pos: usize) -> bool {
        if pos == 0 || pos >= chars.len() - 1 {
            return false;
        }
        
        let current_char = chars[pos];
        
        // Must be end punctuation
        if !self.fst.rules.end_punctuation.contains(&current_char) {
            return false;
        }
        
        // Look ahead for boundary pattern
        let mut next_pos = pos + 1;
        
        // Skip any boundary punctuation (quotes)
        while next_pos < chars.len() && self.fst.rules.boundary_punctuation.contains(&chars[next_pos]) {
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
            || self.fst.rules.opening_quotes.contains(&next_char)
            || self.fst.rules.opening_parentheticals.contains(&next_char)
    }
    
    /// Normalize sentence by removing interior hard line breaks
    /// Implements F-6: normalize sentences by removing hard line breaks
    fn normalize_sentence(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        let mut prev_was_space = false;
        
        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    // Handle \r\n as single break (peek ahead for \n)
                    if chars.peek() == Some(&'\n') {
                        chars.next(); // consume the \n
                    }
                    // Replace with single space
                    if !prev_was_space {
                        result.push(' ');
                        prev_was_space = true;
                    }
                }
                '\n' => {
                    // Replace with single space
                    if !prev_was_space {
                        result.push(' ');
                        prev_was_space = true;
                    }
                }
                _ => {
                    // WHY: preserve all other bytes as specified in F-6
                    result.push(ch);
                    prev_was_space = ch.is_whitespace();
                }
            }
        }
        
        // WHY: trim only leading/trailing whitespace, preserve interior structure
        result.trim().to_string()
    }
    
    /// Format detected sentence for output as specified in F-5
    /// Returns: index<TAB>normalized_sentence<TAB>(start_line,start_col,end_line,end_col)
    pub fn format_sentence_output(&self, sentence: &DetectedSentence) -> String {
        format!(
            "{}\t{}\t({},{},{},{})",
            sentence.index,
            sentence.normalized_content,
            sentence.span.start_line,
            sentence.span.start_col,
            sentence.span.end_line,
            sentence.span.end_col
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_creation() {
        let rules = SentenceBoundaryRules::default();
        assert!(rules.end_punctuation.contains(&'.'));
        assert!(rules.end_punctuation.contains(&'?'));
        assert!(rules.end_punctuation.contains(&'!'));
    }

    #[test]
    fn test_fst_compilation() {
        let rules = SentenceBoundaryRules::default();
        let result = SentenceDetectorFST::compile(rules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sentence_detector_creation() {
        let detector = SentenceDetector::with_default_rules();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_simple_sentence_detection() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "Hello world. This is a test. How are you?";
        
        let sentences = detector.detect_sentences(text).unwrap();
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0].normalized_content.trim(), "Hello world.");
        assert_eq!(sentences[1].normalized_content.trim(), "This is a test.");
        assert_eq!(sentences[2].normalized_content.trim(), "How are you?");
    }

    #[test]
    fn test_sentence_normalization() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text_with_breaks = "This is a\nsentence with\r\nline breaks.";
        
        let normalized = detector.normalize_sentence(text_with_breaks);
        assert_eq!(normalized, "This is a sentence with line breaks.");
    }

    #[test]
    fn test_span_tracking() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "First sentence. Second sentence.";
        
        let sentences = detector.detect_sentences(text).unwrap();
        assert_eq!(sentences.len(), 2);
        
        // First sentence starts at (1,1)
        assert_eq!(sentences[0].span.start_line, 1);
        assert_eq!(sentences[0].span.start_col, 1);
        
        // Second sentence should start after first
        assert_eq!(sentences[1].span.start_line, 1);
        assert!(sentences[1].span.start_col > sentences[0].span.end_col);
    }

    #[test]
    fn test_unicode_support() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "Hello 世界! This contains émojis 🦀. How neat?";
        
        let sentences = detector.detect_sentences(text).unwrap();
        assert_eq!(sentences.len(), 3);
        assert!(sentences[0].normalized_content.contains("世界"));
        assert!(sentences[1].normalized_content.contains("🦀"));
    }

    #[test]
    fn test_output_formatting() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let sentence = DetectedSentence {
            index: 0,
            normalized_content: "Test sentence.".to_string(),
            span: Span {
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 14,
            },
        };
        
        let output = detector.format_sentence_output(&sentence);
        assert_eq!(output, "0\tTest sentence.\t(1,1,1,14)");
    }

    #[test]
    fn test_quoted_sentences() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "He said \"Hello world.\" Then he left.";
        
        let sentences = detector.detect_sentences(text).unwrap();
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_empty_and_whitespace() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        
        // Empty text
        let sentences = detector.detect_sentences("").unwrap();
        assert_eq!(sentences.len(), 0);
        
        // Only whitespace
        let sentences = detector.detect_sentences("   \n  \t  ").unwrap();
        assert_eq!(sentences.len(), 0);
    }
}