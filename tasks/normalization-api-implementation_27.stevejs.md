# Normalization API Implementation

* **Task ID:** normalization-api-implementation_27.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Task 26 analysis shows 15-22x performance difference between raw detection and normalization
  - Current API always normalizes, making benchmarks measure extraneous normalization cost rather than detection work
  - Need to implement dual-mode API: raw detection for benchmarks, on-demand normalization for CLI
  - Isolate implementations in separate files and remove dead code allowed attributes for cleaner codebase
  - Enable fair benchmark comparisons by separating detection work from normalization overhead

* **Acceptance Criteria:**
  1. **Raw Detection API**: DetectedSentence stores raw content, provides on-demand normalization methods
  2. **Supplied Buffer Support**: normalize_into() method for zero-allocation batch processing
  3. **Convenience Methods**: detect_sentences_normalized() for CLI, detect_sentences_raw() for benchmarks
  4. **Code Organization**: Move DFA implementation to separate file, remove #[allow(dead_code)] attributes
  5. **Benchmark Compatibility**: Update existing benchmarks to use appropriate API (raw for performance, normalized for validation)
  6. **Implementation Coverage**: Update both FST and DFA implementations with new API
  7. **Test Compatibility**: Ensure existing tests pass with API changes

* **Deliverables:**
  - **src/sentence_detector/mod.rs**: Main detector interface with dual-mode API
  - **src/sentence_detector/fst_detector.rs**: FST implementation moved from main file
  - **src/sentence_detector/dfa_detector.rs**: DFA implementation moved from main file  
  - **src/sentence_detector/normalization.rs**: Standalone normalization logic with buffer support
  - **Updated benchmarks**: Use raw API for performance measurement, normalized for validation
  - **Updated tests**: Verify new API functionality and backward compatibility where feasible

* **Implementation Plan:**
  1. **Extract normalization logic** to dedicated module with buffer support
  2. **Restructure DetectedSentence** to store raw content with on-demand normalization
  3. **Split detector implementations** into separate files (FST, DFA)
  4. **Add convenience methods** for common use cases (CLI normalized, benchmark raw)
  5. **Update benchmarks** to use appropriate API for fair performance measurement
  6. **Verify tests pass** with new API structure

* **API Design (from Task 26 analysis):**
```rust
#[derive(Debug, Clone)]
pub struct DetectedSentence {
    pub index: usize,
    pub raw_content: String,
    pub span: Span,
}

impl DetectedSentence {
    pub fn raw(&self) -> &str;
    pub fn normalize(&self) -> String;  // New allocation
    pub fn normalize_into(&self, buffer: &mut String);  // Supplied buffer
}

impl SentenceDetector {
    pub fn detect_sentences_raw(&self, text: &str) -> Result<Vec<DetectedSentence>>;
    pub fn detect_sentences_normalized(&self, text: &str) -> Result<Vec<(usize, String, Span)>>;
}
```

* **Performance Target:**
  - Raw detection should match current DFA performance (~12µs per 10KB)
  - Normalization should only occur when explicitly requested
  - Supplied buffer should reduce allocations in batch scenarios

* **Code Organization Changes:**
  - Remove all #[allow(dead_code)] attributes by making implementations active
  - Split large sentence_detector.rs into focused modules
  - Clean separation between detection logic and normalization logic

* **References:**
  - Task 26: Normalization API design analysis showing 15-22x performance impact
  - Current benchmark infrastructure in benches/sentence_detector_bench.rs
  - Existing normalization logic in src/sentence_detector.rs:normalize_sentence()

## Pre-commit checklist:
- [ ] Raw detection API implemented for both FST and DFA
- [ ] Supplied buffer normalization implemented and tested
- [ ] Convenience methods added for common use cases
- [ ] Code split into focused modules (fst_detector, dfa_detector, normalization)
- [ ] All #[allow(dead_code)] attributes removed
- [ ] Benchmarks updated to use appropriate APIs
- [ ] Tests passing with new API structure
- [ ] Performance verified: raw detection fast, normalization only on-demand