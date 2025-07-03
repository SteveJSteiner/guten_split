# Implement Incremental Processing for Safe Re-runs (F-9)

* **Task ID:** incremental-processing-implementation_11
* **Reviewer:** stevejs  
* **Area:** code
* **Motivation (WHY):**
  - Complete PRD requirement F-9: skip processing when aux file exists and is complete
  - Enable safe re-runs on large Project Gutenberg datasets without reprocessing everything
  - Detect and overwrite partial/incomplete auxiliary files automatically
  - Critical for production use - without this, tool always reprocesses all files

* **Acceptance Criteria:**
  1. Skip processing when complete aux file exists (unless --overwrite_all flag)
  2. Detect partial files by checking trailing newline + EOF and overwrite them
  3. --overwrite_all flag forces reprocessing of all files regardless of aux file status
  4. Integration test validates incremental behavior on multiple runs
  5. Performance test shows dramatic speedup on second run (near-zero processing time)

* **Deliverables:**
  - Add aux file existence and completeness checking logic
  - Implement partial file detection (missing trailing newline + EOF)
  - Update main processing loop to respect incremental processing rules
  - Add --overwrite_all flag handling (already parsed, needs implementation)
  - Integration test demonstrating incremental processing behavior

* **Technical Approach:**
  - Check for existing aux file before processing each source file
  - Use async file I/O to check aux file completeness (ends with newline + EOF)
  - Skip processing if aux file exists and is complete (unless --overwrite_all)
  - Log skipped files for observability
  - Maintain same error handling and --fail_fast behavior

* **References:**
  - PRD F-9: Skip processing when aux file exists and completes without truncation
  - PRD 3.1: Skip if complete aux file exists, unless --overwrite_all
  - PRD 3.1: Overwrite partial aux file
  - PRD 8.5: Re-run without --overwrite_all touches zero unchanged aux files

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (incremental processing skips complete files, overwrites partial files)
- [ ] Documentation updated if needed
- [ ] Clippy warnings addressed