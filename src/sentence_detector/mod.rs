// WHY: Main detector interface with dual API for borrowed vs owned data experiments
// Enables performance comparison between mmap-friendly and async I/O-friendly approaches

use anyhow::Result;

pub mod normalization;
pub mod fst_detector;
pub mod dfa_detector;
pub mod dialog_detector;
pub mod abbreviations;

// Re-export core types
pub use normalization::{normalize_sentence, normalize_sentence_into};
// pub use dialog_detector::SentenceDetectorDialog;
pub use abbreviations::AbbreviationChecker;

/// Position in a text file using 1-based indexing as specified in PRD section 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
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

/// Borrowed variant - zero allocation detection (mmap-optimized)
#[derive(Debug, Clone)]
pub struct DetectedSentenceBorrowed<'a> {
    pub index: usize,
    pub raw_content: &'a str,  // Borrowed from source text
    pub span: Span,
}

impl<'a> DetectedSentenceBorrowed<'a> {
    /// Get raw content without normalization
    pub fn raw(&self) -> &str {
        self.raw_content
    }
    
    /// Normalize content with new allocation
    pub fn normalize(&self) -> String {
        normalize_sentence(self.raw_content)
    }
    
    /// Normalize content into supplied buffer (zero allocation)
    pub fn normalize_into(&self, buffer: &mut String) {
        normalize_sentence_into(self.raw_content, buffer);
    }
}

/// Owned variant - convenience for async I/O scenarios
#[derive(Debug, Clone)]
pub struct DetectedSentenceOwned {
    pub index: usize,
    pub raw_content: String,  // Owned copy
    pub span: Span,
}

impl DetectedSentenceOwned {
    /// Get raw content without normalization
    pub fn raw(&self) -> &str {
        &self.raw_content
    }
    
    /// Normalize content with new allocation
    pub fn normalize(&self) -> String {
        normalize_sentence(&self.raw_content)
    }
    
    /// Normalize content into supplied buffer (zero allocation)
    pub fn normalize_into(&self, buffer: &mut String) {
        normalize_sentence_into(&self.raw_content, buffer);
    }
}

/// Legacy struct for backward compatibility - will be deprecated
#[derive(Debug, Clone)]
pub struct DetectedSentence {
    pub index: usize,
    pub normalized_content: String,
    pub span: Span,
}

/// Main sentence detector interface with dual API support
pub struct SentenceDetector {
    rules: SentenceBoundaryRules,
}

impl SentenceDetector {
    /// Create new sentence detector with custom rules
    pub fn new(rules: SentenceBoundaryRules) -> Result<Self> {
        Ok(Self { rules })
    }
    
    /// Create sentence detector with default rules
    pub fn with_default_rules() -> Result<Self> {
        Self::new(SentenceBoundaryRules::default())
    }
    
    /// Detect sentences with borrowed API (mmap-friendly, zero allocations)
    pub fn detect_sentences_borrowed<'a>(&self, text: &'a str) -> Result<Vec<DetectedSentenceBorrowed<'a>>> {
        fst_detector::detect_sentences_borrowed(text, &self.rules)
    }
    
    /// Detect sentences with owned API (async I/O-friendly)
    pub fn detect_sentences_owned(&self, text: &str) -> Result<Vec<DetectedSentenceOwned>> {
        fst_detector::detect_sentences_owned(text, &self.rules)
    }
    
    /// Legacy API - detect and normalize sentences immediately
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DetectedSentence>> {
        fst_detector::detect_sentences_legacy(text, &self.rules)
    }
    
    /// Convenience method - detect and normalize sentences (for CLI)
    pub fn detect_sentences_normalized(&self, text: &str) -> Result<Vec<(usize, String, Span)>> {
        let borrowed_sentences = self.detect_sentences_borrowed(text)?;
        let mut buffer = String::new();
        let mut result = Vec::with_capacity(borrowed_sentences.len());
        
        for sentence in borrowed_sentences {
            sentence.normalize_into(&mut buffer);
            result.push((sentence.index, buffer.clone(), sentence.span));
        }
        
        Ok(result)
    }
    
    /// Format detected sentence for output as specified in F-5
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

/// DFA-based sentence detector for performance comparison
pub struct SentenceDetectorDFA {
    // Implementation will be moved to dfa_detector module
}

impl SentenceDetectorDFA {
    /// Create new DFA-based sentence detector
    pub fn new() -> Result<Self> {
        dfa_detector::SentenceDetectorDFA::new().map(|_| Self {})
    }
    
    /// Legacy API for backward compatibility
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DetectedSentence>> {
        dfa_detector::detect_sentences_legacy(text)
    }
    
    /// Borrowed API for zero-allocation detection
    pub fn detect_sentences_borrowed<'a>(&self, text: &'a str) -> Result<Vec<DetectedSentenceBorrowed<'a>>> {
        dfa_detector::detect_sentences_borrowed(text)
    }
    
    /// Owned API for async I/O scenarios
    pub fn detect_sentences_owned(&self, text: &str) -> Result<Vec<DetectedSentenceOwned>> {
        dfa_detector::detect_sentences_owned(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_api_equivalence() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "Hello world. This is a test. How are you?";
        
        let borrowed_result = detector.detect_sentences_borrowed(text).unwrap();
        let owned_result = detector.detect_sentences_owned(text).unwrap();
        
        assert_eq!(borrowed_result.len(), owned_result.len());
        
        for (borrowed, owned) in borrowed_result.iter().zip(owned_result.iter()) {
            assert_eq!(borrowed.index, owned.index);
            assert_eq!(borrowed.raw(), owned.raw());
            assert_eq!(borrowed.span, owned.span);
            assert_eq!(borrowed.normalize(), owned.normalize());
        }
    }

    #[test]
    fn test_normalization_buffer_reuse() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "First sentence. Second sentence.";
        
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        let mut buffer = String::new();
        let mut normalized = Vec::new();
        
        for sentence in sentences {
            sentence.normalize_into(&mut buffer);
            normalized.push(buffer.clone());
        }
        
        assert_eq!(normalized.len(), 2);
        assert!(normalized[0].contains("First sentence"));
        assert!(normalized[1].contains("Second sentence"));
    }

    #[test]
    fn test_convenience_method() {
        let detector = SentenceDetector::with_default_rules().unwrap();
        let text = "Hello world. This is a test.";
        
        let result = detector.detect_sentences_normalized(text).unwrap();
        assert_eq!(result.len(), 2);
        
        let (index, content, span) = &result[0];
        assert_eq!(*index, 0);
        assert!(content.contains("Hello world"));
        assert_eq!(span.start_line, 1);
    }
}