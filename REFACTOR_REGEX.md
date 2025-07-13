# Regex Pattern Refactor: From Manual Duplication to Table-Driven DFA Generation

## Current Problem Analysis

The current `src/sentence_detector/dialog_detector.rs` exhibits massive code duplication across dialog states:

### Duplication Evidence
- **775 lines** of pattern generation code
- **7 dialog states** × **8+ patterns each** = ~56 nearly identical pattern definitions
- Each state repeats the same pattern structure with different closing characters:
  ```rust
  // Double quotes (lines 464-476)
  let dialog_external_separation_split = format!("{double_quote_close}{sentence_end_punct}...");
  let dialog_external_separation_continue = format!("{double_quote_close}{sentence_end_punct}...");
  // ... 6 more patterns
  
  // Single quotes (lines 520-532) - IDENTICAL STRUCTURE
  let dialog_external_separation_split = format!("{single_quote_close}{sentence_end_punct}...");
  let dialog_external_separation_continue = format!("{single_quote_close}{sentence_end_punct}...");
  // ... 6 more patterns
  
  // Smart quotes, parentheses, brackets, braces - ALL IDENTICAL
  ```

### Maintenance Problems
1. **GAP #1 Fix Complexity**: Required manual replication across 6 quote types (lines 516-773)
2. **Pattern Inconsistency Risk**: Easy to introduce bugs when hand-copying patterns
3. **Modification Amplification**: Any pattern change requires 7× manual updates
4. **Code Review Burden**: 700+ lines of nearly identical code obscures actual logic

---

## Proposed Solution: Route 2 - Table-Driven DFA Generation

### Architecture: SEAM Pattern Lattice → Compile-Time DFA Fleet

**Goal**: Replace 700+ lines of duplicated Rust code with a **declarative table** that generates optimized DFAs at build time.

### Table Schema Design

```toml
# seams.toml - The single source of truth
[pattern_components]
sentence_end_punct = "[.!?]"
non_sentence_ending_punct = "[,:;]"
soft_separator = "[ \\t]+"
sentence_starts = "[A-Z0-9\"'\"\"'([{]"
not_sentence_starts = "[^A-Z0-9\"'\"\"'([{]"
dialog_open_chars = "[\"'\"\"'([{]"

[dialog_states.double_quote]
open_char = "\""
close_char = "\""
state_name = "DialogDoubleQuote"

[dialog_states.single_quote]
open_char = "'"
close_char = "'"
state_name = "DialogSingleQuote"

[dialog_states.smart_double]
open_char = """
close_char = """
state_name = "DialogSmartDoubleOpen"

[dialog_states.smart_single]
open_char = "'"
close_char = "'"
state_name = "DialogSmartSingleOpen"

[dialog_states.round_paren]
open_char = "("
close_char = ")"
state_name = "DialogParenthheticalRound"

[dialog_states.square_bracket]
open_char = "["
close_char = "]"
state_name = "DialogParenthheticalSquare"

[dialog_states.curly_brace]
open_char = "{"
close_char = "}"
state_name = "DialogParenthheticalCurly"

# Pattern templates - applied to ALL dialog states
[[pattern_templates]]
name = "external_separation_split"
regex = "{close}{sentence_end_punct}({soft_separator})[{sentence_starts}]"
match_type = "DialogEnd"
next_state = "Narrative"
comment = "External separation + sentence start → D→N + Split"

[[pattern_templates]]
name = "external_separation_continue"
regex = "{close}{sentence_end_punct}({soft_separator}){not_sentence_starts}"
match_type = "DialogSoftEnd"
next_state = "Narrative"
comment = "External separation + lowercase → D→N + Continue (lowercase overrides)"

[[pattern_templates]]
name = "external_separation_to_dialog"
regex = "{close}{sentence_end_punct}({soft_separator}){dialog_open_chars}"
match_type = "DialogOpen"
next_state = "Unknown"
comment = "External separation + dialog open → D→D + Split"

[[pattern_templates]]
name = "external_continuation"
regex = "{close}{non_sentence_ending_punct}({soft_separator})."
match_type = "DialogSoftEnd"
next_state = "Narrative"
comment = "External continuation → D→N + Continue"

[[pattern_templates]]
name = "internal_hard_end"
regex = "{sentence_end_punct}{close}[,:;]*({soft_separator})[{sentence_starts}]"
match_type = "DialogEnd"
next_state = "Narrative"
comment = "Hard End (internal punctuation)"

[[pattern_templates]]
name = "internal_soft_end_punctuated"
regex = "{sentence_end_punct}{close}[,:;]*({soft_separator}){not_sentence_starts}"
match_type = "DialogSoftEnd"
next_state = "Narrative"
comment = "Soft End (punctuated)"

[[pattern_templates]]
name = "internal_continuation"
regex = "[^.!?]{close}[,:;]*({soft_separator}){dialog_open_chars}"
match_type = "DialogOpen"
next_state = "Unknown"
comment = "Dialog Continuation"

[[pattern_templates]]
name = "internal_soft_end_unpunctuated"
regex = "[^.!?]{close}[,:;]*({soft_separator})[^{dialog_open_chars}]"
match_type = "DialogSoftEnd"
next_state = "Narrative"
comment = "Soft End (unpunctuated)"
```

