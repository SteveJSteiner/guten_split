# Enhanced Sentence Metadata Statistics

* **Task ID:** sentence-metadata-statistics_55.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - PRD Open Question: "Should stats include per-sentence length histograms?"
  - Current stats only include basic counts (total sentences, total chars)
  - Enhanced metadata would provide valuable insights into text characteristics
  - Useful for corpus analysis and quality assessment
  - Aligns with research/academic use cases for sentence boundary detection

* **Acceptance Criteria:**
  1. Stats output includes sentence length statistics (min, max, average, median)
  2. Character count distribution metrics for detected sentences
  3. Sentence count per file statistics
  4. Optional histogram data for sentence lengths
  5. Statistics are aggregated across all processed files
  6. JSON schema is well-documented and backwards compatible

* **Deliverables:**
  - Enhanced statistics collection during sentence detection
  - Updated run_stats.json format with new metadata fields
  - Aggregation logic for sentence metadata across files
  - Documentation of statistics schema
  - Optional histogram generation (configurable detail level)

* **Statistics to Include:**
  - **Sentence Length**: min, max, average, median character counts
  - **Sentence Distribution**: count per file, files with 0 sentences
  - **Character Distribution**: total chars in sentences vs source files
  - **Detection Quality**: sentences with unusual characteristics
  - **Optional Histogram**: sentence length distribution buckets

* **Implementation Considerations:**
  - Statistics collection should have minimal performance impact
  - Memory usage should remain bounded for large corpora
  - Consider streaming statistics calculation to avoid storing all lengths
  - Backwards compatibility with existing stats consumers

* **JSON Schema Example:**
```json
{
  "files_processed": 1250,
  "total_sentences": 45823,
  "total_characters": 2847392,
  "sentence_stats": {
    "length_chars": {
      "min": 3,
      "max": 847,
      "average": 62.1,
      "median": 58
    },
    "distribution": {
      "files_with_sentences": 1248,
      "files_empty": 2,
      "avg_sentences_per_file": 36.7
    }
  }
}
```

* **References:**
  - PRD Open Question 2: per-sentence length histograms
  - Current stats implementation in main.rs
  - run_stats.json format requirements

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Tests passing (`cargo test`)
- [ ] Claims validated (statistics are accurate and useful)
- [ ] Documentation updated (JSON schema documented)
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] **Performance impact**: Statistics collection has minimal overhead
- [ ] **Backwards compatibility**: Existing stats consumers still work