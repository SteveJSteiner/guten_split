# Parallel Performance Benchmark for CLI Use Case

* **Task ID:** parallel-performance-benchmark_51.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Current benchmarks test individual file processing but not the actual CLI parallel use case
  - Need to validate performance claims about parallel mmap processing with real workloads
  - Critical to measure end-to-end throughput including file discovery, cache management, and concurrent processing
  - Performance visibility is essential for demonstrating the benefits of the mmap-parallel architecture
  - Benchmark should test the actual `process_files_parallel()` function used by the CLI

* **Acceptance Criteria:**
  1. Benchmark tests actual CLI parallel processing pipeline (discovery → cache → parallel processing)
  2. Measures realistic workloads with multiple files processed concurrently
  3. Reports throughput in chars/sec and MB/s matching CLI output format
  4. Tests different concurrency levels (1, 2, 4, 8+ cores) to show scaling
  5. Validates performance claims made in code comments ("parallel", "concurrent", "high-performance")
  6. Benchmark integrates with existing criterion setup and produces HTML reports

* **Deliverables:**
  - New benchmark file: `benches/parallel_cli_bench.rs`
  - Benchmark function testing `process_files_parallel()` with realistic file sets
  - Performance measurement across different concurrency levels
  - Throughput reporting in chars/sec and MB/s (matching CLI metrics)
  - Integration with existing benchmark infrastructure
  - Update Cargo.toml to include new benchmark

* **Implementation Details:**
  - Use actual `process_files_parallel()` function from main.rs
  - Test with sample Gutenberg files (environment-dependent like other benchmarks)
  - Measure end-to-end performance including:
    - File discovery time
    - Cache loading/saving
    - Parallel mmap processing
    - Aux file writing
  - Test concurrency levels: 1, 2, 4, 8, 16 (bounded by system cores)
  - Report both individual file throughput and aggregate throughput
  - Validate that parallel processing actually improves throughput vs sequential

* **Performance Validation Requirements:**
  - Verify "parallel" claims: concurrency > 1 should show throughput gains
  - Verify "mmap" claims: measure memory usage stays bounded
  - Verify "high-performance" claims: achieve >10 MB/s as per PRD requirements
  - Compare against sequential processing to demonstrate scaling benefits

* **References:**
  - PRD.md: Performance requirement ≥ 10 MB/s sustained throughput
  - src/main.rs: `process_files_parallel()` function to benchmark
  - Existing benchmark setup in benches/ directory
  - CLI performance output format for consistency

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Benchmark runs successfully**: `cargo bench --bench parallel_cli_bench` produces results
- [ ] **Performance claims validated**: Parallel processing shows measurable throughput improvements over sequential