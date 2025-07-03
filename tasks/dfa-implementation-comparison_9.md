# Add regex-automata DFA Implementation for Comparison

* **Task ID:** dfa-implementation-comparison_9
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Need to compare manual pattern matching vs high-performance DFA approach
  - regex-automata produces dense DFA tables for O(n) sentence boundary detection
  - Establish baseline performance comparison between manual and FST-based approaches
  - Validate that DFA approach produces identical results to manual implementation

* **Acceptance Criteria:**
  1. All existing tests pass with both manual and DFA implementations
  2. Both implementations produce identical sentence detection results
  3. DFA implementation available as alternative detector alongside manual one
  4. Performance benchmarks show DFA vs manual implementation comparison

* **Deliverables:**
  - Add regex-automata dependency to Cargo.toml (keep existing manual implementation)
  - New `SentenceDetectorDFA` struct alongside existing `SentenceDetector`
  - Basic DFA pattern: `[.!?]\s+[A-Z]` for sentence boundary detection
  - Side-by-side performance benchmarks: manual vs DFA approaches
  - Identical output validation tests ensuring both implementations agree

* **Technical Approach:**
  - Single pattern: `[.!?]\s+[A-Z]` for basic sentence boundaries
  - Use `dense::DFA::new()` with simple pattern compilation
  - Stream processing: `dfa.find_earliest_fwd()` over input bytes
  - Keep same interface as manual detector for easy comparison

* **References:**
  - regex-automata dense DFA documentation and PatternID examples
  - PRD F-3: deterministic sentence boundary detection (satisfied by DFA)
  - Abbreviation handling without variable-length lookbehind
  - Memory mapping for O(n) streaming without heap allocation

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [ ] Documentation updated if needed
- [ ] Clippy warnings addressed