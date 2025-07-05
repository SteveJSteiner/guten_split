# Testing Strategy & Process Improvements

## Unit Test Strategy & 10-Second Budget

Current baseline: 21 tests complete in ~0.01s, leaving 9.99s budget for expansion.

### Optimization Priorities
- **I/O-intensive tests** — Maximize file system, network, and concurrent I/O test coverage since these are the primary performance-critical paths.
- **Error path coverage** — Test all fail_fast vs resilient behavior modes thoroughly.
- **Integration scenarios** — Test module boundaries (discovery→reader→sentence_detector) with realistic data.

### Efficient Test Patterns
- Use `tempfile::TempDir` for isolated I/O tests rather than mocking.
- Test with varying file sizes: empty, small (1KB), medium (100KB), large (10MB).
- Parametrize buffer sizes and concurrency levels in reader tests.
- Use `tokio::test` with timeouts to prevent runaway async tests.

### Test Budget Allocation
```
├─ I/O & Async (7s budget)     │  File operations, UTF-8 validation, streaming
├─ FST & Detection (2s budget) │  Sentence boundary logic, Unicode handling  
├─ Integration (1s budget)     │  End-to-end workflows, error propagation
```

## Performance Testing Assessment

### Current Benchmarks
- File discovery speed (`benches/reader_bench.rs`)
- File reading throughput
- Basic regression detection (10 samples each)

### Performance Testing Gaps
Missing benchmarks for:
- **Sentence detection throughput** — FST performance on various text types
- **Memory usage validation** — Peak memory during large file processing
- **Concurrent vs sequential timing** — Validate any concurrency claims with measurements
- **End-to-end pipeline performance** — Discovery→reading→detection→output

### Benchmark Enhancement Recommendations
1. Add sentence detection benchmarks with varying text complexity
2. Memory profiling integration for large dataset processing
3. Comparative benchmarks (concurrent vs sequential implementations)
4. Real-world dataset benchmarks (Project Gutenberg subset)

### Benchmark Isolation Requirements
**Critical**: Multiple benchmark functions in the same file can cause performance interference through compilation/memory overhead.

**Best Practices:**
- **Isolated Execution**: Run individual benchmarks with `cargo bench <benchmark_name>` rather than full benchmark suites
- **Separate Benchmark Groups**: Use separate Criterion groups for unrelated benchmarks to prevent interference
- **Gutenberg Throughput**: Use `cargo bench gutenberg_throughput` for clean isolated measurements
- **Avoid Artificial Benchmarks**: Focus on real-world Gutenberg throughput; artificial benchmarks can contaminate results

**Validated Performance Baselines (Isolated):**
- **DFA**: 182.12 MiB/s (regex-automata implementation)
- **Manual FST**: 102.81 MiB/s (production baseline)
- **Dialog State Machine**: 490.36 MiB/s (with coalescing optimizations)

## Integration Testing Strategy

### Current Gap Analysis
Unit tests validate individual modules but miss issues in module boundaries and end-to-end workflows. Need systematic integration testing for discovery→reader→sentence_detector pipeline.

**Note: This section provides specifications for future implementation. The actual integration test framework is listed in TODO_FEATURES.md as a Process-Feature task.**

### Integration Test Budget: 2 Minutes (120 seconds)

### Integration Test Categories

**1. Happy Path Integration (60s budget)**
- Small dataset: 3 files, basic sentence structures
- Medium dataset: 20 files, mixed complexity  
- Large dataset: 100+ files for throughput validation
- Validate output format matches F-5 specification exactly

**2. Error Propagation Testing (30s budget)**
- Invalid UTF-8 files → error handling through pipeline
- Permission errors → fail_fast vs resilient behavior
- Corrupted files → graceful degradation
- Mixed valid/invalid file batches

**3. Performance Integration (30s budget)**
- Memory usage during large file processing
- Pipeline throughput (files/second)
- Resource cleanup verification
- Concurrent vs sequential processing validation

### Integration Test Framework
```rust
// Integration test structure in tests/integration_tests.rs
#[tokio::test]
async fn test_complete_pipeline() {
    // 1. Set up test data (various file types, sizes, encodings)
    // 2. Run discovery
    // 3. Feed results to reader  
    // 4. Feed reader output to sentence detector
    // 5. Validate end-to-end output format and correctness
}
```

### Integration Test Implementation Plan

**Phase 1: Basic Pipeline Tests**
```rust
tests/
├── integration_tests.rs        │ Main integration test suite
├── fixtures/                   │ Test data
│   ├── valid_gutenberg/       │ Clean UTF-8 files with various sentence patterns
│   ├── invalid_encoding/      │ Non-UTF-8 test cases
│   └── edge_cases/            │ Empty files, huge files, unicode edge cases
└── helpers/                   │ Shared test utilities
    └── test_data_generator.rs │ Generate consistent test datasets
```

**Phase 2: End-to-End Validation**
- Create integration tests that mirror real Gutenberg processing
- Validate against known-good outputs for regression detection
- Test with realistic file sizes and content complexity

**Phase 3: Error Scenario Testing**
- Systematic testing of all error paths through the pipeline
- Validate that errors in one module don't corrupt subsequent modules
- Test fail_fast vs resilient behavior at integration level

### Integration Test Execution Strategy
- Run integration tests in CI after unit tests pass
- Use separate test data directory to avoid polluting unit tests
- Parallel execution where possible to maximize coverage within 120s budget
- Focus on realistic scenarios that unit tests cannot cover


## To-Do Features System

### Implementation
The to-do features system is implemented in `TODO_FEATURES.md` in the repo root. This system captures both features and process improvements without context bloat.

### Feature Backlog Management  
- See `TODO_FEATURES.md` for current backlog
- Update during planning sessions, not during implementation
- Reference from task files but don't duplicate details
- Archive completed items to preserve decision history
- Focus on clear descriptions and prerequisites rather than time estimates or priority rankings

## Streamlined Task-to-Commit Workflow

### Current Friction Points
- Awkward transitions between task completion and commit preparation
- Context switching between implementation and documentation
- Manual correlation between task deliverables and actual changes

### Improved Workflow
1. **Task completion checklist** built into task template:
   ```markdown
   ## Pre-commit checklist:
   - [ ] All deliverables implemented
   - [ ] Tests passing (`cargo test`)
   - [ ] Claims validated (see CLAUDE.md section 3.1)
   - [ ] Documentation updated if needed
   ```

2. **Commit message automation** using task metadata:
   ```bash
   # Generated from task file metadata
   feat: implement FST-based sentence detection (see tasks/sentence-fst_3.md)
   
   - Compile boundary rules into immutable FST at startup
   - Detect sentence boundaries with linear-time lookup
   - Add comprehensive test coverage for Unicode handling
   ```

3. **Task archival flow**:
   - Complete task → validate deliverables → commit changes → archive task file
   - No gap between implementation and commit preparation
   - Task file becomes commit message source of truth

### Implementation Notes
- Consider simple shell script to generate commit messages from task files
- Archive completed tasks to `tasks/archive/` to preserve history
- Link archived tasks in commit messages for full traceability