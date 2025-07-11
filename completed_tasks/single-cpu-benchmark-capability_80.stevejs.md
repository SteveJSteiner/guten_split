# Single CPU Benchmark Capability

* **Task ID:** single-cpu-benchmark-capability_80.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Need single-threaded performance baseline for publication benchmarks
  - Enable fair comparisons with other tools that may not be multi-threaded
  - Provide CPU-agnostic performance metrics independent of core count
  - Required for credible performance claims in publication materials
* **Acceptance Criteria:**
  1. Add CLI flag to restrict seams to single CPU operation
  2. Ensure single-CPU mode bypasses all parallel processing paths
  3. Maintain full functionality (file discovery, processing, output) in single-CPU mode
  4. Verify benchmarks can run with single-CPU restriction
  5. Document single-CPU performance characteristics
  6. Ensure flag works with existing benchmark infrastructure
* **Deliverables:**
  - CLI flag implementation (e.g., `--single-cpu` or `--max-threads 1`)
  - Modified parallel processing logic to respect CPU limitation
  - Updated file discovery to work in single-threaded mode
  - Benchmark validation with single-CPU restriction
  - Performance documentation comparing single vs. multi-CPU throughput
* **References:**
  - Current parallel processing implementation using tokio and semaphores
  - Benchmark infrastructure in benchmarks/ directory
  - Publication requirements for performance validation

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely