# Dialog State Machine Over-Coalescing Fix

* **Task ID:** dialog-state-machine-over-coalescing-fix_19.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Algorithm:** Dialog State Machine implementation in `/tests/dialog_state_machine_exploration.rs`
* **Motivation (WHY):**
  - Test `test_false_negative_dialog_over_coalescing` in `dialog_state_machine_exploration.rs` fails
  - Dialog State Machine algorithm produces 4 sentences instead of expected 5+
  - Based on FALSE_NEGATIVE_examples.txt showing over-coalescing issue
  - **Critical**: When running Dialog State Machine against full Oliver Twist text, the entire problematic section becomes 1 massive sentence instead of proper splitting

* **Test Failure:**
```rust
thread 'tests::test_false_negative_dialog_over_coalescing' panicked at tests/dialog_state_machine_exploration.rs:720:9:
Expected at least 5 sentences but got 4. Dialog state machine is over-coalescing!
```

* **Actual Output (4 sentences):**
1. `(He stirred the gin-and-water.)`
2. `"I—I drink your health with cheerfulness, Mrs.`
3. `Mann"; and he swallowed half of it. "And now about business," said the beadle, taking out a leathern pocket-book. "The child that was half-baptized Oliver Twist, is nine year old today."`
4. `"Bless him!" interposed Mrs. Mann, inflaming her left eye with the corner of her apron.`

* **Test Input:**
```rust
let text = r#"(He stirred the gin-and-water.) "I—I drink your health with cheerfulness, Mrs. Mann"; and he swallowed half of it. "And now about business," said the beadle, taking out a leathern pocket-book. "The child that was half-baptized Oliver Twist, is nine year old today." "Bless him!" interposed Mrs. Mann, inflaming her left eye with the corner of her apron."#;
```

* **Specific Tests Required:**
  1. **Unit Test**: `cargo test --test dialog_state_machine_exploration test_false_negative_dialog_over_coalescing`
  2. **Regression Tests**: `cargo test --test dialog_state_machine_exploration` (all 6 tests must pass)
  3. **Binary Generation**: `GUTENBERG_MIRROR_DIR=~/gutenberg_texts cargo run --bin generate_gutenberg_sentences` (generates .norm_sm_sents files)
  4. **Manual Validation**: Inspect `/Users/stevejs/gutenberg_texts/7/3/730/730-0.txt.norm_sm_sents` output for proper dialog splitting

* **Acceptance Criteria:**
  1. `test_false_negative_dialog_over_coalescing` test passes (produces 5+ sentences)
  2. All existing Dialog State Machine tests continue to pass (6/6 tests)
  3. **Run against Oliver Twist**: Process `/Users/stevejs/gutenberg_texts/7/3/730/730-0.txt` with Dialog State Machine algorithm
  4. **Validate by hand**: Manual inspection of Oliver Twist `.norm_sm_sents` output to confirm proper sentence boundaries in dialog sections

* **Deliverables:**
  - Fix Dialog State Machine implementation in `/tests/dialog_state_machine_exploration.rs`
  - Ensure all unit tests pass
  - Generate and manually validate Oliver Twist sentence output using Dialog State Machine

## Pre-commit checklist:
- [ ] `cargo test --test dialog_state_machine_exploration test_false_negative_dialog_over_coalescing` passes
- [ ] `cargo test --test dialog_state_machine_exploration` passes (all 6 tests)
- [ ] Oliver Twist processed with Dialog State Machine: `/Users/stevejs/gutenberg_texts/7/3/730/730-0.txt.norm_sm_sents` generated
- [ ] Manual validation of Oliver Twist `.norm_sm_sents` output shows proper dialog sentence splitting