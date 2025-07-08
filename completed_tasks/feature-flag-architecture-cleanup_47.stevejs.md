# Feature Flag Architecture Cleanup

* **Task ID:** feature-flag-architecture-cleanup_47.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Core CLI functionality incorrectly positioned as "optional" in documentation
  - Memory-mapped I/O (`mmap`) is incorrectly gated behind optional feature
  - Public incremental API usage by CLI should be mandatory, not feature-gated
  - Feature flags being used for core functionality instead of truly optional features
  - Current architecture confuses essential vs optional functionality
  - `mmap` is a core performance feature that should work by default

* **Acceptance Criteria:**
  1. Core CLI functionality works without any feature flags
  2. Only truly optional features are behind feature gates
  3. Documentation clearly distinguishes mandatory vs optional functionality
  4. CLI's use of public incremental API is unconditional and standard
  5. Feature flag usage follows Rust best practices
  6. Zero confusion about what's required vs optional

* **Deliverables:**
  - Review and fix feature flag architecture
  - Ensure CLI core functionality is unconditional
  - Update documentation to clarify feature flag usage
  - Validate that public API adoption is standard behavior
  - Clean separation between core and optional functionality

* **Current Issues:**
  - `mmap` feature incorrectly gates core performance functionality (WRONG)
  - `test-helpers` feature gates test utilities (correct) 
  - Documentation suggests core functionality is optional (incorrect)
  - Core performance features hidden behind optional flags
  - Need clear architectural principles for feature usage

* **Architecture Principles:**
  **Mandatory (no feature flags):**
  - CLI using public incremental API (`cache_exists`, `aux_file_exists`, etc.)
  - Memory-mapped I/O support (`mmap`) - core performance feature
  - Basic sentence detection functionality
  - Standard file I/O operations
  - Core CLI argument parsing and execution
  - All benchmarks should work by default

  **Optional (feature flags):**
  - `test-helpers` - Integration test utilities only
  - Future experimental features
  - Platform-specific experimental optimizations
  - Debug/development-only features

* **References:**
  - Cargo Book on features: https://doc.rust-lang.org/cargo/reference/features.html
  - Rust API Guidelines on feature flags
  - Current Cargo.toml feature definitions
  - Manual commands documentation

## Implementation Plan:

### Phase 1: Architecture Review
- [ ] Audit all current feature flag usage
- [ ] Identify what should vs shouldn't be optional
- [ ] Document architectural principles

### Phase 2: Code Cleanup  
- [ ] Move `mmap` from optional feature to default dependency
- [ ] Ensure all benchmarks work without feature flags
- [ ] Ensure CLI core functionality has no feature dependencies
- [ ] Validate public API usage is unconditional
- [ ] Fix any incorrect feature gating of core functionality

### Phase 3: Documentation Cleanup
- [ ] Update manual commands documentation
- [ ] Clarify what features are for
- [ ] Remove confusion about optional vs mandatory

## Pre-commit checklist:
- [x] Core CLI works with `cargo build` (no features)
- [x] Core CLI works with `cargo run -- --help` (no features)
- [x] All tests pass with default features
- [x] Optional features work when enabled
- [x] Documentation accurately reflects architecture
- [x] No core functionality behind feature flags

## COMPLETION NOTES
**Date:** 2025-07-08
**Completed through:** File discovery cache optimization work

**Changes Made:**
- memmap2 moved from optional to core dependency in Cargo.toml
- Only `test-helpers` remains as optional feature
- CLI core functionality works without any feature flags
- All benchmarks work by default
- Architecture follows documented principles

**Current Feature Architecture:**
- **Mandatory (no flags):** All CLI functionality, mmap, benchmarks, core API
- **Optional (flagged):** Only `test-helpers` for integration test utilities

**Verification Results:**
- ✅ `cargo build` works without features
- ✅ `cargo run --bin seams -- --help` works
- ✅ All tests pass with default features
- ✅ Feature architecture matches principles

**Task Status:** COMPLETED - Feature flag architecture cleaned up, core functionality unconditional