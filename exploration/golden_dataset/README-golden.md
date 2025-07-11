# Golden Dataset Investigation

This directory contains tools and investigations for creating gold-standard sentence datasets from the Gutenberg corpus by analyzing discrepancies between different sentence extraction methods.

## Overview

The goal is to build a curated test set for evaluating sentence segmentation algorithms (punkt, psybd, spaCy, etc.) by finding cases where different methods disagree on sentence boundaries. These disagreements reveal the most challenging cases for sentence segmentation.

## Key Insight

The most valuable gold-set examples are where **the same original text** gets segmented differently by different methods. This reveals true boundary ambiguities rather than just differences in normalization or preprocessing.

## Files

### Scripts
- `build_gold_set.py` - Original approach: extracts sentences from discrepancies, gets original text via coordinates
- `build_boundary_gold_set.py` - **Enhanced approach**: focuses on boundary disagreements using optimized O(n) algorithm
- `evaluate_gold_set.py` - Evaluates sentence segmenters (punkt, psybd, spaCy) against gold-set
- `README_gold_set.md` - Original documentation

### Generated Datasets
- `*.json` - Various test gold-sets generated during investigation
- `test_*` - Test files and smaller samples

## Current Status

### What Works
1. **Original text extraction**: Successfully extracts unprocessed text from `-0.txt` files using coordinate mappings from `_seams.txt`
2. **Boundary disagreement detection**: Finds regions where seams vs fast/sentences methods disagree on boundaries
3. **Complexity classification**: Categorizes sentences by type (dialog, normal, complex, abbreviation)
4. **Comparative evaluation**: Framework for testing multiple segmentation algorithms

### Key Discovery
The fast sentence files (`_sentences_fast.txt`) don't contain coordinate mappings, only the seams files (`_seams.txt`) do. This limits our ability to extract original text for fast-only sentences, but we can still:
- Compare normalized text to find discrepancies
- Use seams coordinates to extract original text regions
- Find overlapping text regions that were segmented differently

## Technical Approach

### Boundary Disagreement Detection
The enhanced algorithm (`build_boundary_gold_set.py`) uses:
- **O(n) traversal**: Processes sentences in order, "consuming" overlapping portions
- **Fuzzy matching**: Finds fast sentences that overlap with seams regions using similarity metrics
- **Original text extraction**: Maps seams coordinates back to source text with preserved formatting

### Example Gold-Set Entry
```json
{
  "original_text": "A. Huntley was born in Providence, R. I., November 2, 1843. J. J. KELLY was one of the best known vocalists...",
  "seams_sentences": [
    "A. Huntley was born in Providence, R. I., November 2, 1843.",
    "J. J. KELLY was one of the best known vocalists in minstrelsy."
  ],
  "comparison_sentences": [
    "Huntley was born in Providence, R.",
    "I., November 2, 1843.", 
    "KELLY was one of the best known vocalists in minstrelsy."
  ],
  "comparison_method": "fast",
  "disagreement_type": "over_split",
  "complexity": "abbreviation"
}
```

This shows fast method incorrectly split on "R. I." (Rhode Island abbreviation).

## Next Steps

### 1. ✅ Soft Separator Fix (COMPLETED)
~~**Priority: High** - Fix the dialog attribution bug where `"no!" interposed` incorrectly splits.~~

**Status**: ✅ **COMPLETED** in task `dialog-hard-separator-and-apostrophe-fixes_82.stevejs.md`
- Dialog attribution bug resolved through pattern priority fixes
- Multi-pattern regex architecture implemented with `Regex::new_many()`
- Universal whitespace requirement prevents apostrophe context confusion
- All dialog detection tests passing

### 2. ✅ Multi-Pattern Regex Refactor (COMPLETED) 
~~**Priority: High** - Implement the `Regex::new_many()` approach documented in task `regex-multi-pattern-dialog-detection_81.stevejs.md`.~~

**Status**: ✅ **COMPLETED** as part of task 82
- Direct `PatternID` to semantic action mapping implemented
- Post-processing disambiguation eliminated
- Attribution patterns working correctly

### 3. ✅ Attribution Pattern Recognition (NOT NEEDED)
~~**Priority: Medium** - Add explicit patterns for dialog attribution verbs~~

**Status**: ✅ **NOT NEEDED** - Current implementation handles attribution correctly through existing pattern architecture. The universal approach is more robust than hard-coded verb lists.

