# Compositional Sentence Boundary Detection Design

## Executive Summary

This document outlines a systematic compositional approach to sentence boundary detection that combines three fundamental components: sentence endings, separators, and sentence starts. The approach moves from ad-hoc pattern enumeration to a principled design that can handle all valid sentence boundary combinations.

## Design Principles

### 1. Boundary Detection as Composition

Sentence boundaries are detected using the pattern:
```
<SENTENCE_END><SENTENCE_SEPARATOR><SENTENCE_START>
```

Where:
- **SENTENCE_END**: Punctuation that can terminate a sentence
- **SENTENCE_SEPARATOR**: Whitespace that separates sentences (the actual boundary point)
- **SENTENCE_START**: Characters that can begin a sentence

### 2. Separator as Split Point

The **SENTENCE_SEPARATOR** (whitespace) is the actual boundary where sentences are divided:
- **Before separator** = end of previous sentence (includes any closing punctuation/quotes)
- **After separator** = start of next sentence (includes any opening quotes/parentheticals)

### 3. Component-Based Regex Construction

Each component is built from fundamental regex expressions combined with OR operators:

```rust
SENTENCE_END = BASIC_END | QUOTE_END | PAREN_END | ...
SENTENCE_SEPARATOR = WHITESPACE_PATTERNS
SENTENCE_START = CAPITAL_START | QUOTE_START | PAREN_START | ...
```

## Component Definitions

### SENTENCE_END Components

```rust
// Basic punctuation endings
BASIC_END = [.!?]

// Quoted endings (punctuation + closing quote)
QUOTE_END = [.!?]['"\u{201D}\u{2019}]

// Parenthetical endings (punctuation + closing paren/bracket)
PAREN_END = [.!?][)\]}]

// Future extensions
ELLIPSIS_END = \.{3}
EMOJI_END = [specific_emoji_patterns]
```

### SENTENCE_SEPARATOR Components

```rust
// Basic whitespace
BASIC_SEP = \s+

// Line breaks (for future multi-line support)
LINE_SEP = \s*\n\s*

// Future extensions
FORMATTED_SEP = \s*<br>\s*  // HTML breaks
```

### SENTENCE_START Components

```rust
// Capital letter starts
CAPITAL_START = [A-Z]

// Quoted starts
QUOTE_START = ['"\u{201C}\u{2018}]

// Parenthetical starts  
PAREN_START = [({\[]

// Numeric starts (for lists, etc.)
NUMERIC_START = [0-9]

// Future extensions
UNICODE_CAPITAL_START = [\p{Lu}]  // Unicode uppercase
SPECIAL_START = [@#$]  // Social media, special formatting
```

## Pattern Generation Algorithm

### 1. Cartesian Product Approach

Generate all valid combinations:
```rust
fn generate_boundary_patterns() -> Vec<String> {
    let mut patterns = Vec::new();
    
    for end in SENTENCE_END_VARIANTS {
        for sep in SENTENCE_SEPARATOR_VARIANTS {
            for start in SENTENCE_START_VARIANTS {
                patterns.push(format!("{}{}{}", end, sep, start));
            }
        }
    }
    
    patterns
}
```

### 2. Combined Regex Construction

```rust
// Current approach: Enumerate specific combinations
let combined_pattern = format!(
    "(?:{})|(?:{})|(?:{})",
    basic_pattern,
    dialog_end_pattern, 
    quote_start_pattern
);

// Proposed approach: Compositional generation
let sentence_end = format!("(?:{})", SENTENCE_END_COMPONENTS.join("|"));
let sentence_sep = format!("(?:{})", SENTENCE_SEPARATOR_COMPONENTS.join("|"));
let sentence_start = format!("(?:{})", SENTENCE_START_COMPONENTS.join("|"));

let boundary_pattern = format!("{}{}{}", sentence_end, sentence_sep, sentence_start);
```

## False Boundary Filtering

### 1. Dictionary-Based Filtering

After pattern matching, apply contextual filters:

