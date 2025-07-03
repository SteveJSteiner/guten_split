# Exploration Workflow

## Purpose

Define the process for research and investigation tasks that require experimentation, prototyping, and iterative discovery before implementation decisions can be made.

## Key Principles

1. **Short iterations**: Commit frequently with tiny changes, even incomplete work
2. **Branch isolation**: All exploration work stays off main branch during investigation
3. **Decision focus**: Results emphasize the decision made, not exhaustive documentation
4. **Git-native transport**: Use cherry-pick to transfer final results to main branch
5. **Preserve investigation trail**: Keep exploration branches forever for full context

## Workflow Steps

### 1. Start Exploration
- Create exploration task in `/tasks/` using exploration template
- Start `explore/<task-id>` branch from main
- Task describes decision to be made, not predetermined outcome

### 2. Explore & Iterate
- Work directly in source tree on exploration branch
- Commit frequently: `explore: <summary> (see tasks/<task-file>)`
- Modify task file as exploration evolves
- Keep iterations short and focused

### 3. Complete Exploration
- Create final `<task-id>_results.md` with decision/recommendation
- Commit ONLY the results file: `explore: final results for <task-id>`
- Update task file to reflect what was actually done

### 4. Transfer Results (Automated)
- `git finalize <final-commit-hash>` (single command handles everything)
- Alias automatically:
  - Switches to main branch
  - Cherry-picks results commit to main
  - Creates immutable tag `refs/tags/explore/<task-id>`
  - Pushes tag to origin
  - Deletes remote exploration branch
- Manually move task file to `/tasks/completed/` with branch reference

## Branch Management

- **Branch naming**: `explore/<task-id>` (e.g., `explore/abbreviation-detection-exploration_14`)
- **Branch lifecycle**: Create → work → finalize → delete (preserved as immutable tag)
- **Preservation**: Branch content preserved in `refs/tags/explore/<task-id>` tag
- **Linear history**: Cherry-pick commit becomes part of main's linear history
- **Remote cleanup**: `git finalize` deletes remote branch automatically

## Commit Messages

- **Exploration work**: `explore: <summary> (see tasks/<task-file>)`
- **Final results**: `explore: final results for <task-id>`
- **Cherry-pick**: Git automatically formats, becomes part of main history

## File Organization

- **Exploration branch**: Work anywhere in source tree as needed
- **Main branch**: No exploration artifacts except final results
- **Completed tasks**: `/tasks/completed/` contains task file + results file

## Setup (One-time)

After cloning the repository, run:
```bash
./scripts/bootstrap.sh
```

This automatically configures:
- `git finalize` alias for exploration workflows
- Hidden exploration tags (refs/tags/explore/*)
- Repository-specific git settings

### Prerequisites
- Branch naming must follow `explore/<task-id>` pattern
- If your push remote isn't `origin`, set `GIT_REMOTE` in your environment

## Integration with Implementation

- **Task evolution**: Task file changes naturally occur in exploration branch
- **Final state**: Results file contains final context, tag preserves full history
- **Follow-up explorations**: Create new task in main branch if needed
- **Reviewer involvement**: Reviewer participates in human-led exploration process
- **Inconclusive results**: Document failure/abandonment in results file
- **Implementation tasks**: Reference exploration results via tag: `refs/tags/explore/<task-id>`
- **Context management**: Use standard process within exploration branch

## Accessing Historical Explorations

- **List exploration tags**: `git tag -l 'explore/*'`
- **View exploration content**: `git show refs/tags/explore/<task-id>`
- **Checkout exploration state**: `git checkout refs/tags/explore/<task-id>`

## Templates

### Exploration Task Template
```markdown
# <Exploration Title>

* **Task ID:** <task-id>
* **Reviewer:** stevejs
* **Area:** exploration
* **Decision Question:** 
  - What specific decision needs to be made?
* **Hypothesis/Approaches:**
  - List of approaches to investigate
* **Success Criteria:**
  - How will we know when the exploration is complete?
* **Constraints:**
  - Time, scope, or resource limitations

## Exploration Areas:
- Specific areas to investigate
- Questions to answer
- Experiments to conduct

## Expected Deliverables:
- Research findings
- Prototype implementations
- Performance data
- Decision recommendation

## Pre-completion checklist:
- [ ] All investigation areas explored
- [ ] Decision criteria evaluated
- [ ] Results documented for future reference
- [ ] Recommendation made with rationale
```

### Results File Template
```markdown
# <Task ID> Results

## Decision Made
- Clear statement of the decision/recommendation

## Rationale
- Why this decision was made
- Key factors that influenced the choice

## Key Findings
- Important discoveries during exploration
- Data that supports the decision

## Implementation Implications
- What this means for future implementation
- Next steps or follow-up tasks needed

## References
- Tag: refs/tags/explore/<task-id>
- Related tasks or documentation
```