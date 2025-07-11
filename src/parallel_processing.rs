// WHY: Parallel processing functionality for benchmarking and external use
// Extracted from main.rs to enable benchmark access while maintaining functionality

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncWriteExt, BufWriter};



/// Sentence length distribution statistics
/// WHY: Provides statistical analysis of sentence lengths for literary research
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SentenceLengthStats {
    /// Minimum sentence length in characters
    pub min_length: u64,
    /// Maximum sentence length in characters
    pub max_length: u64,
    /// Mean sentence length in characters
    pub mean_length: f64,
    /// Median sentence length in characters
    pub median_length: f64,
    /// 25th percentile sentence length
    pub p25_length: f64,
    /// 75th percentile sentence length
    pub p75_length: f64,
    /// 90th percentile sentence length
    pub p90_length: f64,
    /// Standard deviation of sentence lengths
    pub std_dev: f64,
}

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
    /// Sentence length distribution statistics
    pub sentence_length_stats: Option<SentenceLengthStats>,
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
    let file = tokio::fs::File::create(aux_path).await
        .map_err(|e| anyhow::anyhow!(
            "Cannot create output file: {}\nError: {}\n\nSUGGESTIONS:\n• Check write permissions for the directory\n• Ensure sufficient disk space is available\n• Verify the directory exists and is writable",
            aux_path.display(), e
        ))?;
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
        writer.write_all(formatted_line.as_bytes()).await
            .map_err(|e| anyhow::anyhow!(
                "Cannot write to output file: {}\nError: {}\n\nSUGGESTIONS:\n• Check available disk space\n• Ensure write permissions are maintained\n• File system may be full or read-only",
                aux_path.display(), e
            ))?;
        writer.write_all(b"\n").await
            .map_err(|e| anyhow::anyhow!(
                "Cannot write to output file: {}\nError: {}\n\nSUGGESTIONS:\n• Check available disk space\n• Ensure write permissions are maintained\n• File system may be full or read-only",
                aux_path.display(), e
            ))?;
    }
    
    writer.flush().await
        .map_err(|e| anyhow::anyhow!(
            "Cannot finalize output file: {}\nError: {}\n\nSUGGESTIONS:\n• Check available disk space\n• Ensure write permissions are maintained\n• File system may be full or read-only",
            aux_path.display(), e
        ))?;
    Ok(())
}

/// Calculate sentence length statistics for literary analysis
/// WHY: Provides statistical distribution of sentence lengths for research
pub fn calculate_sentence_length_stats(
    sentences: &[crate::sentence_detector::DetectedSentenceBorrowed<'_>]
) -> Option<SentenceLengthStats> {
    if sentences.is_empty() {
        return None;
    }
    
    // Calculate lengths for each sentence
    let mut lengths: Vec<u64> = sentences.iter()
        .map(|s| s.normalize().chars().count() as u64)
        .collect();
    
    if lengths.is_empty() {
        return None;
    }
    
    // Sort for percentile calculations
    lengths.sort_unstable();
    
    let min_length = *lengths.first().unwrap();
    let max_length = *lengths.last().unwrap();
    
    // Calculate mean
    let sum: u64 = lengths.iter().sum();
    let mean_length = sum as f64 / lengths.len() as f64;
    
    // Calculate median
    let median_length = if lengths.len() % 2 == 0 {
        let mid = lengths.len() / 2;
        (lengths[mid - 1] + lengths[mid]) as f64 / 2.0
    } else {
        lengths[lengths.len() / 2] as f64
    };
    
    // Calculate percentiles
    let p25_idx = (lengths.len() as f64 * 0.25) as usize;
    let p75_idx = (lengths.len() as f64 * 0.75) as usize;
    let p90_idx = (lengths.len() as f64 * 0.90) as usize;
    
    let p25_length = lengths[p25_idx.min(lengths.len() - 1)] as f64;
    let p75_length = lengths[p75_idx.min(lengths.len() - 1)] as f64;
    let p90_length = lengths[p90_idx.min(lengths.len() - 1)] as f64;
    
    // Calculate standard deviation
    let variance = lengths.iter()
        .map(|&x| {
            let diff = x as f64 - mean_length;
            diff * diff
        })
        .sum::<f64>() / lengths.len() as f64;
    let std_dev = variance.sqrt();
    
    Some(SentenceLengthStats {
        min_length,
        max_length,
        mean_length,
        median_length,
        p25_length,
        p75_length,
        p90_length,
        std_dev,
    })
}

/// Calculate aggregate sentence length statistics from multiple files
/// WHY: Provides overall sentence length distribution across entire dataset
pub fn calculate_aggregate_sentence_length_stats(file_stats: &[FileStats]) -> Option<SentenceLengthStats> {
    // Collect all sentence lengths from files that have statistics
    let mut all_lengths: Vec<u64> = Vec::new();
    
    for stats in file_stats {
        if let Some(ref length_stats) = stats.sentence_length_stats {
            // We need to reconstruct individual lengths from the statistics
            // For now, we'll use a simplified approach with mean and count
            let count = stats.sentences_detected;
            if count > 0 {
                // Approximate distribution using mean length
                for _ in 0..count {
                    all_lengths.push(length_stats.mean_length as u64);
                }
            }
        }
    }
    
    if all_lengths.is_empty() {
        return None;
    }
    
    // Sort for percentile calculations
    all_lengths.sort_unstable();
    
    let min_length = *all_lengths.first().unwrap();
    let max_length = *all_lengths.last().unwrap();
    
    // Calculate mean
    let sum: u64 = all_lengths.iter().sum();
    let mean_length = sum as f64 / all_lengths.len() as f64;
    
    // Calculate median
    let median_length = if all_lengths.len() % 2 == 0 {
        let mid = all_lengths.len() / 2;
        (all_lengths[mid - 1] + all_lengths[mid]) as f64 / 2.0
    } else {
        all_lengths[all_lengths.len() / 2] as f64
    };
    
    // Calculate percentiles
    let p25_idx = (all_lengths.len() as f64 * 0.25) as usize;
    let p75_idx = (all_lengths.len() as f64 * 0.75) as usize;
    let p90_idx = (all_lengths.len() as f64 * 0.90) as usize;
    
    let p25_length = all_lengths[p25_idx.min(all_lengths.len() - 1)] as f64;
    let p75_length = all_lengths[p75_idx.min(all_lengths.len() - 1)] as f64;
    let p90_length = all_lengths[p90_idx.min(all_lengths.len() - 1)] as f64;
    
    // Calculate standard deviation
    let variance = all_lengths.iter()
        .map(|&x| {
            let diff = x as f64 - mean_length;
            diff * diff
        })
        .sum::<f64>() / all_lengths.len() as f64;
    let std_dev = variance.sqrt();
    
    Some(SentenceLengthStats {
        min_length,
        max_length,
        mean_length,
        median_length,
        p25_length,
        p75_length,
        p90_length,
        std_dev,
    })
}

