# Reorganize Task Structure for Better Workflow (Process Improvement)

* **Task ID:** task-structure-reorganization_12
* **Reviewer:** stevejs  
* **Area:** process
* **Motivation (WHY):**
  - Current `/tasks/completed/` subdirectory mixes active and completed tasks in Claude Code auto-complete
  - Determining next task number requires scanning both active and completed directories
  - File navigation becomes cumbersome with nested structure
  - Better separation improves workflow efficiency and reduces cognitive overhead

* **Acceptance Criteria:**
  1. Completed tasks moved to peer directory structure (not subdirectory)
  2. Next task number easily determinable from single source of truth
  3. Claude Code auto-complete cleanly separates active vs completed tasks
  4. All existing task references updated to new structure
  5. Documentation updated to reflect new workflow

* **Proposed Structure:**
```
/tasks/                    # Active tasks only
  task-name_13.md
  another-task_14.md
  
/completed_tasks/          # Completed tasks only (peer directory)
  project-setup_1.md
  file-discovery_2.md
  async-file-reader_3.md
  ...
  incremental-processing-implementation_11.md
  task-structure-reorganization_12.md  # This task when complete
```

* **Deliverables:**
  - Create `/completed_tasks/` directory as peer to `/tasks/`
  - Move all files from `/tasks/completed/` to `/completed_tasks/`
  - Remove `/tasks/completed/` subdirectory
  - Update CLAUDE.md workflow documentation to reflect new structure
  - Update any existing task references or documentation
  - Create simple script or guideline for determining next task number

* **Next Task Number Strategy:**
  - **Option A**: Simple incrementing counter in CLAUDE.md or README
  - **Option B**: Script that scans both directories and reports next number
  - **Option C**: Convention: always check highest number in both directories

* **Technical Approach:**
  1. Create `/completed_tasks/` directory
  2. Move all completed tasks from `/tasks/completed/` to `/completed_tasks/`
  3. Update CLAUDE.md section 2.1 task naming guidelines
  4. Update CLAUDE.md section 3 task lifecycle documentation
  5. Add clear next-task-number determination process
  6. Test workflow with Claude Code auto-complete

* **Migration Commands:**
```bash
# Create new structure
mkdir completed_tasks

# Move all completed tasks
mv tasks/completed/* completed_tasks/

# Remove old subdirectory
rmdir tasks/completed

# Update any references in documentation
```

* **Benefits:**
  - **Cleaner auto-complete**: `/tasks/` shows only active work
  - **Faster navigation**: No need to drill into subdirectories
  - **Simpler task numbering**: Clear convention for next task ID
  - **Better mental model**: Active vs completed as peer concepts
  - **Reduced cognitive load**: Less directory traversal during development

* **References:**
  - Current CLAUDE.md section 2.1 (task naming)
  - Current CLAUDE.md section 3 (task lifecycle)
  - Claude Code auto-complete behavior patterns

## Pre-commit checklist:
- [ ] All completed tasks moved to `/completed_tasks/`
- [ ] `/tasks/completed/` directory removed
- [ ] CLAUDE.md updated with new workflow
- [ ] Next task number determination process documented
- [ ] All file references updated
- [ ] Test auto-complete behavior works as expected