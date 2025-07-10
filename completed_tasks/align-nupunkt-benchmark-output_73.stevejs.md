# Align Nupunkt Benchmark Output Format

* **Task ID:** align-nupunkt-benchmark-output_73.stevejs.md
* **Reviewer:** stevejs
* **Area:** tests
* **Motivation (WHY):**
  - The `nupunkt` benchmark currently produces a different JSON output format for file-level statistics compared to the `seams` benchmark.
  - Aligning the output format will simplify the comparison logic in `run_comparison.py` and make the benchmark results consistent and easier to parse.
  - The current `nupunkt` output includes the full list of sentences, which is unnecessary and consumes a large amount of space in the output JSON.
* **Acceptance Criteria:**
  1. The `python_nupunkt_benchmark.py` script is modified to produce the following JSON structure for each file processed:
     ```json
     {
       "path": "/path/to/file.txt",
       "chars_processed": 12345,
       "sentences_detected": 123,
       "processing_time_ms": 45.6,
       "sentence_detection_time_ms": 40.1,
       "chars_per_sec": 270725.8,
       "status": "success",
       "error": null
     }
     ```
  2. The `sentences` array is removed from the output.
  3. The field names are updated to match the `seams` benchmark output:
     - `file_path` -> `path`
     - `sentence_count` -> `sentences_detected`
     - `throughput_chars_per_sec` -> `chars_per_sec`
  4. The following fields are added:
     - `sentence_detection_time_ms` (if not already present, requires adding timing logic)
     - `status` (`"success"` or `"failed"`)
     - `error` (error message string or `null`)
  5. The `success` field is removed (replaced by `status`).
* **Deliverables:**
  - Modified `benchmarks/python_nupunkt_benchmark.py` script.
* **References:**
  - User request to align benchmark output formats.

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [x] Documentation updated if needed
- [x] Clippy warnings addressed
