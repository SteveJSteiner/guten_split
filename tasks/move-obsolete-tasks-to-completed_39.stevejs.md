# Move Obsolete Tasks to Completed

* **Task ID:** move-obsolete-tasks-to-completed_39.stevejs.md
* **Reviewer:** stevejs
* **Area:** docs
* **Motivation (WHY):**
  - Assessment found active tasks based on problems that no longer exist
  - abbreviation-detection-exploration_14.stevejs.md explores solved problems
  - Other tasks may be superseded by recent implementation work
  - Clean active task list needed for accurate current state

* **Acceptance Criteria:**
  1. Review each active task against current implementation reality
  2. Move obsolete/superseded tasks to completed_tasks/
  3. Update task files with completion notes explaining why moved
  4. Ensure remaining active tasks reflect genuine current needs
  5. Update .next_id to reflect cleaned task list

* **Deliverables:**
  - Obsolete tasks moved to completed_tasks/ with completion notes
  - Clean /tasks directory with only genuinely active work
  - Updated task completion documentation

* **References:**
  - comprehensive-implementation-reality-check_36.stevejs.md findings
  - Assessment FINDING 5.3: Active tasks based on outdated assumptions

## Tasks to Review:
- [ ] abbreviation-detection-exploration_14.stevejs.md (explores problem that's solved)
- [ ] dialog-state-machine-performance-optimization_18.stevejs.md (may be addressed by recent fixes)
- [ ] comprehensive-state-assessment-and-validation_23.stevejs.md (may be superseded by assessment)
- [ ] dialog-state-machine-over-coalescing-fix_19.stevejs.md (may be resolved by recent fixes)

## Pre-commit checklist:
- [ ] All active tasks reviewed against current implementation
- [ ] Obsolete tasks moved to completed_tasks/ with completion notes
- [ ] Remaining active tasks reflect genuine current needs
- [ ] Task directory cleaned and organized
- [ ] Next ID updated appropriately