# Validate --fail-fast Flag with Parallel Processing

* **Task ID:** fail-fast-parallel-validation_53.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - PRD F-10 requires --fail-fast to abort entire run on first I/O/UTF-8/DFA error
  - Current parallel processing implementation may not handle fail-fast correctly
  - Need to ensure fail-fast behavior works with concurrent file processing
  - Critical for deterministic error handling in production pipelines

* **Acceptance Criteria:**
  1. --fail-fast immediately aborts entire run on first error across all parallel tasks
  2. Error propagation works correctly from any parallel worker
  3. Cleanup happens properly when aborting parallel processing
  4. Exit code is non-zero when fail-fast triggers
  5. Error logging provides clear context about which file/task failed
  6. No race conditions between error detection and shutdown

* **Deliverables:**
  - Validate current fail-fast implementation with parallel processing
  - Fix any issues with error propagation across parallel tasks
  - Add integration tests for fail-fast scenarios
  - Ensure proper cleanup when aborting parallel execution
  - Document fail-fast behavior with parallel processing

* **Test Scenarios:**
  1. I/O error (permission denied, missing file) during parallel processing
  2. UTF-8 encoding error in one of many files being processed
  3. DFA error during sentence detection
  4. Multiple simultaneous errors across different parallel tasks
  5. Error occurs while other tasks are still starting up

* **Implementation Considerations:**
  - Review Semaphore and tokio::spawn usage for proper cancellation
  - Ensure error propagation works with bounded concurrent processing
  - Verify cache save doesn't happen on failed runs

* **References:**
  - PRD F-10: --fail-fast abort behavior
  - Current parallel processing implementation in process_files_parallel()

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (fail-fast works correctly with parallel processing)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Integration tests**: Manual testing of fail-fast with various error scenarios