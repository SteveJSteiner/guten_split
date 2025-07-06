# Test Architecture Simplification

* **Task ID:** test-architecture-simplification_41.stevejs.md
* **Reviewer:** stevejs
* **Area:** tests
* **Motivation (WHY):**
  - Assessment found 60% reduction in test code possible through elimination of redundancy
  - 6+ separate tests for abbreviation handling (should be 1-2)
  - Over-engineered test infrastructure for current needs
  - Test complexity preventing maintainability and clarity

* **Acceptance Criteria:**
  1. Consolidate 6+ abbreviation tests into 1-2 comprehensive tests
  2. Eliminate redundant golden file testing for simple cases
  3. Simplify test infrastructure (remove over-engineered TestFixture/assert_golden_file)
  4. Separate unit/integration concerns properly
  5. Maintain test coverage while reducing code volume
  6. All simplified tests pass and validate behavior correctly

* **Deliverables:**
  - Consolidated abbreviation test suite (1-2 tests vs 6+)
  - Simplified test infrastructure removing over-engineering
  - Clean unit/integration test separation
  - Reduced test code volume while maintaining coverage

* **References:**
  - comprehensive-implementation-reality-check_36.stevejs.md FINDING 4
  - Assessment: 60%+ reduction in test code possible through consolidation

## Specific Simplification Areas:
- [ ] Consolidate abbreviation tests: test_abbreviation_detection, test_geographic_abbreviations, test_measurement_abbreviations, test_multiple_title_abbreviations, test_dialog_with_abbreviations → 1-2 comprehensive tests
- [ ] Remove over-engineered TestFixture infrastructure for simple cases
- [ ] Eliminate golden file testing for trivial 3-sentence scenarios  
- [ ] Consolidate dialog test files (dialog_hard_separator_bug.rs + dialog_state_machine_exploration.rs)
- [ ] Remove redundant detector creation patterns (22+ instances)
- [ ] Simplify integration test utilities (tests/integration/mod.rs complexity)

## Pre-commit checklist:
- [ ] Abbreviation tests consolidated (6+ → 1-2)
- [ ] Test infrastructure simplified
- [ ] All tests pass with reduced code volume (`cargo test`)
- [ ] Test coverage maintained despite simplification
- [ ] Clean unit/integration separation achieved
- [ ] Test maintainability improved through simplification