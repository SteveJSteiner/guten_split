# Fix Dialog Detector Initial State Detection Issue

* **Task ID:** dialog-initial-state-detection_93.claude
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current implementation always starts in `DialogState::Narrative` (dialog_detector.rs:736) regardless of text content
  - When text begins with dialog (e.g., `"Hello world"`), the detector fails to recognize SEAM patterns properly
  - Results in 0 debug transitions and incorrect sentence detection for dialog-starting text
  - Debug evidence shows patterns like `!" I` work correctly mid-text but fail at text start
  - According to SEAMS design, detector should start in appropriate state based on first character

* **Acceptance Criteria:**
  1. Dialog detector analyzes first character(s) to determine initial state
  2. Text starting with dialog quotes (`"`, `'`, `"`, `'`) begins in appropriate DialogState
  3. Text starting with parentheses `(`, brackets `[`, braces `{` begins in appropriate DialogState
  4. Text starting with other characters begins in `DialogState::Narrative`
  5. All existing tests continue to pass
  6. Debug transitions are properly generated for dialog-starting text

* **Deliverables:**
  - Modify `detect_sentences_internal` to detect initial state from first character
  - Add helper function to determine initial dialog state from text prefix
  - Update state machine logic to handle dialog-starting text correctly
  - Add unit tests for various dialog-starting scenarios
  - Verify debug output shows proper state transitions for dialog-starting text

* **References:**
  - Current bug: dialog_detector.rs line 736 `let mut current_state = DialogState::Narrative;`
  - Debug evidence: `cargo run --bin seams -- --debug-text '"Stop!" I shouted loudly.'` shows 0 transitions
  - Working example: Text with narrative prefix correctly detects `!" I` as `DialogUnpunctuatedHardEnd`
  - SEAMS-Design.md: State machine should handle dialog appropriately regardless of text position

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely