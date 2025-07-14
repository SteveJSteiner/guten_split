# Dialog Detection Debugging Log

## Issue: Over-splitting after soft_separator change

### Background
Changed `soft_separator` from `r"[ \t]+"` to `r"[ \t]|\r?\n"` to treat single newlines as equivalent to spaces. This caused 4 tests to fail due to over-splitting.

### Root Cause Analysis

#### Problem 1: Abbreviation Handling Failure
**Test case**: `"The U.S.A. declared independence. It was 1776."`
**Expected**: 2 sentences
**Actual**: 3 sentences (`'The U.S.A.'`, `'declared independence.'`, `'It was 1776.'`)

**Debug output**:
```
DEBUG: State=Narrative, PatternID=1, MatchedText='. ', MatchType=NarrativeGestureBoundary, NextState=Narrative
```

**Key insight**: The pattern is matching `'. '` (dot + space) WITHOUT considering the next character. By design, this should NOT match because the following character `'d'` is lowercase and in `not(sentence_start_chars)`.

**Issue**: The `narrative_soft_boundary` pattern appears to be `[.!?]([ \t]|\r?\n)[A-Z...]` which should only match when followed by uppercase, but somehow `'. d'` (dot + space + lowercase) is still creating a boundary.

**Hypothesis**: The `sentence_start_chars` constraint is not being applied correctly in the narrative pattern, or a different pattern without this constraint is matching.

#### Problem 2: Dialog Hard Separator Minimal
**Debug output**:
```
DEBUG: State=DialogDoubleQuote, PatternID=1, MatchedText='."

', MatchType=DialogSoftEnd, NextState=Narrative
```

**Issue**: Dialog pattern consuming too much text (including newlines) when it should only match the quote character context.

### Solution Attempted
Implemented explicit hard/soft dialog patterns:
- **Hard**: `[.!?]"([ \t]|\r?\n)[A-Z...]` (sentence boundary when followed by uppercase)
- **Soft**: `[.!?]"([ \t]|\r?\n)[^A-Z...]` (continue sentence when followed by lowercase)

Used helper function:
```rust
fn negate_char_class(char_class: &str) -> String {
    format!("[^{}]", &char_class[1..char_class.len()-1])
}
```

### Current Status
- ✅ Dialog transitions fixed (test_hard_dialog_transitions now passes)
- ✅ Abbreviation handling fixed (test_abbreviation_handling now passes)
- ❌ Dialog hard separator minimal still broken

### Update
After adding parentheses to the soft_separator in narrative_soft_boundary pattern:
```rust
let narrative_soft_boundary = format!("{sentence_end_punct}({soft_separator}){sentence_start_chars}");
```

The pattern is now correctly formed as:
```
'[.!?]([ \t]|\r?\n)[A-Z\x22\x27\u{201C}\u{2018}\(\[\{]'
```

And debug output shows proper matching:
```
DEBUG: State=Narrative, PatternID=1, MatchedText='. S', MatchType=NarrativeGestureBoundary, NextState=Narrative
```

This indicates the pattern correctly requires uppercase letters after punctuation + separator.

### Key Architectural Insight: Abstraction Levels

**Problem**: Mixing paragraph breaks with soft separators
- **Paragraph breaks** (`\n\n`, `\r\n\r\n`, `\ns+`) are **HARD_SEP** abstraction level
- **Soft separators** (single space, single newline) are different abstraction level
- Current `soft_separator = r"[ \t]|\r?\n"` allows patterns to match across paragraph boundaries

**Solution**: Soft separators should explicitly exclude paragraph breaks
```rust
let paragraph_break = r"(?:\r\n\r\n|\n\n|\n\s+)";  // All paragraph break patterns
let soft_separator = r"(?:[ \t]|\r?\n)(?!paragraph_break_lookahead)";  // NOT paragraph breaks
```

### Questions for Clarification:

1. **Paragraph break patterns**: Should `\ns+` (newline + whitespace) be considered a paragraph break? What about `\n   \n` (newline + spaces + newline)?

2. **Negative lookahead**: Should soft_separator use negative lookahead `(?!\n\n)` to prevent matching when followed by paragraph breaks?

3. **Pattern precedence**: Should paragraph break detection happen BEFORE soft separator patterns, or should soft patterns explicitly exclude paragraph breaks?

4. **Edge cases**: What about mixed line endings like `\r\n\n` or `\n\r\n`?

### Architecture: State Machine Transitions and Hard Separators

**Core Problem**: Unclosed dialog spanning forever is bad
- Dialog states can "capture" text indefinitely if no proper end transition
- Hard separators serve as "escape hatches" to force state transitions
- **IMPORTANT**: Hard separators help protect against the problem, but are NOT an invariant
- **Context matters**: Some patterns like `:` + hard_sep + `<dialog>` indicate continuation, not breaks

