# Performance Results

This document contains benchmark results from different systems and configurations for SEAMS sentence segmentation performance.

## Current Benchmark Results

### System: macOS 15.5 ARM64 (M1 Macbook Air 2020), 16 GB RAM
**Test Corpus:** Project Gutenberg texts (39 files, ~32M characters, ~237K sentences)  
**Command:** `python run_comparison.py ~/gutenberg_texts`

| Benchmark (version) | Cores | End-to-end time | Speed-up vs nupunkt | Sentences / s | Sentence detection throughput | Total e2e throughput | Note |
|---------------------|:----:|---------------:|--------------------:|--------------:|-----------------------------:|--------------------:|------|
| **seams** | 8 | **4 s** | **3 ×** | **53 k** | 118.2 MB/s | **8.0 MB/s** | line offsets included |
| seams-single-cpu | 1 | 19 s | 0.81 × | 12 k | **163.0 MB/s** | 2.6 MB/s | single-CPU baseline |
| nupunkt (0.5.1) | 1 | 15 s | 1 × | 17 k | 18.0 MB/s | 2.0 MB/s | pure-Python |

### System: Intel i9-13900KF (16 cores, 32 threads) running Linux 5.15 WSL2, 32GB RAM
**Test Corpus:** Project Gutenberg texts (20,440 files, ~7.4B characters, ~56M sentences)  
**Command:** `python run_comparison.py ~/gutenberg`

| Benchmark (version) | Cores | End-to-end time | Speed-up vs nupunkt | Sentences / s | Sentence detection throughput | Total e2e throughput | Note |
|---------------------|:----:|---------------:|--------------------:|--------------:|-----------------------------:|--------------------:|------|
| **seams** | 32 | **6 s** | **59 ×** | **8.6 M** | 105.4 MB/s | **1176.2 MB/s** | line offsets included |
| seams-single-cpu | 1 | 1 m 31 s | 4 × | 611 k | **450.6 MB/s** | 90.4 MB/s | single-CPU baseline |
| nupunkt (0.5.1) | 1 | 6 m 23 s | 1 × | 179 k | 19.7 MB/s | 19.3 MB/s | pure-Python |

## Notes

- **End-to-end throughput** includes complete pipeline: file discovery, reading, boundary detection, span tracking, normalization, and writing output
- **Sentence detection throughput** measures pure boundary detection + line coordinate tracking
- **Speed-up vs nupunkt** shows relative performance compared to the Python nupunkt baseline
- Performance varies significantly by hardware, corpus size, and system configuration
- Results demonstrate SEAMS' effectiveness across different CPU architectures and core counts

## Running Your Own Benchmarks

To contribute benchmark results from your system:

1. **Install dependencies:**
   ```bash
   cd benchmarks
   ./setup_benchmarks.sh
   source .venv/bin/activate
   ```

2. **Run comparison:**
   ```bash
   python run_comparison.py /path/to/gutenberg/texts
   ```

3. **Submit results:** Open an issue or PR with your system specs and benchmark output table.

## Methodology

All benchmarks follow equivalent processing patterns:
- File discovery using `**/*-0.txt` pattern  
- UTF-8 text reading
- Sentence segmentation with boundary detection
- Performance measurement with hardware context
- Statistics collection and comparison

For complete benchmark scripts and methodology, see the Python comparison tools in this directory.