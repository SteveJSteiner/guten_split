# Investigate Character Count Discrepancy Between Seams and Python Tools

* **Task ID:** investigate-character-count-discrepancy_70.stevejs.md
* **Reviewer:** stevejs
* **Area:** benchmarks
* **Motivation (WHY):**
  - Seams reports 31,974,932 characters processed while Python tools report ~30,751,046 characters on the same files
  - This is a significant discrepancy of over 1.2 million characters that needs investigation
  - Could indicate: bytes vs characters counting, text normalization differences, or processing scope differences
  - Essential for ensuring fair performance comparisons and data integrity

* **Acceptance Criteria:**
  1. Identify root cause of character count discrepancy between seams and Python tools
  2. Determine if seams is counting bytes vs Unicode characters
  3. Analyze if Python tools are modifying/normalizing text during processing
  4. Verify that all tools are processing the same set of files
  5. Document the difference and ensure fair comparison methodology
  6. Update benchmark comparison to account for any processing differences

* **Deliverables:**
  - Analysis script to compare character counting methods between tools
  - Investigation report documenting the root cause
  - Verification that benchmark comparisons are fair and accurate
  - Documentation of any text processing differences between tools

* **Investigation Areas:**
  - Character vs byte counting in seams
  - Text normalization in Python tools (pysbd, spaCy, nupunkt)
  - File encoding handling differences
  - Processing scope differences (which files are actually processed)

* **References:**
  - Benchmark comparison showing seams: 31,974,932 chars vs Python: ~30,751,046 chars
  - Python benchmark scripts in benchmarks/ directory
  - Seams character counting implementation in src/

## Investigation Update: Dialog State Machine Backward Seek Bug

**DISCOVERY**: While investigating character count discrepancies with the large problem reproduction file (`exploration/problem_repro-0.txt`), we discovered a critical bug in the dialog sentence detection state machine that causes backward seek errors and processing failures.

### Problem Characterization

**Root Cause**: The `find_sent_sep_end()` function incorrectly calculates the next sentence start position for dialog hard end patterns, causing `PositionTracker` backward seek violations.

**Affected Patterns**: ALL dialog hard end patterns (6 of 7 tested):
- Single quote: `'meet.' "` → Position tracking error: current 14 > target 13  
- Double quote: `"meet." "` → Position tracking error: current 18 > target 17
- Smart double quote: `"meet." "` → Position tracking error: current 18 > target 17  
- Round parentheses: `(meet.) "` → Position tracking error: current 14 > target 13
- Square brackets: `[meet.] "` → Position tracking error: current 14 > target 13
- Curly braces: `{meet.} "` → Position tracking error: current 14 > target 13

**State Machine Flow Issue**:
1. Dialog opener transitions state: `Narrative` → `Dialog*` state
2. Dialog hard end pattern matches: `{sentence_end_punct}{dialog_close}{soft_separator}{sentence_start_chars}`
3. `find_sent_sep_end()` returns position within match instead of match end
4. Results in `next_sentence_start_byte < match_end_byte`, violating forward-only constraint

**Minimal Reproduction**: `"verb 'meet.' \"Why should"` 
- Match: `.' "` (bytes [46, 39, 32, 34]) at positions 10..14
- `find_sent_sep_end()` returns 3, giving `next_sentence_start_byte = 13`  
- But `match_end_byte = 14`, causing backward seek from 14 → 13

**Impact**: This bug prevents processing of any text containing dialog hard end patterns, causing complete failure on realistic literary texts like Project Gutenberg content.

**Location**: `src/sentence_detector/dialog_detector.rs:745-766` in `find_sent_sep_end()` function.

### Resolution

**FIXED**: The dialog state machine backward seek bug has been resolved by correcting the DialogEnd case to respect the SENT_SEP invariant.

**Root Cause**: The `find_dialog_sent_end()` function was violating the SENT_SEP invariant by trying to find "sentence content end" instead of finding separator bounds. This caused the position tracker to advance beyond where the next sentence actually starts.

**Solution**: Replaced `find_dialog_sent_end()` with `find_sent_sep_start()` in the DialogEnd case to correctly identify separator bounds, ensuring the position tracker never needs to seek backwards.

**Result**: Seams now successfully processes all Gutenberg text files without backward seek errors, enabling accurate character count comparisons.

### Post-Fix Benchmark Results

After fixing the dialog state machine bug, the benchmark comparison now runs successfully:

**Seams**: 39/39 files, 32,268,339 chars, 230,523 sentences
**Nupunkt**: 39/39 files, 31,043,966 chars, 267,924 sentences

**Character Count Discrepancy**: ~1.2M characters (32.27M vs 31.04M)
- Seams processes ~3.9% more characters than nupunkt
- This represents the core discrepancy that still needs investigation

**Sentence Count Differences**: Significant variance in sentence detection
- Seams: 230,523 sentences (avg: 5,910.8 per file)  
- Nupunkt: 267,924 sentences (avg: 6,869.8 per file)
- Nupunkt detects ~16% more sentences than seams

The dialog bug fix enables this investigation to proceed with complete data from both tools.

## Character Count Discrepancy Investigation - COMPLETED

**Final Root Cause Analysis**: Seams was counting **bytes** instead of **Unicode characters** while nupunkt counted actual Unicode characters.

### Investigation Process

1. **Created minimal character counting tools** to isolate the issue from sentence detection complexity
2. **File-by-file comparison** revealed systematic ~4% difference across all files
3. **Code analysis** found seams using `content.len()` (bytes) while nupunkt used `len(content)` (Unicode chars)
4. **Line ending analysis** identified remaining acceptable differences due to `\r\n` handling

### Resolution

**Fixed in `src/main.rs`**: Changed `let byte_count = content.len() as u64;` to properly count Unicode characters:
```rust
let sentence_count = sentences.len() as u64;
let byte_count = content.len() as u64;  // Keep for file I/O metrics
let char_count = content.chars().count() as u64;  // Actual Unicode character count
```

And updated `FileStats` assignment:
```rust
chars_processed: char_count,  // Was: byte_count
```

### Final Results

- **Before fix**: 32,268,339 bytes vs 31,043,966 chars (~1.2M difference, 3.9%)
- **After fix**: 31,650,610 chars vs 31,043,966 chars (~600K difference, 1.9%)

**Remaining 1.9% difference is acceptable**: Due to line ending normalization differences:
- Seams (memmap2): Counts `\r\n` as 2 characters (raw file reading)
- Nupunkt (Python): Counts `\r\n` as 1 character (text mode auto-conversion)

This represents legitimate implementation differences, not errors.

## Pre-commit checklist:
- [x] Root cause of processing failure identified (dialog state machine bug)
- [x] Dialog state machine bug fixed - replaced find_dialog_sent_end() with find_sent_sep_start() to respect SENT_SEP invariant
- [x] **Character count discrepancy RESOLVED**: Fixed bytes vs Unicode character counting in seams
- [x] **Core fix applied**: Updated `src/main.rs` to use `content.chars().count()` for `chars_processed` field
- [x] Fair comparison methodology verified - both tools now count Unicode characters correctly
- [x] Remaining 1.9% difference documented and deemed acceptable (line ending handling differences)  
- [x] Documentation updated with complete findings and solution
- [x] **ZERO WARNINGS**: All tests pass, clippy warnings resolved