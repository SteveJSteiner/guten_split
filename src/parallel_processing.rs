// WHY: Parallel processing functionality for benchmarking and external use
// Extracted from main.rs to enable benchmark access while maintaining functionality

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncWriteExt, BufWriter};



/// Per-file processing statistics
/// WHY: Collects metrics for each file processed to meet PRD F-8 requirements
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStats {
    /// File path relative to root directory
    pub path: String,
    /// Number of characters processed
    pub chars_processed: u64,
    /// Number of sentences detected
    pub sentences_detected: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Sentence detection time in milliseconds (subset of processing_time_ms)
    pub sentence_detection_time_ms: u64,
    /// Throughput in characters per second
    pub chars_per_sec: f64,
    /// Processing status (success, skipped, failed)
    pub status: String,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Write auxiliary file with borrowed sentence data in F-5 format
/// WHY: Zero-allocation async I/O optimized for mmap-based processing
pub async fn write_auxiliary_file_borrowed(
    aux_path: &Path,
    sentences: &[crate::sentence_detector::DetectedSentenceBorrowed<'_>],
    _detector: &crate::sentence_detector::dialog_detector::SentenceDetectorDialog,
) -> Result<()> {
    let file = tokio::fs::File::create(aux_path).await?;
    let mut writer = BufWriter::new(file);
    
    for sentence in sentences {
        // WHY: Call normalize() on-demand to maintain zero-allocation benefits
        let formatted_line = format!("{}\t{}\t({},{},{},{})", 
            sentence.index, 
            sentence.normalize(),
            sentence.span.start_line,
            sentence.span.start_col,
            sentence.span.end_line,
            sentence.span.end_col
        );
        writer.write_all(formatted_line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    
    writer.flush().await?;
    Ok(())
}

