use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::discovery;
#[allow(unused_imports)]
use rs_sft_sentences::reader;
use rs_sft_sentences::sentence_detector::{DetectedSentence, Span, SentenceBoundaryRules, SentenceDetector, SentenceDetectorDFA};
use std::path::PathBuf;
use regex_automata::meta::Regex;

// Import dialog state machine for benchmarking
mod dialog_state_machine_exploration {
    include!("../tests/dialog_state_machine_exploration.rs");
}

const SIMPLE_TEXT: &str = "Hello world. This is a test. How are you?";
const COMPLEX_TEXT: &str = r#"
    "Mr. & Mrs. Smith," she said, "went to Washington, D.C. last week." 
    He replied, 'I saw them there.' It was a surprise!
"#;
const LONG_TEXT: &str = include_str!("../tests/fixtures/long_text.txt");

// Dictionary Post-Processing Strategy for Benchmark
// WHY: Test performance impact of abbreviation handling vs baseline DFA
#[allow(dead_code)]
fn detect_sentences_dictionary_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
    ];
    
    // Phase 1: Find all potential sentence boundaries using simple pattern
    let pattern = Regex::new(r"[.!?]\s+[A-Z]").unwrap();
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    
    for mat in pattern.find_iter(text) {
        let potential_end = mat.start() + 1; // Position after the punctuation
        let preceding_text = &text[last_start..potential_end];
        
        // Phase 2: Check if this ends with a known abbreviation
        let is_abbreviation = abbreviations.iter().any(|abbrev| {
            preceding_text.trim_end().ends_with(abbrev)
        });
        
        if !is_abbreviation {
            // This is a real sentence boundary
            let sentence_text = &text[last_start..potential_end];
            
            // Simple position calculation - just use 1-based line/col
            // WHY: Focus on performance comparison rather than exact span calculation
            sentences.push(DetectedSentence {
                index: sentence_index,
                normalized_content: sentence_text.to_string(),
                span: Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: potential_end + 1,
                },
            });
            sentence_index += 1;
            
            // Skip whitespace to start of next sentence
            let next_start = mat.end() - 1; // Position after the space
            last_start = text[next_start..].chars()
                .position(|c| !c.is_whitespace())
                .map(|pos| next_start + pos)
                .unwrap_or(text.len());
        }
    }
    
    // Add final sentence if there's remaining text
    if last_start < text.len() {
        let final_text = &text[last_start..];
        if !final_text.trim().is_empty() {
            sentences.push(DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.to_string(),
                span: Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: text.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Enhanced Dictionary Strategy for Benchmark - Full Feature Support
// WHY: Matches manual detector functionality while adding abbreviation filtering
#[allow(dead_code)]
fn detect_sentences_dictionary_enhanced_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    // For benchmark performance comparison, use same simple pattern as dictionary strategy
    // but add abbreviation filtering. Complex patterns would require careful UTF-8 handling.
    detect_sentences_dictionary_benchmark(text)
}

// Context Analysis Strategy for Benchmark
// WHY: Benchmark sophisticated context analysis approach  
#[allow(dead_code)]
fn detect_sentences_context_analysis_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
    ];
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    let chars: Vec<char> = text.chars().collect();
    
    let mut i = 0;
    while i < chars.len() {
        if matches!(chars[i], '.' | '!' | '?') {
            if i + 2 < chars.len() && chars[i + 1].is_whitespace() {
                let next_char = chars[i + 2];
                let is_sentence_start = next_char.is_uppercase() || 
                                      matches!(next_char, '"' | '\'' | '(' | '[');
                
                if is_sentence_start {
                    // Simple abbreviation detection for benchmark
                    let context_start = if i >= 10 { i - 10 } else { 0 };
                    let context: String = chars[context_start..=i].iter().collect();
                    let is_abbreviation = abbreviations.iter().any(|abbrev| context.ends_with(abbrev));
                    
                    if !is_abbreviation {
                        let sentence_text: String = chars[last_start..=i].iter().collect();
                        sentences.push(DetectedSentence {
                            index: sentence_index,
                            normalized_content: sentence_text.trim().to_string(),
                            span: Span {
                                start_line: 1,
                                start_col: last_start + 1,
                                end_line: 1,
                                end_col: i + 2,
                            },
                        });
                        sentence_index += 1;
                        
                        last_start = i + 2;
                        while last_start < chars.len() && chars[last_start].is_whitespace() {
                            last_start += 1;
                        }
                        i = last_start.saturating_sub(1);
                    }
                }
            }
        }
        i += 1;
    }
    
    if last_start < chars.len() {
        let final_text: String = chars[last_start..].iter().collect();
        if !final_text.trim().is_empty() {
            sentences.push(DetectedSentence {
                index: sentence_index,
                normalized_content: final_text.trim().to_string(),
                span: Span {
                    start_line: 1,
                    start_col: last_start + 1,
                    end_line: 1,
                    end_col: chars.len() + 1,
                },
            });
        }
    }
    
    Ok(sentences)
}

