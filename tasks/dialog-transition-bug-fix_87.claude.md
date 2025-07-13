# Fix Dialog Quote Transition Regression

* **Task ID:** dialog-transition-bug-fix_87.claude.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - TOML-based dialog pattern refactor introduced a behavioral regression causing -50,617 sentences difference across 20K Project Gutenberg corpus
  - Root cause: Narrative→Dialog state transitions not properly triggered by patterns like "word \"quote"
  - Critical bug prevents the refactor from being a true zero-diff replacement of the original 700-line manual pattern code

* **Acceptance Criteria:**
  1. Unit test `test_dialog_quote_transition_regression` passes (currently failing)
  2. Text `we read of an "Azacari...sword" (A.D. 1656). From this description...` produces exactly 2 sentences (currently produces 1)
  3. Full corpus validation shows exactly zero sentence count difference (currently -50,617)
  4. All existing tests continue to pass
  5. Zero warnings from `./scripts/validate_warning_free.sh`

* **Deliverables:**
  - Fix pattern generation in `build.rs` or `dialog_patterns.toml` to properly handle Narrative→Dialog transitions
  - Ensure "an \"quote" pattern triggers Dialog state entry
  - Verify dialog state machine correctly processes quoted content with parenthetical dates
  - Update dialog transition patterns to match original behavior exactly

* **References:**
  - Failing test: `tests/dialog_detector_tests.rs::test_dialog_quote_transition_regression`
  - Pattern configuration: `dialog_patterns.toml`
  - Code generation: `build.rs`
  - Original implementation: `src/sentence_detector/dialog_detector.rs` (before refactor)
  - Validation data: `test-0.txt` → `test-0_seams2.txt` (1 sentence instead of 2)

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **ZERO BEHAVIORAL DIFFERENCE**: Full corpus validation shows exact match, not -50,617 difference