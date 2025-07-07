# Seams - Gutenberg Sentence Extractor

## Running Benchmarks

### File-by-file Benchmark

This benchmark tests sentence detection performance on real Project Gutenberg texts.

#### Setup:

1. **Copy environment template:**
   ```bash
   cp .env.example .env
   ```

2. **Edit .env to point to your Gutenberg texts:**
   ```bash
   # Edit .env file
   GUTENBERG_MIRROR_DIR=/path/to/your/gutenberg_texts
   ```

3. **Run the benchmark:**
   ```bash
   cargo bench --features mmap --bench file_by_file_bench
   ```

#### Alternative setup (without .env):
```bash
GUTENBERG_MIRROR_DIR=/path/to/texts cargo bench --features mmap --bench file_by_file_bench
```

The benchmark will automatically:
- Load environment variables from `.env` if it exists
- Fall back to `~/gutenberg_texts` if no environment variable is set
- Provide helpful error messages if no test data is found