// Multi-Pattern DFA Strategy for Benchmark
// WHY: Test performance with combined pattern DFA approach
#[allow(dead_code)]
fn detect_sentences_multipattern_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    use regex_automata::dfa::{dense::DFA, Automaton};
    
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
        "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
        "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
        "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
    ];
    
    // Combine all patterns into one optimized DFA
    let combined_pattern = r#"[.!?](?:['"\u{201D}\u{2019}])?\s+(?:[A-Z]|['"\u{201C}\u{2018}]|[({\[])"#;
    let dfa = DFA::new(combined_pattern)?;
    
    let mut sentences = Vec::new();
    let mut sentence_index = 0;
    let mut last_start = 0;
    let mut search_at = 0;
    let text_bytes = text.as_bytes();
    
    while search_at < text_bytes.len() {
        if let Some(match_result) = dfa.try_search_fwd(&regex_automata::Input::new(&text_bytes[search_at..]))? {
            let match_start = search_at + match_result.offset();
            
            // Find punctuation position
            let mut punct_pos = match_start;
            while punct_pos < text_bytes.len() {
                let byte = text_bytes[punct_pos];
                if byte == b'.' || byte == b'!' || byte == b'?' {
                    break;
                }
                punct_pos += 1;
            }
            
            let potential_end = punct_pos + 1;
            let sentence_bytes = &text_bytes[last_start..potential_end];
            
            if let Ok(preceding_text) = std::str::from_utf8(sentence_bytes) {
                let is_abbreviation = abbreviations.iter().any(|abbrev| {
                    preceding_text.trim_end().ends_with(abbrev)
                });
                
                if !is_abbreviation {
                    sentences.push(DetectedSentence {
                        index: sentence_index,
                        normalized_content: preceding_text.to_string(),
                        span: Span {
                            start_line: 1,
                            start_col: last_start + 1,
                            end_line: 1,
                            end_col: potential_end + 1,
                        },
                    });
                    sentence_index += 1;
                    
                    // Move to next sentence
                    last_start = potential_end;
                    while last_start < text_bytes.len() && text_bytes[last_start].is_ascii_whitespace() {
                        last_start += 1;
                    }
                }
            }
            
            search_at = match_start + 1;
        } else {
            break;
        }
    }
    
    // Add final sentence
    if last_start < text_bytes.len() {
        if let Ok(final_text) = std::str::from_utf8(&text_bytes[last_start..]) {
            if !final_text.trim().is_empty() {
                sentences.push(DetectedSentence {
                    index: sentence_index,
                    normalized_content: final_text.to_string(),
                    span: Span {
                        start_line: 1,
                        start_col: last_start + 1,
                        end_line: 1,
                        end_col: text_bytes.len() + 1,
                    },
                });
            }
        }
    }
    
    Ok(sentences)
}

