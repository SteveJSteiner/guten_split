# Fix Dialog Comma + Capital Letter Continuation Issue

* **Task ID:** dialog-comma-capital-continuation_92.steve
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current implementation incorrectly splits sentences at capital letters even when continuation punctuation (comma) signals the sentence should continue
  - Example: `"Right," I told him` incorrectly splits into `"Right,"` + `I told him` instead of keeping as single sentence
  - According to SEAMS-Design.md line 176: `{close}{cont_punct}` + ` ` + Any → D→N Continue
  - Comma is a continuation signifier that should override capital letter sentence-start signal

* **Acceptance Criteria:**
  1. Pattern `"word," <space> Capital` correctly continues sentence (no split)
  2. All existing dialog tests still pass
  3. Test case covers the specific failing example: `"Right," I told him, condescendingly.`

* **Deliverables:**
  - Fix dialog detector pattern matching for continuation punctuation + capital letters
  - Add test case for the specific failing pattern
  - Verify all dialog continuation punctuation works correctly (comma, semicolon, colon)

* **References:**
  - SEAMS-Design.md lines 176, 198: continuation punctuation always maintains sentence flow
  - Example from seams disagreement analysis showing incorrect split

## Implementation Summary:
**Root Cause:** The unpunctuated dialog end patterns didn't distinguish between continuation punctuation (`,;:`) and actual sentence boundaries.

**Solution:** Added continuation punctuation patterns for all dialog closing characters:
- **Before close**: `{non_sentence_ending_punct}{close}({soft_separator}){unified_sentence_start_chars}` → Continue
- **After close**: `{close}{non_sentence_ending_punct}({soft_separator}){unified_sentence_start_chars}` → Continue  
- **Unpunctuated**: `{close}({soft_separator}){non_dialog_sentence_start_chars}` → Split

**Applied to all dialog states:** DoubleQuote, SingleQuote, SmartDoubleQuote, SmartSingleQuote, ParenthheticalRound, ParenthheticalSquare, ParenthheticalCurly

**Validation:** CLI debug shows pattern `," I` now correctly matches as Continue instead of Split.

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [x] Documentation updated if needed
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely