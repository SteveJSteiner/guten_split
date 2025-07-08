pub mod discovery;
pub mod sentence_detector;
pub mod incremental;
pub mod parallel_processing;

// Re-export main types for convenient access
pub use sentence_detector::{
    DetectedSentenceBorrowed, 
    Span
};

// Re-export incremental processing utilities
pub use incremental::{
    generate_aux_file_path, aux_file_exists, 
    create_complete_aux_file, generate_cache_path, cache_exists, read_cache, read_cache_async
};

// Re-export parallel processing types and functions for benchmarking
pub use parallel_processing::{
    ProcessingCache, FileMetadata, FileStats, 
    process_files_parallel, should_process_file
};