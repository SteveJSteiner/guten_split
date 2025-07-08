# Make file_by_file_bench Available Without Feature Flags

* **Task ID:** benchmark-mmap-feature-removal_48.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current file_by_file_bench requires `--features mmap` to compile and run
  - Warning-free validation script doesn't test mmap-dependent benchmarks
  - Benchmark should be available in default configuration for consistent performance testing
  - Simplifies benchmark execution and documentation
  - Ensures all benchmarks can be run with simple `cargo bench` command
  - memmap2 is stable, well-tested, and idiomatic - no need for custom wrappers

* **Acceptance Criteria:**
  1. `cargo bench --bench file_by_file_bench` works without feature flags
  2. Zero warnings in `cargo bench --bench file_by_file_bench` output
  3. memmap2 dependency moved to core dependencies (not optional)
  4. Benchmark functionality preserved (still measures file processing performance)
  5. No regression in benchmark accuracy or usefulness
  6. Updated documentation reflects simplified benchmark execution

* **Deliverables:**
  - Move memmap2 from optional to core dependency in Cargo.toml
  - Remove `mmap = ["memmap2"]` feature definition
  - Remove `required-features = ["mmap"]` from benchmark definition
  - Updated docs/manual-commands.md to reflect simplified benchmark usage
  - Verified warning-free execution across all benchmark scenarios

* **References:**
  - Current benchmark implementation in `benches/file_by_file_bench.rs`
  - Feature configuration in `Cargo.toml`
  - docs/manual-commands.md benchmark documentation
  - Warning-free validation requirements from CLAUDE.md section 2.3

## Implementation Strategy:

### Phase 1: Update Cargo.toml Dependencies
- [ ] Move memmap2 from `[dependencies]` optional to required: `memmap2 = "0.9"`
- [ ] Remove `mmap = ["memmap2"]` from `[features]` section
- [ ] Remove `required-features = ["mmap"]` from `[[bench]]` file_by_file_bench definition

### Phase 2: Validation and Documentation  
- [ ] Verify `cargo bench --bench file_by_file_bench` works without feature flags
- [ ] Confirm zero warnings in benchmark execution
- [ ] Update docs/manual-commands.md to remove `--features mmap` requirement
- [ ] Run warning-free validation script to ensure no regressions

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [x] Documentation updated if needed
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [x] Benchmark runs successfully: `cargo bench --bench file_by_file_bench`
- [x] No feature flags required: benchmark works with default configuration
- [x] memmap2 is now a core dependency, not optional

## COMPLETION NOTES
**Date:** 2025-07-08
**Completed through:** File discovery cache optimization work

**Changes Made:**
- memmap2 moved to core dependencies in Cargo.toml (line 27)
- No mmap feature flag exists in [features] section
- Benchmark verified working without feature flags
- Zero warnings validation passes

**Verification Results:**
- ✅ `cargo bench --bench file_by_file_bench` works without flags
- ✅ Zero warnings across all scenarios
- ✅ memmap2 available by default

**Task Status:** COMPLETED - Benchmark available without feature flags, memmap2 is core dependency