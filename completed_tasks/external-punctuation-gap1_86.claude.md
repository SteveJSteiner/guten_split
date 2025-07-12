# External Punctuation Pattern Support (GAP #1 Fix)

* **Task ID:** external-punctuation-gap1_86.claude
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - SEAMS-Design.md identified GAP #1: Missing external definitive punctuation patterns for all quote types
  - Only double quotes had external punctuation patterns (`"word"! More` scenarios)
  - All other quote types (single, smart quotes, parentheses, brackets, braces) lacked these patterns
  - This caused incorrect sentence splitting for cases like `'word'! More text. New sentence.`

* **Acceptance Criteria:**
  1. All dialog quote types support external punctuation patterns (punctuation AFTER dialog close)
  2. External separation + sentence start patterns → D→N + Split
  3. External separation + lowercase patterns → D→N + Continue (lowercase overrides)
  4. External separation + dialog open patterns → D→D + Split
  5. External continuation patterns → D→N + Continue
  6. Pattern mappings maintain consistent priority ordering across all quote types

* **Deliverables:**
  - Updated `src/sentence_detector/dialog_detector.rs` with external punctuation patterns for:
    - Single quotes (`'`)
    - Smart double quotes (`"` `"`)
    - Smart single quotes (`'` `'`)  
    - Round parentheses (`()`)
    - Square brackets (`[]`)
    - Curly braces (`{}`)
  - Fixed SEAMS-Design.md example: replaced non-SEAM `word <space> more` with real SEAM `Dr. <space> Smith`

* **References:**
  - SEAMS-Design.md GAP #1 analysis (lines 389-391)
  - External punctuation pattern architecture (lines 462-470 double quote implementation)

## Implementation Details

**Pattern Structure Applied to All Quote Types:**
```rust
// External punctuation patterns (punctuation AFTER dialog close) - GAP 1 fix
// Pattern 1: External separation + sentence start → D→N + Split
let dialog_external_separation_split = format!("{close}{sentence_end_punct}({soft_separator})[{sentence_starts}]");
// Pattern 2: External separation + lowercase → D→N + Continue (lowercase overrides)
let dialog_external_separation_continue = format!("{close}{sentence_end_punct}({soft_separator}){not_sentence_starts}");
// Pattern 3: External separation + dialog open → D→D + Split
let dialog_external_separation_to_dialog = format!("{close}{sentence_end_punct}({soft_separator}){dialog_open_chars}");
// Pattern 4: External continuation → D→N + Continue
let dialog_external_continuation = format!("{close}{non_sentence_ending_punct}({soft_separator}).");
```

**Target Cases Now Supported:**
- `'word'! More text.` → 3 sentences (external exclamation + capital)
- `'word'! next text.` → 2 sentences (external exclamation + lowercase override)
- `(word)? More text.` → 3 sentences (external question + capital)
- `[word], more text.` → 1 sentence (external continuation)

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (external punctuation patterns replicated across all quote types)
- [x] Documentation updated (SEAMS-Design.md example corrected)
- [x] **ZERO WARNINGS**: All quote types follow same pattern structure

**Completion Status:** ✅ COMPLETED - All quote types now support complete external punctuation pattern coverage, resolving GAP #1 identified in SEAMS-Design.md.