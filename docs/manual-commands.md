# Manual Commands Reference

Comprehensive guide for running every test, build, and benchmark scenario in the Seams development workflow.

## Core Development Commands

### Build Commands

**Standard Debug Build**
```bash
cargo build
```
- Compiles with debug symbols and optimizations disabled
- Fastest compile time, larger binaries
- Use for development and debugging

**Release Build**
```bash
cargo build --release
```
- Optimized for performance and size
- Slower compile time, smaller binaries
- Use for production, benchmarking, and performance testing

**Build with Features**
```bash
cargo build --features mmap
```
- Builds with optional memory-mapped I/O support
- Required for file-by-file benchmarks

**Build Specific Binary**
```bash
cargo build --bin seams
cargo build --bin generate_boundary_tests
cargo build --bin generate_gutenberg_sentences
```

**Build Library Only**
```bash
cargo build --lib
```

### Test Commands

**Run All Tests**
```bash
cargo test
```
- Runs all unit tests, integration tests, and doc tests
- Current execution time: ~2-4 seconds
- Includes 26 unit tests + 8 pipeline tests + 5 incremental tests + 3 integration tests

**Run Specific Test Types**
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test error_handling_integration
cargo test --test incremental_processing_integration
cargo test --test pipeline_integration

# Doc tests only
cargo test --doc
```

**Run Tests by Pattern**
```bash
# Test specific modules
cargo test reader
cargo test sentence_detector
cargo test discovery

# Test specific functions
cargo test test_dialog_coalescing
cargo test test_abbreviation_detection
```

**Run Tests with Output**
```bash
cargo test -- --nocapture
```
- Shows all println! and info! outputs during testing
- Useful for debugging test failures

**Run Tests with Parallel Control**
```bash
# Single-threaded (useful for debugging)
cargo test -- --test-threads=1

# Specific thread count
cargo test -- --test-threads=4
```

### Benchmark Commands

**Run All Benchmarks**
```bash
cargo bench
```
- Runs all benchmark suites
- Requires clean environment for accurate results

**Run Specific Benchmarks**
```bash
# File reading and discovery benchmarks
cargo bench reader_bench

# Detector instantiation benchmarks  
cargo bench detector_instantiation_bench

# File-by-file processing benchmarks (requires mmap feature)
cargo bench file_by_file_bench

# Run benchmark tests for quick validation
cargo bench --bench detector_instantiation_bench -- --test
```

**Run Benchmarks with Features**
```bash
# Required for file-by-file benchmarks
cargo bench --features mmap file_by_file_bench
```

**Environment Setup for Benchmarks**
```bash
# Set environment variable for Gutenberg mirror
export GUTENBERG_MIRROR_DIR=/path/to/your/gutenberg/mirror

# Run specific benchmark
cargo bench reader_bench
```

## Advanced Development Commands

### Code Quality

**Check Code (No Build)**
```bash
cargo check
```
- Fast syntax and type checking without compilation
- Useful for quick validation during development

**Lint with Clippy**
```bash
cargo clippy
```
- Static analysis for common mistakes and improvements
- Must pass with no warnings for clean builds

**Format Code**
```bash
cargo fmt
```
- Automatically formats all Rust code to standard style
- Run before commits to maintain consistency

### Documentation

**Generate Documentation**
```bash
cargo doc
```
- Generates HTML documentation for the crate and dependencies

**Open Documentation in Browser**
```bash
cargo doc --open
```

**Generate Documentation with Private Items**
```bash
cargo doc --document-private-items
```

### Binary Execution

**Run Main Binary**
```bash
# Debug build
cargo run -- /path/to/gutenberg/files

# Release build  
cargo run --release -- /path/to/gutenberg/files

# With options
cargo run --release -- /path/to/gutenberg/files --overwrite-all --fail-fast
```

**Run Utility Binaries**
```bash
# Generate boundary test data
cargo run --bin generate_boundary_tests

# Generate Gutenberg sentence examples
cargo run --bin generate_gutenberg_sentences
```

## Test Environment Setup

### Integration Test Dependencies

**Required Environment Variables**
```bash
# For benchmarks that use real Gutenberg data
export GUTENBERG_MIRROR_DIR=/path/to/project/gutenberg/mirror

# Optional: for specific test configurations
export RUST_LOG=info
```

**Test Data Setup**
```bash
# Create test fixtures (handled automatically by integration tests)
# Tests use tempfile crate for isolated environments
```

### Test Execution Strategies

**Full Test Suite (Recommended)**
```bash
# Run everything with timing
time cargo test

