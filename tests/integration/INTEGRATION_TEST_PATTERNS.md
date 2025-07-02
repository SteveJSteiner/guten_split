# Integration Test Patterns

This document outlines the patterns and conventions used in the integration test suite for rs-sft-sentences.

## Test Structure

### Directory Organization
```
tests/
├── integration/
│   ├── mod.rs              # Test utilities and common code
│   └── fixtures/
│       └── mod.rs          # Test data and expected outputs
├── pipeline_integration.rs  # End-to-end pipeline tests
└── error_handling_integration.rs  # Error scenario tests
```

### Test Utilities

**TestFixture**: Creates temporary directories with Gutenberg-style test files
- `new()`: Creates a new temporary test environment
- `create_gutenberg_file(path, content)`: Creates test files matching the `*-0.txt` pattern
- `aux_file_exists(path)`: Checks if auxiliary sentence file exists
- `read_aux_file(path)`: Reads auxiliary file content

**assert_golden_file**: Compares actual vs expected output with detailed diff reporting

## Test Categories

### 1. Pipeline Integration Tests (`pipeline_integration.rs`)

Tests the complete discovery→reader→sentence_detector pipeline with known inputs and expected outputs.

**Pattern**:
```rust
#[tokio::test]
async fn test_pipeline_scenario() {
    let fixture = TestFixture::new();
    let file_path = fixture.create_gutenberg_file("test-0.txt", TEST_CONTENT);
    
    // Test discovery
    let files = discovery::find_gutenberg_files(&fixture.root_path).await.unwrap();
    assert_eq!(files.len(), 1);
    
    // Test file reading
    let content = reader::read_file_async(&file_path).await.unwrap();
    
    // Test sentence detection
    let detector = sentence_detector::SentenceDetector::with_default_rules().unwrap();
    let sentences = detector.detect_sentences(&content).unwrap();
    
    // Validate output format
    let output = format_sentences_output(&sentences);
    assert_golden_file(&output, EXPECTED_OUTPUT, "Test description");
}
```

**Test Scenarios**:
- Simple single-line text with clear boundaries
- Complex multi-line text with normalization
- Challenging punctuation patterns
- Performance validation with large texts
- Multiple file discovery and processing

### 2. Error Handling Tests (`error_handling_integration.rs`)

Tests pipeline behavior under error conditions and edge cases.

**Pattern**:
```rust
#[tokio::test]
async fn test_error_scenario() {
    let fixture = TestFixture::new();
    // Create problematic condition
    
    // Verify graceful handling
    let result = pipeline_operation().await;
    assert!(condition_met, "Error should be handled gracefully");
}
```

**Error Scenarios**:
- Invalid UTF-8 files
- Permission denied access
- Empty files and whitespace-only content
- Non-matching filename patterns
- Deeply nested directory structures

## Test Data Patterns

### Fixture Constants
Located in `integration/fixtures/mod.rs`:

- `SIMPLE_TEXT` / `SIMPLE_EXPECTED`: Basic sentence boundary testing
- `COMPLEX_TEXT` / `COMPLEX_EXPECTED`: Multi-line with normalization
- `PUNCTUATION_TEXT` / `PUNCTUATION_EXPECTED`: Challenging punctuation
- `generate_large_text()`: Performance testing with 500+ sentences

### Output Format Validation
All tests validate the PRD-specified format:
```
index<TAB>sentence<TAB>(start_line,start_col,end_line,end_col)
```

Key patterns:
- 1-based line and column numbering
- TAB separation between fields
- Span coordinates in parentheses
- Normalized sentence content (line breaks removed)

## Performance Requirements

### 2-Minute Budget Compliance
- Full integration test suite must complete within 2 minutes
- Large text test (500 sentences) should complete under 1 second
- Individual tests should complete within milliseconds to seconds

### Performance Test Pattern
```rust
#[tokio::test]
async fn test_performance_scenario() {
    let start = std::time::Instant::now();
    
    // Execute pipeline operation
    let result = operation().await;
    
    let duration = start.elapsed();
    assert!(duration.as_secs() < TARGET_SECONDS, 
        "Operation took {:?}, should be under {}s", duration, TARGET_SECONDS);
    
    // Validate correctness alongside performance
    assert_output_correctness(result);
}
```

## Common Assertions

### Golden File Validation
```rust
assert_golden_file(&actual_output, EXPECTED_OUTPUT, "Test context");
```
- Compares line-by-line with detailed error reporting
- Shows exact line number and content on mismatch
- Includes test context for debugging

### File Discovery Validation
```rust
let files = discovery::find_gutenberg_files(&root_path).await.unwrap();
assert_eq!(files.len(), EXPECTED_COUNT);
assert!(files.iter().all(|f| f.file_name().unwrap().to_str().unwrap().ends_with("-0.txt")));
```

### Sentence Count and Content Validation
```rust
assert_eq!(sentences.len(), EXPECTED_COUNT);
assert_eq!(sentences[0].normalized_content, "Expected sentence.");
assert_eq!(sentences[0].span.start_line, 1);
```

## Adding New Integration Tests

### 1. Choose Appropriate Test File
- Pipeline functionality → `pipeline_integration.rs`
- Error scenarios → `error_handling_integration.rs`
- New category → Create new `*_integration.rs` file

### 2. Add Test Fixtures
- Add constants to `integration/fixtures/mod.rs`
- Include both input text and expected output
- Document the test scenario purpose

### 3. Follow Naming Conventions
- `test_pipeline_*` for end-to-end tests
- `test_error_*` for error scenarios
- `test_performance_*` for timing-sensitive tests

### 4. Validate Requirements
- Test must pass within 2-minute budget
- Output format must match PRD specification
- Error handling must be graceful and logged

## Debugging Failed Tests

### Golden File Mismatches
1. Check the exact line and column reported
2. Verify sentence boundary detection logic
3. Confirm span calculation (1-based indexing)
4. Check sentence normalization (line break removal)

### Performance Issues
1. Run with `--nocapture` to see timing output
2. Use `cargo test --release` for realistic performance
3. Check for unexpected blocking operations
4. Validate FST compilation isn't repeated unnecessarily

### Discovery Issues
1. Verify file naming matches `*-0.txt` pattern
2. Check UTF-8 encoding of test files
3. Confirm directory structure creation
4. Validate file permissions in test environment