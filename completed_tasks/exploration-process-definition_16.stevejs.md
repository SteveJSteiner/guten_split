# Exploration Process Definition

* **Task ID:** exploration-process-definition_16.stevejs
* **Reviewer:** stevejs
* **Area:** docs
* **Motivation (WHY):**
  - Current task lifecycle handles feat: and docs: commits but lacks process for research/exploration work
  - Need systematic approach to isolate experimental work from production code
  - Exploration tasks like abbreviation detection need concrete deliverables beyond just "research"
  - Want to preserve exploration findings without cluttering production codebase
  - Establish commit conventions and file organization for investigation work

* **Acceptance Criteria:**
  1. Define exploration task template with concrete deliverable types
  2. Specify file organization strategy to isolate experimental code
  3. Create commit message convention for exploration work (explore: prefix?)
  4. Document how exploration findings transition to implementation tasks
  5. Update CLAUDE.md with exploration process integration
  6. Design system for preserving exploration artifacts without production impact

* **Exploration Process Elements to Define:**

## Git Branch Strategy:
- **Branch naming**: `explore/<task-id>` (e.g., `explore/abbreviation-detection-exploration_14`)
- **Frequent commits**: SHORT, TINY iterations checked in immediately
- **Isolation**: Exploration work stays OFF main branch during investigation
- **Preservation**: Branch preserved after completion for artifact record
- **Merge strategy**: Only final results/recommendations merge to main

## Commit Message Convention:
- `explore: <summary>` for exploration commits
- Reference exploration task in commit messages: `(see tasks/<task-file>)`
- Preserve all exploration commits (no squashing) for investigation trail
- Final summary commit when transitioning findings to implementation

## File Organization Strategy:
- Work directly in source tree on exploration branch
- `/explorations/` directory for preserved artifacts that merge to main
- Separate from `/tests/` to avoid cargo test discovery during exploration
- Archive completed exploration branches (don't delete)


## Integration with Main Process:

### Preferred Workflow:
1. **Exploration branch**: `explore/<task-id>` - all work stays there forever
2. **Main branch**: Task file moves to `/tasks/completed/` with branch pointer + summary  
3. **No explorations/ in main**: Results stay in exploration branch only
4. **Side-band communication**: Convey summary/decision outside main repo

### Side-band Results Transfer Mechanism:

**Selected: Cherry-pick Transport**
- **Final commit requirement**: Last commit in exploration branch must be ONLY the results summary file
- **Transfer mechanism**: `git cherry-pick <final-commit-hash>` from exploration branch to main
- **File location**: Results file goes to `/tasks/completed/<task-id>_results.md`
- **Task update**: Original task file updated with branch reference and moved to `/tasks/completed/`

**Process Steps:**
1. **Exploration work**: All experiments/iterations on `explore/<task-id>` branch
2. **Results compilation**: Create final `<task-id>_results.md` with summary/decision/recommendations
3. **Final commit**: Commit ONLY the results file (no other changes) 
4. **Switch to main**: `git checkout main`
5. **Cherry-pick**: `git cherry-pick <final-commit-hash>` brings results file to main
6. **Task completion**: Update original task file with branch reference, move to completed

**Benefits:**
- Git-native transport mechanism
- Clean separation of exploration vs results
- Preserves authorship and commit metadata
- No custom scripts required
- Results stay in main branch permanently

## Isolation Guarantees:
- Ensure exploration code doesn't affect production builds
- Separate from main test suite but still runnable
- Clear boundaries between exploration and production code
- Documentation of what's exploration vs implementation

* **Deliverables:**
  - **docs/exploration-workflow.md** with complete process documentation ✓
  - **Exploration task template** with fields adapted for research work
  - **Results file template** focused on decision/recommendation
  - **Updated CLAUDE.md** with pointer to exploration workflow
  - **Small test exploration** to validate process before abbreviation detection

* **Technical Approach:**
  - **Review existing task patterns** in feat: and docs: commits
  - **Design exploration directory structure** with clear isolation
  - **Create template matching current task schema** but adapted for research
  - **Document integration points** with existing workflow
  - **Validate against abbreviation exploration requirements** as concrete example

* **Expected Outcomes:**
  - **Clear process boundaries** between exploration and implementation
  - **Systematic artifact preservation** for future reference
  - **Reduced production code clutter** from experimental work
  - **Streamlined transition** from research to implementation
  - **Consistent exploration documentation** across all investigation tasks

* **References:**
  - CLAUDE.md Section 2: Task Lifecycle (extend with exploration)
  - abbreviation-detection-exploration_14.stevejs.md: Concrete example to validate process
  - Existing feat: and docs: commit patterns for consistency
  - TODO_FEATURES.md: Multiple exploration needs identified

## Pre-commit checklist:
- [ ] Exploration task template created with concrete deliverable types
- [ ] File organization strategy documented with isolation guarantees
- [ ] Commit message convention defined for exploration work
- [ ] CLAUDE.md updated with exploration process integration
- [ ] Transition process documented from exploration to implementation
- [ ] Process validated against abbreviation exploration requirements
- [ ] No changes to existing production workflow (additive only)