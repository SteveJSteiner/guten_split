# Fix misleading debug state transition names

* **Task ID:** fix-debug-state-names_94.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current debug output shows misleading pattern names like "DialogUnpunctuatedSoftEnd" for pattern ID 6
  - The example `!" q` demonstrates this issue - the pattern actually matches punctuated endings (`[.!?]"`) but is named "Unpunctuated"
  - State transition names are hardcoded and don't reflect the actual patterns being matched in different dialog states
  - This makes debugging and understanding the state machine behavior confusing for developers

* **Acceptance Criteria:**
  1. Debug output shows accurate pattern names that reflect what each pattern actually matches
  2. Pattern ID 6 in DialogDoubleQuote state shows "DialogPunctuatedSoftEnd" instead of "DialogUnpunctuatedSoftEnd"
  3. Pattern names are state-aware and specific to the actual regex patterns being used
  4. All tests pass (`cargo test`)
  5. Debug output example `!" q` shows correct transition name

* **Deliverables:**
  - Updated `dialog_detector.rs` pattern_name function to be state-aware
  - Accurate pattern names for all dialog states and pattern combinations
  - Updated tests if needed to reflect new naming

* **References:**
  - Current issue in `/Users/stevejs/guten_split/src/sentence_detector/dialog_detector.rs` line 250
  - Debug output example from README.md showing `--debug-text` usage
  - Pattern definitions in dialog detector show actual regex patterns vs names

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely