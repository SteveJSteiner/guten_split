# Dead Code Cleanup and Elimination

* **Task ID:** dead-code-cleanup-elimination_40.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Assessment found massive dead code warnings throughout codebase
  - 40% reduction in API surface area possible by removing unused code
  - DetectedSentenceOwned entire API unused, many methods never called
  - Clean codebase needed for maintainability and clarity

* **Acceptance Criteria:**
  1. Remove unused DetectedSentenceOwned struct and methods
  2. Remove unused API methods (raw(), normalize_into(), detect_sentences_owned())
  3. Clean up unused imports and re-exports
  4. Remove unused internal fields (DialogDetectedSentence unused fields)
  5. Verify all tests still pass after cleanup
  6. Achieve clean compile with no dead code warnings

* **Deliverables:**
  - Cleaned API surface with only actively used methods
  - Removed unused structs, fields, and imports
  - Clean compile with zero dead code warnings
  - Updated documentation reflecting cleaned API

* **References:**
  - comprehensive-implementation-reality-check_36.stevejs.md FINDING 3
  - Assessment dead code summary: ~40% reduction in API surface area possible

## Specific Cleanup Items:
- [ ] Remove DetectedSentenceOwned struct (never constructed)
- [ ] Remove detect_sentences_owned() method (never called)
- [ ] Remove unused raw() and normalize_into() methods
- [ ] Remove unused dialog_detector::SentenceDetectorDialog re-export
- [ ] Remove unused DialogDetectedSentence fields (start_pos, end_pos, content)
- [ ] Remove unused abbreviation methods (is_abbreviation, ends_with_abbreviation)
- [ ] Remove unused PositionTracker::current_position method

## Pre-commit checklist:
- [ ] All unused API methods removed
- [ ] Unused structs and fields eliminated
- [ ] Clean compile achieved (no dead code warnings)
- [ ] All existing tests still pass (`cargo test`)
- [ ] API documentation updated to reflect cleaned interface
- [ ] Implementation validated with reduced surface area