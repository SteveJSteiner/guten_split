# Regex Multi-Pattern Dialog Detection Refactor

* **Task ID:** regex-multi-pattern-dialog-detection_81.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current dialog detection uses complex post-processing logic to disambiguate combined regex patterns
  - The `classify_dialog_end()` function manually inspects matched text to determine if it's a hard or soft boundary
  - This leads to bugs like incorrectly splitting `"no!" interposed` where attribution should continue the sentence
  - `regex-automata` provides `Regex::new_many()` which can directly tell us which pattern matched via `PatternID`
  - This eliminates the need for post-processing disambiguation and makes the logic clearer

* **Acceptance Criteria:**
  1. Replace combined patterns with separate patterns using `Regex::new_many()`
  2. Map `PatternID` directly to `(MatchType, DialogState)` tuples
  3. Eliminate `classify_dialog_end()` function and its manual text inspection
  4. Fix the attribution bug: `"no!" interposed` should be one sentence, not split
  5. All existing tests pass
  6. Add specific test case for the attribution bug

* **Deliverables:**
  - Modified `DialogStateMachine::new()` to use `Regex::new_many()` approach
  - Updated detection loop to use `PatternID` for direct classification
  - Removed `classify_dialog_end()` and related disambiguation logic
  - New test case demonstrating the attribution fix
  - Updated dialog patterns to handle attribution correctly

* **References:**
  - [regex-automata multi-pattern documentation](https://docs.rs/regex-automata/latest/regex_automata/)
  - Current bug: `"Lor bless her dear heart, no!" interposed the nurse` incorrectly splits into 2 sentences instead of 1

## Technical Design

### Current Problem
The current approach uses combined patterns like:
```rust
let dialog_double_end = format!("(?:{dialog_hard_double_end})|(?:{dialog_soft_double_end})");
```

Then tries to figure out which sub-pattern matched in `classify_dialog_end()`:
```rust
fn classify_dialog_end(&self, matched_text: &str) -> (MatchType, DialogState) {
    let has_sentence_punct = matched_text.chars().any(|c| ".!?".contains(c));
    let has_separator = matched_text.chars().any(char::is_whitespace);
    // Complex logic to guess what happened...
}
```

This leads to incorrect classification of `"no!" interposed` as two sentences.

### Proposed Solution
Use `Regex::new_many()` with explicit pattern ordering:

```rust
// Each pattern maps to a specific semantic action
let patterns = [
    &dialog_hard_double_end,      // PatternID 0 = sentence boundary
    &dialog_soft_double_end,      // PatternID 1 = continue sentence  
    &dialog_attribution_pattern,  // PatternID 2 = continue sentence (attribution)
    &narrative_soft_boundary,     // PatternID 3 = sentence boundary
    &pure_hard_sep,              // PatternID 4 = paragraph break
    // ... other patterns
];

let re = Regex::new_many(&patterns)?;
```

In the detection loop:
```rust
if re.captures(&mut cache, text_from_position, &mut caps).is_some() {
    let (match_type, next_state) = match caps.pattern().unwrap().as_usize() {
        0 => (MatchType::DialogEnd, DialogState::Narrative),
        1 => (MatchType::DialogSoftEnd, DialogState::Narrative), 
        2 => (MatchType::DialogSoftEnd, DialogState::Narrative), // Attribution continues
        3 => (MatchType::NarrativeGestureBoundary, DialogState::Narrative),
        4 => (MatchType::HardSeparator, DialogState::Unknown),
        _ => unreachable!(),
    };
    // Use match_type and next_state directly - no disambiguation needed
}
```

### Test Case for Bug Fix
```rust
#[test]
fn test_dialog_attribution_no_split() {
    let detector = get_detector();
    let text = r#""Lor bless her dear heart, no!" interposed the nurse, hastily
depositing in her pocket a green glass bottle, the contents of which
she had been tasting in a corner with evident satisfaction."#;
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // Should be ONE sentence (dialog + attribution), not split at "no!" interposed
    assert_eq!(sentences.len(), 1, "Dialog with attribution should not be split");
    assert!(sentences[0].raw_content.contains("no!"));
    assert!(sentences[0].raw_content.contains("interposed"));
    assert!(sentences[0].raw_content.contains("satisfaction"));
}
```

### Pattern Priority
The order in the patterns array determines priority. Attribution patterns should come before general soft dialog ends to ensure `"no!" interposed` matches the attribution pattern rather than just the soft dialog end.

### Benefits
1. **Direct mapping**: PatternID → behavior, no guessing
2. **Performance**: Stays on DFA fast path, no capture group overhead
3. **Clarity**: Each pattern has one specific semantic meaning
4. **Correctness**: Explicit attribution handling prevents incorrect splits
5. **Maintainability**: Adding new patterns just requires extending the array and match statement

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] Attribution bug test case passes
- [ ] No regression in existing dialog detection behavior