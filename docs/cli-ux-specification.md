# CLI User Experience Specification

## Overview

The `seams` CLI is designed for narrative analysis pipeline builders who need a high-performance, hassle-free tool for sentence extraction from Project Gutenberg texts. This specification defines the user experience requirements for installation, usage, error handling, and integration with automated workflows.

## Target Audience

**Primary Users**: Developers and researchers building narrative analysis pipelines who need:
- Fast sentence extraction with span metadata
- Reliable batch processing of large text corpora
- Integration with existing CI/CD and data processing workflows
- Minimal setup and configuration overhead

## Installation Experience

### Requirement: Zero-friction Installation
```bash
cargo install seams
```

**Success Criteria:**
- Single command installation from crates.io
- No additional dependencies or setup required
- Cross-platform compatibility (macOS, Windows, Linux)
- Clear error messages if installation fails

### Verification Commands
```bash
# Verify installation
seams --version
# Should output: seams 0.1.0

# Verify help access
seams --help
# Should show comprehensive usage guide
```

## Command Line Interface

### Help System

**Comprehensive `--help` Output:**
```
seams 0.1.0
High-throughput sentence extractor for Project Gutenberg texts

USAGE:
    seams [OPTIONS] <ROOT_DIR>

ARGS:
    <ROOT_DIR>    Directory containing Project Gutenberg *-0.txt files

OPTIONS:
    --overwrite_all       Overwrite even complete aux files
    --fail_fast          Abort on first error (default: continue with logging)
    --use_mmap           Use memory-mapped I/O for benchmarking
    --no_progress        Suppress progress bars (useful for CI)
    --stats_out <PATH>   Stats output file [default: run_stats.json]
    -h, --help           Print help information
    -V, --version        Print version information

EXAMPLES:
    # Process Project Gutenberg mirror
    seams /path/to/gutenberg

    # Process with progress suppressed for CI
    seams --no_progress /path/to/gutenberg

    # Overwrite existing results
    seams --overwrite_all /path/to/gutenberg

    # Benchmark with memory-mapped I/O
    seams --use_mmap /path/to/gutenberg

OUTPUT:
    Creates *_seams.txt files alongside source files with format:
    <index>	<sentence>	(<start_line>,<start_col>,<end_line>,<end_col>)
```

### Progress Feedback

**Interactive Mode (default):**
- Multi-line progress bars showing:
  - Files processed/total
  - Current throughput (MB/s, chars/s)
  - ETA
  - Current file being processed

**CI/Pipeline Mode (`--no_progress`):**
- Structured log output only
- No interactive progress bars
- Suitable for automated environments

## Error Handling

### Actionable Error Messages

**File Access Errors:**
```
Error: Cannot access root directory '/nonexistent/path'
→ Check that the path exists and you have read permissions
→ Use 'ls -la /nonexistent' to verify directory status
```

**Invalid UTF-8 Files:**
```
Error: Invalid UTF-8 in file '/path/to/file-0.txt'
→ This file contains non-UTF-8 content and will be skipped
→ Use '--fail_fast' to abort processing on first error
→ Check file encoding with 'file /path/to/file-0.txt'
```

**Permission Errors:**
```
Error: Cannot write auxiliary file '/path/to/file_seams.txt'
→ Check write permissions in the target directory
→ Use 'ls -la /path/to' to verify directory permissions
```

**Memory/Performance Warnings:**
```
Warning: Large file detected (500MB+), consider using --use_mmap
→ Memory-mapped I/O may improve performance for very large files
→ Current file: /path/to/large-file-0.txt
```

### Error Recovery

**Default Behavior (continue processing):**
- Log errors with context
- Mark failed files in statistics
- Continue processing remaining files
- Exit with non-zero code if any files failed

**Fail-fast Mode (`--fail_fast`):**
- Abort immediately on first error
- Provide detailed error context
- Exit with descriptive error code

## Shell Integration

### Shell Completion

**Supported Shells:**
- bash
- zsh
- fish

**Installation:**
```bash
# Generate completion script
seams --generate-completion bash > ~/.bash_completion.d/seams
seams --generate-completion zsh > ~/.zsh/completions/_seams
seams --generate-completion fish > ~/.config/fish/completions/seams.fish
```

**Completion Features:**
- Path completion for `<ROOT_DIR>`
- Flag completion with descriptions
- Context-aware suggestions

## Integration with Workflows

### CI/CD Pipeline Integration

**Recommended Usage:**
```bash
# In CI scripts
seams --no_progress --fail_fast /data/gutenberg
if [ $? -eq 0 ]; then
    echo "Sentence extraction completed successfully"
else
    echo "Sentence extraction failed"
    exit 1
fi
```

**Output Handling:**
- Structured JSON logs for parsing
- Predictable exit codes
- Statistics in machine-readable format

### Data Pipeline Integration

**Batch Processing:**
```bash
# Process multiple corpora
for corpus in /data/gutenberg-*; do
    seams --stats_out "stats_$(basename $corpus).json" "$corpus"
done
```

**Performance Monitoring:**
```bash
# Extract performance metrics
seams /data/gutenberg 2>&1 | grep -E "(MB/s|chars/s|files processed)"
```

## Performance Characteristics

### Throughput Expectations
- **Target**: >30M characters/second on modern hardware
- **Scalability**: Utilizes multiple CPU cores automatically
- **Memory**: Bounded memory usage regardless of file size

### Cold Start Performance
- **Startup Time**: <100ms for DFA initialization
- **First File**: Processing begins immediately after startup
- **Comparison**: Superior to generic sentence detection tools

## Validation and Testing

### User Acceptance Testing
1. **Installation Test**: Fresh system, `cargo install seams` works
2. **Help Test**: `seams --help` provides comprehensive guidance
3. **Error Test**: Invalid inputs produce actionable error messages
4. **Workflow Test**: Integration with common CI/CD patterns works
5. **Performance Test**: Meets throughput requirements on reference hardware

### Success Metrics
- Time-to-first-success < 5 minutes from installation
- Error resolution rate: Users can fix common issues from error messages
- Integration success: Works in automated workflows without manual intervention