# Benchmark Implementation Audit

* **Task ID:** benchmark-implementation-audit_25.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Need to verify what code differences exist between DFA, Manual FST, and Dialog State Machine benchmarks
  - Ensure relative performance comparisons are valid by confirming similar work is being measured
  - Identify any implementation differences that could skew performance comparisons

* **Acceptance Criteria:**
  1. **Code Path Analysis**: Document exactly what code each benchmark executes
  2. **Work Comparison**: Verify all benchmarks do equivalent sentence detection work
  3. **Overhead Identification**: Identify any benchmark-specific overhead differences
  4. **Implementation Equivalence**: Confirm benchmarks measure comparable functionality

* **Analysis Targets:**
  - **DFA Benchmark**: `dfa_detector.detect_sentences()` - what does this call?
  - **Manual FST Benchmark**: `manual_detector.detect_sentences()` - what does this call?  
  - **Dialog State Machine**: `DialogStateMachine.detect_sentences()` - what does this call?

* **Key Files:**
  - `/Users/stevejs/guten_split/benches/sentence_detector_bench.rs` (benchmark functions)
  - `/Users/stevejs/guten_split/src/sentence_detector.rs` (DFA and Manual FST implementations)
  - `/Users/stevejs/guten_split/tests/dialog_state_machine_exploration.rs` (Dialog State Machine implementation)

* **Investigation Points:**
  1. **Input Processing**: Do all three process the same input text format?
  2. **Core Algorithm**: What's the actual sentence detection logic in each?
  3. **Output Generation**: What format conversion/processing happens in each?
  4. **Memory Allocation**: Any differences in string handling, vector allocation?
  5. **Position Calculation**: How does each compute line:col positions?

* **Deliverables:**
  - **Code Path Document**: Step-by-step breakdown of what each benchmark executes
  - **Work Equivalence Analysis**: Comparison of actual sentence detection work performed
  - **Overhead Analysis**: Identification of benchmark-specific overhead in each approach
  - **Performance Validity Assessment**: Whether relative performance comparisons are meaningful

* **Method:**
  1. **Trace DFA benchmark**: Follow `SentenceDetectorDFA::detect_sentences()` implementation
  2. **Trace Manual FST benchmark**: Follow `SentenceDetector::detect_sentences()` implementation
  3. **Trace Dialog State Machine**: Follow `DialogStateMachine::detect_sentences()` implementation
  4. **Compare work performed**: Identify differences in actual computation vs overhead
  5. **Document findings**: Clear breakdown of what each benchmark measures

## Pre-commit checklist:
- [ ] All three benchmark code paths fully documented
- [ ] Equivalent work vs overhead clearly identified for each benchmark
- [ ] Performance comparison validity assessed
- [ ] Any implementation differences that affect benchmarking identified