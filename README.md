# Seams - Lightning-fast detection of natural breaks in narrative text

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

Splits large text corpora into meaningful sentences while preserving narrative flow and dialog structure.

## Narrative Sentence Splitting 

See [examples](docs/examples.md) demonstrating:
- **Dialog spanning paragraph separators** - Dialog attribution stays connected across paragraph breaks
- **Paragraph separators indicating end of never-closed quote** - Letter format with implicit quote boundaries

## Quick Start

### Installation

```bash
cargo install seams
```

**From source:**
```bash
git clone https://github.com/your-username/seams.git
cd seams
cargo install --path .
```

### Basic Usage

Process all Project Gutenberg texts in a directory:
```bash
seams /path/to/gutenberg_texts
```

The tool will:
- Find all `*-0.txt` files recursively
- Extract sentences with boundary detection
- Write results to `*_seams.txt` files alongside originals
- Generate processing statistics in `run_stats.json`

## Usage Examples

### Common Scenarios

**Process a Project Gutenberg mirror:**
```bash
seams ~/gutenberg_mirror
```

**Overwrite existing results:**
```bash
seams --overwrite-all ~/gutenberg_texts
```


**Batch processing with statistics:**
```bash
seams --stats-out batch_results.json ~/large_corpus
```

**Debug mode with detailed progress:**
```bash
seams --fail-fast ~/test_corpus
```

### Output Format

For each input file `book-0.txt`, seams creates `book-0_seams.txt` with:
```
1	This is the first sentence.	(1,1,1,32)
2	Here is the second sentence.	(1,33,2,15)
```

Format: `index<TAB>sentence<TAB>(start_line,start_col,end_line,end_col)`
- Line and column numbers are 1-based
- Sentences are normalized (line breaks removed, whitespace collapsed)
- Span coordinates refer to the original text

### Command Reference

```
seams [OPTIONS] <ROOT_DIR>

Arguments:
  <ROOT_DIR>  Root directory to scan for *-0.txt files

Options:
      --overwrite-all                   Overwrite even complete aux files
      --fail-fast                       Abort on first error
      --no-progress                     Suppress console progress bars
      --stats-out <STATS_OUT>           Stats output file path [default: run_stats.json]
      --clear-restart-log               Clear the restart log before processing
  -h, --help                            Print help
  -V, --version                         Print version
```

## Performance

- **End-to-end throughput:** ≥50 MB/s sustained (includes all processing: reading, boundary detection, span tracking, and writing output)
- **Parallel processing:** Uses up to half the CPU cores each for file enumeration and sentence splitting (no core affinity)
- **Memory efficiency:** Memory-mapped files for large corpora
- **Incremental:** Skip already-processed files automatically

Performance is primarily I/O bound rather than CPU bound. Actual throughput varies by:
- **Hardware:** CPU cores, storage speed, memory bandwidth
- **File characteristics:** Size distribution, text complexity
- **Workload:** Complete pipeline vs. raw boundary detection only

The ≥50 MB/s target represents real-world usable throughput including all processing steps, not just the raw sentence boundary detection which is significantly faster but not representative of actual usage.

## For Developers

**Prerequisites:** Rust 1.88.0+ (see rust-toolchain.toml)

**Quick start:**
```bash
cargo build --release
cargo test
```

For detailed development setup, contribution guidelines, and architecture documentation, see [CLAUDE.md](CLAUDE.md).

## Technical Details

**Algorithm:** DFA-based boundary detection using regex-automata with narrative-aware heuristics for dialog coalescing.

**Performance:** <TBD - comparative benchmark vs. NLTK, spaCy, standard sentence splitters>

**Architecture:** Two-stage pipeline with bounded parallelism (file enumeration + sentence splitting), async I/O with memory-mapped files.

For detailed technical documentation, see [docs/architecture.md](docs/architecture.md).

## License

MIT License - see LICENSE file for details.

## Acknowledgments

Built for Project Gutenberg corpus processing. Designed as a Rust learning resource demonstrating high-performance text processing patterns.