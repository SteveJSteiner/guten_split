# Stats Output Format

The `--stats-out` flag generates a JSON file containing per-file and aggregate statistics for the processing run. This document describes the JSON schema and field meanings.

## JSON Schema

```json
{
  "run_start": "1751992392",
  "total_processing_time_ms": 4,
  "total_chars_processed": 445,
  "total_sentences_detected": 12,
  "overall_chars_per_sec": 98415.94559478064,
  "files_processed": 2,
  "files_skipped": 0,
  "files_failed": 0,
  "file_stats": [
    {
      "path": "test_stats_output/test-0.txt",
      "chars_processed": 239,
      "sentences_detected": 6,
      "processing_time_ms": 0,
      "chars_per_sec": 402667.3765293779,
      "status": "success",
      "error": null
    },
    {
      "path": "test_stats_output/second-0.txt",
      "chars_processed": 206,
      "sentences_detected": 6,
      "processing_time_ms": 0,
      "chars_per_sec": 324111.8783856711,
      "status": "success",
      "error": null
    }
  ]
}
```

## Field Descriptions

### Top-level Fields

| Field | Type | Description |
|-------|------|-------------|
| `run_start` | String | Unix timestamp when the run started (seconds since epoch) |
| `total_processing_time_ms` | Integer | Total processing time in milliseconds |
| `total_chars_processed` | Integer | Total number of characters processed across all files |
| `total_sentences_detected` | Integer | Total number of sentences detected across all files |
| `overall_chars_per_sec` | Float | Overall processing throughput in characters per second |
| `files_processed` | Integer | Number of files successfully processed |
| `files_skipped` | Integer | Number of files skipped (already complete) |
| `files_failed` | Integer | Number of files that failed processing |
| `file_stats` | Array | Array of per-file statistics objects |

### Per-file Statistics (`file_stats` array elements)

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | File path relative to the root directory |
| `chars_processed` | Integer | Number of characters processed in this file |
| `sentences_detected` | Integer | Number of sentences detected in this file |
| `processing_time_ms` | Integer | Processing time for this file in milliseconds |
| `chars_per_sec` | Float | Processing throughput for this file in characters per second |
| `status` | String | Processing status: "success", "skipped", or "failed" |
| `error` | String/null | Error message if processing failed, null otherwise |

## Status Values

- **"success"**: File was processed successfully
- **"skipped"**: File was skipped because aux file already exists and is up-to-date
- **"failed"**: File processing failed (error message provided in `error` field)

## Usage Examples

### Command Line

```bash
# Default stats output (creates run_stats.json in current directory)
seams /path/to/gutenberg/texts

# Custom stats output location
seams /path/to/gutenberg/texts --stats-out /custom/path/stats.json

# Absolute path
seams /path/to/gutenberg/texts --stats-out /tmp/processing_stats.json

# Relative path with directory
seams /path/to/gutenberg/texts --stats-out reports/daily_stats.json
```

### Throughput Calculation

The `overall_chars_per_sec` field matches the throughput displayed in the CLI output. It's calculated as:

```
overall_chars_per_sec = total_chars_processed / (total_processing_time_ms / 1000)
```

Per-file throughput is calculated similarly:

```
chars_per_sec = chars_processed / (processing_time_ms / 1000)
```

### Parallel Processing Notes

- Each file is processed in parallel, so individual file processing times may be very small (0ms)
- The `total_processing_time_ms` represents the overall wall-clock time for the parallel processing
- The sum of individual file processing times may be less than the total time due to parallel execution
- Failed files still appear in `file_stats` with status "failed" and error details

## PRD Compliance

This format meets PRD F-8 requirements:
- ✅ Per-file stats (chars processed, sentences, wall-clock ms)
- ✅ Aggregate stats with total chars/sec
- ✅ Matches CLI throughput display
- ✅ Includes failed file counts and error information
- ✅ Works correctly with parallel processing