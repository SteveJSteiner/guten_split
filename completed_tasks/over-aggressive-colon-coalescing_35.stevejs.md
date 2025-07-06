# Over-Aggressive Colon Coalescing Across Paragraph Breaks

* **Task ID:** over-aggressive-colon-coalescing_35.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Dialog coalescing logic is too aggressive when rejecting hard separators after colons
  - Text with colon followed by paragraph break (blank line) and dialog should create sentence boundary
  - Currently `text:\n\n"dialog"` is being coalesced into one sentence when it should be two
  - This breaks proper sentence detection for dialog that follows narrative description with paragraph separation
  - Real-world case: Gutenberg text 74-0.txt sentence 51 incorrectly spans multiple logical sentences

* **Acceptance Criteria:**
  1. Create unit test reproducing the bug with colon + paragraph break + dialog pattern
  2. Diagnose root cause in `should_reject_hard_separator()` logic  
  3. Fix over-aggressive coalescing while preserving proper coalescing for direct continuation
  4. Distinguish between direct continuation (`text:\n\n"dialog"`) and paragraph breaks (`text:\n\n\n"dialog"`)
  5. All existing coalescing tests continue to pass
  6. New test case passes with correct sentence boundaries
  7. Gutenberg benchmark maintains performance (no significant regression)

* **Deliverables:**
  - Add unit test `test_colon_paragraph_break_dialog_separation()` reproducing the bug
  - Diagnose and document root cause in hard separator rejection logic
  - Implement refined coalescing logic that respects paragraph breaks
  - Update or add documentation explaining coalescing rules
  - Verify fix with real Gutenberg text case

* **References:**
  - /Users/stevejs/gutenberg_texts/7/74/74-0.txt sentence 51 (real-world bug case)
  - completed_tasks/dialog-coalescing-hard-sep-rejection_33.stevejs.md (original implementation)
  - src/sentence_detector/dialog_detector.rs (main implementation location)

## Test Case:

**Input text:**
```
She looked perplexed for a moment, and then said, not fiercely, but still loud enough for the furniture to hear:

"Well, I lay if I get hold of you I'll—"

She did not finish, for by this time she was bending down and punching
under the bed with the broom, and so she needed breath to punctuate the
punches with. She resurrected nothing but the cat.
```

**Expected behavior:** 2 sentences
1. `"She looked perplexed... furniture to hear: "Well, I lay if I get hold of you I'll—"`
2. `She did not finish, for by this time she was bending down and punching under the bed with the broom, and so she needed breath to punctuate the punches with.`

**Current (buggy) behavior:** 1 sentence spanning both parts

## Pre-commit checklist:
- [ ] Unit test created reproducing the bug
- [ ] Root cause diagnosed and documented  
- [ ] Fix implemented with refined coalescing logic
- [ ] All existing tests passing (`cargo test`)
- [ ] Gutenberg benchmark shows no significant performance regression
- [ ] Claims validated (paragraph breaks properly respected)
- [ ] Documentation updated explaining refined coalescing rules

NOTE from an analysis:
🎯 Root Cause Identified

  The issue is in our should_reject_hard_separator() function. When we encounter:

  "Well, I lay if I get hold of you I'll—"

  She did not finish...

  What happens:
  1. We find the hard separator \n\n after the closing quote "
  2. Our algorithm probes backward past the closing quote "
  3. It finds the em dash — (which is internal punctuation)
  4. It incorrectly rejects the hard separator, causing over-coalescing

  The problem:
  - The closing quote " is unambiguous terminal punctuation by itself
  - We should stop at the closing quote and accept the hard separator
  - We should NOT probe past closing quotes to find earlier internal punctuation

  The fix:
  In our should_reject_hard_separator() function, when we encounter closing delimiters like "')}], we should:
  - Accept the hard separator immediately (return false)
  - NOT continue scanning past the closing delimiter

  Currently we have:
  // Closing delimiters - skip and continue looking for meaningful punctuation
  b'"' | b'\'' | b')' | b']' | b'}' => continue,

  But it should be:
  // Closing delimiters - accept hard separator (they are terminal)
  b'"' | b'\'' | b')' | b']' | b'}' => return false,

  This perfectly explains why our coalescing logic is over-aggressive! The closing quote should be treated as
  definitive terminal punctuation, not something to skip over. 🎯