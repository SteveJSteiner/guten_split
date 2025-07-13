# Interactive Debug Environment for SEAM Detection Regression

* **Task ID:** interactive-debug-seam_88.steve
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Dialog quote transition regression: `sword" (A.D. 1656)` produces 1 sentence instead of expected 2
  - Need interactive debugging leveraging 7GB corpus processing speed (~10 seconds)
  - Missing external definitive punctuation patterns `[.!?]` for dialog exit transitions
  - Current _seams.txt vs _seams2.txt comparison shows this specific pattern failure
* **Acceptance Criteria:**
  1. Enhanced _seams-debug.txt TSV output with state transition and pattern columns
  2. Debug mode that processes 7GB corpus with detailed SEAM analysis in ~10 seconds
  3. State transition logging for the failing `sword" (A.D. 1656)` pattern
  4. TSV columns: existing seam data + state_transition + matched_pattern + regex_name
* **Deliverables:**
  - `--debug-seams` CLI flag that outputs _seams-debug.txt files
  - Enhanced TSV format with state transition and pattern match columns
  - Debug output showing which regex patterns match (or fail to match) specific SEAMs
  - Documentation for analyzing debug TSV output to identify pattern gaps
* **References:**
  - NOTE-Debug.md line 5: failing test case
  - SEAMS-Design.md line 389-392: missing external definitive punctuation patterns
  - Current _seams.txt TSV format for extending with debug columns

## Pre-commit checklist:
- [x] All deliverables implemented  
- [x] Tests passing (`cargo test`)
- [x] Claims validated (debug TSV shows state transitions for failing pattern)
- [ ] Documentation updated if needed
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely

## Debug TSV Format:
Extend existing _seams.txt columns with:
- `state_before`: Dialog state before SEAM
- `state_after`: Dialog state after SEAM  
- `transition_type`: Continue/Split decision
- `matched_pattern`: Which regex pattern matched
- `pattern_name`: Human-readable pattern name