// Dialog State Machine Strategy for Benchmark
// WHY: Test performance of dialog-aware sentence boundary detection with coalescing
fn detect_sentences_dialog_state_machine_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    use dialog_state_machine_exploration::DialogStateMachine;
    
    let dialog_machine = DialogStateMachine::new()?;
    let dialog_sentences = dialog_machine.detect_sentences(text)?;
    
    // Convert from dialog state machine format to benchmark format
    let mut sentences = Vec::new();
    for (index, sentence) in dialog_sentences.iter().enumerate() {
        sentences.push(DetectedSentence {
            index,
            normalized_content: sentence.content.clone(),
            span: Span {
                start_line: sentence.start_line.into(),
                start_col: sentence.start_col.into(),
                end_line: sentence.end_line.into(),
                end_col: sentence.end_col.into(),
            },
        });
    }
    
    Ok(sentences)
}

fn bench_sentence_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("sentence_detection");

    // Benchmark FST compilation vs DFA compilation
    group.bench_function("manual_fst_compilation", |b| {
        b.iter(|| {
            let rules = SentenceBoundaryRules::default();
            black_box(SentenceDetector::new(rules).unwrap());
        })
    });

    group.bench_function("dfa_compilation", |b| {
        b.iter(|| {
            black_box(SentenceDetectorDFA::new().unwrap());
        })
    });

    let manual_detector = SentenceDetector::with_default_rules().unwrap();
    let dfa_detector = SentenceDetectorDFA::new().unwrap();

    // Benchmark simple text - manual vs DFA
    group.bench_function("manual_simple_text", |b| {
        b.iter(|| {
            manual_detector.detect_sentences(black_box(SIMPLE_TEXT)).unwrap();
        })
    });

    group.bench_function("dfa_simple_text", |b| {
        b.iter(|| {
            dfa_detector.detect_sentences(black_box(SIMPLE_TEXT)).unwrap();
        })
    });

    // Benchmark complex text - manual vs DFA
    group.bench_function("manual_complex_text", |b| {
        b.iter(|| {
            manual_detector.detect_sentences(black_box(COMPLEX_TEXT)).unwrap();
        })
    });

    group.bench_function("dfa_complex_text", |b| {
        b.iter(|| {
            dfa_detector.detect_sentences(black_box(COMPLEX_TEXT)).unwrap();
        })
    });

    // Benchmark long text - manual vs DFA
    group.bench_function("manual_long_text", |b| {
        b.iter(|| {
            manual_detector.detect_sentences(black_box(LONG_TEXT)).unwrap();
        })
    });

    group.bench_function("dfa_long_text", |b| {
        b.iter(|| {
            dfa_detector.detect_sentences(black_box(LONG_TEXT)).unwrap();
        })
    });

    group.finish();
}

