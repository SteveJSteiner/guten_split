# Dialog Hard Separator and Apostrophe Context Fixes

* **Task ID:** dialog-hard-separator-and-apostrophe-fixes_82.stevejs
* **Reviewer:** stevejs  
* **Area:** code
* **Motivation (WHY):**
  - Dialog soft patterns were consuming hard separators (`\n\n`) inappropriately, treating paragraph breaks as sentence continuations
  - Apostrophes in contractions (like "hermit's") were incorrectly triggering dialog mode, causing entire narrative sections to be treated as dialog
  - These issues caused incorrect sentence splitting in both dialog and narrative contexts

* **Acceptance Criteria:**
  1. `test_dialog_hard_separator_minimal` passes - hard separators properly create sentence boundaries in dialog
  2. `test_user_hermit_scroll_text` passes - apostrophes in contractions don't trigger dialog mode
  3. All existing dialog detection tests continue to pass
  4. Zero warnings across all development scenarios

* **Deliverables:**
  - Fixed dialog pattern priority to prevent hard separator consumption
  - Implemented universal whitespace requirement for dialog opening characters
  - Enhanced test assertions with detailed error messages for better debugging
  - Documentation of dialog detection architecture and edge cases

* **References:**
  - DIAG.md - comprehensive debugging log of hard separator issue
  - DIALOG-OPEN-WHITESPACE-REQUIREMENT.md - analysis and solution for apostrophe context

## Root Cause Analysis

### Problem 1: Hard Separator Over-Consumption
**Issue**: Dialog soft patterns like `[.!?]"([ \t]|\r?\n)[^A-Z...]` were matching `."` + `\n\n` (hard separator), consuming paragraph breaks and treating separate dialog elements as one sentence.

**Test case**: `"He said:\n\n\"Hello.\"\n\n\"World.\""`
- **Expected**: 2 sentences (`He said: "Hello."` and `"World."`)  
- **Actual**: 1 sentence (all combined)

### Problem 2: Apostrophe Context Confusion  
**Issue**: Apostrophes in contractions were treated as dialog opening single quotes, causing narrative text to incorrectly enter dialog mode.

**Test case**: `"He had thus sat...the hermit's scroll..."`
- **Expected**: 3 sentences with proper narrative boundaries
- **Actual**: 1 sentence (entire text treated as unclosed dialog)

## Solutions Implemented

### Solution 1: Pattern Priority (Hard Before Soft)
Reordered all dialog state pattern arrays to prioritize hard separators:

```rust
// BEFORE: Soft patterns could consume hard separators
let dialog_patterns = vec![
    dialog_hard_end.as_str(),     // PatternID 0
    dialog_soft_end.as_str(),     // PatternID 1  
    pure_hard_sep.as_str(),       // PatternID 2
];

// AFTER: Hard separators have highest priority
let dialog_patterns = vec![
    pure_hard_sep.as_str(),       // PatternID 0 (HIGHEST PRIORITY)
    dialog_hard_end.as_str(),     // PatternID 1
    dialog_soft_end.as_str(),     // PatternID 2
];
```

**Key insight**: Even with pattern reordering, the regex engine chooses longest matches. However, when combined with restrictive soft_separator definitions, this prevents inappropriate consumption.

### Solution 2: Universal Whitespace Requirement for Dialog Opening
Implemented context-aware dialog detection requiring preceding whitespace:

```rust
let dialog_prefix_whitespace = r"[ \t\n]";  // Space, tab, or newline (\r omitted - Windows \r\n has \n)
let dialog_open_chars = r"[\x22\x27\u{201C}\u{2018}\(\[\{]";  // All dialog opening characters
let dialog_start = format!("{dialog_prefix_whitespace}{dialog_open_chars}");
```

**Results**:
- ✅ `hermit's` does NOT trigger dialog (no preceding whitespace)
- ✅ ` "Hello"` DOES trigger dialog (space before)
- ✅ `\n"Hello"` DOES trigger dialog (newline before)
- ✅ `\t'Quote'` DOES trigger dialog (tab before)

**Rationale**: Legitimate dialog opening characters are typically preceded by whitespace or appear at text boundaries, distinguishing them from inline punctuation like contractions, measurements (`6"tall`), or function calls (`file(name)`).

### Solution 3: Restrictive Soft Separator Definition
Used `soft_separator = r"[ \t]+"` (spaces/tabs only) in combination with explicit line boundary patterns to prevent cross-paragraph matching while preserving sentence detection.

## Implementation Details

### Pattern Architecture Changes
1. **Multi-pattern arrays**: All dialog states now use `pure_hard_sep` as PatternID 0 for consistent priority
2. **Context-aware dialog detection**: `dialog_start` pattern replaces context-free `dialog_open_chars`  
3. **Explicit line boundaries**: Added `narrative_line_boundary` pattern for cross-line sentence detection
4. **Character class negation**: Helper function `negate_char_class()` for generating complementary patterns

### Enhanced Test Coverage
- Added `test_user_hermit_scroll_text` with detailed assertion messages
- Improved existing test assertions to show actual vs expected sentence splits
- Removed debug print statements in favor of descriptive test failures

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] `test_dialog_hard_separator_minimal` passes (hard separator fix verified)
- [x] `test_user_hermit_scroll_text` passes (apostrophe fix verified)  
- [x] All existing dialog detection tests pass (no regressions)
- [x] Enhanced test assertions provide clear debugging information
- [x] **ZERO WARNINGS**: All development scenarios are warning-free
- [x] Documentation updated (DIAG.md and DIALOG-OPEN-WHITESPACE-REQUIREMENT.md)

## Final Status
✅ **COMPLETE**: Both hard separator over-consumption and apostrophe context confusion issues resolved. Dialog detection now correctly distinguishes between legitimate dialog boundaries and inline punctuation while maintaining proper paragraph-level sentence separation.