# Enable Single File Processing in Seams CLI

* **Task ID:** single-file-processing_74.stevejs.md
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Currently, the `seams` CLI only supports processing entire directory trees.
  - Adding the ability to process a single file will provide more flexibility for users who may only want to process a specific file without scanning a whole directory.
  - This will also make the tool easier to use for testing and debugging on individual files.
* **Acceptance Criteria:**
  1. The `seams` CLI can accept a single file path as an argument.
  2. If the provided path is a file, the CLI processes that file and exits.
  3. If the provided path is a directory, the CLI processes the directory recursively as it does now.
  4. The CLI help text is updated to reflect the new capability, explaining that the tool accepts either a file or a directory.
  5. The existing UX for directory processing is preserved.
* **Deliverables:**
  - Modified `src/main.rs` with the updated CLI argument parsing and processing logic.
* **References:**
  - User request to add single-file processing.

## Implementation Summary

Successfully implemented single file processing capability for the seams CLI:

### Key Changes Made

1. **Updated CLI Argument Handling** (`src/main.rs`):
   - Modified argument description from "Root directory" to "Directory to scan recursively for *-0.txt files, or single *-0.txt file to process"
   - Added automatic detection of file vs directory paths
   - Preserved all existing directory processing behavior

2. **Added Single File Processing Function**:
   - Implemented `process_single_file_mode()` function for handling individual files
   - Added file validation (exists, is file, matches *-0.txt pattern)
   - Provides detailed performance metrics for single files
   - Generates stats output compatible with existing tooling

3. **Enhanced Help Documentation**:
   - Updated CLI help text to include single file usage example: `seams /path/to/single-file-0.txt`
   - Added clear documentation in BASIC USAGE section

4. **Validated Implementation**:
   - Tested successfully on files that were previously failing UTF-8 validation
   - Confirmed compatibility with existing benchmarking and stats infrastructure
   - Maintains all existing functionality for directory processing

### Usage Examples
```bash
# Process single file
seams /path/to/book-0.txt --stats-out single_stats.json

# Process directory (unchanged behavior)  
seams /path/to/gutenberg/ --stats-out dir_stats.json
```

## Pre-commit checklist:
- [x] All deliverables implemented
- [x] Tests passing (`cargo test`)
- [x] Claims validated (`cargo test -- --nocapture | grep -E "(concurrent|parallel|faster|optimized)"` + manual verification)
- [x] Documentation updated if needed
- [x] **ZERO WARNINGS**: `./scripts/validate_warning_free.sh` passes completely
