# Python Benchmark Comparison

This directory contains Python benchmark scripts for comparing seams performance against established Python sentence segmentation libraries.

## Tools Compared

- **seams** - This Rust implementation
- **pysbd** - Python Sentence Boundary Disambiguation 
- **spaCy sentencizer** - spaCy's rule-based sentence segmenter
- **nupunkt** - Zero-dependency Punkt algorithm implementation

## Usage

### Prerequisites

Install Python dependencies using uv:
```bash
cd benchmarks
./setup_benchmarks.sh
```

This will create a `.venv` virtual environment and install all required dependencies from `pyproject.toml`.

### Running Individual Benchmarks

Each tool has its own benchmark script:

```bash
# Activate virtual environment
source .venv/bin/activate

# pysbd benchmark
python benchmarks/python_pysbd_benchmark.py /path/to/gutenberg/files

# spaCy benchmark  
python benchmarks/python_spacy_benchmark.py /path/to/gutenberg/files

# nupunkt benchmark
python benchmarks/python_nupunkt_benchmark.py /path/to/gutenberg/files
```

### Running Comprehensive Comparison

Run all benchmarks and generate comparative analysis:

```bash
# Activate virtual environment
source .venv/bin/activate

python benchmarks/run_comparison.py /path/to/gutenberg/files
```

This will:
1. Build and run seams benchmark
2. Run all Python benchmarks
3. Generate `benchmark_comparison.json` with hardware-contextualized results
4. Print performance comparison summary

### Testing with Limited Files

For quick testing, limit the number of files processed:

```bash
source .venv/bin/activate
python benchmarks/run_comparison.py /path/to/gutenberg/files --max_files 10
```

## Output Format

Each benchmark produces JSON stats with:
- Tool name and version
- Processing statistics (files, characters, sentences)
- Throughput metrics (chars/sec, MB/sec)
- Hardware context (CPU, memory, platform)
- Sample sentences for accuracy comparison

## Methodology

All benchmarks follow equivalent processing patterns:
1. File discovery using `**/*-0.txt` pattern
2. UTF-8 text reading
3. Sentence segmentation
4. Performance measurement
5. Statistics collection

This ensures fair comparison across different implementations.

## Hardware Context

Performance numbers are meaningless without hardware context. The comparison runner automatically collects:
- Platform and architecture
- CPU information
- Memory specifications
- Python version
- Tool versions

Results are reported as relative performance ratios, not absolute numbers.