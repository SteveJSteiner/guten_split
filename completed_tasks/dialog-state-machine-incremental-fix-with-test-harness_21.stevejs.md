# Dialog State Machine Incremental Fix with Test Harness

* **Task ID:** dialog-state-machine-incremental-fix-with-test-harness_21.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Algorithm:** Dialog State Machine implementation in `/tests/dialog_state_machine_exploration.rs`
* **Motivation (WHY):**
  - Test `test_false_negative_dialog_over_coalescing` fails (produces 4 sentences instead of 5+)
  - Known issues: `. "` misclassified as `DialogOpen` instead of `NarrativeGestureBoundary`
  - Known issues: Bare `"` incorrectly creates hard sentence boundary instead of soft transition
  - Need systematic fix with regression protection using new comprehensive test harness
  - Current ad-hoc fixes risk breaking working patterns without detection

* **Strategy:**
  - **Phase 1**: Establish comprehensive baseline using test harness (all 162 patterns)
  - **Phase 2**: Mark working patterns as validated to lock in correct behavior
  - **Phase 3**: Fix known issues incrementally with test harness validation
  - **Phase 4**: Validate against Oliver Twist full text processing

* **Acceptance Criteria:**
  1. **Baseline Population**: All 162 boundary test patterns have recorded baseline behavior
  2. **Validation State**: Working patterns marked as `validated: true` in `boundary_validation_state.json`
  3. **Period-Quote-Space Fix**: `. "` correctly classified as `NarrativeGestureBoundary`
  4. **Soft Dialog End Fix**: Bare `"` creates soft transition, not hard sentence boundary
  5. **Regression Protection**: All validated patterns continue to pass after fixes
  6. **Oliver Twist Validation**: Process `/Users/stevejs/gutenberg_texts/7/3/730/730-0.txt` with 5+ sentences in test case
  7. **Test Suite Success**: `test_false_negative_dialog_over_coalescing` passes

* **Deliverables:**
  - Updated `tests/boundary_validation_state.json` with full baseline and validation flags
  - Fixed classification logic in `classify_match()` function
  - Fixed dialog end logic in `classify_dialog_end()` function
  - Passing test suite including `test_false_negative_dialog_over_coalescing`
  - Generated `.norm_sm_sents` file for Oliver Twist with proper dialog splitting

* **Implementation Plan:**

## Phase 1: Establish Comprehensive Baseline
1. **Modify test limit**: Change `test_populate_baseline_behavior` from 50 to 162 tests
2. **Run baseline population**: `cargo test test_populate_baseline_behavior -- --nocapture`
3. **Review generated state**: Examine `tests/boundary_validation_state.json` for patterns
4. **Document current behavior**: Note which patterns work vs. need attention

## Phase 2: Lock in Working Patterns
1. **Identify working patterns**: Review baseline results for correct classifications
2. **Mark as validated**: Set `validated: true` for confirmed correct patterns in `boundary_validation_state.json`
3. **Leave problematic patterns unvalidated**: Keep known issues as `validated: false`
4. **Test validation workflow**: Ensure test harness correctly identifies validated vs. unvalidated changes

## Phase 3: Fix Classification Logic
1. **Fix period-quote-space issue**:
   - Location: `classify_match()` around line 503
   - Issue: Pattern `. "` incorrectly matches dialog open logic
   - Fix: Ensure narrative boundary logic takes precedence over dialog open detection
2. **Fix soft dialog end issue**:
   - Location: `classify_dialog_end()` around line 547
   - Issue: Bare `"` creates hard boundary instead of soft transition
   - Fix: Distinguish between hard end (with sentence punctuation) and soft end (just close)
3. **Test each fix**: Run test harness after each change to detect regressions

## Phase 4: Validate Against Real Text
1. **Process Oliver Twist**: Run Dialog State Machine against full text
2. **Check sentence count**: Verify test case produces 5+ sentences instead of 4
3. **Manual inspection**: Review `.norm_sm_sents` output for proper dialog splitting
4. **Update test expectations**: Adjust test assertions based on fixed behavior

* **Specific Test Cases to Address:**
  - **FALSE_POSITIVE #7**: `"They had been strangers too long. \"It's all over, Mrs. Thingummy!\" said the surgeon at last."`
    - Expected: 2 sentences with proper boundaries
    - Current: Incorrect boundary at `. "`
  - **FALSE_NEGATIVE Oliver Twist**: Complex dialog conversation
    - Expected: 5+ sentences with proper dialog/attribution splitting
    - Current: 4 sentences due to over-coalescing

* **Error Patterns to Fix:**
```rust
// ISSUE 1: classify_match() line ~508
if matched_text.contains('"') {
    (MatchType::DialogOpen, DialogState::DialogDoubleQuote)
} 
// Should check for narrative boundary first: contains("[.!?]") && contains(char::is_whitespace)

// ISSUE 2: classify_dialog_end() line ~547
// All dialog ends currently return DialogEnd, should distinguish soft vs hard
```

## Pre-commit checklist:
- [ ] All 162 boundary patterns have baseline behavior recorded
- [ ] Working patterns marked as `validated: true` in boundary_validation_state.json
- [ ] Period-quote-space pattern correctly classified as NarrativeGestureBoundary
- [ ] Bare quote creates soft transition, not hard sentence boundary
- [ ] `cargo test test_generated_boundary_cases` shows no validated pattern regressions
- [ ] `cargo test test_false_negative_dialog_over_coalescing` passes
- [ ] Oliver Twist processing generates proper dialog sentence splitting
- [ ] Manual validation of `.norm_sm_sents` output confirms correct boundaries