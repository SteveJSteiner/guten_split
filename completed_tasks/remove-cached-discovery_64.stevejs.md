# Remove Cached File Discovery Feature

* **Task ID:** remove-cached-discovery_64.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Cached file discovery feature has been removed from the implementation
  - CLI flag `--overwrite-use-cached-locations` is no longer functional
  - PRD references and documentation still mention this obsolete feature
  - Users may be confused by non-functional CLI options
  - Clean up reduces maintenance burden and API surface

* **Acceptance Criteria:**
  1. Remove `--overwrite-use-cached-locations` CLI flag completely
  2. Remove any code related to file discovery caching
  3. Update PRD.md to remove cached discovery references (F-11)
  4. Update README.md to remove cached discovery examples
  5. Update CLI help text and documentation
  6. All tests pass after removal
  7. No dead code or unused imports remain

* **Deliverables:**
  - Remove CLI flag from argument parser
  - Remove cached discovery code from file enumeration
  - Update PRD.md to remove F-11 requirement
  - Update README.md usage examples
  - Update any other documentation references
  - Remove related tests and test data

* **References:**
  - PRD.md F-11: "Cache discovered file locations to avoid slow directory traversal"
  - CLI help output showing --overwrite-use-cached-locations flag
  - Current README.md usage examples

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] CLI flag completely removed from argument parser
- [x] No cached discovery code remains in file enumeration
- [x] PRD.md updated to remove F-11 requirement
- [x] README.md updated to remove cached discovery examples (not needed - no references found)
- [x] Tests passing (`cargo test`)
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [x] No dead code warnings for removed functionality
- [x] CLI help output no longer shows removed flag

## Implementation Summary

**COMPLETED:**
1. **Removed CLI flag** `--overwrite-use-cached-locations` from Args struct in main.rs
2. **Updated function signatures** removed `overwrite_use_cached_locations` parameter from:
   - `should_process_file()` in restart_log.rs
   - `process_single_file_restart()` in main.rs  
3. **Updated all call sites** to use the new 3-parameter function signature
4. **Removed PRD F-11 requirement** about cached file discovery
5. **Updated all tests** to remove references to the cached discovery flag
6. **Verified CLI help output** no longer shows the removed flag

**RESULT:** The non-functional `--overwrite-use-cached-locations` CLI flag has been completely removed, eliminating user confusion about a broken feature. The restart log functionality remains intact, providing file tracking without the complexity of cached discovery.