Product Requirements Document (PRD)

1 Purpose

Provide a high-throughput CLI tool that scans a local mirror of Project Gutenberg, tokenises each “*-0.txt” UTF-8 text into sentences using a sentence-boundary detector built from a startup-generated sentence boundary detector (DFA), and writes an auxiliary file alongside each source containing normalised sentences and span metadata. The tool must support fully asynchronous I/O and be safe to re-run incrementally.

2 Glossary

Term	Definition
DFA	Deterministic Finite Automaton compiled at startup from sentence-boundary patterns.
Sentence normalisation	Removal of intra-sentence hard line breaks (\n, \r\n) with whitespace collapsed to a single space. Everything else remains byte-exact.
Aux file	<orig>_rs_sft_sentences.txt written next to its source.
Span	(start_line, start_col, end_line, end_col)—all one-based, columns measured in Unicode scalar values (characters).

3 Scope

3.1 In Scope
	•	Discover every file matching the glob **/*-0.txt under a user-supplied root.
	•	For each source file:
	•	Skip if a complete aux file already exists, unless --overwrite_all.
	•	Overwrite a partial aux file.
	•	Produce per-run statistics and aggregate them in run_stats.json.
	•	Provide a --fail_fast switch to abort on first error; otherwise continue and log.
	•	Offer two I/O back-ends: standard async buffered I/O and memory-mapped (mmap) comparison mode for benchmarking.

3.2 Out of Scope (Now)
	•	Distributed processing across hosts.
	•	Optimising the DFA beyond linear-time guarantees.
	•	Sentence boundary training—assume rule set is externally supplied.

4 Functional Requirements

ID	Requirement
F-1	CLI accepts: root_dir, --overwrite_all, --fail_fast, --use_mmap, --no_progress, --stats_out.
F-2	Recursively locate and stream-process every *-0.txt.
F-3	At startup, compile sentence-boundary patterns into a high-performance DFA stored in memory for all worker tasks.
F-4	Read each file with async buffered reader (Tokio) or memory-map (when --use_mmap).
F-5	Detect sentence boundaries with the DFA, producing: index<TAB>sentence<TAB>(start_line,start_col,end_line,end_col) per line.
F-6	Normalise sentences by removing hard line breaks; treat \r\n as single break; preserve all other bytes.
F-7	Write results via async buffered writer to <path>_rs_sft_sentences.txt.
F-8	Generate per-file stats (chars processed, sentences, wall-clock ms) and aggregate into run_stats.json with total chars/sec.
F-9	Skip processing when aux file exists and completes without truncation; detect partial files by trailing newline + EOF.
F-10	Respect --fail_fast: abort entire run on first I/O/UTF-8/DFA error.
F-11	Cache discovered file locations to avoid slow directory traversal on subsequent runs.

5 Non-Functional Requirements

Category	Requirement
Tech Stack	Rust (2021 edition) with the Tokio async runtime; sentence DFA via regex-automata crate; progress bars via indicatif; built with cargo --release.
Performance	≥ 10 MB/s sustained on NVMe SSD with async buffered I/O; ≤ 30 % single-core CPU when I/O-bound.
Scalability	Should saturate multiple cores using Tokio’s work-stealing scheduler.
Portability	Linux (x86-64, aarch64) and Windows 10+.
Reliability	Deterministic output; idempotent reruns.
Observability	Structured logs (JSONLines), console progress bars (on by default), and run_stats.json.

6 Error Handling
	•	Log all recoverable errors with context.
	•	For non---fail_fast runs, continue after logging, mark file as failed in stats.

7 CLI & Config

rs-sft-sentences <root_dir>
    [--overwrite_all]   # overwrite even complete aux files
    [--fail_fast]       # abort on first error
    [--use_mmap]        # use memory-mapped I/O instead of async buffered
    [--no_progress]     # suppress console progress bars
    [--stats_out <path>]# default: run_stats.json in CWD

Progress: uses the indicatif crate to render multi-progress bars: files processed, bytes/s, chars/s, ETA.

8 Acceptance Criteria
	1.	Unit tests covering DFA boundary detection and normalisation rules.
	2.	Golden-file tests on at least five Gutenberg texts; diff is empty.
	3.	Throughput benchmark meets performance NFR on reference hardware.
	4.	run_stats.json includes per-file counts plus aggregate chars/sec.
	5.	Re-run without --overwrite_all touches zero unchanged aux files.
	6.	--fail_fast causes exit code ≠ 0 after first injected UTF-8 error.

9 Milestones

Milestone	Deliverable	Target Date
M1	CLI skeleton; file discovery; async read/write
M2	DFA pattern compilation; integration tests
M3	Stats aggregation; mmap mode; perf bench; progress bars
M4	Docs, CI, release v0.1

10 Risks & Mitigations

Risk	Likelihood	Impact	Mitigation
DFA patterns change frequently	M	M	Hot-reload DFA per file batch when patterns change.
Very large files cause memory spikes	L	M	Stream processing and bounded buffers.
Windows newline variants mishandled	M	L	Normalisation unit tests on mixed CRLF data.

11 Open Questions
	1.	Which concrete DSL or regex subset will define the sentence-boundary spec?
	2.	Should stats include per-sentence length histograms?
	3.	Need coloured CLI progress bars or silent by default? (Current default = on.)
	4.	Future: move from DFA to PDA for cross-paragraph context?