```rust
// Current implementation
let is_title_false_positive = {
    let words: Vec<&str> = preceding_text.split_whitespace().collect();
    if let Some(last_word) = words.last() {
        let clean_word = last_word.trim_matches(QUOTE_CHARS);
        TITLE_FALSE_POSITIVES.contains(&clean_word)
    } else {
        false
    }
};
```

### 2. Context-Aware Filtering Categories

```rust
// Title abbreviations (Dr. Smith, Mr. Johnson)
TITLE_FALSE_POSITIVES = ["Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr."]

// Measurement abbreviations (5 ft. tall, 10 lbs. weight)
MEASUREMENT_FALSE_POSITIVES = ["ft.", "in.", "lbs.", "oz.", "mi.", "km.", "deg."]

// Geographic abbreviations (U.S.A. exports, N.Y.C. traffic)
GEOGRAPHIC_FALSE_POSITIVES = ["U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C."]

// Time abbreviations (5 p.m. meeting, 9 a.m. start)
TIME_FALSE_POSITIVES = ["a.m.", "p.m."]

// Academic abbreviations (Smith et al. published, vs. other studies)
ACADEMIC_FALSE_POSITIVES = ["et al.", "vs.", "etc."]
```

## Current Pattern Gaps

### Missing Combinations

Based on the compositional framework, we're missing:

1. **QUOTE_END + QUOTE_START**: `[.!?]['"\u{201D}\u{2019}]\s+['"\u{201C}\u{2018}]`
   - Example: `"He said, 'Hello.' 'Goodbye,' she replied."`

2. **QUOTE_END + PAREN_START**: `[.!?]['"\u{201D}\u{2019}]\s+[({\[]`
   - Example: `"He said, 'Hello.' (She left quietly.)"`

3. **PAREN_END + QUOTE_START**: `[.!?][)\]}\s+['"\u{201C}\u{2018}]`
   - Example: `"He left (quietly.) 'Where did he go?' she asked."`

4. **PAREN_END + PAREN_START**: `[.!?][)\]}\s+[({\[]`
   - Example: `"He left (quietly.) (She followed.)"`

## Implementation Strategy

### Phase 1: Immediate Fix (Current Session)
- Add missing `QUOTE_END + QUOTE_START` pattern to achieve 5/5 dialog detection
- Validate against current test scenarios

### Phase 2: Compositional Refactor (Future Session)
- Implement component-based pattern generation
- Replace manual pattern enumeration with systematic composition
- Add comprehensive test coverage for all combinations

### Phase 3: Advanced Context (Future Enhancement)
- Implement context-aware false positive detection
- Add support for multi-line boundaries
- Handle Unicode punctuation and international quotes

## Benefits of Compositional Approach

1. **Systematic Coverage**: Ensures all valid combinations are handled
2. **Maintainability**: Adding new sentence end/start types automatically generates all combinations
3. **Testability**: Can systematically test each component and combination
4. **Extensibility**: Easy to add new punctuation types, quote styles, or languages
5. **Documentation**: Self-documenting through component names and structure

## Performance Considerations

### Regex Optimization
- Single combined regex vs multiple pattern matching
- DFA compilation time vs runtime performance
- Memory usage for large pattern combinations

### Benchmarking Strategy
- Compare compositional approach against current manual enumeration
- Measure impact on Gutenberg throughput benchmarks
- Validate quality improvements don't significantly impact performance

## Risk Assessment

### Low Risk
- Adding missing patterns to current approach
- Component organization and documentation

### Medium Risk  
- Refactoring to compositional pattern generation
- Ensuring all edge cases are covered in systematic approach

### High Risk
- Performance regression from more complex regex patterns
- Over-generalization leading to false positives

## Conclusion

The compositional approach provides a principled foundation for sentence boundary detection that can systematically handle all valid sentence ending and starting combinations. The immediate focus should be on fixing the missing `QUOTE_END + QUOTE_START` pattern, followed by a systematic refactor to the compositional approach in a future session.

This design enables both the current quality goals (9/9 abbreviations, 5/5 dialog) and provides a robust foundation for future enhancements including international text support and advanced formatting scenarios.