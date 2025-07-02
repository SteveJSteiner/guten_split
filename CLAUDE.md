Claude Workflow Guide

Scope → This guide applies to the Gutenberg Sentence Extractor repository and complements the main PRD. It explains how Claude (or any LLM automation) should propose, document, and commit granular development steps while turning the repo into a Rust‑learning resource.

⸻

1 Philosophy
	•	One thought → one task → one validated commit.  Each atomic improvement must compile, be covered by tests, and land as its own git commit.
	•	Explain why, not just how.  Inline comments focus on rationale—trade‑offs, Rust idioms, async pitfalls—assuming a reader familiar with systems concepts but new to Rust.
	•	Rolling window of ten tasks.  Only the ten most‑recent task files stay in /tasks; older ones are deleted in the same commit that adds a new task (history preserved in git).
	•	Human in the loop.  Claude drafts tasks; reviewer stevejs approves, amends, or rejects them before merge.

⸻

2 Task Lifecycle

flowchart LR
    idea["Next engineering step"] -->|draft| claude[Claude writes task file]
    claude --> pr[Pull Request + commit]
    pr --> stevejs[Review by stevejs]
    stevejs -->|approve| main

2.1 When drafting a task Claude must:
	1.	Ensure atomic scope—it must pass tests in isolation.
	2.	Delete the oldest file in /tasks when there are already ten present.
	3.	**3. Name the new task file <semantic-name>_<index>.md, where <semantic-name> is a concise, kebab-case description and <index> is a monotonically increasing integer (1, 2, 3, …) regardless of date.
	4.	Commit message should begin with feat: / fix: / docs: etc., include a brief summary, and end with (see tasks/<file>).

⸻

3 Task File Schema

Each file lives under /tasks/ and follows this template:

# <Task Title>

* **Task ID:** <same as filename>
* **Reviewer:** stevejs
* **Area:** <code|docs|tests|build>
* **Motivation (WHY):**
  - <bullet‑point rationale>
* **Acceptance Criteria:**
  1. <unit tests|integration tests pass>
  2. <behavioural description>
* **Deliverables:**
  - <list of rust files / modules / docs>
* **References:**
  - PRD sections, related tasks, docs links

(Claude auto‑fills the template when generating the task.)

⸻

4 Coding Conventions

Topic	Guideline	WHY
Edition	Rust 2021	Widest stable baseline; async‑await mature.
Async runtime	Tokio 1.x	Matches PRD; ecosystem standard; IOCP/epoll under hood.
File IO	tokio::fs & optional memmap2	Non‑blocking default; mmap for benchmark parity.
FST	fst crate	Guarantees linear‑time lookup; zero‑alloc reads.
Progress	indicatif multi‑progress bars	Ergonomic, cross‑platform; keeps users informed during long scans.
Lock file	Commit Cargo.lock	Reproducible builds and tutorial consistency.
Docs folder	/docs/ for high‑level explainers	Separates narrative docs from code comments.
Tests	cargo test in CI	Ensures every task is validated.

Inline WHY comments use this style:

// WHY: using BufWriter lowers syscalls and boosts throughput on network filesystems


⸻

5 Next‑Step Selection Heuristics

Claude chooses the next task by scanning open tasks & git diff:
	1.	Red state first.  Fix failing tests or clippy warnings before new features.
	2.	Shortest critical path.  Prioritise tasks unblocking others (e.g., FST loader before progress bars).
	3.	Knowledge debt.  Add docs when unfamiliar Rust design is introduced.

⸻

6 Example Commit Sequence

Commit	Task	Sample Message
d4e5b3c	fst-loader_1	feat: load sentence FST at startup (see tasks/fst-loader_1.md)
c3a9fed	async-reader_2	feat: async buffered read for source files (see tasks/async-reader_2.md)
…	…	…

⸻

This document is version‑controlled. Amend via PRs when conventions evolve.