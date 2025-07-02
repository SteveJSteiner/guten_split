# File Discovery and Validation

* **Task ID:** file-discovery_2
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Implement F-2: recursively locate and stream-process every *-0.txt file
  - Establish foundation for file processing pipeline before sentence extraction
  - Enable early validation of input directory structure and file accessibility
  - Create reusable async file discovery that can be extended with progress tracking
* **Acceptance Criteria:**
  1. `cargo test` passes with unit tests for file discovery logic
  2. Recursively finds all files matching glob `**/*-0.txt` under root directory
  3. Returns async iterator/stream of file paths for processing pipeline
  4. Handles permission errors gracefully (log and continue vs fail_fast)
  5. Validates UTF-8 encoding for discovered files (log encoding issues)
  6. CLI runs end-to-end: discovers files, logs count, exits cleanly with ~/gutenberg_texts
* **Deliverables:**
  - `src/discovery.rs` module with async file discovery implementation
  - Unit tests covering glob matching, error handling, and UTF-8 validation
  - Integration with main.rs to demonstrate file discovery in action
  - Structured logging for discovery progress and errors
* **References:**
  - PRD F-2: Recursively locate and stream-process every *-0.txt
  - PRD F-10: Respect --fail_fast for I/O errors
  - PRD section 6: Error handling requirements (log recoverable errors)