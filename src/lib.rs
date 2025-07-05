pub mod discovery;
pub mod reader;
pub mod sentence_detector;

// Re-export main types for convenient access
pub use sentence_detector::{
    SentenceDetector, SentenceDetectorDFA, 
    DetectedSentence, DetectedSentenceBorrowed, DetectedSentenceOwned, 
    Span, SentenceBoundaryRules
};
pub use sentence_detector::dialog_detector::SentenceDetectorDialog;