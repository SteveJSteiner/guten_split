# Dialog State Machine Regex Design

## Overview

This document specifies the regex patterns and state machine design for dialog-aware sentence boundary detection. The approach uses a single-pass state machine where each state has a specific regex pattern optimized for that context.

## Design Principles

### 1. Context-Specific Boundary Detection

Different contexts require different sentence boundary patterns:
- **Narrative**: Standard punctuation + whitespace + capital letter
- **Dialog**: Punctuation + closing quote + whitespace + any sentence start
- **Parentheticals**: Punctuation + closing bracket + whitespace + any sentence start

### 2. State-Specific Regex Patterns

Each state has a compiled regex pattern that matches valid sentence boundaries for that context. The transducer determines exit reason and next state based on which part of the pattern matched.

### 3. Unicode Escape Codes

All Unicode characters use explicit escape codes (e.g., `\u{201C}`) to ensure proper tool handling and compilation.

## State Machine States

### Core States
- `Narrative` - normal text processing
- `DialogDoubleQuote` - inside ASCII double quotes `"`
- `DialogSingleQuote` - inside ASCII single quotes `'`
- `DialogSmartDoubleOpen` - inside Unicode smart double quotes `\u{201C}` 
- `DialogSmartSingleOpen` - inside Unicode smart single quotes `\u{2018}`
- `ParenthheticalRound` - inside round parentheses `(`
- `ParenthheticalSquare` - inside square brackets `[`
- `ParenthheticalCurly` - inside curly braces `{`
- `Unknown` - after HARD_SENT_SEP, next boundary determines state

## Pattern Components

### Sentence Separators
```rust
BASIC_SENT_SEP = "\\s+"
HARD_SENT_SEP = "\\n\\n"
```

### Sentence Endings by Context
```rust
// Basic punctuation
NARRATIVE_SENT_END = "[.!?]"

// Quote endings (punctuation + specific closing quote)
DOUBLE_QUOTE_SENT_END = "[.!?]\""
SINGLE_QUOTE_SENT_END = "[.!?]'"
SMART_DOUBLE_SENT_END = "[.!?]\\u{201D}"
SMART_SINGLE_SENT_END = "[.!?]\\u{2019}"

// Parenthetical endings (punctuation + specific closing bracket)
PAREN_ROUND_SENT_END = "[.!?]\\)"
PAREN_SQUARE_SENT_END = "[.!?]\\]"
PAREN_CURLY_SENT_END = "[.!?]\\}"
```

### Sentence Starts
```rust
CAPITAL_START = "[A-Z]"
QUOTE_START = "[\"'\\u{201C}\\u{2018}]"
PAREN_START = "[\\(\\[\\{]"
BASIC_START = "(?:[A-Z]|[\"'\\u{201C}\\u{2018}]|[\\(\\[\\{])"
```

## State-Specific Patterns

### Narrative State
```rust
NARRATIVE_PATTERN = "(?:" + NARRATIVE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
```

### Dialog States
```rust
// ASCII quotes
DIALOG_DOUBLE_PATTERN = "(?:" + DOUBLE_QUOTE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
DIALOG_SINGLE_PATTERN = "(?:" + SINGLE_QUOTE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"

// Unicode smart quotes
DIALOG_SMART_DOUBLE_PATTERN = "(?:" + SMART_DOUBLE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
DIALOG_SMART_SINGLE_PATTERN = "(?:" + SMART_SINGLE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
```

### Parenthetical States
```rust
PAREN_ROUND_PATTERN = "(?:" + PAREN_ROUND_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
PAREN_SQUARE_PATTERN = "(?:" + PAREN_SQUARE_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
PAREN_CURLY_PATTERN = "(?:" + PAREN_CURLY_SENT_END + BASIC_SENT_SEP + BASIC_START + ")|(?:" + HARD_SENT_SEP + ")"
```

## Transducer Logic

### Exit Reason Detection
The transducer examines the matched pattern to determine why the boundary was detected:

1. **HARD_SENT_SEP Match**: `\n\n` pattern matched
   - Exit reason: Hard separator
   - Next state: `Unknown`

2. **Quote-Ending Match**: Quote-specific ending pattern matched
   - Exit reason: Dialog end
   - Next state: `Narrative`

3. **Parenthetical-Ending Match**: Bracket-specific ending pattern matched
   - Exit reason: Parenthetical end
   - Next state: `Narrative`

4. **Narrative-Ending Match**: Basic punctuation pattern matched
   - Exit reason: Potential state transition
   - Next state: Determined by examining `SENT_START` for dialog/parenthetical opening

### State Transition Logic
```rust
match current_state {
    Narrative => {
        if hard_sep_matched => Unknown
        else if quote_start_detected => appropriate_dialog_state
        else if paren_start_detected => appropriate_paren_state
        else => Narrative
    }
    DialogDoubleQuote => {
        if hard_sep_matched => Unknown
        else if quote_end_matched => Narrative
        else => DialogDoubleQuote  // Continue in dialog
    }
    ParenthheticalRound => {
        if hard_sep_matched => Unknown
        else if paren_end_matched => Narrative
        else => ParenthheticalRound  // Continue in parenthetical
    }
    Unknown => {
        // Next boundary determines state based on context
        determine_state_from_context()
    }
}
```

## Processing Algorithm

### Main Loop
1. **Initialize**: Start in `Narrative` state
2. **Find Boundary**: Use current state's regex pattern to find next sentence boundary
3. **Determine Exit**: Use transducer to determine why boundary was detected
4. **Record Sentence**: Create DetectedSentence with correct span information
5. **Transition State**: Move to next state based on exit reason and context
6. **Continue**: Repeat from boundary position until end of text

### Span Information
- **Sentence Start**: Position after previous boundary separator
- **Sentence End**: Position before current boundary separator
- **Boundary Point**: The whitespace/hard separator (not included in sentence content)

## Implementation Notes

### Regex Compilation
- Each state maintains a pre-compiled regex pattern
- Patterns are built compositionally from named components
- Unicode escapes ensure proper cross-platform behavior

### Performance Considerations
- Single-pass processing with state-specific patterns
- Minimal backtracking through careful pattern design
- State machine overhead vs. pattern complexity trade-off

### Error Handling
- Unknown state provides recovery mechanism after hard separators
- Malformed quote/parenthetical matching falls back to narrative rules
- Invalid Unicode sequences handled by regex engine

## Test Scenarios

### Dialog Boundaries
```
Input: "He said, 'Hello.' 'Goodbye,' she replied."
Expected: Two sentences, dialog boundaries properly detected

Input: "Stop her, sir! Ting-a-ling-ling!" The headway ran almost out.
Expected: Dialog coalesced into single sentence
```

### Parenthetical Boundaries
```
Input: "He left (quietly.) She followed."
Expected: Two sentences, parenthetical boundary detected

Input: "The measurement (5 ft. by 3 ft.) was accurate."
Expected: Single sentence, abbreviation not falsely detected
```

### Hard Separator Handling
```
Input: "He spoke.\n\nShe listened."
Expected: Two sentences, hard separator forces boundary
```

## Future Extensions

### Additional Quote Types
- Guillemets: `«»` `‹›`
- CJK quotes: `「」` `『』`
- Custom quote pairs per language

### Complex Parentheticals
- Nested parentheticals: `(outer (inner) text)`
- Mixed bracket types: `[outer (inner) text]`
- Math expressions: `f(x) = y. Next equation...`

### State Machine Enhancements
- Recursive dialog tracking for nested quotes
- Context-aware abbreviation handling per state
- Performance optimization through pattern consolidation