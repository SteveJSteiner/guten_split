# Sentence Detection Throughput Analysis and Fair Benchmarking

* **Task ID:** sentence-detection-throughput-analysis_71.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current benchmark comparisons mix end-to-end pipeline performance with pure sentence detection algorithm performance
  - Seams reports overall throughput (file I/O + detection + aux writing) vs competitors' pure detection time
  - Seams uses multi-threading while Python alternatives are single-threaded, making comparisons potentially unfair
  - Need to isolate sentence detection algorithm performance for accurate benchmarking
* **Acceptance Criteria:**
  1. Benchmark script shows both overall and pure detection throughput for seams
  2. Fair comparison methodology documented between multi-threaded seams and single-threaded alternatives
  3. Optional single-threaded mode for seams to enable direct algorithm comparison
  4. Performance analysis shows where time is spent: I/O vs detection vs aux file writing
* **Deliverables:**
  - Enhanced benchmark reporting with sentence detection throughput breakdown
  - Analysis of multi-threading impact on benchmark fairness
  - Optional `--single-threaded` flag for fair algorithm-to-algorithm comparison
  - Documentation of performance characteristics and bottlenecks
* **References:**
  - Current benchmark shows nupunkt: 19.4M chars/sec vs seams overall: 7.5M chars/sec
  - Seams sentence_detection_time_ms data available in JSON output
  - Need to extract pure detection performance from total pipeline time

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely