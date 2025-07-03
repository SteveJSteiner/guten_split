use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::discovery;
use rs_sft_sentences::reader;
use rs_sft_sentences::sentence_detector::{SentenceBoundaryRules, SentenceDetector, SentenceDetectorDFA};
use std::path::PathBuf;

const SIMPLE_TEXT: &str = "Hello world. This is a test. How are you?";
const COMPLEX_TEXT: &str = r#"
    "Mr. & Mrs. Smith," she said, "went to Washington, D.C. last week." 
    He replied, 'I saw them there.' It was a surprise!
"#;
const LONG_TEXT: &str = include_str!("../tests/fixtures/long_text.txt");

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

    group.finish();
}

criterion_group!(benches, bench_sentence_detection, bench_gutenberg_throughput);
criterion_main!(benches);
