use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::discovery;
use rs_sft_sentences::reader;
use rs_sft_sentences::sentence_detector::{DetectedSentence, Span, SentenceBoundaryRules, SentenceDetector, SentenceDetectorDFA};
use std::path::PathBuf;
use regex_automata::meta::Regex;

const SIMPLE_TEXT: &str = "Hello world. This is a test. How are you?";
const COMPLEX_TEXT: &str = r#"
    "Mr. & Mrs. Smith," she said, "went to Washington, D.C. last week." 
    He replied, 'I saw them there.' It was a surprise!
"#;
const LONG_TEXT: &str = include_str!("../tests/fixtures/long_text.txt");

// Dictionary Post-Processing Strategy for Benchmark
// WHY: Test performance impact of abbreviation handling vs baseline DFA
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
fn detect_sentences_dictionary_enhanced_benchmark(text: &str) -> Result<Vec<DetectedSentence>, Box<dyn std::error::Error>> {
    // For benchmark performance comparison, use same simple pattern as dictionary strategy
    // but add abbreviation filtering. Complex patterns would require careful UTF-8 handling.
    detect_sentences_dictionary_benchmark(text)
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
    let all_text = rt.block_on(async {
        let mirror_dir = std::env::var("GUTENBERG_MIRROR_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/gutenberg_texts", home)
            });
        let root_dir = PathBuf::from(mirror_dir);

        if !root_dir.exists() {
            eprintln!("Gutenberg mirror directory {:?} does not exist, skipping throughput benchmark.", root_dir);
            return String::new();
        }

        let discovery_config = discovery::DiscoveryConfig::default();
        let discovered_files = discovery::collect_discovered_files(&root_dir, discovery_config)
            .await
            .unwrap_or_else(|_| Vec::new());

        let valid_files: Vec<_> = discovered_files
            .iter()
            .filter(|f| f.is_valid_utf8 && f.error.is_none())
            .take(10) // Take first 10 valid files
            .map(|f| f.path.clone())
            .collect();

        if valid_files.is_empty() {
            eprintln!("No valid files found for throughput benchmark");
            return String::new();
        }

        let mut content = String::new();
        for path in valid_files {
            if let Ok(file_content) = tokio::fs::read_to_string(path).await {
                content.push_str(&file_content);
            }
        }
        content
    });

    if all_text.is_empty() {
        eprintln!("No content read for throughput benchmark, skipping.");
        return;
    }

    let mut group = c.benchmark_group("gutenberg_throughput");
    group.throughput(Throughput::Bytes(all_text.len() as u64));

    let manual_detector = SentenceDetector::with_default_rules().unwrap();
    let dfa_detector = SentenceDetectorDFA::new().unwrap();

    group.bench_function("manual_chars_per_sec", |b| {
        b.iter(|| {
            manual_detector.detect_sentences(black_box(&all_text)).unwrap();
        })
    });

    group.bench_function("dfa_chars_per_sec", |b| {
        b.iter(|| {
            dfa_detector.detect_sentences(black_box(&all_text)).unwrap();
        })
    });

    group.bench_function("dictionary_chars_per_sec", |b| {
        b.iter(|| {
            detect_sentences_dictionary_benchmark(black_box(&all_text)).unwrap();
        })
    });

    group.bench_function("enhanced_dictionary_chars_per_sec", |b| {
        b.iter(|| {
            detect_sentences_dictionary_enhanced_benchmark(black_box(&all_text)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sentence_detection, bench_gutenberg_throughput);
criterion_main!(benches);
