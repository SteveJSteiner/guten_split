# Python Module Specification

* **Task ID:** python-module-spec_42.stevejs
* **Reviewer:** stevejs
* **Area:** docs
* **Motivation (WHY):**
  - Enable Python developers to use seams functionality programmatically for high-throughput sentence extraction
  - Provide both individual text processing and bulk file processing capabilities
  - Leverage Rust performance while maintaining Python ergonomics
  - Support async workflows for I/O-bound corpus processing
* **Acceptance Criteria:**
  1. Complete API specification covering all core functionality
  2. Clear documentation of class-based primary API with functional wrappers
  3. Performance characteristics documented (DFA reuse benefits)
  4. Async file discovery and processing pipeline specified
  5. PyPI package structure and naming defined
* **Deliverables:**
  - Updated PRD.md with Python module requirement section
  - Detailed API specification document
  - Python package structure and build requirements
  - Integration points between Rust crate and Python bindings
* **References:**
  - Current lib.rs exports: DetectedSentence, Span, SentenceDetectorDialog
  - PRD.md functional requirements F-1 through F-11
  - PyO3/maturin ecosystem for native extensions

## API Design Specification

### Core Classes

**SentenceDetector** (primary API)
- `__init__()` - Compile sentence boundary DFA once
- `detect_sentences(text: str) -> List[DetectedSentence]` - Process single text
- `process_file(path: str, use_mmap: bool = False) -> List[DetectedSentence]` - Process file with backend choice
- `process_files(paths: List[str], use_mmap: bool = False) -> Dict[str, List[DetectedSentence]]` - Batch process

**FileDiscovery** (async pipeline)
- `discover_files(root_dir: str, pattern: str = "**/*-0.txt") -> AsyncIterator[str]` - Async file discovery
- `process_corpus(root_dir: str, output_dir: str = None, **kwargs) -> CorpusStats` - Full pipeline

### Data Types

**DetectedSentence** (mirrors Rust)
- `sentence: str` - Normalized sentence text
- `span: Span` - Source location (line/col positions)
- `index: int` - Sentence index in source

**Span** (mirrors Rust)
- `start_line: int`, `start_col: int`
- `end_line: int`, `end_col: int`

**CorpusStats** (mirrors run_stats.json)
- `total_files: int`
- `total_sentences: int`
- `total_chars: int`
- `chars_per_second: float`
- `per_file_stats: Dict[str, FileStats]`

### Functional API (convenience wrappers)

```python
# Simple usage
sentences = seams.detect_sentences(text)
file_sentences = seams.process_file(path)

# Corpus processing
stats = seams.process_corpus(root_dir)
```

### Async API

```python
async def process_large_corpus():
    async for filepath in seams.discover_files("/gutenberg"):
        sentences = await seams.process_file_async(filepath)
        # ... handle sentences
```

## Technical Requirements

### PyPI Package
- **Name**: `knowseams` (or similar available variant)
- **Structure**: Separate repository with seams as crates.io dependency
- **Build**: maturin for PyO3 native extensions
- **Distribution**: Python wheels for major platforms (Linux x86_64/aarch64, Windows, macOS)

### Rust Integration
- seams crate published to crates.io as pure Rust library
- Python bindings crate depends on published seams crate
- Expose async runtime integration (tokio compatibility)
- Support both buffered I/O and mmap backends

### Performance Considerations
- DFA compilation happens once per SentenceDetector instance
- Async file processing uses same work-stealing scheduler as CLI
- Memory efficiency through Rust borrow semantics where possible
- Zero-copy string handling where PyO3 allows

## Implementation Phases

1. **Phase 1**: Core SentenceDetector class with sync API
2. **Phase 2**: File processing and discovery integration
3. **Phase 3**: Async API and corpus processing pipeline
4. **Phase 4**: PyPI packaging and distribution setup

## Open Questions
- Should we expose mmap backend selection per-method or per-instance?
- How to handle Rust panic boundaries in Python context?
- What level of logging integration with Python logging module?
- Should functional API create new detector instances or reuse singleton?

## Pre-commit checklist:
- [ ] All deliverables implemented
- [ ] PRD.md updated with Python module requirements
- [ ] API specification covers all core functionality
- [ ] Performance characteristics documented
- [ ] PyPI package structure defined
- [ ] Integration points with Rust crate specified