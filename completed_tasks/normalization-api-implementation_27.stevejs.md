# Normalization API Implementation

* **Task ID:** normalization-api-implementation_27.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Task 26 analysis shows 15-22x performance difference between raw detection and normalization
  - Current API always normalizes, making benchmarks measure extraneous normalization cost rather than detection work
  - Need experimental framework comparing borrowed vs owned data approaches for mmap vs async I/O optimization
  - Mmap support (300M+ char/sec target) favors borrowed slices, async I/O favors owned strings
  - Gutenberg texts <2MB allow whole-file processing, normalization into reusable buffers per sentence
  - Enable fair benchmark comparisons by separating detection work from normalization overhead

* **Acceptance Criteria:**
  1. **Dual API Implementation**: Both borrowed (`DetectedSentence<'a>`) and owned (`DetectedSentence`) variants
  2. **Supplied Buffer Support**: normalize_into() method for zero-allocation batch processing in both variants
  3. **Convenience Methods**: detect_sentences_normalized() for CLI, detect_sentences_raw() for benchmarks
  4. **Performance Comparison**: Benchmarks comparing borrowed vs owned approaches on same Gutenberg texts
  5. **Code Organization**: Move DFA implementation to separate file, remove #[allow(dead_code)] attributes
  6. **Implementation Coverage**: Update both FST and DFA implementations with dual API variants
  7. **Test Compatibility**: Ensure existing tests pass with API changes
  8. **Memory Management**: Demonstrate buffer reuse patterns for both mmap and async I/O scenarios

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

* **Dual API Design (experimental comparison):**

**Borrowed Variant (mmap-optimized):**
```rust
#[derive(Debug, Clone)]
pub struct DetectedSentenceBorrowed<'a> {
    pub index: usize,
    pub raw_content: &'a str,  // Borrowed from source text
    pub span: Span,
}

impl<'a> DetectedSentenceBorrowed<'a> {
    pub fn raw(&self) -> &str { self.raw_content }
    pub fn normalize(&self) -> String;  // New allocation
    pub fn normalize_into(&self, buffer: &mut String);  // Reusable buffer
}
```

**Owned Variant (async I/O-optimized):**
```rust
#[derive(Debug, Clone)]
pub struct DetectedSentenceOwned {
    pub index: usize,
    pub raw_content: String,  // Owned copy
    pub span: Span,
}

impl DetectedSentenceOwned {
    pub fn raw(&self) -> &str { &self.raw_content }
    pub fn normalize(&self) -> String;  // New allocation
    pub fn normalize_into(&self, buffer: &mut String);  // Reusable buffer
}
```

**Detector Interface:**
```rust
impl SentenceDetector {
    // Borrowed API (mmap-friendly)
    pub fn detect_sentences_borrowed(&self, text: &str) -> Result<Vec<DetectedSentenceBorrowed>>;
    
    // Owned API (async I/O-friendly)  
    pub fn detect_sentences_owned(&self, text: &str) -> Result<Vec<DetectedSentenceOwned>>;
    
    // Convenience methods
    pub fn detect_sentences_normalized(&self, text: &str) -> Result<Vec<(usize, String, Span)>>;
}
```

* **Performance Target:**
  - Raw detection should match current DFA performance (~12µs per 10KB)
  - Borrowed API should minimize allocations (target: zero per sentence detection)
  - Owned API should demonstrate allocation cost for comparison
  - Normalization buffer reuse should reduce allocations in batch scenarios
  - Benchmarks should quantify borrowed vs owned performance difference on Gutenberg texts

* **Code Organization Changes:**
  - Remove all #[allow(dead_code)] attributes by making implementations active
  - Split large sentence_detector.rs into focused modules
  - Clean separation between detection logic and normalization logic

* **References:**
  - Task 26: Normalization API design analysis showing 15-22x performance impact
  - Current benchmark infrastructure in benches/sentence_detector_bench.rs
  - Existing normalization logic in src/sentence_detector.rs:normalize_sentence()

## Pre-commit checklist:
- [x] Dual API implemented: both borrowed and owned variants for FST, DFA, and Dialog detectors
- [x] Supplied buffer normalization implemented and tested for both variants
- [x] Convenience methods added for common use cases (CLI normalized, benchmark raw)
- [x] Performance benchmarks comparing borrowed vs owned on Gutenberg texts
- [x] Code split into focused modules (fst_detector, dfa_detector, dialog_detector, normalization)
- [x] Dialog state machine moved from tests to official implementation with dual API
- [x] Tests passing with dual API structure and dialog integration
- [x] Benchmarks updated to include all three implementations (FST, DFA, Dialog)
- [x] Memory allocation patterns documented: borrowed API ~0 allocations, owned API minimal overhead
- [x] Performance verified: DFA borrowed ~60.2µs, FST borrowed ~70.8µs, Dialog borrowed ~76.3µs, owned APIs competitive
- [x] O(n^2) performance issues identified and fixed in FST detector
- [x] File-by-file benchmarking implemented with realistic throughput measurements (407M chars/sec Dialog, 339M chars/sec DFA, 165M chars/sec FST)