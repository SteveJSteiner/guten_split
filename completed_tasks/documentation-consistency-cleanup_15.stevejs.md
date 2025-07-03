# Documentation Consistency Cleanup

* **Task ID:** documentation-consistency-cleanup_15.stevejs
* **Reviewer:** stevejs
* **Area:** docs
* **Motivation (WHY):**
  - PRD.md contains outdated FST references when we implemented superior DFA (regex-automata) achieving 149.8 MiB/s
  - TODO_FEATURES.md duplicates completion tracking that's already in /completed_tasks/ directory
  - CLAUDE.md tech stack table references obsolete "fst crate" instead of actual regex-automata implementation
  - Inconsistencies between documented requirements and actual implementation create confusion
  - Documentation should reflect the reality of our high-performance DFA implementation

* **Acceptance Criteria:**
  1. PRD.md updated to reflect DFA implementation instead of outdated FST references
  2. TODO_FEATURES.md "Completed & Archived" section removed (redundant with /completed_tasks/)
  3. CLAUDE.md tech stack table updated with correct regex-automata dependency
  4. All prerequisite references updated to reflect completed task 9 status
  5. Documentation consistency validated across all three files
  6. No functional changes to code, purely documentation updates

* **Specific Inconsistencies to Fix:**

## PRD.md Updates:
```
Line 5: "finite-state transducer (FST)" → "sentence boundary detector (DFA)"
Line 10: "FST" definition → "DFA" definition  
Line 36: "compile sentence-boundary spec into an immutable FST" → "compile patterns into high-performance DFA"
Line 38: "Detect sentence boundaries with the FST" → "Detect sentence boundaries with the DFA"
Line 49: "sentence FST via the fst crate" → "sentence DFA via regex-automata crate"
Line 72: "FST boundary detection" → "DFA boundary detection"
Line 82: "FST generation from spec" → "DFA pattern compilation"
```

## TODO_FEATURES.md Cleanup:
- **Remove entire "Completed & Archived" section** (lines 96-102)
- **Reason**: /completed_tasks/ directory is single source of truth
- **Update prerequisite references**: "Basic DFA implementation (dfa-implementation-comparison_9)" → "High-performance DFA implementation (completed)"

## CLAUDE.md Tech Stack Update:
```
Line 119: "FST fst crate Guarantees linear-time lookup; zero-alloc reads." 
→ "Sentence Detection regex-automata crate High-performance DFA with 149.8 MiB/s throughput."
```

* **Implementation Reality Context:**
  - **Task 9 Results**: DFA implementation achieved 149.8 MiB/s (49-72% faster than manual)
  - **Tech Stack**: regex-automata with dense DFA, not fst crate
  - **Performance**: Exceeds PRD requirements (≥10 MB/s) by 15x
  - **Status**: Production-ready DFA replaces theoretical FST approach

* **Deliverables:**
  - Updated PRD.md with DFA terminology and regex-automata tech stack
  - Cleaned TODO_FEATURES.md without redundant completion tracking
  - Updated CLAUDE.md coding conventions table
  - Consistency validation across all documentation files
  - Git diff showing only documentation changes, no code modifications

* **Validation Steps:**
  1. Search for remaining "FST" references: `rg "FST|fst crate" *.md`
  2. Verify /completed_tasks/ contains all completion records
  3. Check prerequisite references point to completed tasks
  4. Ensure DFA performance claims match task 9 results (149.8 MiB/s)

* **References:**
  - Task 9 (dfa-implementation-comparison_9.md): 149.8 MiB/s DFA implementation
  - Current PRD.md lines 5, 10, 36, 38, 49, 72, 82
  - TODO_FEATURES.md lines 96-102 (removal target)
  - CLAUDE.md line 119 (tech stack correction)

## Pre-commit checklist:
- [x] PRD.md updated with DFA terminology replacing FST references
- [x] TODO_FEATURES.md "Completed & Archived" section removed
- [x] CLAUDE.md tech stack table corrected to regex-automata
- [x] GEMINI.md tech stack table corrected to regex-automata
- [x] All prerequisite references updated for completed task 9
- [x] Documentation consistency validated with rg searches
- [x] No code changes, only documentation updates
- [x] Performance claims match actual task 9 results (149.8 MiB/s)