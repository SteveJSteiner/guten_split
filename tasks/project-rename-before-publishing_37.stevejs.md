# Project Rename Before Publishing

* **Task ID:** project-rename-before-publishing_37.stevejs.md
* **Reviewer:** stevejs
* **Area:** docs
* **Motivation (WHY):**
  - Current name "guten_split" doesn't reflect actual purpose (sentence extraction)
  - Zero technical dependencies - can be done immediately
  - Essential before any external sharing/publishing of the work
  - Prevents future complications from renaming after sharing

* **Acceptance Criteria:**
  1. Choose appropriate name reflecting sentence extraction purpose
  2. Update Cargo.toml package name and metadata
  3. Update all README/documentation references
  4. Update repository name if applicable
  5. Verify project builds and tests pass with new name

* **Deliverables:**
  - Updated Cargo.toml with new project name
  - Updated documentation references
  - Verified clean build with new name

* **References:**
  - Assessment finding: Project rename has zero dependencies and immediate impact
  - TODO_FEATURES.md process improvement (still valid)

## Pre-commit checklist:
- [ ] New project name chosen and validated
- [ ] Cargo.toml updated with new name
- [ ] Documentation updated with new references
- [ ] Clean build verified (`cargo test`)
- [ ] Repository name updated if applicable