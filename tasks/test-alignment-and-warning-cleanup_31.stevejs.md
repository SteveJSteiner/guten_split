# Test Alignment and Warning Cleanup

* **Task ID:** test-alignment-and-warning-cleanup_31.stevejs
* **Reviewer:** stevejs
* **Area:** tests
* **Motivation (WHY):**
  - Current tests compare experimental functions against broken production code, hiding real issues
  - Test warnings indicate dead code and unused functionality that needs cleanup
  - Tests should validate actual production Dialog detector, not experimental functions
  - Test coverage gaps prevent catching real sentence boundary detection issues
  - Warning-free tests are required for clean CI builds

* **Acceptance Criteria:**
  1. **Production Test Coverage**: All tests validate production Dialog detector behavior
  2. **Warning-Free Tests**: All compiler warnings in test files resolved
  3. **Test Data Validation**: Test cases use realistic sentence boundary scenarios
  4. **Abbreviation Test Integration**: Tests validate abbreviation handling in Dialog detector
  5. **Performance Test Alignment**: Benchmark tests focus on Dialog detector performance
  6. **Dead Code Cleanup**: Remove unused test functions and imports

* **Deliverables:**
  - **Updated Test Files**: All test files validate production Dialog detector
  - **Cleaned Test Code**: Remove experimental functions and dead code from tests
  - **Warning-Free Build**: All compiler warnings in test files resolved
  - **Comprehensive Test Suite**: Tests cover all Dialog detector functionality including abbreviations
  - **Performance Validation**: Benchmark tests validate Dialog detector performance claims

* **Implementation Plan:**
  1. **Audit Test Coverage**: Identify tests that validate experimental vs production code
  2. **Update Test Functions**: Replace experimental function calls with production Dialog detector
  3. **Clean Dead Code**: Remove unused test functions and imports
  4. **Fix Warnings**: Address all compiler warnings in test files
  5. **Validate Coverage**: Ensure all Dialog detector functionality is tested

* **References:**
  - Test files in `tests/` directory
  - Dialog detector implementation in `src/sentence_detector/dialog_detector.rs`
  - Abbreviation test cases in `tests/abbreviation_exploration.rs`
  - Current benchmark implementations

## Pre-commit checklist:
- [ ] All tests validate production Dialog detector (not experimental functions)
- [ ] All compiler warnings in test files resolved
- [ ] Dead code and unused imports removed from tests
- [ ] Test cases cover all Dialog detector functionality
- [ ] Abbreviation handling tests integrated with Dialog detector
- [ ] Performance tests validate Dialog detector benchmarks
- [ ] Test suite passes completely without warnings