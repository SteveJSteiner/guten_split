# Integration Test Framework Implementation

* **Task ID:** integration-test-framework_6
* **Reviewer:** stevejs
* **Area:** tests
* **Motivation (WHY):**
  - Core components (discovery, reader, sentence_detector) exist but lack end-to-end validation
  - Unit tests validate individual modules but miss integration issues between components
  - PRD requires integration tests with golden-file validation (acceptance criteria 2-3)
  - Current development relies on manual testing for complete pipeline validation
  - Need automated verification that discovery→reader→sentence_detector pipeline produces correct output format
  - Integration tests will catch regressions during refactoring and enable confident concurrent processing implementation
* **Acceptance Criteria:**
  1. Integration tests validate complete discovery→reader→sentence_detector pipeline
  2. Tests execute within 2-minute budget as defined in testing strategy
  3. Golden-file validation using known Gutenberg text samples
  4. Output format validation: index<TAB>sentence<TAB>(start_line,start_col,end_line,end_col)
  5. Tests detect false concurrency claims (e.g., reader.rs:147 sequential processing)
  6. Aux file generation and incremental processing validation
  7. Error handling scenarios covered (malformed files, permission issues)
* **Deliverables:**
  - `tests/integration/` directory with pipeline integration tests
  - Test fixtures: sample Gutenberg texts with expected sentence outputs
  - Golden-file comparison framework for deterministic output validation
  - Integration test runner that respects 2-minute budget constraint
  - Documentation of integration test patterns for future features
* **References:**
  - PRD section 8 (acceptance criteria 2-3) for golden-file requirements
  - docs/testing-strategy.md for 2-minute budget optimization
  - reader.rs:147 false concurrency example for validation target
  - Current unit test structure for integration patterns

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Integration tests complete within 2-minute budget
- [ ] Golden-file validation working with sample texts
- [ ] Claims validated (integration tests actually test end-to-end pipeline)
- [ ] Documentation updated with integration test patterns
- [ ] Clippy warnings addressed