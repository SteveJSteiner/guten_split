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
- [ ] All deliverables implemented
- [ ] CLI flag completely removed from argument parser
- [ ] No cached discovery code remains in file enumeration
- [ ] PRD.md updated to remove F-11 requirement
- [ ] README.md updated to remove cached discovery examples
- [ ] Tests passing (`cargo test`)
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] No dead code warnings for removed functionality
- [ ] CLI help output no longer shows removed flag