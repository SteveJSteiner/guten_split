=== Benchmark Comparison Summary ===
| Benchmark (version) | Cores | End-to-end time | Speed-up vs nupunkt | Sentences / s | Sentence detection throughput | Total e2e throughput | Note |
|---------------------|:----:|---------------:|--------------------:|--------------:|-----------------------------:|--------------------:|------|
| **seams** | **32** | **8 s** | **0 ×** | **6.4 M** | **135.1 MB/s** | **1213.4 MB/s** | line offsets included |
| nupunkt (0.5.1) | 1 | 3 s | 1 × | 177 k | 20.7 MB/s | 17.4 MB/s | pure-Python |



 python run_comparison.py ~/gutenberg
- 🚀 Starting sentence segmentation benchmark comparison...
- 📁 Processing files from: /home/steve/gutenberg
- 💻 System: Linux-5.15.167.4-microsoft-standard-WSL2-x86_64-with-glibc2.39
- 🧠 Memory: 31.26 GB

- 🔨 Building seams...
- 🏃 Running seams benchmark...
- ✅ seams completed:
   - Total e2e time: 6.14s
   - Files: 20440/20440
   - Chars: 7,465,634,379
   - Sentences: 51,466,856 (min: 1, Q25: 959, avg: 2517.9, median: 2167.0, Q75: 3465, max: 91608)
   - Sentence detection throughput: 149,244,035 chars/sec (142.33 MB/sec)
   - Total e2e throughput: 1,271,437,099 chars/sec (1212.54 MB/sec)

- 🐍 Running nupunkt benchmark...
- ✅ nupunkt completed:
   - Total e2e time: 361.01s
   - Files: 20440/20440
   - Chars: 7,396,878,768
   - Sentences: 68,763,351 (min: 1, Q25: 1282, avg: 3364.2, median: 2798.0, Q75: 4504, max: 152447)
   - Sentence detection throughput: 20,520,140 chars/sec (19.57 MB/sec)
   - Total e2e throughput: 20,489,635 chars/sec (19.54 MB/sec)

- 📊 Current leaderboard:
   1. seams: 1,271,437,099 chars/sec [1.00x] (6.1s)
   2. nupunkt: 20,520,140 chars/sec [0.02x] (361.0s)

=== Benchmark Comparison Summary ===
| Benchmark (version) | Cores | End-to-end time | Speed-up vs nupunkt | Sentences / s | Sentence detection throughput | Total e2e throughput | Note |
|---------------------|:----:|---------------:|--------------------:|--------------:|-----------------------------:|--------------------:|------|
| **seams** | **32** | **6 s** | **59 ×** | **8.4 M** | **149.2 MB/s** | **1271.4 MB/s** | line offsets included |
| nupunkt (0.5.1) | 1 | 6 m 1 s | 1 × | 190 k | 20.5 MB/s | 20.5 MB/s | pure-Python |