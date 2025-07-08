# File Discovery Cache Optimization for Faster Restarts

* **Task ID:** file-discovery-cache-optimization_56.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - PRD F-11 requires caching discovered file locations to avoid slow directory traversal
  - Current incremental processing cache tracks completion but not file discovery
  - Large Gutenberg mirrors have thousands of files - directory traversal is expensive
  - Second processing runs should skip directory search entirely when possible
  - Critical for user experience with large corpora (10+ minute discovery vs instant)

* **Acceptance Criteria:**
  1. File discovery results are cached and reused on subsequent runs
  2. Cache invalidation works correctly when directory structure changes
  3. Cache includes file metadata (size, modified time) for change detection
  4. Significant performance improvement for repeated runs on same directory
  5. Cache works correctly with --overwrite_all flag
  6. Graceful fallback to full discovery if cache is corrupted/missing

* **Deliverables:**
  - Extended cache format to include discovered file list with metadata
  - Cache invalidation logic based on directory modification times
  - Integration with existing incremental processing cache system
  - Performance benchmarks showing discovery time improvements
  - Cache management utilities (clear, validate, etc.)

* **Implementation Strategy:**
  - **Option A**: Extend existing ProcessingCache with file discovery data
  - **Option B**: Separate discovery cache with cross-references to processing cache
  - **Option C**: Unified cache with layered invalidation (directory vs file level)

* **Cache Invalidation Triggers:**
  - Root directory modification time changed
  - Cache file missing or corrupted
  - Explicit cache clear command
  - Version mismatch (cache format changes)

* **Performance Targets:**
  - **Cold start**: No regression in initial discovery time
  - **Warm start**: >90% reduction in discovery time for unchanged directories
  - **Mixed scenario**: Fast detection of new/changed files in mostly-stable directory

* **Implementation Considerations:**
  - Memory usage for large file lists (consider compression/pagination)
  - Cross-platform directory change detection
  - Race conditions with concurrent file system changes
  - Cache size limits and cleanup policies

* **Integration Points:**
  - Existing `discovery::collect_discovered_files()` function
  - Current ProcessingCache implementation in main.rs
  - Cache path generation utilities in incremental.rs

* **References:**
  - PRD F-11: Cache discovered file locations
  - Current discovery implementation in src/discovery/
  - Existing cache management in src/incremental.rs

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (performance improvement measured)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Performance benchmark**: Discovery time improvement on repeated runs measured
- [ ] **Cache reliability**: Invalidation logic tested with various scenarios