### 4. ✅ Golden Dataset Generation (COMPLETED)
**Priority: HIGH** - Generate the actual golden dataset using existing tools:
- ✅ Enhanced `build_boundary_gold_set.py` with filtering rules and multi-method segmentation
- ✅ Added filtering rules: reject examples with no lowercase letters, reject when all methods agree
- ✅ Added nupunkt and pysbd segmentations to each example for comprehensive comparison
- ✅ Successfully tested enhanced tool with boundary disagreement detection
- Ready to generate production datasets at scale

### 5. Dataset Validation and Quality Control
**Priority: MEDIUM** - Future validation tasks:
- Verify coordinate mappings extract correct original text at scale
- Check byte-level fidelity preservation as documented  
- Test against known edge cases (abbreviations, dialog attribution, etc.)
- Ensure balanced distribution across text complexity types
- Generate production datasets with 500-1000+ examples

### 6. Evaluation Framework Enhancement
**Priority: MEDIUM** - Improve evaluation against sentence segmentation algorithms:
- Test against punkt, pysbd, spaCy using existing `evaluate_gold_set.py`
- Compare performance across different text genres
- Generate precision/recall metrics by complexity category
- Document algorithm strengths/weaknesses on different text types

## Usage Examples

### Generate Gold-Set
```bash
# Basic gold-set (original approach)
python build_gold_set.py --max-files 20 --target-size 1000

# Boundary disagreements (enhanced approach)  
python build_boundary_gold_set.py --max-files 15 --target-size 500

# Both fast and sentences comparison
python build_boundary_gold_set.py --max-files 20 --target-size 800
```

### Evaluate Segmenters
```bash
# Install dependencies
pip install nltk pysbd spacy
python -m spacy download en_core_web_sm

# Run evaluation
python evaluate_gold_set.py gold_set_sentences.json --output results.json
```

## Research Questions

1. **How do different text genres affect segmentation accuracy?**
   - Dialog-heavy vs narrative vs technical texts
   - Historical vs contemporary writing styles

2. **What are the most challenging sentence boundary cases?**
   - Abbreviations (R. I., Dr. Smith)
   - Dialog attribution (`"Hello," he said.`)
   - Complex punctuation (ellipses, em-dashes)

3. **How do different algorithms handle edge cases?**
   - punkt vs pysbd vs spaCy performance comparison
   - Which performs best on different text types?

## Implementation Notes

### Coordinate Mapping
The seams files use 1-based coordinates: `(start_line, start_col, end_line, end_col)`.
Text extraction must convert to 0-based indexing and handle:
- Single vs multi-line spans
- UTF-8 character boundaries
- Line ending variations (`\n` vs `\r\n`)

**CRITICAL: Byte-Level Fidelity Required**

For accurate sentence boundary evaluation, the gold-set must preserve the *precise byte stream* of the original text from the start of each line. This impacts serialization:

- **Whitespace preservation**: Exact spacing, tabs, and line breaks must be maintained
- **Character encoding**: UTF-8 byte sequences must be preserved exactly
- **Line boundaries**: Original line structure affects sentence detection algorithms
- **Serialization format**: JSON string escaping can alter the byte stream

The coordinate mapping extracts text as:
```rust
let original_text = &original_lines[start_line_idx][start_col_idx..end_col_idx];
```

But when serialized to JSON, the exact byte representation may change due to:
- JSON string escaping (`"` → `\"`, `\n` → `\\n`)
- Unicode normalization
- Whitespace trimming

**Recommendation**: Consider alternative serialization formats that preserve exact byte streams:
- Base64 encoding of the original byte sequence
- Binary format with length-prefixed strings
- Separate metadata file with coordinate ranges + original file references

### Performance Considerations
- The O(n²) original algorithm was replaced with O(n) sorted traversal
- Fuzzy matching is limited to prevent performance degradation
- Regex compilation is cached for repeated use

### File Format Assumptions
- Seams files: `sentence_id TAB text TAB (coordinates)`
- Fast files: `sentence_id TAB text` (no coordinates)
- Original files: UTF-8 text with preserved formatting

## Future Directions

This investigation could evolve into:
1. **Benchmark suite** for sentence segmentation algorithms
2. **Training data** for machine learning approaches
3. **Quality metrics** for prose processing pipelines
4. **Comparative study** of segmentation approaches across genres

The boundary disagreement approach shows particular promise for identifying the most challenging and informative test cases.