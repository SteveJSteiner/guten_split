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

## Implementation Results:

**✅ COMPLETED SUCCESSFULLY**

### Key Findings:
1. **Missing Implementation**: The `--stats-out` flag was defined but had no actual implementation
2. **PRD F-8 Compliance**: Required complete stats system with per-file and aggregate metrics
3. **Always-On Behavior**: Stats generation is always enabled (default: run_stats.json in CWD)

### Implementation Details:
- **Added Stats Structures**: `FileStats` and `RunStats` with comprehensive metrics
- **Modified `process_files_parallel()`**: Now collects per-file timing and performance data
- **JSON Output**: Complete stats written to configurable path with pretty formatting
- **Error Handling**: Failed files tracked with error messages in stats output
- **Parallel Processing**: Thread-safe stats collection with accurate aggregate calculations

### JSON Schema Created:
```json
{
  "run_start": "timestamp",
  "total_processing_time_ms": 123,
  "total_chars_processed": 456,
  "total_sentences_detected": 12,
  "overall_chars_per_sec": 98415.94,
  "files_processed": 2,
  "files_skipped": 0,
  "files_failed": 0,
  "file_stats": [
    {
      "path": "file.txt",
      "chars_processed": 239,
      "sentences_detected": 6,
      "processing_time_ms": 0,
      "chars_per_sec": 402667.37,
      "status": "success|skipped|failed",
      "error": null
    }
  ]
}
```

### Testing Coverage:
- **4 Integration Tests**: JSON structure, multiple files, default behavior, skipped files
- **Path Validation**: Relative, absolute, nested directories all work correctly
- **Error Scenarios**: Invalid UTF-8 files handled gracefully
- **Performance Matching**: Aggregate throughput exactly matches CLI display

### Files Modified:
- `src/main.rs`: Added stats structures and generation logic
- `tests/stats_output_integration.rs`: Comprehensive test suite
- `docs/stats-output-format.md`: Complete JSON schema documentation

### Fixed Issues:
- **Test Failure**: Resolved `test_overwrite_all_flag` failure by ensuring stats generation works with empty file sets
- **Clippy Warnings**: Fixed format string warnings to achieve zero-warning build

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (stats output matches CLI performance display)
- [x] Documentation updated if needed
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [x] **Stats validation**: Manual testing of --stats-out with various scenarios

## Follow-up Task Created:
- **Task 59**: Review stats-output default behavior for less noisy UX (consider opt-in vs always-on)