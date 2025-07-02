# FST Sentence Boundary Detection

* **Task ID:** fst-sentence-detection_4
* **Reviewer:** stevejs
* **Area:** code
* **Motivation (WHY):**
  - Implement F-3: compile sentence-boundary spec into an immutable FST at startup
  - Implement F-5: detect sentence boundaries with FST, producing indexed output format
  - Implement F-6: normalize sentences by removing interior hard line breaks (preserving all other bytes)
  - Create configurable ruleset system allowing simple initial rules and future complex rulesets
* **Acceptance Criteria:**
  1. `cargo test` passes with unit tests for FST compilation and sentence detection
  2. Compiles sentence boundary ruleset into FST at startup using `fst` crate
  3. Default ruleset detects: end punctuation (. ? ! " ') + space + (Capital letter | open quote | open parenthetical)
  4. Uses Unicode-aware character matching for punctuation and letter classification
  5. Normalizes sentences: replaces interior `\n` and `\r\n` with single spaces, preserves all other bytes
  6. Produces output format: `index<TAB>normalized_sentence<TAB>(start_line,start_col,end_line,end_col)`
  7. Supports configurable ruleset specification (simple format for initial implementation)
  8. Integrates with existing file reader to process discovered text files
* **Deliverables:**
  - `src/sentence_detector.rs` module with FST compilation and boundary detection
  - Default sentence boundary ruleset specification (embedded or external file)
  - Unit tests covering sentence boundary detection, normalization, and Unicode characters
  - Integration with reader module to demonstrate end-to-end sentence extraction
  - Span tracking implementation (1-based line/column positions in Unicode scalar values)
  - Sentence normalization logic (interior line break removal)
* **References:**
  - PRD F-3: Compile sentence-boundary spec into immutable FST at startup
  - PRD F-5: Detect sentence boundaries producing indexed output with spans
  - PRD F-6: Normalize sentences by removing hard line breaks, treat \r\n as single break
  - PRD section 2: Span definition (1-based, Unicode scalar values)
  - PRD section 11: Open question about sentence-boundary DSL format