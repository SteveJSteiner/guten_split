# Dialog State-Specific Exit Patterns

* **Task ID:** dialog-state-specific-exit-patterns_90.steve
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Dialog states incorrectly exit on mismatched delimiters (e.g., `(` ending `DialogDoubleQuote`)
  - Current `DialogSoftEnd` pattern allows ANY dialog delimiter to end ANY dialog state
  - Debug analysis shows: `DialogDoubleQuote → Narrative` on `(` instead of only on `"`
  - Each dialog type must only exit on its specific matching close character via REGEX patterns
* **Acceptance Criteria:**
  1. `DialogDoubleQuote` state only exits on `"` character matches
  2. `DialogSingleQuote` state only exits on `'` character matches  
  3. `DialogParenthheticalRound` state only exits on `)` character matches
  4. `DialogParenthheticalSquare` state only exits on `]` character matches
  5. `DialogParenthheticalCurly` state only exits on `}` character matches
  6. `DialogSmartDoubleOpen` state only exits on `"` (smart close) character matches
  7. `DialogSmartSingleOpen` state only exits on `'` (smart close) character matches
  8. Test case `sword" (A.D. 1656)` produces 2 sentences instead of 1
* **Deliverables:**
  - Modified dialog patterns in pattern configuration files
  - State-specific exit patterns replacing generic `DialogSoftEnd`
  - Updated regex patterns for each dialog state's exit conditions
  - Validation that parentheses no longer prematurely exit quote dialog states
* **References:**
  - Debug analysis showing `DialogDoubleQuote → Narrative` on `(` transition
  - SEAMS-Design.md line 389-392: missing external definitive punctuation patterns
  - Task interactive-debug-seam_88.steve: debug infrastructure revealing this issue

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (debug shows state-specific exit patterns working)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely