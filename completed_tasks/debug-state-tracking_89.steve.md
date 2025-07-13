# Debug State Transition Tracking Implementation

* **Task ID:** debug-state-tracking_89.steve
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current debug output uses placeholder data instead of actual state transitions
  - Need to capture real SEAM pattern matches and state changes during detection
  - Critical for debugging the `sword" (A.D. 1656)` regression where 1 sentence is produced instead of 2
  - Debug tracking must have zero performance impact when debug mode is disabled
* **Acceptance Criteria:**
  1. Capture actual state transitions (before/after states) for each sentence boundary
  2. Record which regex pattern matched for each SEAM detection
  3. Show transition type (Continue/Split) decisions with reasoning
  4. Zero performance overhead when `debug_seams=false`
  5. Real data replaces placeholder values in `_seams-debug.txt` TSV output
* **Deliverables:**
  - Debug-enabled `detect_sentences_with_debug()` method in DialogStateMachine
  - `DebugTransitionInfo` struct to capture SEAM analysis details
  - Conditional debug tracking that only activates when `debug_seams=true`
  - Updated parallel_processing to pass debug info to TSV writer
  - Real state transition data in debug TSV format
* **References:**
  - task_88: Basic debug framework (completed)
  - NOTE-Debug.md: Failing test case pattern
  - SEAMS-Design.md: Expected pattern types and state transitions

## Pre-commit checklist:
- [ ] All deliverables implemented  
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (debug TSV shows real pattern matches for failing case)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely

## Implementation Strategy:
1. **Conditional API**: Add `detect_sentences_with_debug()` method that returns `(Vec<DialogDetectedSentence>, Vec<DebugTransitionInfo>)`
2. **Debug Data Structure**: 
   ```rust
   struct DebugTransitionInfo {
       sentence_index: usize,
       state_before: DialogState,
       state_after: DialogState,
       transition_type: TransitionType, // Continue/Split
       matched_pattern: String,
       pattern_name: String,
       seam_text: String, // The actual SEAM that was analyzed
   }
   ```
3. **Zero-Cost Abstraction**: Only collect debug info when explicitly requested
4. **Threading Support**: Debug info collection must work with parallel processing

## Expected Debug Output for Test Case:
For `He said "word" (A.D. 1656). From this description we know.`:
- Should show the specific pattern that matches (or fails to match) at `1656). From`
- Should reveal why it's not creating a sentence boundary at that SEAM
- Enable precise diagnosis of the missing external definitive punctuation patterns