### Build Script Implementation

```rust
// build.rs
use regex_automata::dfa::dense;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct SeamConfig {
    pattern_components: HashMap<String, String>,
    dialog_states: HashMap<String, DialogStateConfig>,
    pattern_templates: Vec<PatternTemplate>,
}

#[derive(Deserialize)]
struct DialogStateConfig {
    open_char: String,
    close_char: String,
    state_name: String,
}

#[derive(Deserialize)]
struct PatternTemplate {
    name: String,
    regex: String,
    match_type: String,
    next_state: String,
    comment: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: SeamConfig = toml::from_str(include_str!("seams.toml"))?;
    let out_dir = std::env::var("OUT_DIR")?;
    
    // Generate DFAs for each dialog state
    for (state_key, state_config) in &config.dialog_states {
        let mut patterns = Vec::new();
        let mut mappings = Vec::new();
        
        // Apply each pattern template to this dialog state
        for template in &config.pattern_templates {
            let pattern = expand_template(template, state_config, &config.pattern_components)?;
            patterns.push(pattern);
            mappings.push((template.match_type.clone(), template.next_state.clone()));
        }
        
        // Build multi-pattern DFA for this state
        let dfa = dense::DFA::new_many(&patterns)?;
        let bytes = dfa.to_bytes_little_endian()?;
        
        // Write DFA binary
        let dfa_path = format!("{}/dialog_{}_dfa.bin", out_dir, state_key);
        std::fs::write(&dfa_path, bytes)?;
        
        // Write pattern mappings
        let mappings_path = format!("{}/dialog_{}_mappings.json", out_dir, state_key);
        let mappings_json = serde_json::to_string(&mappings)?;
        std::fs::write(&mappings_path, mappings_json)?;
    }
    
    // Generate Rust code with static DFA loading
    generate_rust_code(&config, &out_dir)?;
    
    println!("cargo:rerun-if-changed=seams.toml");
    Ok(())
}

fn expand_template(
    template: &PatternTemplate, 
    state: &DialogStateConfig, 
    components: &HashMap<String, String>
) -> Result<String, Box<dyn std::error::Error>> {
    let mut pattern = template.regex.clone();
    
    // Replace state-specific placeholders
    pattern = pattern.replace("{close}", &state.close_char);
    pattern = pattern.replace("{open}", &state.open_char);
    
    // Replace component placeholders
    for (key, value) in components {
        pattern = pattern.replace(&format!("{{{}}}", key), value);
    }
    
    Ok(pattern)
}

fn generate_rust_code(config: &SeamConfig, out_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n");
    code.push_str("use regex_automata::dfa::dense::DFA;\n");
    code.push_str("use std::collections::HashMap;\n\n");
    
    // Generate DFA constants
    for (state_key, state_config) in &config.dialog_states {
        code.push_str(&format!(
            "pub static {}_DFA: DFA<&[u32], u32> = {{\n",
            state_key.to_uppercase()
        ));
        code.push_str(&format!(
            "    const BYTES: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/dialog_{}_dfa.bin\"));\n",
            state_key
        ));
        code.push_str("    unsafe { DFA::from_bytes_unchecked(BYTES) }\n");
        code.push_str("};\n\n");
    }
    
    // Generate mapping loader
    code.push_str("pub fn load_dialog_patterns() -> HashMap<DialogState, (DFA<&'static [u32], u32>, Vec<(MatchType, DialogState)>)> {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    for (state_key, state_config) in &config.dialog_states {
        code.push_str(&format!(
            "    let {}_mappings: Vec<(MatchType, DialogState)> = serde_json::from_str(include_str!(concat!(env!(\"OUT_DIR\"), \"/dialog_{}_mappings.json\"))).unwrap();\n",
            state_key, state_key
        ));
        code.push_str(&format!(
            "    map.insert(DialogState::{}, ({}_DFA, {}_mappings));\n",
            state_config.state_name, state_key.to_uppercase(), state_key
        ));
    }
    
    code.push_str("    map\n");
    code.push_str("}\n");
    
    std::fs::write(format!("{}/generated_patterns.rs", out_dir), code)?;
    Ok(())
}
```

### Refactored DialogStateMachine

```rust
// src/sentence_detector/dialog_detector.rs (refactored)
include!(concat!(env!("OUT_DIR"), "/generated_patterns.rs"));

impl DialogStateMachine {
    pub fn new() -> Result<Self> {
        let pattern_data = load_dialog_patterns();
        let mut state_patterns = HashMap::new();
        let mut state_pattern_mappings = HashMap::new();
        
        for (state, (dfa, mappings)) in pattern_data {
            state_patterns.insert(state.clone(), dfa);
            state_pattern_mappings.insert(state, mappings);
        }
        
        Ok(DialogStateMachine {
            state_patterns,
            state_pattern_mappings,
            abbreviation_checker: AbbreviationChecker::new(),
        })
    }
}
```

