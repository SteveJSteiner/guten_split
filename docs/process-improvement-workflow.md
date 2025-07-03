# Process Improvement Workflow

This document covers how to handle process improvements (like the work we just completed) - changes to development workflow, testing strategy, documentation systems, and tooling.

## Process Work Session Planning

### Scope Definition
Process improvements should be scoped as complete, self-contained changes:
- **Good scope**: "Optimize unit testing strategy for 10-second budget"
- **Bad scope**: "Improve testing and also add some benchmarks and maybe update docs"

### Session Structure
1. **Problem identification** - What specific process friction are we addressing?
2. **Solution design** - What concrete changes will solve it?
3. **Implementation** - Update docs, templates, workflows
4. **Validation** - Test the new process (often by applying it immediately)

### Context Management for Process Work
Process work has different context dynamics than feature work:
- **High up-front context cost** - Understanding existing process
- **Lower incremental cost** - Making systematic changes
- **Front-load analysis** - Examine current state thoroughly before changing

**Process Work Context Rules:**
- Start with comprehensive analysis of current state
- Make all related changes in one session (avoid partial process updates)
- Document rationale extensively (process changes affect future sessions)
- Test new process immediately if possible

## Process Work Task Template

Use this template for process improvement tasks:

```markdown
# <Process Improvement Title>

* **Task ID:** <process-name>_<index>
* **Reviewer:** stevejs  
* **Area:** process
* **Problem Statement:**
  - What specific friction/inefficiency are we addressing?
  - What breaks or fails with current process?
* **Solution Design:**
  - Concrete changes to workflow/docs/templates
  - How will this solve the identified problems?
* **Acceptance Criteria:**
  1. Process documentation updated
  2. Templates/examples updated if applicable
  3. New process validated (tested immediately if possible)
* **Deliverables:**
  - Updated CLAUDE.md sections
  - New/updated docs in /docs/
  - Template changes
* **Validation Plan:**
  - How will we test that the new process works?
  - What would indicate success/failure?

## Pre-commit checklist:
- [ ] All process documentation updated
- [ ] Examples and templates reflect changes  
- [ ] Changes are internally consistent
- [ ] New process tested/validated if possible
- [ ] Related documents cross-reference correctly
```

## Process Change Validation

### Testing New Processes
- **Immediate application** - Use the new process right away if possible
- **Consistency checks** - Ensure all related docs align with changes
- **Edge case consideration** - What could break with the new process?

### Validation Methods
1. **Self-application** - Use new process in same session that creates it
2. **Cross-reference verification** - Check that all related docs are consistent
3. **Edge case analysis** - Identify what could go wrong
4. **Rollback planning** - How would we revert if it doesn't work?

## Process Change Deployment

### Documentation Updates
- Update CLAUDE.md for workflow changes
- Update /docs/ for detailed guidance  
- Update task templates for new requirements
- Ensure cross-references are correct
- Move completed tasks to /tasks/completed/ during completion commits

### Change Communication
- Process changes should be committed with clear rationale
- Include examples of how new process works
- Document what problems the change solves

### Rollback Strategy
- Keep old process documented until new one is validated
- Use git history to preserve previous process versions
- Have clear success/failure criteria for new processes

## Common Process Work Patterns

### Documentation Restructuring
- Move detailed guidance out of CLAUDE.md to preserve space budget
- Maintain clear cross-references between documents
- Keep CLAUDE.md focused on workflow essentials

### Template Updates  
- Update task templates immediately when process changes
- Include concrete examples in templates
- Test templates by using them in the same session

### Workflow Optimization
- Address specific friction points with concrete solutions
- Integrate new requirements into existing workflows rather than adding overhead
- Validate that optimizations actually reduce friction

## Context Boundaries for Process Work

### When to Ask for Continuation
- **Heavy context + major changes remaining** - Ask before continuing with large changes
- **Multiple unrelated process areas** - Don't mix testing strategy with git workflow changes
- **Validation requires significant work** - If testing new process would consume major context

### Prime Directive
**When in doubt about process changes, ASK.** Process changes affect all future work, so getting them right matters more than completing them quickly.

### Process Work Session Limits
- Focus on one process area per session (testing, workflow, documentation, etc.)
- Complete all related changes in one session rather than partial updates
- Ask before mixing process improvements with feature development