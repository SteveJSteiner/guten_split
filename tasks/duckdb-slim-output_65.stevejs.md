# DuckDB Slim Output Format

* **Task ID:** duckdb-slim-output_65.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current aux file output creates many individual files alongside sources
  - Database format enables efficient querying and analysis of sentence data
  - Slim output reduces filesystem clutter and improves data management
  - Byte offsets are more efficient than line/column coordinates for processing
  - DuckDB provides high-performance columnar storage for analytical workloads
  - Single database file is easier to manage than thousands of aux files

* **Acceptance Criteria:**
  1. Add `--slim-output <DATABASE_PATH>` CLI flag to enable DuckDB output
  2. Create DuckDB database with two tables: `files` and `sentences`
  3. `files` table: `fileId INTEGER PRIMARY KEY, local_file_path TEXT`
  4. `sentences` table: `fileId INTEGER, sentenceId INTEGER, start_byte INTEGER, end_byte INTEGER`
  5. Process files and populate database instead of writing aux files when flag is used
  6. Maintain existing aux file output as default behavior
  7. Database should be created/opened efficiently with proper error handling
  8. All existing functionality (progress bars, stats, incremental processing) works with slim output
  9. Foreign key relationship between files and sentences tables

* **Deliverables:**
  - Add duckdb crate dependency to Cargo.toml
  - Implement DuckDB database creation and schema setup
  - Add CLI flag parsing for --slim-output option
  - Create database output writer alongside existing aux file writer
  - Update processing pipeline to support both output formats
  - Add tests for database output format
  - Update documentation for new output option

* **Database Schema:**
```sql
CREATE TABLE files (
    fileId INTEGER PRIMARY KEY,
    local_file_path TEXT NOT NULL
);

CREATE TABLE sentences (
    fileId INTEGER NOT NULL,
    sentenceId INTEGER NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    FOREIGN KEY (fileId) REFERENCES files(fileId)
);
```

* **Implementation Notes:**
  - Use incremental fileId assignment starting from 1
  - sentenceId should be 1-based per file (reset for each file)
  - Byte offsets should be relative to original file content
  - Database should be created if it doesn't exist
  - Consider batch inserts for performance
  - Handle database locking and concurrent access properly

* **References:**
  - Current aux file output format in src/
  - DuckDB Rust crate documentation
  - Existing CLI argument parsing structure

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] DuckDB dependency added to Cargo.toml
- [ ] CLI flag parsing implemented and tested
- [ ] Database schema created correctly
- [ ] Both output formats work independently
- [ ] Tests passing (`cargo test`)
- [ ] Database output produces correct fileId and byte offset mappings
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] Foreign key constraints working properly
- [ ] Error handling for database operations
- [ ] Documentation updated for new output format