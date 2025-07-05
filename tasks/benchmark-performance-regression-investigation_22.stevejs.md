# Benchmark Performance Regression Investigation

* **Task ID:** benchmark-performance-regression-investigation_22.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Significant performance regression detected in benchmark results
  - DFA throughput dropped from ~181 MiB/s to 47.9 MiB/s (-73%)
  - Manual FST throughput dropped from ~102 MiB/s to 85.4 MiB/s (-16%)
  - DFA changed from being fastest to slowest among original strategies
  - This regression occurred during dialog state machine work when no changes should have affected DFA/Manual FST

* **Acceptance Criteria:**
  1. **Root Cause Identified**: Determine what caused the performance regression
  2. **Changes Documented**: List all changes made during dialog state machine task that could affect benchmarks
  3. **Baseline Comparison**: Document before/after performance numbers clearly
  4. **Regression Source**: Identify if regression is in benchmark code, test data, or underlying implementations
  5. **Performance Restored**: Either revert problematic changes or fix regression to restore baseline performance

* **Performance Regression Details:**

## Before (Original Baseline):
- **Manual FST**: 101.58-101.83 MiB/s 
- **DFA**: 181.25-181.69 MiB/s
- **Performance Order**: DFA (fastest) > Manual FST

## After (Current Results):
- **Manual FST**: 85.4 MiB/s (-16%)
- **DFA**: 47.9 MiB/s (-73%)
- **Dialog State Machine**: 9.5 MiB/s (new)
- **Performance Order**: Manual FST > DFA > Dialog State Machine

## Changes Made During Dialog State Machine Task:
1. **Modified test limit** in `test_populate_baseline_behavior` from 50 to 162
2. **Fixed classification logic** in `classify_match()` line 512 (literal string bug)
3. **Updated benchmark import** in `sentence_detector_bench.rs` line 271 (removed missing functions)
4. **Added dialog state machine benchmark** function and criterion group

* **Deliverables:**
  - Root cause analysis of performance regression
  - Documentation of all changes that could affect benchmarks
  - Performance comparison between git states before/after changes
  - Fix or revert to restore baseline performance for DFA and Manual FST
  - Verification that dialog state machine benchmark doesn't interfere with existing benchmarks

* **Investigation Plan:**

## Phase 1: Change Analysis
1. **Git diff analysis**: Review all changes made during dialog state machine task
2. **Benchmark code review**: Check if benchmark modifications affected existing strategies
3. **Import changes**: Verify the benchmark import fix didn't introduce performance issues
4. **Test data changes**: Confirm test data and environment are consistent

## Phase 2: Isolation Testing
1. **Revert to baseline**: Test performance before any dialog state machine changes
2. **Incremental testing**: Apply changes one at a time to identify regression point
3. **Benchmark isolation**: Run DFA/Manual FST benchmarks independently from dialog state machine
4. **Environment validation**: Verify system performance, CPU scaling, thermal throttling

## Phase 3: Performance Restoration
1. **Fix identified issues**: Address root cause of regression
2. **Validate restoration**: Confirm DFA returns to ~181 MiB/s, Manual FST to ~102 MiB/s
3. **Ensure dialog state machine**: Verify new benchmark still works at expected ~9.5 MiB/s
4. **Document final state**: Record corrected performance baseline for all strategies

* **References:**
  - Original benchmark output showing DFA at 181 MiB/s, Manual FST at 102 MiB/s
  - Current benchmark results in `/target/criterion/gutenberg_throughput/`
  - Dialog state machine implementation in `tests/dialog_state_machine_exploration.rs`
  - Benchmark modifications in `benches/sentence_detector_bench.rs`

## Pre-commit checklist:
- [ ] Root cause of regression identified and documented
- [ ] All changes during dialog state machine task catalogued
- [ ] Performance regression fixed or reverted
- [ ] DFA performance restored to ~181 MiB/s baseline
- [ ] Manual FST performance restored to ~102 MiB/s baseline
- [ ] Dialog state machine benchmark isolated and non-interfering
- [ ] Final performance baselines documented for all strategies