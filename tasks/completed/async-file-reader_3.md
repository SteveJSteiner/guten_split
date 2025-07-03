# Async File Reader Implementation

* **Task ID:** async-file-reader_3
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Implement F-4: async buffered file reading foundation before sentence processing
  - Establish UTF-8 text streaming that can handle large Gutenberg files efficiently
  - Create reusable async reader that integrates with existing file discovery
  - Build toward F-9: detect partial/complete aux files by reading to EOF
* **Acceptance Criteria:**
  1. `cargo test` passes with unit tests for async file reading
  2. Reads discovered *-0.txt files using Tokio's async buffered reader
  3. Validates UTF-8 encoding and reports invalid sequences with file context
  4. Handles I/O errors gracefully (respects --fail_fast flag)
  5. Streams file contents line-by-line to avoid memory spikes on large files
  6. Logs reading progress and performance metrics per file
* **Deliverables:**
  - `src/reader.rs` module with async buffered file reading
  - Unit tests covering UTF-8 validation, error handling, and streaming
  - Integration with discovery module to process found files
  - Basic line-by-line processing loop in main.rs
  - Structured logging for read progress and errors
  - Criterion benchmark measuring real Gutenberg file processing performance (~226μs for 5 files)
* **References:**
  - PRD F-4: Read each file with async buffered reader (Tokio)
  - PRD F-10: Respect --fail_fast for I/O/UTF-8 errors
  - PRD section 10: Risk mitigation for large files (stream processing)