fn bench_gutenberg_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Discover and read files
    let (all_text, char_count, _file_info) = rt.block_on(async {
        let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/gutenberg_texts", home)
            });
        let root_dir = PathBuf::from(mirror_dir);

        if !root_dir.exists() {
            eprintln!("Gutenberg mirror directory {:?} does not exist, skipping throughput benchmark.", root_dir);
            return (String::new(), 0, Vec::new());
        }

        let discovery_config = discovery::DiscoveryConfig::default();
        let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
            .await
            .unwrap_or_else(|_| Vec::new());

        let valid_files: Vec<_> = discovered_files
            .iter()
            .filter(|f| f.is_valid_utf8 && f.error.is_none())
            .take(10) // Take first 10 valid files
            .collect();

        if valid_files.is_empty() {
            eprintln!("No valid files found for throughput benchmark");
            return (String::new(), 0, Vec::new());
        }

        let mut content = String::new();
        let mut file_info = Vec::new();
        
        println!("=== Benchmark Data Transparency ===");
        println!("Processing {} files:", valid_files.len());
        
        for file in &valid_files {
            if let Ok(file_content) = tokio::fs::read_to_string(&file.path).await {
                let file_bytes = file_content.len();
                let file_chars = file_content.chars().count();
                
                println!("  {:?}: {} bytes, {} characters", file.path, file_bytes, file_chars);
                file_info.push((file.path.clone(), file_bytes, file_chars));
                content.push_str(&file_content);
            }
        }
        
        let total_chars = content.chars().count();
        println!("Total: {} files, {} bytes, {} characters", valid_files.len(), content.len(), total_chars);
        println!("======================================");
        
        (content, total_chars, file_info)
    });

    if all_text.is_empty() {
        eprintln!("No content read for throughput benchmark, skipping.");
        return;
    }

    let mut group = c.benchmark_group("gutenberg_throughput");
    group.throughput(Throughput::Elements(char_count as u64));

    // Create manual FST detector
    let manual_detector = SentenceDetector::with_default_rules().unwrap();
    let dfa_detector = SentenceDetectorDFA::new().unwrap();

    // Validate equivalent work by comparing sentence detection results
    println!("=== Benchmark Work Equivalence Validation ===");
    
    let manual_result = manual_detector.detect_sentences(&all_text).unwrap();
    let dfa_result = dfa_detector.detect_sentences(&all_text).unwrap();
    let dialog_result = detect_sentences_dialog_state_machine_benchmark(&all_text).unwrap();
    
    println!("Manual FST: {} sentences detected", manual_result.len());
    println!("DFA: {} sentences detected", dfa_result.len());
    println!("Dialog State Machine: {} sentences detected", dialog_result.len());
    
    // Calculate average sentence lengths for comparison
    let manual_avg_len = if manual_result.is_empty() { 0.0 } else {
        manual_result.iter().map(|s| s.normalized_content.chars().count()).sum::<usize>() as f64 / manual_result.len() as f64
    };
    let dfa_avg_len = if dfa_result.is_empty() { 0.0 } else {
        dfa_result.iter().map(|s| s.normalized_content.chars().count()).sum::<usize>() as f64 / dfa_result.len() as f64
    };
    let dialog_avg_len = if dialog_result.is_empty() { 0.0 } else {
        dialog_result.iter().map(|s| s.normalized_content.chars().count()).sum::<usize>() as f64 / dialog_result.len() as f64
    };
    
    println!("Average sentence length:");
    println!("  Manual FST: {:.1} characters", manual_avg_len);
    println!("  DFA: {:.1} characters", dfa_avg_len);
    println!("  Dialog State Machine: {:.1} characters", dialog_avg_len);
    
    // Check for significant differences that would indicate non-equivalent work
    let max_sentences = manual_result.len().max(dfa_result.len()).max(dialog_result.len());
    let min_sentences = manual_result.len().min(dfa_result.len()).min(dialog_result.len());
    let sentence_variance = if max_sentences > 0 {
        ((max_sentences - min_sentences) as f64 / max_sentences as f64) * 100.0
    } else {
        0.0
    };
    
    println!("Sentence count variance: {:.1}%", sentence_variance);
    if sentence_variance > 10.0 {
        println!("⚠️  WARNING: >10% variance suggests non-equivalent work!");
    } else {
        println!("✅ Sentence counts are comparable (≤10% variance)");
    }
    println!("==============================================");

    group.bench_function("manual_fst_chars_per_sec", |b| {
        b.iter(|| {
            manual_detector.detect_sentences(black_box(&all_text)).unwrap();
        })
    });

    group.bench_function("dfa_chars_per_sec", |b| {
        b.iter(|| {
            dfa_detector.detect_sentences(black_box(&all_text)).unwrap();
        })
    });

    group.bench_function("dialog_state_machine_chars_per_sec", |b| {
        b.iter(|| {
            detect_sentences_dialog_state_machine_benchmark(black_box(&all_text)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sentence_detection, bench_gutenberg_throughput);
criterion_main!(benches);
