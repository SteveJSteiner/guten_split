# Configurable File Pattern Support

* **Task ID:** configurable-file-pattern_75.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Make seams useful for literary corpora beyond Project Gutenberg (Harvard, HathiTrust, etc.)
  - Remove hardcoded `*-0.txt` pattern while keeping it as optimized default
  - Enable processing of different file extensions and naming conventions in academic collections
* **Acceptance Criteria:**
  1. Add `--file-pattern` CLI flag with default value `*-0.txt`
  2. Update file discovery logic to use configurable pattern
  3. Update help text to show pattern examples for different corpora
  4. All existing tests pass with default pattern
  5. New tests verify custom pattern functionality
* **Deliverables:**
  - Updated CLI argument structure with `--file-pattern` flag
  - Modified discovery module to accept pattern parameter
  - Updated help text with multi-corpus examples
  - Test coverage for custom file patterns
* **References:**
  - Harvard corpus discussion
  - HathiTrust future support requirements
  - Current hardcoded pattern in discovery module

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely