# Polars to Pandas Python Integration Support

* **Task ID:** polars-pandas-support_67.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - NLP researchers primarily use Python pandas for data analysis and processing
  - Current TSV output requires manual parsing and DataFrame construction
  - Polars provides high-performance DataFrame operations with pandas compatibility
  - Direct integration reduces friction for NLP research workflows
  - Enables efficient querying, filtering, and analysis of sentence data
  - Leverages existing DuckDB infrastructure for optimal performance
  - Provides both native Polars API and pandas-compatible interface

* **Acceptance Criteria:**
  1. Add `--polars-output <OUTPUT_PATH>` CLI flag for Polars-compatible output
  2. Generate Polars DataFrame with columns: file_id, file_path, sentence_id, sentence_text, start_line, start_col, end_line, end_col
  3. Support export to common formats: Parquet, Arrow, JSON, CSV
  4. Provide Python integration examples showing pandas conversion
  5. Maintain compatibility with existing DuckDB slim output (reference duckdb-slim-output_65.stevejs.md)
  6. Performance comparable to DuckDB output for large datasets
  7. All existing functionality works with Polars output enabled

* **Deliverables:**
  - Add polars crate dependency to Cargo.toml
  - Implement Polars DataFrame construction from sentence data
  - Add CLI flag parsing for --polars-output option
  - Create output writer for Polars format alongside existing writers
  - Add Python integration examples in docs/
  - Add tests for Polars output format and DataFrame structure
  - Update documentation for new output option

* **DataFrame Schema:**
```
file_id: i32,
file_path: String,
sentence_id: i32,
sentence_text: String,
start_line: i32,
start_col: i32,
end_line: i32,
end_col: i32
```

* **Python Integration Examples:**
```python
# Load Polars output into pandas
import polars as pl
import pandas as pd

# Native Polars
df = pl.read_parquet("sentences.parquet")
filtered = df.filter(pl.col("sentence_text").str.contains("dialog"))

# Convert to pandas
pandas_df = df.to_pandas()
```

* **Implementation Notes:**
  - Consider using Polars LazyFrame for memory efficiency with large datasets
  - Support both eager and lazy evaluation modes
  - Provide format-specific optimizations (Parquet columnar compression, etc.)
  - Reference DuckDB slim output task for database schema consistency
  - Consider streaming writes for very large corpora
  - Maintain column-oriented storage benefits

* **References:**
  - DuckDB slim output task (duckdb-slim-output_65.stevejs.md) for schema consistency
  - Polars crate documentation and DataFrame API
  - Current sentence processing pipeline
  - Python NLP research workflow patterns

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] Polars dependency added to Cargo.toml
- [ ] CLI flag parsing implemented and tested
- [ ] DataFrame schema matches specification
- [ ] Multiple output formats supported (Parquet, Arrow, JSON, CSV)
- [ ] Python integration examples provided
- [ ] Tests passing (`cargo test`)
- [ ] Performance comparable to DuckDB output
- [ ] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
- [ ] Documentation updated for new output format
- [ ] Schema consistency with DuckDB slim output
- [ ] Memory efficiency verified with large datasets