# Boundary Test Harness

The boundary test harness provides systematic validation of Dialog State Machine pattern matching and state transitions. It uses a **data-driven approach** with production rules that generate comprehensive test cases.

## Overview

The test harness separates **test structure** (regenerated from rules) from **validation state** (preserved between runs):

- **Rule files**: Define pattern categories and test generation rules
- **Generated tests**: Fresh test cases created from rules each time
- **Validation state**: Baseline behavior and validation flags (preserved)

## Architecture

```
tests/boundary_rules/*.json  →  [Generator]  →  Temp test cases
                                     ↓
tests/boundary_validation_state.json  →  [Merger]  →  [Test Runner]
```

## Files Structure

### Rule Files (Source of Truth)
- `tests/boundary_rules/narrative_boundaries.json` - Narrative sentence boundaries
- `tests/boundary_rules/dialog_opens.json` - Dialog opening patterns  
- `tests/boundary_rules/dialog_ends.json` - Dialog ending patterns
- `tests/boundary_rules/known_issues.json` - Problematic patterns to track

### Generated Files
- `tests/generated_boundary_tests.json` - Generated test cases (git ignored, auto-recreated)
- `tests/boundary_validation_state.json` - Validation state (committed, preserves work)

### Test Functions
- `test_generated_boundary_cases()` - Read-only validation (10 tests)
- `test_populate_baseline_behavior()` - Baseline population (50 tests)

## Usage Workflow

### 1. Running Tests

```bash
# Quick validation (10 tests)
cargo test test_generated_boundary_cases -- --nocapture

# Populate/update baselines (50 tests)  
cargo test test_populate_baseline_behavior -- --nocapture
```

### 2. Adding New Test Patterns

Edit the appropriate rule file in `tests/boundary_rules/`:

```json
{
  "id": "new_pattern_test",
  "description": "Test description",
  "end_chars": [".", "!", "?"],
  "separators": [" ", "\t"], 
  "start_chars": ["A"],
  "context_template": "Context {end}{sep}{start}nother sentence.",
  "expected_match_type": "UNKNOWN",
  "expected_next_state": "UNKNOWN", 
  "creates_sentence_boundary": true,
  "validated": false
}
```

**Rule components:**
- `end_chars`: Sentence ending punctuation
- `separators`: Whitespace between end and start
- `start_chars`: Sentence starting characters
- `context_template`: Text template with `{end}{sep}{start}` placeholders
- `expected_match_type`: `"UNKNOWN"` for new patterns (populated by harness)
- `validated`: `false` initially, set to `true` after manual verification

### 3. Validation Workflow

The test harness implements a validation state machine:

```
UNKNOWN → BASELINE_RECORDED (when baseline populated)
    ↓
UNVALIDATED → manually set validated: true  
    ↓
VALIDATED → behavior locked in
```

**Validation rules:**
- `validated + no_change` → **PASS** ✅
- `validated + change` → **ERROR** ❌ (breaks build)
- `unvalidated + change` → **ATTENTION_REQUIRED** ⚠️ (needs review)
- `unvalidated + no_change` → **PASS** ✅

### 4. Managing Validation State

```bash
# Populate baselines for new patterns
cargo test test_populate_baseline_behavior -- --nocapture

# Review validation state file
cat tests/boundary_validation_state.json

# Mark patterns as validated after manual review
# Edit boundary_validation_state.json: "validated": true
```

### 5. Adding Known Issues

For patterns with known bugs, add to `tests/boundary_rules/known_issues.json`:

```json
{
  "id": "period_quote_space_bug",
  "description": "KNOWN BUG: Period + space + quote misclassified",
  "pattern": ". \"",
  "current_state": "Narrative", 
  "context_before": "They had been strangers too long",
  "context_after": "It's all over!\" said the surgeon.",
  "actual_behavior": {
    "match_type": "DialogOpen",
    "next_state": "DialogDoubleQuote"
  },
  "expected_behavior": {
    "match_type": "NarrativeGestureBoundary",
    "next_state": "Narrative"
  },
  "source": "FALSE_POSITIVE_examples.txt #7",
  "validated": false,
  "fixed": false
}
```

## Test Output

### Summary Section
```
=== BOUNDARY VALIDATION SUMMARY ===
Total Tests: 50
Passed: 47 (94.0%)
No Match: 3 (6.0%)
```

### Category Breakdown
```
=== RESULTS BY CATEGORY ===
narrative_boundaries: 95.2% success (20/21 tests)
  No Match: 1 
dialog_opens: 100.0% success (7/7 tests)
dialog_ends: 90.9% success (20/22 tests)
  No Match: 2 
```

### Status Indicators
- ✅ **PASSED**: All tests behaving as expected
- ⚠️ **ATTENTION**: N unvalidated tests need review  
- ❌ **FAILED**: N validated tests changed behavior

## Common Workflows

### Adding a New Boundary Pattern
1. Add rule to appropriate `tests/boundary_rules/*.json` file
2. Run `cargo test test_populate_baseline_behavior` to record baseline
3. Review the behavior in `tests/boundary_validation_state.json`
4. If correct, set `"validated": true` in validation state file

### Investigating Pattern Failures
1. Look at test output for specific failing patterns
2. Check context templates in rule files
3. Use `classify_match()` function directly for debugging
4. Add to `known_issues.json` if it's a Dialog State Machine bug

### Regression Testing
1. Make changes to Dialog State Machine
2. Run `cargo test test_generated_boundary_cases`
3. Any **ERROR** results indicate validated behavior changed
4. Any **ATTENTION_REQUIRED** results need manual review

### Updating Rule Files
1. Edit `tests/boundary_rules/*.json` files
2. Tests automatically regenerate from updated rules
3. Validation state preserved across rule changes
4. Only commit rule files and validation state (generated files ignored)

## Technical Details

### Rule Expansion
Each rule generates multiple test cases by expanding the Cartesian product of:
- `end_chars` × `separators` × `start_chars`

Example: `[".", "!"] × [" ", "\t"] × ["A", "\""]` = 8 test cases

### Context Templates
Templates use placeholder substitution:
- `{end}` → end character
- `{sep}` → separator  
- `{start}` → start character

Example: `"Context {end}{sep}{start}nother sentence."` becomes `"Context . Another sentence."`

### State Management
- **Generated structure**: Always fresh from rules (temp directory)
- **Validation state**: Preserved between runs (committed file)
- **Automatic merging**: Validation state merged into generated structure

This approach ensures tests stay in sync with rules while preserving validation work.