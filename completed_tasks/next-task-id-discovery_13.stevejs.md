# Simplify Next Task ID Discovery with Multi-User Support

* **Task ID:** next-task-id-discovery_13
* **Reviewer:** stevejs
* **Area:** process
* **Motivation (WHY):**
  - With new /tasks/ and /completed_tasks/ peer directory structure, determining next task number requires manual scanning of both directories
  - Current process is error-prone and inefficient - need to check highest number across both directories
  - Frequent task creation makes this a repeated workflow bottleneck
  - Need to support multiple collaborators without ID conflicts
  - Simple automation can eliminate this friction and prevent ID conflicts

* **Acceptance Criteria:**
  1. Next task ID can be determined instantly by reading a single file
  2. File automatically maintained as part of task creation workflow
  3. Works reliably with multiple users and prevents ID conflicts
  4. Integrated into CLAUDE.md and GEMINI.md workflow documentation
  5. Maintains monotonic ordering across all users
  6. Simple to use and maintain

* **Proposed Solution: .next_id File + Username Suffix**

## New Task Naming Convention:
`<semantic-name>_<next_id>.<username>.md`

Examples:
- `sentence-detection_14.stevejs.md`
- `performance-optimization_15.claude.md` 
- `bug-fix_16.alice.md`

## Workflow with `/tasks/.next_id`:
Create `/tasks/.next_id` containing just the next task number:
```
17
```

**Task Creation Workflow:**
1. Read next ID: `next_id=$(cat tasks/.next_id)`
2. Create task: `sentence-detection_${next_id}.$(whoami).md`
3. Increment counter: `echo $((next_id + 1)) > tasks/.next_id`

**Multi-User Benefits:**
- ✅ No ID collisions (each user gets unique filename)
- ✅ Monotonic ordering (IDs always increase)
- ✅ Clear ownership (username in filename)
- ✅ Partial ordering preserved (can sort by ID then user)
- ✅ Git-friendly (merge conflicts unlikely)

## Implementation Changes:

**File Operations:**
- Current: `task-name_13.md` → New: `task-name_13.stevejs.md`
- Commit messages: `(see tasks/task-name_13.stevejs.md)`

**Directory Structure:**
```
/tasks/
  .next_id                           # Contains: 17
  sentence-detection_14.stevejs.md
  performance-test_15.claude.md
  
/completed_tasks/  
  project-setup_1.stevejs.md
  file-discovery_2.stevejs.md
  ...
  task-structure-reorganization_12.stevejs.md
```

* **Deliverables:**
  - Create `/tasks/.next_id` file with current next ID
  - Update CLAUDE.md section 2.1 with new naming convention and workflow
  - Update GEMINI.md section 2.1 with new naming convention and workflow
  - Rename this task file to follow new convention when completed
  - Update commit message format in documentation

* **Technical Approach:**
  1. Determine current highest task ID from both directories (scan all existing files)
  2. Create `tasks/.next_id` with next number
  3. Update workflow documentation in both guide files
  4. Add username suffix to task naming convention
  5. Update commit message format examples

* **Migration for Existing Files:**
  - Existing files keep current names (grandfather clause)
  - New files use new convention going forward
  - Documentation reflects both patterns during transition

* **References:**
  - CLAUDE.md section 2.1 (task naming guidelines)
  - GEMINI.md section 2.1 (task naming guidelines)
  - Task structure reorganization (task-structure-reorganization_12.md)
  - TODO_FEATURES.md process improvement entry

## Pre-commit checklist:
- [ ] `/tasks/.next_id` file created with correct next ID
- [ ] CLAUDE.md updated with new naming convention and .next_id workflow
- [ ] GEMINI.md updated with new naming convention and .next_id workflow
- [ ] This task file renamed to new convention when moved to completed_tasks
- [ ] Commit message format updated in documentation
- [ ] Workflow tested and validated