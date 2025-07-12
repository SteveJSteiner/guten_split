# Fix Parenthetical Dialog Patterns

**STATUS: SUPERCEDED BY TASK 85** - `unify-dialog-pattern-partitioning_85.claude.md`

This task identified the parenthetical issue but the root cause affects all dialog types. Task 85 provides a comprehensive unified solution with 4-pattern partitioning instead of ad-hoc parenthetical fixes.

* **Task ID:** fix-parenthetical-dialog-patterns_84.claude.md
* **Reviewer:** stevejs  
* **Area:** code
* **Motivation (WHY):**
  - Parenthetical expressions like `(rightly)` incorrectly merge entire sentences due to flawed closing patterns
  - Current patterns require sentence punctuation BEFORE closing `)` which doesn't match real usage
  - Parentheticals differ from quotes: they contain single words/phrases without internal punctuation
  - Bug causes test case "She doubted (rightly) if her nature would endure.  Stenography was unknown." to become 1 sentence instead of 2

* **Acceptance Criteria:**
  1. Test `test_parenthetical_state_transitions` passes with correct sentence splitting
  2. Parenthetical patterns detect any `)` regardless of preceding punctuation
  3. Patterns correctly distinguish hard vs soft endings based on what follows the `)`
  4. All existing dialog tests continue to pass (no regression)
  5. **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely

* **Deliverables:**
  - Update parenthetical closing patterns in `src/sentence_detector/dialog_detector.rs`
  - Differentiate parenthetical patterns from quote patterns
  - Validate with existing test suite

* **References:**
  - Bug diagnosis in `tests/dialog_detector_tests.rs::test_parenthetical_state_transitions`
  - Current flawed patterns at lines 509-510 in dialog_detector.rs
  - Debug tracing capability added with `debug-states` feature

## Implementation Plan:

1. **Analyze Current Quote vs Parenthetical Patterns**
   - Quotes: `"Hello."` - expect punctuation before close
   - Parentheticals: `(rightly)` - no punctuation expected

2. **Design New Parenthetical Patterns**
   - Hard end: `)` + sentence punct + whitespace + sentence start → `DialogEnd`
   - Soft end: `)` + non-sentence context → `DialogSoftEnd`  
   - Simple close: `)` + whitespace + lowercase/continuation → `DialogSoftEnd`

3. **Update Pattern Definitions**
   - Replace `dialog_hard_paren_round_end` pattern
   - Replace `dialog_soft_paren_round_end` pattern
   - Apply same logic to square brackets and curly braces

4. **Validation**
   - Test with debug tracing enabled
   - Verify state transitions: Narrative → DialogParenthheticalRound → Narrative
   - Ensure proper sentence boundary creation

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Debug tracing shows correct state transitions (`cargo test --features debug-states`)
- [ ] Claims validated (parentheticals no longer merge sentences)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely