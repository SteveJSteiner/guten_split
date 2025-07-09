# Remove Tabs During Sentence Normalization

* **Task ID:** remove-tabs-normalization_66.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current normalization preserves tabs within sentences, breaking TSV output format
  - TSV (Tab-Separated Values) format requires tab characters only as field separators
  - Tabs within sentence content corrupt the structured output format
  - Essential for downstream processing tools that parse TSV data
  - Enables reliable import into spreadsheets, databases, and analysis tools
  - Maintains consistency with other whitespace normalization (line breaks → spaces)

* **Acceptance Criteria:**
  1. Tabs within sentences are converted to spaces during normalization
  2. TSV output format remains valid with tabs only as field separators
  3. Existing normalization behavior preserved for other whitespace
  4. All tests pass with updated normalization logic
  5. No regression in sentence boundary detection accuracy
  6. Documentation updated to reflect tab handling in normalization

* **Deliverables:**
  - Update sentence normalization logic to convert tabs to spaces
  - Add tests for tab handling in normalization
  - Verify TSV output format integrity
  - Update documentation about normalization behavior
  - Ensure no impact on sentence boundary detection

* **Implementation Notes:**
  - Tabs should be converted to single spaces (consistent with line break handling)
  - Only affect tabs within sentence content, not TSV field separators
  - Consider edge cases: multiple consecutive tabs, tabs at sentence boundaries
  - Maintain existing normalization for \n, \r\n → single space
  - Test with various tab scenarios in Project Gutenberg texts

* **References:**
  - Current normalization logic in sentence processing
  - TSV output format specification
  - Existing whitespace normalization tests

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tabs converted to spaces during sentence normalization
- [ ] TSV output format remains valid and parseable
- [ ] Tests passing (`cargo test`)
- [ ] No regression in sentence boundary detection
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] Documentation updated for normalization behavior
- [ ] Edge cases handled (multiple tabs, boundary tabs)
- [ ] Verified with real Project Gutenberg texts containing tabs