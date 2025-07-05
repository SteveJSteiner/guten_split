# Dialog Soft Transition Fix

* **Task ID:** dialog-soft-transition-fix_34.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Dialog soft transitions are generically broken - no code distinguishes soft from hard transitions
  - Lines 574-575 in `classify_dialog_end()` show both SOFT_END and HARD_END return the same state: `DialogState::Narrative`
  - SOFT_END patterns are defined (lines 242, 246, 250, etc.) but logic to handle them differently is missing
  - Soft transitions should NOT create sentence boundaries, only state transitions
  - Hard transitions should create sentence boundaries AND state transitions
  - Current implementation treats all dialog endings as hard boundaries, breaking dialog coalescing

* **Acceptance Criteria:**
  1. **Distinguish Soft vs Hard Transitions**: SOFT_END should not create sentence boundaries, only state changes
  2. **Proper Soft Transition Logic**: When dialog ends with just closing punctuation (no sentence terminator), continue sentence
  3. **Maintain Hard Transition Logic**: When dialog ends with sentence punctuation + separator + sentence start, create boundary
  4. **State Machine Consistency**: Soft transitions should maintain or transition dialog state appropriately
  5. **Comprehensive Testing**: Add tests covering all soft transition scenarios (quotes, parentheticals, etc.)
  6. **Fix Existing Tests**: The dialog hard separator bug test should benefit from proper soft transition logic

* **Deliverables:**
  - Fix `classify_dialog_end()` to return different states/actions for SOFT_END vs HARD_END
  - Implement proper soft transition handling in main detection loop
  - Add comprehensive test suite for soft dialog transitions
  - Update dialog state machine logic to handle soft transitions correctly
  - Document soft vs hard transition rules and examples

* **Current Problem Analysis:**

### Broken Code Location (dialog_detector.rs:564-577):
```rust
fn classify_dialog_end(&self, matched_text: &str) -> (MatchType, DialogState) {
    // Check if this is a HARD_END (sentence punctuation + close + separator) or SOFT_END (just close)
    let has_sentence_punct = matched_text.chars().any(|c| ".!?".contains(c));
    let has_separator = matched_text.chars().any(char::is_whitespace);
    
    if has_sentence_punct && has_separator {
        // HARD_END: This creates a sentence boundary and transitions to Narrative
        (MatchType::DialogEnd, DialogState::Narrative)
    } else {
        // SOFT_END: Just dialog close, creates soft transition, not hard boundary
        // Return DialogEnd but maintain dialog state for soft transition
        (MatchType::DialogEnd, DialogState::Narrative)  // ❌ BROKEN: Same action as HARD_END
    }
}
```

### Missing Functionality:
1. **No `MatchType::DialogSoftEnd`** - only `DialogEnd` exists, treated as hard boundary
2. **No soft state handling** - both soft and hard return same `DialogState::Narrative`
3. **No sentence continuation logic** - soft ends should continue current sentence
4. **No proper state transitions** - soft ends should transition dialog state without sentence boundary

### Example Broken Scenarios:
- `"Hello," she said` - comma + quote should soft transition, continue sentence
- `"Yes"` followed by narrative - quote alone should soft transition  
- `(thinking quietly)` - parenthetical close should soft transition
- `"Wait!" he shouted` - exclamation + space + capital should hard transition, create boundary

* **Implementation Strategy:**
0. **Reproduce the problem** by extending the unit tests from the examples above.
1. **Add `MatchType::DialogSoftEnd`** to distinguish from `DialogEnd` (hard)
2. **Update `classify_dialog_end()`** to return `DialogSoftEnd` for soft cases
3. **Add soft transition handling** in main detection loop (around line 405)
4. **Implement sentence continuation logic** for soft transitions
5. **Add comprehensive test coverage** for all dialog ending types

* **References:**
  - `src/sentence_detector/dialog_detector.rs:564-577` - broken `classify_dialog_end()`
  - `src/sentence_detector/dialog_detector.rs:238-267` - soft pattern definitions  
  - `tests/dialog_hard_separator_bug.rs` - may benefit from soft transition fix
  - Dialog coalescing design in exploration docs

## Implementation Summary:

### Changes Made:
1. **Added `MatchType::DialogSoftEnd` enum variant** - distinguishes soft transitions from hard dialog endings
2. **Fixed `classify_dialog_end()` logic** - now returns `DialogSoftEnd` for soft cases (just closing punctuation) vs `DialogEnd` for hard cases (punctuation + separator + capital)
3. **Implemented soft transition handling** - added `MatchType::DialogSoftEnd` case in main detection loop that performs state transition without creating sentence boundary
4. **Added comprehensive test cases** - `test_soft_dialog_transitions()` and `test_hard_dialog_transitions()` to verify correct behavior

### Verification:
- ✅ All 30 unit tests pass including new soft/hard transition tests  
- ✅ Soft transitions like `"Hello," she said` now continue as one sentence
- ✅ Hard transitions like `"Wait!" He shouted. Then he left.` still create proper boundaries
- ✅ Core dialog coalescing functionality maintained

### Technical Details:
- **Line 205**: Added `DialogSoftEnd` to `MatchType` enum
- **Line 576**: Fixed `classify_dialog_end()` to return `DialogSoftEnd` for soft cases
- **Line 442-446**: Added soft transition handling that skips sentence boundary creation
- **Lines 857-898**: Added test cases reproducing and verifying the fix

## Pre-commit checklist:
- [x] Add `MatchType::DialogSoftEnd` enum variant
- [x] Fix `classify_dialog_end()` to return different types for soft vs hard
- [x] Implement soft transition handling in main detection loop  
- [x] Add comprehensive test cases for soft dialog transitions
- [x] Verify all existing tests still pass
- [x] Document soft vs hard transition rules with examples
- [x] Claims validated (soft transitions don't create sentence boundaries)