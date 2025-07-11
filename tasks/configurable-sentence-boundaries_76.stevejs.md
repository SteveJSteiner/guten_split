# Configurable Sentence Boundary Detection

* **Task ID:** configurable-sentence-boundaries_76.stevejs
* **Reviewer:** stevejs  
* **Area:** code
* **Motivation (WHY):**
  - Enable customization of sentence detection rules for different literary corpora
  - Support academic texts, historical texts, and non-dialog literary works
  - Make "seam" regex configurable while maintaining performance
  - Allow users to define custom boundary patterns without code changes
* **Acceptance Criteria:**
  1. Design TOML configuration format for sentence boundary rules
  2. Add `--seam-config` CLI flag to specify configuration file
  3. Maintain current dialog-aware detection as default behavior
  4. Validate configuration at startup with clear error messages
  5. Ensure custom regexes maintain performance characteristics
  6. Configuration format should be intuitive for non-programmers
* **Deliverables:**
  - TOML configuration schema design document
  - Configuration parsing and validation logic
  - Updated sentence detector to accept configurable rules
  - Example configuration files for different use cases
  - Integration tests with custom configurations
  - Performance benchmarks with custom regexes
* **References:**
  - Current regex-automata based sentence detector
  - Discussion of academic vs. literary text requirements
  - Performance requirement of >1GB/sec throughput

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated
- [ ] Documentation updated if needed
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely