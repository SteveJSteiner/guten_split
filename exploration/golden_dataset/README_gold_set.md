# Gold-Set Sentence Extraction

This directory contains scripts for building and evaluating a gold-set of sentences for comparative analysis against punkt, psybd, and other sentence segmentation algorithms.

## Overview

The gold-set is built by analyzing discrepancies between two sentence extraction methods and extracting the **original text** from source files:
- **Seams extraction**: Conservative approach with coordinate mapping to original text
- **Fast extraction**: More aggressive approach (used for comparison only)

The key innovation is that we extract the **original, unprocessed text** from the `-0.txt` files using coordinate mappings from the seams files, preserving all original formatting, punctuation, and structure. This provides a true gold-set for evaluating sentence segmentation algorithms.

## Scripts

### `build_gold_set.py`
Generates a curated gold-set of sentences from Gutenberg corpus discrepancies.

**Usage:**
```bash
python scripts/build_gold_set.py [options]
```

**Options:**
- `--gutenberg-root`: Root directory of Gutenberg corpus (default: `/home/steve/gutenberg`)
- `--output`: Output JSON file (default: `gold_set_sentences.json`)
- `--max-files`: Maximum number of file pairs to analyze (default: 50)
- `--target-size`: Target number of sentences in gold-set (default: 1000)
- `--seed`: Random seed for reproducibility (default: 42)

**Example:**
```bash
python scripts/build_gold_set.py --max-files 30 --target-size 1000 --output gold_set_sentences.json
```

### `evaluate_gold_set.py`
Evaluates sentence segmentation algorithms against the gold-set.

**Usage:**
```bash
python scripts/evaluate_gold_set.py gold_set_sentences.json [--output results.json]
```

**Requires (install as needed):**
- `nltk` for punkt evaluation
- `pysbd` for pysbd evaluation  
- `spacy` with English model for spaCy evaluation

**Example:**
```bash
# Install dependencies
pip install nltk pysbd spacy
python -m spacy download en_core_web_sm

# Run evaluation
python scripts/evaluate_gold_set.py gold_set_sentences.json --output evaluation_results.json
```

## Gold-Set Structure

The generated JSON contains original text extracted from source files:

```json
{
  "metadata": {
    "total_examples": 1000,
    "complexity_distribution": {
      "dialog": 300,
      "normal": 400, 
      "complex": 200,
      "abbreviation": 100
    },
    "method_distribution": {
      "seams_only": 893,
      "both": 53,
      "seams": 54
    }
  },
  "sentences": [
    {
      "text": "Original text with formatting preserved.",
      "source_file": "6/9/8/2/69826/69826-0.txt",
      "method": "seams_only|both|seams",
      "complexity": "dialog|normal|complex|abbreviation",
      "length": 37,
      "line_number": 1234,
      "context_before": "",
      "context_after": ""
    }
  ]
}
```

## Sentence Complexity Classification

Sentences are automatically classified into categories:

- **dialog**: Contains quoted speech, dialog tags, or dialog punctuation
- **normal**: Standard narrative prose without complex features
- **complex**: Contains semicolons, parentheticals, em-dashes, or complex conjunctions
- **abbreviation**: Contains abbreviations that may confuse sentence boundaries

## Example Workflow

1. **Generate gold-set:**
   ```bash
   python scripts/build_gold_set.py --target-size 1000
   ```

2. **Evaluate algorithms:**
   ```bash
   python scripts/evaluate_gold_set.py gold_set_sentences.json --output results.json
   ```

3. **Analyze results:**
   The evaluation will show precision, recall, and F1 scores for each algorithm broken down by sentence complexity.

## Sample Output

```
SENTENCE SEGMENTATION EVALUATION RESULTS
================================================================================

NORMAL SENTENCES:
--------------------------------------------------
Algorithm       Precision  Recall     F1         Correct  Total   
--------------------------------------------------
punkt           0.847      0.847      0.847      339      400     
pysbd           0.892      0.892      0.892      357      400     
spacy           0.825      0.825      0.825      330      400     
simple_regex    0.723      0.723      0.723      289      400     

DIALOG SENTENCES:
--------------------------------------------------
Algorithm       Precision  Recall     F1         Correct  Total   
--------------------------------------------------
punkt           0.653      0.653      0.653      196      300     
pysbd           0.713      0.713      0.713      214      300     
spacy           0.667      0.667      0.667      200      300     
simple_regex    0.487      0.487      0.487      146      300     
```

This gold-set provides a systematic way to evaluate and compare sentence segmentation approaches, with particular focus on challenging cases where different methods disagree.