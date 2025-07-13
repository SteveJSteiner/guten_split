# CLI Debug Convenience Function

* **Task ID:** cli-debug-convenience_91.steve
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current CLI requires creating temporary files to debug single text snippets
  - Pattern debugging workflow is cumbersome for development and testing
  - Need immediate debug output for SEAM analysis without file I/O overhead
  - Developers frequently need to test specific text patterns during dialog detector work
* **Acceptance Criteria:**
  1. CLI accepts `--debug-text "text here"` flag for direct text input
  2. CLI accepts `--debug-stdin` flag to read from standard input
  3. Debug output shows state transitions, patterns, and SEAM analysis
  4. Output format matches existing `_seams-debug.txt` format but to stdout
  5. Works without creating temporary files or auxiliary outputs
  6. Integrates with existing debug infrastructure (`detect_sentences_borrowed_with_debug`)
* **Deliverables:**
  - New CLI flags: `--debug-text` and `--debug-stdin`
  - Debug output handler that writes to stdout instead of files
  - Integration with existing sentence detector debug functionality
  - Documentation of new convenience flags
* **References:**
  - Current debug workflow requires temporary files and navigation
  - Existing `--debug-seams` flag and infrastructure in `src/main.rs`
  - Dialog detector debug functionality in `src/sentence_detector/dialog_detector.rs`

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (convenience flags work as expected)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely