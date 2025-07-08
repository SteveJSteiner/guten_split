# Overlapped File Discovery and Sentence Processing Pipeline

* **Task ID:** overlapped-discovery-processing_57.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current pipeline waits for complete file discovery before starting sentence processing
  - No technical reason to block processing on full discovery completion
  - Large corpora discovery can take significant time (minutes for thousands of files)
  - Processing can begin immediately as files are discovered
  - Overlapped pipeline can improve total throughput and reduce perceived latency
  - Better resource utilization with parallel I/O and CPU work

* **Acceptance Criteria:**
  1. File processing begins as soon as first valid files are discovered
  2. Discovery and processing pipelines run concurrently without blocking
  3. Processing respects existing concurrency limits and cache behavior
  4. Error handling works correctly for both discovery and processing phases
  5. Cache updates happen correctly for both discovery results and completed processing
  6. Total pipeline time is reduced compared to sequential discovery → processing
  7. All existing functionality preserved (incremental processing, --overwrite flags, etc.)

* **Deliverables:**
  - Refactored main function to use streaming discovery + processing pipeline
  - Concurrent discovery and processing with shared state management
  - Updated cache management to handle overlapped operations
  - Performance benchmarks showing total pipeline time improvement
  - Integration tests validating overlapped pipeline behavior
  - Error handling that gracefully manages failures in either phase

* **Implementation Strategy:**

### Current Architecture (Sequential)
```
Discovery Phase: [░░░░░░░░░░] → Processing Phase: [████████████]
Total Time: Discovery + Processing
```

### Target Architecture (Overlapped)
```
Discovery:  [░░░░░░░░░░]
Processing:    [████████████████]
Total Time: max(Discovery, Processing) + startup overlap
```

### Technical Approach

**Option A: Channel-Based Pipeline**
- Discovery sends files through channel to processing workers
- Processing workers consume from channel as files become available
- Natural backpressure through bounded channels

**Option B: Shared State with Notifications**
- Discovery updates shared state with discovered files
- Processing polls/waits for new files to become available
- More complex synchronization but more control

**Option C: Async Stream Integration**
- Convert discovery to async stream of files
- Processing consumes stream with concurrency limits
- Leverages existing async/await patterns

### Cache Management Considerations
- Discovery cache updates can happen incrementally as files are found
- Processing cache updates happen per-file as before
- Need to handle case where discovery completes before processing
- Ensure cache consistency when both phases are updating simultaneously

### Error Handling Strategy
- Discovery errors should not block processing of already-found files
- Processing errors should be handled per-file as before
- --fail-fast should abort both discovery and processing
- Graceful shutdown when one phase completes or fails

* **Performance Targets:**
  - **Small corpora (1-10 files):** No regression in total time
  - **Medium corpora (100-1000 files):** 20-30% reduction in total pipeline time
  - **Large corpora (1000+ files):** 40-60% reduction in total pipeline time
  - **Memory usage:** No significant increase (bounded by existing concurrency limits)

* **Integration Points:**
  - Current `discovery::collect_discovered_files()` function
  - Current `process_files_parallel()` function  
  - Cache management in main.rs
  - Error handling and --fail-fast logic

* **Test Scenarios:**
  1. Normal operation with various corpus sizes
  2. Discovery errors (permission denied, invalid patterns)
  3. Processing errors (UTF-8 issues, DFA failures)
  4. --fail-fast behavior with overlapped pipeline
  5. Cache consistency with concurrent discovery/processing updates
  6. Graceful shutdown scenarios

* **References:**
  - Current sequential pipeline in main.rs lines 426-456
  - Discovery implementation in src/discovery.rs
  - Parallel processing in process_files_parallel()
  - Cache management and incremental processing logic

## Implementation Phases

### Phase 1: Research and Design
- [ ] Analyze current discovery and processing bottlenecks
- [ ] Choose between Channel, Shared State, or Async Stream approach
- [ ] Design cache update strategy for overlapped operations
- [ ] Create performance measurement baseline

### Phase 2: Core Pipeline Implementation  
- [ ] Implement overlapped discovery + processing pipeline
- [ ] Update cache management for concurrent operations
- [ ] Preserve all existing CLI flags and behaviors
- [ ] Ensure bounded concurrency and memory usage

### Phase 3: Error Handling and Edge Cases
- [ ] Implement proper error propagation between phases
- [ ] Handle --fail-fast correctly with overlapped pipeline
- [ ] Add graceful shutdown for both discovery and processing
- [ ] Test edge cases (empty directories, permission errors, etc.)

### Phase 4: Performance Validation
- [ ] Benchmark overlapped vs sequential pipeline
- [ ] Validate performance targets across different corpus sizes
- [ ] Ensure no regression in small corpus performance
- [ ] Measure memory usage and resource efficiency

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (overlapped pipeline shows performance improvement)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Performance benchmarks**: Overlapped pipeline faster than sequential
- [ ] **Integration tests**: All existing functionality preserved
- [ ] **Error handling**: --fail-fast and error propagation work correctly
- [ ] **Cache consistency**: Discovery and processing cache updates work correctly
- [ ] **Memory usage**: No significant increase in peak memory consumption