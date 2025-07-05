pub mod discovery;
pub mod reader;
pub mod sentence_detector;

// Re-export main types for convenient access
pub use sentence_detector::{
    DetectedSentence, DetectedSentenceBorrowed, DetectedSentenceOwned, 
    Span, SentenceDetectorDialog
};