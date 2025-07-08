# Validate and Fix --stats-out Flag Implementation

* **Task ID:** stats-out-flag-validation_52.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - PRD F-8 requires stats aggregation into run_stats.json with total chars/sec
  - Current CLI shows --stats-out flag but implementation may be broken
  - Need to validate stats output works correctly with parallel mmap processing
  - Stats should include per-file and aggregate performance metrics

* **Acceptance Criteria:**
  1. --stats-out flag creates valid JSON file at specified path
  2. run_stats.json includes per-file stats (chars processed, sentences, wall-clock ms)
  3. Aggregate stats include total chars/sec matching CLI throughput display
  4. Stats work correctly with parallel processing (no race conditions)
  5. Default behavior creates run_stats.json in current working directory
  6. Stats include failed file counts and error information

* **Deliverables:**
  - Validate current stats implementation with parallel processing
  - Fix any issues with JSON generation or file writing
  - Ensure stats match CLI performance output (chars/sec consistency)
  - Add tests for stats output functionality
  - Document stats JSON schema

* **Investigation Required:**
  1. Test --stats-out with various file paths (relative, absolute, existing/non-existing dirs)
  2. Verify JSON format matches PRD F-8 requirements
  3. Check if parallel processing affects stats accuracy
  4. Validate aggregate throughput calculation matches CLI display

* **References:**
  - PRD F-8: Generate per-file stats and aggregate into run_stats.json
  - CLI throughput display implementation in main.rs

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (stats output matches CLI performance display)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Stats validation**: Manual testing of --stats-out with various scenarios