---

## Benefits Analysis

### Code Reduction
- **Before**: 775 lines of repetitive pattern generation
- **After**: ~50 lines of table data + generated code
- **Maintenance Burden**: 95% reduction

### Pattern Consistency  
- **Single Source of Truth**: All patterns defined once in table
- **Impossible Duplication Bugs**: Compiler ensures consistency across states
- **Easy Pattern Evolution**: Modify template once, affects all states

### Performance Optimization
- **Compile-Time DFA Generation**: Zero runtime regex compilation cost
- **Optimized DFAs**: `regex-automata` produces highly optimized state machines
- **Binary Size**: Pre-built DFAs vs runtime regex compiler

### Development Workflow
- **Pattern Authors**: Edit declarative TOML (readable, reviewable)
- **Compiler**: Generates optimized Rust code automatically  
- **Runtime**: Zero-cost abstractions with compile-time guarantees

---

## Implementation Strategy

### Phase 1: Validation Using Individual Sentence Output Files
**CRITICAL**: Use existing `benchmarks/run_comparison.py` infrastructure to generate `*_seams2.txt` files for behavioral equivalence testing.

**Validation Strategy**:
1. Current implementation creates: `/home/steve/gutenberg/4/2/7/0/42701/42701-0_seams.txt` (existing)
2. Refactored implementation creates: `/home/steve/gutenberg/4/2/7/0/42701/42701-0_seams2.txt` (new suffix)
3. Diff comparison across 20K file pairs for comprehensive validation
4. Success = identical sentence detection outputs across real-world corpus

```bash
# Validation workflow
# 1. Baseline already exists from previous runs (42701-0_seams.txt files)

# 2. Run refactored implementation with modified output suffix
# Modify seams binary to output *_seams2.txt instead of *_seams.txt
cargo build --release --features="table-driven-patterns"
python benchmarks/run_comparison.py /home/steve/gutenberg --seams-only

# 3. Compare sentence outputs file-by-file
# For each Project Gutenberg file, compare:
#   42701-0_seams.txt  (baseline)
#   42701-0_seams2.txt (refactored)
find /home/steve/gutenberg -name "*_seams.txt" | while read baseline; do
    refactored="${baseline%_seams.txt}_seams2.txt"
    if [ -f "$refactored" ]; then
        if ! diff -q "$baseline" "$refactored" >/dev/null; then
            echo "DIFFERENCE: $baseline vs $refactored"
        fi
    else
        echo "MISSING: $refactored"
    fi
done

# Success = no differences reported across all 20K file pairs
```

**Benefits of Individual File Validation**:
- **Exact sentence-level comparison**: Line-by-line diff of actual sentence detection output
- **Real-world data**: 20K Project Gutenberg files with complex dialog patterns  
- **Non-destructive**: `*_seams2.txt` suffix preserves existing baseline files
- **Comprehensive coverage**: Every sentence boundary decision validated across corpus

### Phase 2: Table-to-Code Implementation
1. TOML schema already designed and created (`seams.toml`)
2. Update build script to parse TOML instead of hardcoded patterns
3. Implement template expansion system

### Phase 3: Full Migration
1. Generate all dialog states from table
2. Run comprehensive differential testing
3. Replace manual code with generated code
4. Remove duplicated pattern definitions

### Phase 4: Enhancement Opportunities
1. **Pattern Verification**: Build-script can verify pattern completeness
2. **Documentation Generation**: Auto-generate pattern documentation from table
3. **Test Case Generation**: Generate test cases from pattern combinations

---

## Risk Mitigation

### Behavioral Equivalence
- **Comprehensive Test Suite**: Cover all dialog state × pattern combinations
- **Golden Dataset Validation**: Ensure identical results on large text corpus
- **Edge Case Testing**: Multi-byte characters, complex nesting, abbreviations

### Build Complexity
- **Fallback Strategy**: Keep current code during transition period
- **Incremental Migration**: Start with one dialog state, expand gradually
- **CI Integration**: Fail fast on table→DFA generation errors

### Debug Experience  
- **Pattern Traceability**: Generated code includes comments linking back to table
- **Debug Mode**: Optional feature to dump expanded patterns for inspection
- **Error Messages**: Clear mapping from table entries to compilation errors

---

## Decision Required

**Should we proceed with Route 2 (Table-Driven DFA Generation) for the dialog_detector.rs refactor?**

**Advantages**: 
- Massive code reduction (95% less pattern code)
- Compile-time optimization (pre-built DFAs)
- Perfect pattern consistency (impossible to have duplication bugs)
- Declarative authoring experience (edit table, not Rust code)

**Risks**:
- Build script complexity  
- Requires comprehensive differential testing
- Learning curve for pattern table schema

**Recommendation**: **PROCEED** - The duplication problem is severe enough to justify the refactor investment, and the table-driven approach aligns perfectly with the DFA-first architecture goal.