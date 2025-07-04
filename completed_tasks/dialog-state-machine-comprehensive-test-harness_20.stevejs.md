# Dialog State Machine Comprehensive Test Harness

* **Task ID:** dialog-state-machine-comprehensive-test-harness_20.stevejs
* **Reviewer:** stevejs
* **Area:** tests
* **Algorithm:** Dialog State Machine implementation in `/tests/dialog_state_machine_exploration.rs`
* **Motivation (WHY):**
  - Current Dialog State Machine has incorrect pattern matching and state transitions
  - Pattern `. "` incorrectly classified as `DialogOpen` instead of `NarrativeGestureBoundary`
  - Pattern `"` incorrectly creates sentence boundary when it should be `SOFT_DIALOG_END`
  - Need exhaustive testing of every possible boundary pattern and state transition
  - Current ad-hoc tests miss critical edge cases and pattern interactions

* **Acceptance Criteria:**
  1. Data-driven test harness that enumerates ALL boundary character combinations
  2. Explicit test cases for every regex pattern vs expected state transition
  3. Comprehensive coverage of narrative boundaries, dialog opens, dialog ends
  4. Each test case specifies: input pattern, current state, expected match type, expected next state
  5. Test harness validates actual behavior against expected behavior for each case
  6. All existing Dialog State Machine tests continue to pass
  7. New test harness identifies all current incorrect behaviors

* **Deliverables:**
  - Comprehensive test harness in `/tests/dialog_state_machine_exploration.rs`
  - Exhaustive test case data structure covering all boundary patterns
  - Test runner that validates every pattern/state combination
  - Documentation of expected vs actual behavior for each pattern

* **Test Case Categories:**
  1. **Narrative Boundaries**: All combinations of `[.!?] + [space/tab] + [A-Z/quotes/parens]`
  2. **Dialog Opens**: All combinations of quote/paren characters in narrative context
  3. **Hard Dialog Ends**: All combinations of `[.!?] + [close] + [space] + [sentence_start]`
  4. **Soft Dialog Ends**: All combinations of `[close]` without sentence_start following
  5. **Hard Separators**: `\n\n` patterns in all states
  6. **State Transitions**: Every valid state transition with its triggering pattern

* **Test Data Structure:**
```rust
struct BoundaryTestCase {
    name: String,
    input_pattern: String,
    current_state: DialogState,
    expected_match_type: MatchType,
    expected_next_state: DialogState,
    context_before: String,  // Text before the pattern
    context_after: String,   // Text after the pattern
}
```

* **Pattern Enumeration:**
  - **Sentence ending punctuation**: `.`, `!`, `?`
  - **Separators**: ` ` (space), `\t` (tab), `\n\n` (hard)
  - **Sentence start chars**: `A-Z`, `"`, `'`, `"`, `'`, `(`, `[`, `{`
  - **Quote closes**: `"`, `'`, `"`, `'`
  - **Paren closes**: `)`, `]`, `}`
  - **Non-sentence chars**: `a-z` (lowercase), `,`, `;`, `:`

## Pre-commit checklist:
- [x] Comprehensive test case data structure implemented
- [x] Test harness validates all boundary pattern combinations
- [x] All existing dialog state machine tests continue to pass
- [x] Test harness identifies current incorrect pattern classifications
- [x] Documentation shows expected vs actual behavior for each test case
- [x] Test coverage includes all state transitions and boundary types