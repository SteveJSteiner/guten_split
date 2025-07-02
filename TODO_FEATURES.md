# Feature & Process Improvement Backlog

## Active Features

### High Priority
- **Feature**: Integration test framework implementation
- **Type**: Process-Feature (builds development infrastructure)
- **Effort**: Medium
- **Prerequisites**: Testing strategy document (completed)
- **Context**: Implements the integration testing strategy defined in docs/testing-strategy.md
- **Acceptance**: Complete discovery→reader→sentence_detector pipeline tests within 2min budget

### Medium Priority
- **Feature**: Sentence detection throughput benchmarks
- **Type**: Process-Feature (performance validation)
- **Effort**: Small
- **Prerequisites**: Current benchmark suite
- **Context**: Address performance testing gaps identified in testing strategy
- **Acceptance**: Benchmarks for FST performance on various text complexity levels

- **Feature**: Memory usage profiling integration
- **Type**: Process-Feature (performance validation)  
- **Effort**: Medium
- **Prerequisites**: Integration test framework
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

## Completed & Archived

- Process improvements and testing strategy (process-improvements-testing_5.md) - COMPLETED
- PRD compliance validation framework - COMPLETED  
- Context management guidelines - COMPLETED

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