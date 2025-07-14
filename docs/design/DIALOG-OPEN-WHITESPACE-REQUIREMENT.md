# Dialog Opening Character Whitespace Requirement Analysis

## Problem Statement

The apostrophe in "hermit's" is being incorrectly recognized as an open single quote, causing the algorithm to enter dialog mode inappropriately. This raises a broader question about when punctuation should trigger dialog mode.

## Root Cause

Current `dialog_open_chars = r"[\x22\x27\u{201C}\u{2018}\(\[\{]"` includes all dialog characters without context, making it impossible to distinguish between:

### Problematic Cases
- **Apostrophes in contractions**: `hermit's`, `don't`, `it's` vs **Dialog single quotes**: `'Hello there,' he said.`
- **Measurements/dimensions**: `6"tall`, `8'wide` vs **Dialog double quotes**: `"Hello world"`
- **Function calls**: `file(name)`, `array[index]` vs **Parentheticals**: `(aside)`, `[note]`

## Design Question: Universal Whitespace Requirement

Should ALL dialog opening characters require preceding whitespace/newline to distinguish them from inline punctuation?

### Current State
- All quote/parenthetical characters trigger dialog mode immediately when encountered
- No context checking for whether they're truly dialog vs inline punctuation

### Proposed: Context-Aware Dialog Detection

**Hypothesis**: Legitimate dialog opening characters are typically preceded by whitespace or appear at text boundaries.

## Implementation Decisions Needed

### 1. What constitutes "preceding whitespace"?
- **Option A**: `[ \t]` (space and tab only)
- **Option B**: `[ \t\r\n]` (space, tab, and newlines)  
- **Option C**: `\s` (all whitespace including vertical)
- **Option D**: Text boundary OR whitespace (handles start-of-text cases)

### 2. Which characters should require this?
- **All quotes**: `"` (double), `'` (single), `"` (smart double), `'` (smart single)
- **All parentheticals**: `(` (round), `[` (square), `{` (curly)
- **Selective approach**: Only ambiguous ones like `'` and `"`

### 3. Edge cases to consider
- **Start of text**: `"Hello world"` (no preceding whitespace possible)
- **After punctuation**: `sentence. "Next quote"` vs `sentence."embedded`  
- **Line boundaries**: `word\n"dialog"` should work
- **Multiple whitespace**: `word   "dialog"` should work

## Test Cases to Validate

### Should NOT trigger dialog mode:
```
hermit's scroll
don't go
it's working
6"tall
8'wide  
file(name)
array[index]
word"embedded
```

### SHOULD trigger dialog mode:
```
"Hello world"           // start of text
He said "hello"         // space before
word
"Next line"             // newline before
sentence. "Quote"       // space after punctuation
He said: 'Single'       // space before single
Note (parenthetical)    // space before paren
See [reference]         // space before bracket
```

## Current Status

- ✅ Hard separator priority fix is working
- ❌ Apostrophe in "hermit's" incorrectly triggers dialog mode
- ❓ Need decision on universal whitespace requirement scope

## Recommended Next Steps

1. **Decide scope**: Should all dialog chars require preceding whitespace, or just ambiguous ones?
2. **Define whitespace**: What pattern counts as "preceding whitespace"?
3. **Handle text boundaries**: How to handle dialog at start of text?
4. **Implement incrementally**: Start with single quotes, validate, then expand
5. **Test thoroughly**: Ensure no regressions in existing dialog detection

## Impact Assessment

**Low Risk**: Only affects problematic cases like contractions
**Medium Risk**: May break legitimate dialog at text boundaries  
**High Risk**: Could break all dialog detection if implemented incorrectly

## Chosen Solution

**Decision**: Require preceding whitespace for ALL dialog opening characters.

**Whitespace Definition**: `[ \t\n]` (space, tab, or newline)
- **Note**: `\r` is omitted because Windows line endings are `\r\n`, so the `\n` byte is always present and sufficient for matching. Unix systems use `\n` only.

**Pattern Structure**: 
```rust
let dialog_prefix_whitespace = r"[ \t\n]";  // Space, tab, or newline (no \r needed)
let dialog_open_chars = r"[\x22\x27\u{201C}\u{2018}\(\[\{]";
let dialog_start = format!("{dialog_prefix_whitespace}{dialog_open_chars}");
```

**Implementation**: Make this explicit as `dialog_start = {dialog_prefix_whitespace}{dialog_open_chars}`

This ensures:
- ✅ `hermit's` does NOT trigger dialog (no preceding whitespace)
- ✅ ` "Hello"` DOES trigger dialog (space before)
- ✅ `\n"Hello"` DOES trigger dialog (newline before)
- ✅ `\t'Quote'` DOES trigger dialog (tab before)

**Edge Case**: Start-of-text dialog will need special handling OR we accept that `"Hello` at the very beginning won't trigger dialog mode (which may be acceptable for most use cases).

**Next Step**: Implement and test this universal whitespace requirement.