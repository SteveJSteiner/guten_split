# Benchmark Data Transparency

* **Task ID:** benchmark-data-transparency_24.stevejs
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Throughput benchmarks report MiB/s but production needs characters/sec
  - Unknown which specific Project Gutenberg files are being benchmarked
  - Unknown total data size being processed for throughput calculations
  - Need concrete data transparency for benchmark validation

* **Acceptance Criteria:**
  1. **Character Count Reporting**: Benchmarks report characters/sec instead of MiB/s
  2. **File List Documentation**: Clear list of which specific files are processed
  3. **Data Size Transparency**: Total character count and file size reported
  4. **Reproducible Data**: Same files processed consistently across benchmark runs

* **Technical Changes:**
  - **File**: `/Users/stevejs/guten_split/benches/sentence_detector_bench.rs`
  - **Function**: `bench_gutenberg_throughput()`
  - **Lines**: ~360-400 (text discovery and loading)

* **Implementation:**
  1. **Add file logging**: Print list of discovered files with paths and sizes
  2. **Add character counting**: Count total characters (not bytes) in `all_text`
  3. **Change throughput metric**: Use `Throughput::Elements(char_count)` instead of `Throughput::Bytes`
  4. **Add summary output**: Print total files, total characters, total bytes for transparency

* **Deliverables:**
  - Modified benchmark that reports characters/sec
  - Console output showing which files are processed
  - Clear documentation of total data volume (characters + bytes)
  - Consistent file selection across runs

* **Validation:**
  - Run `GUTENBERG_MIRROR_DIR=~/gutenberg_texts cargo bench gutenberg_throughput`
  - Verify output shows: file list, character count, chars/sec metrics
  - Confirm same files selected on repeat runs

## Results:

**Benchmark Data Transparency Successfully Implemented:**
- **Character throughput reporting**: Benchmarks now report characters/sec (Melem/s) instead of MiB/s
- **File transparency**: Console output lists all 10 processed files with paths, bytes, and character counts
- **Data volume**: Total 9,323,006 characters across 9,555,158 bytes from Project Gutenberg texts
- **Deterministic file selection**: Same files processed consistently across runs

**Performance Results (characters/second):**
- **Manual FST**: 104.73 Melem/s (104.73 million characters/second)
- **DFA**: 185.19 Melem/s (185.19 million characters/second)  
- **Dialog State Machine**: 473.15 Melem/s (473.15 million characters/second)

**Additional improvements:**
- **Warning cleanup**: Added `#[allow(dead_code)]` attributes to eliminate all benchmark warnings
- **Clean output**: Benchmark runs without noise or distracting warnings

## Pre-commit checklist:
- [x] Benchmark reports characters/sec instead of MiB/s
- [x] Console output lists specific files processed with sizes
- [x] Total character count displayed
- [x] File selection is deterministic and documented
- [x] All benchmark warnings eliminated for clean output