### Implementation Options

#### Option B: Restrictive soft_separator Definition
```rust
let soft_separator = r"[ \t]+";  // Only spaces/tabs within same line
let line_break = r"\r?\n";       // Single newlines handled separately
```
**Pros**: Clear separation, prevents cross-paragraph matching
**Cons**: Need to handle single newlines explicitly in patterns

#### Option C: Pattern Priority (Hard Before Soft)
```rust
// In multi-pattern arrays, put hard separator patterns FIRST
let dialog_patterns = vec![
    pure_hard_sep.as_str(),           // PatternID 0 = paragraph break (HIGHEST PRIORITY)
    dialog_hard_end.as_str(),         // PatternID 1 = sentence boundary
    dialog_soft_end.as_str(),         // PatternID 2 = continue sentence
];
```
**Pros**: Explicit precedence, paragraph breaks always win
**Cons**: Need to reorder all pattern arrays

#### Option D: "Outer" State Machine with END_OF_LINE Transitions
```rust
let end_of_line_narrative = r"{non_abbreviation}\.\r?\n{sentence_start}";
```
**Additional transition**: Force exit from Dialog → Narrative on line boundaries with sentence structure
**Pros**: Handles unclosed dialog gracefully, respects sentence structure
**Cons**: More complex state machine, need robust abbreviation detection

### Test Cases to Guide Decision

#### Case 1: Unclosed Dialog Spanning Paragraphs
```
"He said, 'This dialog never closes

And this should be narrative again.
```
**Desired**: Force transition to Narrative at paragraph break

#### Case 2: Dialog with Internal Paragraph Break
```
"This is a long speech.

Still the same speaker continuing."
```
**Desired**: Stay in Dialog, but recognize paragraph structure

#### Case 2b: Colon + Hard Separator + Dialog (Current Failing Test)
**Input**: `"He said:\n\n\"Hello.\"\n\n\"World.\""`
```
He said:

"Hello."

"World."
```
**Expected**: 2 sentences
1. `He said: "Hello."`  
2. `"World."`

**Current behavior**: 1 sentence (all combined)
**Issue**: The soft dialog pattern `'."` + newlines is consuming the hard separator `\n\n` between dialog elements, treating the entire sequence as one soft continuation instead of recognizing the hard separator as a sentence boundary between the two dialog pieces.

#### Case 3: Dialog End at Line Boundary
```
"Short speech." He turned away.
Next sentence starts here.
```
**Desired**: Transition at `." H` (hard end), continue at line boundary

#### Case 4: Abbreviation + Line Break
```
He lived in Washington D.C.
The next sentence starts here.
```
**Desired**: NO transition at `C.\n` (abbreviation), transition at `. T`

### Recommended Hybrid Approach
1. **Option C**: Prioritize hard separators in multi-pattern arrays
2. **Option D**: Add END_OF_LINE narrative transitions as safety net
3. **Option B**: Make soft_separator more restrictive for within-line only

### Current Failing Test Analysis
The `test_dialog_hard_separator_minimal` is testing the colon + hard separator + dialog pattern:
```
DEBUG: State=DialogDoubleQuote, PatternID=1, MatchedText='."

', MatchType=DialogSoftEnd, NextState=Narrative
```

**Issue**: The soft dialog pattern is consuming the hard separator and treating it as soft continuation, when it should recognize the hard separator as a sentence boundary between dialog elements.

### Questions for Implementation
1. Should END_OF_LINE transitions require both sentence punctuation AND uppercase following?
2. How do we distinguish "paragraph breaks that force state transition" vs "paragraph breaks within dialog"?
3. Should abbreviation checking happen before or after line break transitions?
4. **NEW**: How do we preserve colon + hard_sep + dialog continuation behavior while fixing over-aggressive soft pattern matching?

### Next Steps
1. ~~Investigate why `narrative_soft_boundary` pattern is matching without considering `sentence_start_chars`~~ ✅ FIXED
2. ~~Check actual compiled pattern values~~ ✅ DONE  
3. Implement paragraph break exclusion in soft_separator
4. Test remaining dialog hard separator minimal failure

### Pattern Definitions (Current)
```rust
let sentence_start_chars = r"[A-Z\x22\x27\u{201C}\u{2018}\(\[\{]";
let not_sentence_start_chars = Self::negate_char_class(sentence_start_chars);
let narrative_soft_boundary = format!("{sentence_end_punct}{soft_separator}{sentence_start_chars}");
```

**Expected `not_sentence_start_chars`**: `[^A-Z\x22\x27\u{201C}\u{2018}\(\[\{]`

### Debug Questions
1. What is the actual value of `sentence_start_chars` and `not_sentence_start_chars`?
2. Which exact pattern is matching `'. d'` in the abbreviation case?
3. Why is `PatternID=1` matching instead of no match for lowercase following character?