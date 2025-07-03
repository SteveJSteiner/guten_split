# Feature & Process Improvement Backlog

## Active Features

### High Priority
- **Feature**: Complete sentence boundary rules implementation
- **Type**: Core Feature
- **Effort**: Large
- **Prerequisites**: Integration test framework (completed)
- **Context**: Implement complete rules that handle abbreviations (Dr., Mr., U.S.A.) correctly vs current simple rules
- **Acceptance**: All punctuation complete rules tests pass, abbreviations not split incorrectly

### Medium Priority
- **Feature**: Complex multi-line text integration tests
- **Type**: Process-Feature (test coverage)
- **Effort**: Small
- **Prerequisites**: Integration test framework (completed)
- **Context**: Re-enable test_pipeline_complex_text with correct span calculations for line break normalization
- **Acceptance**: Complex text scenarios with line breaks validate correctly

- **Feature**: Performance integration tests with large datasets
- **Type**: Process-Feature (performance validation)
- **Effort**: Small
- **Prerequisites**: Integration test framework (completed)
- **Context**: Re-enable test_pipeline_performance with correct column position calculations
- **Acceptance**: 500+ sentence processing validates within performance budget

- **Feature**: Multi-pattern DFA with PatternID for narrative vs dialog boundaries
- **Type**: Core Feature
- **Effort**: Medium
- **Prerequisites**: High-performance DFA implementation (completed)
- **Context**: Extend single-pattern DFA to distinguish sentence types using PatternID
- **Acceptance**: Pattern 0 for narrative boundaries, Pattern 1 for dialog boundaries, identical output to manual implementation

- **Feature**: Memory-mapped file processing with streaming DFA
- **Type**: Core Feature
- **Effort**: Medium
- **Prerequisites**: High-performance DFA implementation (completed)
- **Context**: Add mmap support for large file processing with DFA streaming
- **Acceptance**: DFA processes memory-mapped files without loading into heap memory

- **Feature**: 3-char lookbehind abbreviation checking
- **Type**: Core Feature
- **Effort**: Small
- **Prerequisites**: High-performance DFA implementation (completed)
- **Context**: O(1) post-processing to handle abbreviations after DFA match
- **Acceptance**: Abbreviations like "Dr." correctly detected and skipped

- **Feature**: Sentence detection throughput benchmarks
- **Type**: Process-Feature (performance validation)
- **Effort**: Small
- **Prerequisites**: DFA implementation alongside manual implementation
- **Context**: Compare manual vs DFA performance on various text complexity levels
- **Acceptance**: Benchmarks show performance difference between manual and DFA approaches

- **Feature**: Memory usage profiling integration
- **Type**: Process-Feature (performance validation)  
- **Effort**: Medium
- **Prerequisites**: Integration test framework (completed)
- **Context**: Validate memory claims and detect leaks during large file processing
- **Acceptance**: Automated memory profiling in CI pipeline

### Low Priority
- **Feature**: Concurrent file processing implementation
- **Type**: Core Feature
- **Effort**: Large
- **Prerequisites**: Integration tests, memory profiling
- **Context**: Replace false concurrency in reader.rs:147 with actual concurrent processing
- **Acceptance**: Measurable performance improvement with concurrent benchmarks

## Active Process Improvements

### High Priority
- **Process**: Rename project before publishing
- **Type**: Process Improvement
- **Effort**: Small
- **Prerequisites**: None
- **Context**: Current name "guten_split" doesn't reflect sentence extraction purpose, needs better name before any publishing/sharing
- **Acceptance**: Project renamed with updated Cargo.toml, README references, and repository name

- **Process**: Task-to-commit automation tooling
- **Type**: Process Improvement
- **Effort**: Small
- **Prerequisites**: Current task template with checklist
- **Context**: Automate commit message generation from task metadata per testing-strategy.md
- **Acceptance**: Shell script that generates commit messages from task files

### Medium Priority
- **Process**: Claim validation automation
- **Type**: Process Improvement
- **Effort**: Medium
- **Prerequisites**: Current rg-based validation
- **Context**: Integrate claim validation into cargo test workflow
- **Acceptance**: `cargo test` automatically detects and reports unvalidated claims


## Usage Guidelines

### Adding New Items
1. Use this format for both features and process improvements
2. Be specific about effort (Small: 1-2 hours, Medium: half day, Large: full day+)
3. Identify prerequisites clearly to avoid dependency issues
4. Link to relevant context (PRD sections, existing tasks, documents)

### Prioritization Rules
- **High**: Blocks other work or addresses critical technical debt
- **Medium**: Improves development experience or addresses known issues  
- **Low**: Nice-to-have improvements or experimental features

### Maintenance
- Archive completed items to preserve decision history
- Review and re-prioritize monthly
- Keep active list under 10 items to avoid context bloat
- Reference from task files but don't duplicate details