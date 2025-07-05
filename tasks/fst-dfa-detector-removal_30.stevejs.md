# FST and DFA Detector Removal

* **Task ID:** fst-dfa-detector-removal_30.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - FST and DFA detectors are secondary algorithms that create maintenance burden
  - Dialog detector is the primary algorithm (407M chars/sec) and should be the single focus
  - Removing secondary detectors simplifies the codebase and reduces complexity
  - Current module exports include unused detector implementations
  - Build warnings indicate dead code in FST and DFA modules

* **Acceptance Criteria:**
  1. **FST Detector Removal**: Remove FST detector implementation and all references
  2. **DFA Detector Removal**: Remove DFA detector implementation and all references
  3. **Export Cleanup**: Update module exports to only include Dialog detector
  4. **Test Cleanup**: Remove or update tests that reference removed detectors
  5. **Warning-Free Build**: All compiler warnings related to dead code resolved
  6. **Documentation Update**: Update documentation to reflect Dialog-only architecture

* **Deliverables:**
  - **Removed FST Module**: Delete `src/sentence_detector/fst_detector.rs`
  - **Removed DFA Module**: Delete `src/sentence_detector/dfa_detector.rs`
  - **Updated Module Exports**: Clean `src/sentence_detector/mod.rs` to only export Dialog detector
  - **Updated Main Interface**: Remove `SentenceDetector` and `SentenceDetectorDFA` from public API
  - **Cleaned Tests**: Remove tests for deleted detectors
  - **Updated Documentation**: Reflect Dialog-focused architecture in docs

* **Implementation Plan:**
  1. **Remove FST Implementation**: Delete FST detector module and all references
  2. **Remove DFA Implementation**: Delete DFA detector module and all references
  3. **Update Exports**: Clean up module exports to only include Dialog detector
  4. **Update Tests**: Remove or update tests that reference removed detectors
  5. **Update Documentation**: Reflect simplified architecture in documentation

* **References:**
  - Dialog detector implementation in `src/sentence_detector/dialog_detector.rs`
  - Current module structure in `src/sentence_detector/mod.rs`
  - Test files that may reference removed detectors

## Pre-commit checklist:
- [ ] FST detector module deleted
- [ ] DFA detector module deleted
- [ ] Module exports updated to only include Dialog detector
- [ ] All references to removed detectors cleaned up
- [ ] Tests updated or removed for deleted detectors
- [ ] All compiler warnings resolved
- [ ] Documentation updated to reflect Dialog-only architecture
- [ ] Public API simplified to only include Dialog detector