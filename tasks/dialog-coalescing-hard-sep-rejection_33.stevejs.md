# Dialog Coalescing: Internal Punctuation Hard Separator Rejection

* **Task ID:** dialog-coalescing-hard-sep-rejection_33.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current dialog detector splits dialog sections at hard separators (`\n\n`) even when they should be coalesced
  - Internal (continuative) punctuation before hard separators indicates sentence continuation, not termination
  - Hard separators after internal punctuation should be rejected as false positive sentence boundaries
  - Example: `he said:\n\n"Hello."` should be one sentence, not two
  - This implements proper dialog coalescing behavior based on standard punctuation semantics

* **Acceptance Criteria:**
  1. Hard separators (`\n\n`) are rejected as sentence boundaries when preceded by internal punctuation
  2. Internal punctuation set includes: comma (,), semicolon (;), colon (:), em dash (—), en dash (–), hyphen (-), slash (/), opening brackets/parentheses, opening quotes
  3. Only terminal punctuation (. ? !) before hard separators allows sentence boundaries
  4. Dialog following internal punctuation is properly coalesced across hard separators
  5. Regular hard separators (preceded by terminal punctuation or no punctuation) still work correctly
  6. All existing tests pass (`cargo test`)
  7. The specific case from BAD_SEP_HANDLING.md now produces the expected 2 sentences instead of 4

* **Deliverables:**
  - Implement internal punctuation detection logic in dialog state machine
  - Update hard separator detection to check for preceding internal punctuation
  - Add comprehensive test cases for all internal punctuation types before hard separators
  - Verify fix against original bug report cases
  - Document internal punctuation rules and examples

* **References:**
  - exploration/BAD_SEP_HANDLING.md - original bug report expecting 2 sentences
  - exploration/dialog-state-machine-regex-design.md - dialog coalescing design intent
  - src/sentence_detector/dialog_detector.rs - main implementation location

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (dialog coalescing produces expected sentence count)
- [ ] Documentation updated if needed
- [ ] Clippy warnings addressed

## Internal Punctuation Reference:

### Guaranteed Sentence Continuation (Never Terminal)
- **Comma (,)** U+002C - separates items, sets off modifiers
- **Semicolon (;)** U+003B - joins related clauses or separates complex list items  
- **Colon (:)** U+003A - introduces explanation, list, quotation, or summary
- **Em dash (—)** U+2014 - sets off parentheticals or replaces colon/semicolon
- **En dash (–)** U+2013 - British alternative to em dash (spaced usage)
- **Hyphen (-)** U+002D - binds compounds (word-internal)
- **Slash (/)** U+002F - couples alternatives (and/or)
- **Opening brackets/parentheses** - insert side material
- **Opening quotes** - never ends a sentence

### Terminal Punctuation (Definitively Ends Sentences)
- **Period (.)** U+002E
- **Question mark (?)** U+003F  
- **Exclamation point (!)** U+0021

### Ambiguous Cases (Handle with Caution)
- **Ellipsis (…)** - can appear mid-sentence or replace period
- **Short dash interruptions** - in fiction can replace closing quote + period

## Implementation Strategy:
1. **Internal Punctuation Detection**: Create function to identify internal vs terminal punctuation
2. **Context-Aware Hard Separator Logic**: Before accepting `\n\n` as boundary, check if preceded by internal punctuation
3. **State Machine Enhancement**: Update hard separator classification to reject false positives
4. **Comprehensive Testing**: Test all internal punctuation types with hard separators