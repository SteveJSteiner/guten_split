// WHY: Main detector interface with dual API for borrowed vs owned data experiments
// Enables performance comparison between mmap-friendly and async I/O-friendly approaches


pub mod normalization;
pub mod dialog_detector;
pub mod abbreviations;

// Re-export core types
pub use normalization::normalize_sentence;
pub use abbreviations::AbbreviationChecker;

/// Position in a text file using 1-based indexing as specified in PRD section 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}


/// Borrowed variant - zero allocation detection (mmap-optimized)
#[derive(Debug, Clone)]
pub struct DetectedSentenceBorrowed<'a> {
    pub index: usize,
    pub raw_content: &'a str,  // Borrowed from source text
    pub span: Span,
}

impl<'a> DetectedSentenceBorrowed<'a> {
    /// Normalize content with new allocation
    pub fn normalize(&self) -> String {
        normalize_sentence(self.raw_content)
    }
}


