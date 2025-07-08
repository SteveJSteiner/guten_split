# CLI Incremental API Adoption

* **Task ID:** cli-incremental-api-adoption_46.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Public incremental processing API created but shows dead code warnings
  - CLI contains custom logic that duplicates public API functionality
  - Inconsistent implementation between CLI and public API reduces maintainability
  - External users should be able to rely on same utilities that CLI uses internally
  - Zero-warning builds require eliminating unused public API warnings

* **Acceptance Criteria:**
  1. Zero warnings in `cargo build` output
  2. Zero warnings in `cargo test` output  
  3. CLI uses public incremental API consistently throughout
  4. No functionality regression (all tests pass)
  5. Public API functions are utilized by CLI where appropriate
  6. Eliminate custom CLI implementations that duplicate public API

* **Deliverables:**
  - Updated CLI to use `aux_file_exists()` instead of custom logic
  - Updated CLI to use `read_aux_file()` where applicable
  - Updated CLI to use `cache_exists()` and `read_cache()` where applicable
  - Clean zero-warning builds across all configurations
  - Consistent incremental processing implementation

* **References:**
  - `src/incremental.rs` - Public incremental processing API
  - `src/main.rs` - CLI implementation with custom logic
  - Previous warning cleanup comprehensive task

## Current State Analysis:

### Public API Functions with Warnings:
```rust
// These show as unused but are legitimate public API
pub fn aux_file_exists<P: AsRef<Path>>(source_path: P) -> bool
pub fn read_aux_file<P: AsRef<Path>>(source_path: P) -> Result<String, io::Error>
pub fn create_complete_aux_file<P: AsRef<Path>>(source_path: P, content: &str) -> Result<PathBuf, io::Error>
pub fn cache_exists<P: AsRef<Path>>(root_dir: P) -> bool
pub fn read_cache<P: AsRef<Path>>(root_dir: P) -> Result<String, io::Error>
```

### CLI Custom Logic to Replace:
- Manual cache file existence checking
- Custom aux file existence logic
- Inconsistent error handling patterns
- Duplicated cache reading logic

## Implementation Strategy:

### Phase 1: Audit CLI Usage Patterns
- [ ] Identify all places CLI checks file existence manually
- [ ] Identify all places CLI reads cache/aux files manually
- [ ] Map current CLI patterns to available public API functions

### Phase 2: Replace Custom Logic
- [ ] Replace manual cache existence checks with `cache_exists()`
- [ ] Replace manual aux file checks with `aux_file_exists()`
- [ ] Replace manual file reading with `read_aux_file()` and `read_cache()`
- [ ] Ensure error handling consistency

### Phase 3: Validation
- [ ] All CLI functionality works identically
- [ ] All tests pass unchanged
- [ ] Zero warnings in build output
- [ ] Public API functions show as used in dead code analysis

## Pre-commit checklist:
- [ ] Zero warnings in `cargo build`
- [ ] Zero warnings in `cargo test`
- [ ] All tests pass (`cargo test`)
- [ ] CLI functionality unchanged (manual verification)
- [ ] Public API consistently used throughout CLI
- [ ] No custom logic duplicating public API functions