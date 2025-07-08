# Sentence Splitting Fix for Project Gutenberg Text 4300

* **Task ID:** sentence-splitting-pg4300-fix_45.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - PG text 4300 (Ulysses by James Joyce) at /Users/stevejs/gutenberg_texts/4/3/0/4300/4300-0.txt splits incorrectly
  - Missing significant content at the end of extraction
  - NOTE: The final sentences of Ulysses are legitimately massive, spanning hundreds of lines (Joyce's stream-of-consciousness style with minimal punctuation)
  - Specific challenging section breaks incorrectly into 3 sentences instead of 1:
    ```
    Listener, S. E. by E.: Narrator, N. W. by W.: on the 53rd parallel of
    latitude, N., and 6th meridian of longitude, W.: at an angle of 45° to
    the terrestrial equator.
    ```
  - Root cause: Single capital abbreviations like 'S.' not handled properly
  - This is a concrete test case for improving abbreviation detection while respecting Joyce's intentional massive sentences

* **Acceptance Criteria:**
  1. Text 4300 splits cleanly without missing content at the end
  2. The challenging section remains as one sentence (not split into 3)
  3. Single capital abbreviations (S., E., N., W.) are handled correctly
  4. No regression on other Project Gutenberg texts
  5. All existing tests continue to pass

* **Deliverables:**
  - Enhanced abbreviation detection for single capital letter patterns
  - Updated AbbreviationChecker to handle compass directions and similar patterns
  - Test case covering the specific PG 4300 challenging section
  - Verification that 4300-0.txt processes completely without truncation
  - Preservation of Joyce's intentionally massive stream-of-consciousness sentences

* **References:**
  - Current abbreviation handling in src/sentence_detector/abbreviations.rs
  - Existing TITLE_ABBREVIATIONS pattern
  - Dialog detector coalescing logic that may interact with abbreviations

## Analysis:

### Current Issue
The problematic text contains navigational/geographic abbreviations:
- `S. E. by E.` (Southeast by East)
- `N. W. by W.` (Northwest by West) 
- `N.` (North)
- `W.` (West)

These single-letter abbreviations followed by periods are incorrectly treated as sentence boundaries.

### Implementation Strategy

1. **Expand abbreviation patterns** to include:
   - Compass directions: N., S., E., W., NE., NW., SE., SW., etc.
   - Single capital letters that are commonly abbreviations
   - Geographic/scientific notation patterns

2. **Test against PG 4300** specifically:
   - Create test case with the exact problematic text
   - Verify it remains as one sentence after fix
   - Ensure no content truncation

3. **Validate broadly**:
   - Run against sample of other PG texts to ensure no regressions
   - Check that legitimate sentence boundaries are still detected
   - Ensure Joyce's massive sentences (legitimate literary style) are preserved as single units

## Pre-commit checklist:
- [ ] Single capital abbreviations (S., E., N., W.) handled correctly
- [ ] PG 4300 challenging section stays as one sentence
- [ ] No content truncation in 4300-0.txt processing
- [ ] All existing tests pass (`cargo test`)
- [ ] New test case covers the specific problematic pattern
- [ ] No regressions on other Project Gutenberg sample texts
- [ ] Abbreviation detection maintains precision (doesn't over-coalesce)