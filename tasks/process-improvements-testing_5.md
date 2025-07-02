# Process Improvements and Testing Strategy

* **Task ID:** process-improvements-testing_5
* **Reviewer:** stevejs
* **Area:** docs|tests
* **Motivation (WHY):**
  - Current testing strategy relies on unit tests only; e2e validation is manual and ad-hoc
  - Need systematic approach to validate complete workflows from file discovery through sentence extraction
  - Integration tests would catch issues between modules that unit tests miss
  - Automated e2e tests would prevent regressions and enable confident refactoring
  - Process improvements could streamline development workflow and reduce manual verification overhead
* **Acceptance Criteria:**
  1. Define PRD compliance validation framework to catch implementation drift
  2. Optimize unit test strategy to leverage full 10-second budget with comprehensive I/O testing
  3. Assess if existing perf tests are sufficient for validating performance claims
  4. Create context management guidelines for 50% limit and work chunking
  5. Define clear boundaries between feature work vs process work
  6. Establish `to_do` features system for capturing requirements without context overload
  7. Streamline task completion workflow to flow naturally into commits
* **Deliverables:**
  - PRD compliance check framework (automated detection of false performance/concurrency claims)
  - Unit test optimization strategy: maximize I/O test coverage within 10-second budget
  - Performance validation assessment: existing perf tests vs new framework needs
  - Context management rules and work chunking guidelines for 50% limit
  - Feature/Process work separation guidelines with concrete examples
  - `to_do` features system design for requirement capture without context bloat
  - Improved task-to-commit workflow eliminating awkward transitions
* **References:**
  - `src/reader.rs:147` false concurrency example requiring detection
  - Current unit test performance baseline for 10-second budget optimization
  - Existing benchmark suite for performance validation assessment
  - CLAUDE.md atomic commit workflow for improvement opportunities
  - Context management challenges in current task completion flow