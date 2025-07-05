# Normalization API Design

* **Task ID:** normalization-api-design_26.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Task 25 revealed normalization disparity: Manual FST & DFA normalize sentences, Dialog State Machine skips normalization
  - Need consistent API that allows both normalized and raw sentence output
  - Must optimize for both ease of use and performance (avoid unnecessary O(n) operations)
  - Current approaches either always normalize (performance cost) or never normalize (missing feature)
  - Production API needs flexibility to handle both use cases efficiently

* **Acceptance Criteria:**
  1. **API Design**: Unified interface that supports both normalized and raw sentence output
  2. **Performance Optimization**: Avoid unnecessary normalization when raw content is sufficient
  3. **Ease of Use**: Simple API for common use cases (normalized output as default)
  4. **Memory Efficiency**: Minimize allocations and avoid duplicate processing
  5. **Lazy Evaluation**: Normalization only performed when requested
  6. **Benchmark Support**: Enable benchmarking with and without normalization for fair comparisons

* **EXPLICIT NON-REQUIREMENTS:**
  - **NO Backward Compatibility**: Breaking changes to existing APIs are acceptable for better design

* **Design Considerations:**
  - **When to normalize**: At detection time vs. on-demand vs. configurable
  - **Memory strategy**: Store both raw and normalized vs. compute on demand vs. supplied buffer
  - **API ergonomics**: Method chaining vs. configuration vs. separate types
  - **Performance impact**: Benchmark normalization overhead vs. detection overhead
  - **Production requirements**: CLI needs normalized output, benchmarks may want raw
  - **Benchmark fairness**: Must allow equivalent work comparison (with/without normalization)

* **API Design Options to Explore:**

## Option 1: Lazy Normalization with Getters
```rust
struct DetectedSentence {
    raw_content: String,
    normalized_content: OnceCell<String>, // Computed on first access
    span: Span,
}

impl DetectedSentence {
    pub fn raw(&self) -> &str { &self.raw_content }
    pub fn normalized(&self) -> &str { 
        self.normalized_content.get_or_init(|| normalize(&self.raw_content))
    }
}
```

## Option 2: Configuration-Based Normalization
```rust
#[derive(Default)]
struct DetectionConfig {
    normalize_output: bool,
    preserve_raw: bool,
}

impl SentenceDetector {
    pub fn detect_sentences_with_config(&self, text: &str, config: DetectionConfig) -> Vec<DetectedSentence>
}
```

## Option 3: Separate Raw and Normalized APIs
```rust
impl SentenceDetector {
    pub fn detect_sentences_raw(&self, text: &str) -> Vec<RawSentence>
    pub fn detect_sentences(&self, text: &str) -> Vec<NormalizedSentence> // Default normalized
    pub fn detect_sentences_both(&self, text: &str) -> Vec<DetectedSentence> // Both available
}
```

## Option 4: Post-Processing API
```rust
struct SentenceProcessor;
impl SentenceProcessor {
    pub fn normalize_batch(sentences: &mut [DetectedSentence])
    pub fn normalize_single(sentence: &mut DetectedSentence)
}
```

## Option 5: Supplied Buffer Strategy
```rust
struct NormalizationBuffer {
    buffer: String,
}

impl SentenceDetector {
    pub fn detect_sentences_into_buffer(&self, text: &str, buffer: &mut NormalizationBuffer) -> Vec<DetectedSentence>
    pub fn detect_sentences_raw(&self, text: &str) -> Vec<RawSentence> // No normalization
}
```

* **Investigation Areas:**
  1. **Current normalization logic**: Analyze `normalize_sentence()` implementation cost
  2. **Usage patterns**: How often is normalized vs raw content needed in production
  3. **Memory impact**: Cost of storing both raw and normalized content vs supplied buffer strategy
  4. **Performance benchmarks**: Measure normalization overhead vs detection overhead
  5. **API usability**: Test different API designs for ergonomics
  6. **Benchmark equivalence**: Ensure fair performance comparisons possible with/without normalization

* **Deliverables:**
  - **API design proposal** with recommended approach and rationale
  - **Performance analysis** of normalization overhead and supplied buffer benefits
  - **Usage pattern analysis** for different API options
  - **Implementation plan** for chosen API design
  - **Benchmark strategy** for fair performance comparisons with/without normalization

* **Technical Analysis:**
  1. **Benchmark current normalization cost** in isolation
  2. **Analyze memory usage** of different storage strategies including supplied buffer
  3. **Test API ergonomics** with real use cases
  4. **Measure lazy vs eager normalization** performance characteristics
  5. **Design benchmark parity** strategy for fair comparisons

* **References:**
  - Task 25: Benchmark implementation audit revealing normalization disparity
  - Current `normalize_sentence()` implementation in src/sentence_detector.rs
  - PRD requirements for sentence output format (F-6: normalize hard line breaks)
  - Production CLI requirements for clean sentence output

## Pre-commit checklist:
- [x] Current normalization implementation analyzed for performance cost
- [x] API design options prototyped and evaluated including supplied buffer strategy
- [x] Performance benchmarks completed for different approaches
- [x] Recommended API design documented with rationale
- [x] Implementation plan created for chosen approach
- [x] Benchmark parity strategy designed for fair performance comparisons

## Results Summary

### Performance Analysis (Real Gutenberg Text):
- **Current API** (always normalize): ~240µs per 10KB text
- **Lazy API** (normalize on demand): ~15µs initial (16x faster)
- **Raw API** (no normalization): ~12µs per 10KB (22x faster) 
- **Buffer reuse**: Similar to current but reduces allocations

**Key Finding**: Normalization adds 15-22x overhead to sentence detection.

### Recommended API Design: Dual-Mode with Supplied Buffer
```rust
#[derive(Debug, Clone)]
pub struct DetectedSentence {
    pub index: usize,
    pub raw_content: String,  // Always store raw
    pub span: Span,
}

impl DetectedSentence {
    pub fn raw(&self) -> &str;
    pub fn normalize(&self) -> String;  // On-demand with allocation
    pub fn normalize_into(&self, buffer: &mut String);  // Zero-allocation
}

impl SentenceDetector {
    // For benchmarks (raw performance measurement)
    pub fn detect_sentences_raw(&self, text: &str) -> Result<Vec<DetectedSentence>>;
    
    // For CLI (always needs normalized)
    pub fn detect_sentences_normalized(&self, text: &str) -> Result<Vec<(usize, String, Span)>>;
}
```

### Rationale:
1. **E2E Performance**: Raw detection enables fast benchmarks (22x improvement)
2. **Benchmark Fairness**: Clear separation between detection vs normalization work
3. **Memory Control**: Supplied buffer eliminates allocations in batch processing
4. **Flexibility**: CLI can normalize on-demand, benchmarks use raw content
5. **Breaking Changes**: Acceptable per explicit non-requirements

### Implementation Plan:
Created task 27 (normalization-api-implementation_27.stevejs) to implement this design with:
- Restructured DetectedSentence storing raw content
- Supplied buffer normalization support
- Code organization: split into focused modules
- Remove #[allow(dead_code)] attributes
- Update benchmarks for fair performance measurement

### Benchmark Strategy:
- **Performance benchmarks**: Use raw API to measure pure detection speed
- **Validation benchmarks**: Use normalized API to verify equivalent sentence splitting
- **Throughput reporting**: Characters/second for both raw and normalized paths