# Run with verbose output
cargo test --verbose
```

**Targeted Testing for Development**
```bash
# Test specific feature during development
cargo test dialog_detector

# Test error handling
cargo test error_handling_integration

# Test incremental processing
cargo test incremental_processing_integration
```

## Performance Testing

### Benchmark Execution Best Practices

**Isolated Benchmark Runs**
```bash
# Run individual benchmarks to prevent interference
cargo bench reader_bench
cargo bench detector_instantiation_bench

# Clean environment between runs
cargo clean && cargo bench reader_bench
```

**Benchmark with Custom Configuration**
```bash
# Set custom measurement time
CRITERION_MEASUREMENT_TIME=30 cargo bench

# Generate HTML reports
cargo bench -- --output-format html
```

**Performance Validation**
```bash
# Check performance claims with specific benchmarks
cargo bench gutenberg_throughput

# Validate concurrency vs sequential performance
cargo bench file_by_file_bench --features mmap
```

### Memory and Resource Testing

**Memory Usage Monitoring**
```bash
# Run with memory profiling (requires external tools)
valgrind --tool=massif cargo test
heaptrack cargo test

# Monitor resource usage during benchmarks
/usr/bin/time -v cargo bench
```

## Debugging and Troubleshooting

### Debug Builds with Symbols

**Full Debug Information**
```bash
cargo build --profile dev
```

**Debug with Backtraces**
```bash
RUST_BACKTRACE=1 cargo test
RUST_BACKTRACE=full cargo test
```

### Verbose Output

**Detailed Compilation**
```bash
cargo build --verbose
cargo test --verbose
```

**Show Command Execution**
```bash
cargo build -vv
```

## CI/CD Pipeline Commands

### Full Validation Pipeline

**Complete Validation (Recommended CI sequence)**
```bash
# 1. Code formatting check
cargo fmt --check

# 2. Lint check
cargo clippy -- -D warnings

# 3. Build check
cargo build

# 4. Test execution
cargo test

# 5. Release build validation
cargo build --release

# 6. Benchmark regression check
cargo bench
```

### Feature Matrix Testing

**Test All Feature Combinations**
```bash
# Default features
cargo test

# With mmap feature
cargo test --features mmap

# With test helpers (for incremental processing integration tests)
cargo test --features test-helpers

# Multiple features
cargo test --features "mmap,test-helpers"

# All features
cargo test --all-features

# No default features
cargo test --no-default-features
```

## Environment-Specific Commands

### Development Environment

**Quick Development Cycle**
```bash
# Fast iteration loop
cargo check && cargo test --lib
```

**Full Development Validation**
```bash
# Complete pre-commit validation
cargo fmt && cargo clippy && cargo test && cargo build --release
```

### Production Environment

**Production Build**
```bash
# Optimized release build
cargo build --release

# Strip debug symbols for minimal size
cargo build --release --config strip=true
```

**Performance Validation**
```bash
# Validate production performance
cargo bench --release
```

## Notes and Best Practices

### Benchmark Isolation
- **Critical**: Run benchmarks individually to prevent performance interference
- Use clean builds between different benchmark types
- Set consistent environment variables for reproducible results

### Test Budget Management
- Unit tests: ~0.01-0.04 seconds (target: under 10 seconds)
- Integration tests: ~2-4 seconds (target: under 2 minutes)
- Benchmarks: Variable (depends on dataset size)

### Feature Flags
- **`mmap`**: Enables memory-mapped I/O support
  - Required for: `file_by_file_bench` benchmark
  - Usage: `cargo test --features mmap`
- **`test-helpers`**: Enables integration test helper functions
  - Required for: Incremental processing integration tests
  - Usage: `cargo test --features test-helpers`
  - Eliminates false positive dead code warnings for test utilities

### Memory-Mapped I/O
- Benchmarks requiring `mmap` feature: `file_by_file_bench`
- Build with `--features mmap` when testing memory-mapped functionality

### Environment Dependencies
- `GUTENBERG_MIRROR_DIR`: Required for realistic benchmark data
- `RUST_LOG`: Controls logging verbosity during testing
- `CRITERION_*`: Controls benchmark execution parameters
- `SEAMS_DEBUG_API`: Enables public API demonstration in CLI (optional)

This reference covers all scenarios needed for comprehensive development, testing, and performance validation of the Seams sentence extractor.