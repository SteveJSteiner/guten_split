# Investigate File Discovery Performance Issue

* **Task ID:** investigate-discovery-performance_62.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - File discovery is taking 8-10 seconds to find just 32 files in `/Users/stevejs/gutenberg_texts`
  - This is unacceptable performance for a file discovery operation
  - Log analysis shows discovery time: "2025-07-08T22:43:10.461652Z" → "2025-07-08T22:43:18.912485Z" (8.5 seconds)
  - Initial investigation revealed UTF-8 validation during discovery was reading entire files (hundreds of KB each)
  - However, even after optimization attempts, discovery is still slow, indicating deeper performance issues
  - Root cause needs to be identified and fixed for optimal user experience

* **Acceptance Criteria:**
  1. Identify the specific bottleneck causing 8+ second discovery time for 32 files
  2. Discovery time should be under 1 second for 32 files (target: sub-500ms)
  3. Maintain all existing functionality while optimizing performance
  4. All tests continue to pass after optimization
  5. Document the root cause and solution for future reference

* **Investigation Areas:**
  - **File I/O Operations**: Check if discovery is still reading file contents unnecessarily
  - **Synchronous vs Async**: Verify all operations are properly async and non-blocking
  - **Parallel Processing**: Ensure parallel discovery is actually running in parallel
  - **File System Access**: Check for slow file system operations (metadata, permissions)
  - **UTF-8 Validation**: Even after optimization, verify no residual file reading
  - **Channel/Stream Performance**: Check if async stream implementation has bottlenecks
  - **Semaphore Contention**: Verify semaphore limits aren't causing unnecessary serialization
  - **Glob Pattern Performance**: Check if glob pattern matching is inefficient

* **Deliverables:**
  - Root cause analysis with specific performance bottleneck identified
  - Performance optimization implementation
  - Updated discovery code that achieves target performance (< 1 second for 32 files)
  - Consider removing `is_valid_utf8` field entirely and deferring all validation to processing phase
  - Benchmark validation showing before/after performance improvement
  - Documentation of the fix for future reference

* **Implementation Strategy:**
  1. **Profiling**: Add detailed timing logs to isolate the bottleneck within discovery
  2. **Minimal Discovery**: Consider stripping discovery to absolute minimum (only file existence)
  3. **Remove UTF-8 Field**: Eliminate `is_valid_utf8` from `FileValidation` struct entirely
  4. **Processing-Time Validation**: Move all file validation to processing phase where content is read anyway
  5. **Parallel Optimization**: Ensure discovery tasks are truly running in parallel without contention
  6. **Benchmark**: Measure discovery time before and after optimization

* **References:**
  - src/discovery.rs - Current discovery implementation with performance issues
  - src/main.rs - Processing phase where UTF-8 validation already occurs
  - Performance target: Discovery should be filesystem-metadata-speed only (< 1 second for 32 files)

## Pre-commit checklist:
- [ ] Root cause identified and documented
- [ ] Discovery performance optimized to target (< 1 second for 32 files)
- [ ] All tests passing (`cargo test`)
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] Benchmark showing performance improvement documented
- [ ] UTF-8 validation properly moved to processing phase