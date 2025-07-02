use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::discovery;
use rs_sft_sentences::reader;
use rs_sft_sentences::sentence_detector::{SentenceBoundaryRules, SentenceDetector};
use std::path::PathBuf;

const SIMPLE_TEXT: &str = "Hello world. This is a test. How are you?";
const COMPLEX_TEXT: &str = r#"
    "Mr. & Mrs. Smith," she said, "went to Washington, D.C. last week." 
    He replied, 'I saw them there.' It was a surprise!
"#;
const LONG_TEXT: &str = include_str!("../tests/fixtures/long_text.txt");

fn bench_sentence_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("sentence_detection");

    // Benchmark FST compilation
    group.bench_function("fst_compilation", |b| {
        b.iter(|| {
            let rules = SentenceBoundaryRules::default();
            black_box(SentenceDetector::new(rules).unwrap());
        })
    });

    let detector = SentenceDetector::with_default_rules().unwrap();

    // Benchmark simple text
    group.bench_function("simple_text_detection", |b| {
        b.iter(|| {
            detector.detect_sentences(black_box(SIMPLE_TEXT)).unwrap();
        })
    });

    // Benchmark complex text
    group.bench_function("complex_text_detection", |b| {
        b.iter(|| {
            detector.detect_sentences(black_box(COMPLEX_TEXT)).unwrap();
        })
    });

    // Benchmark long text
    group.bench_function("long_text_throughput", |b| {
        b.iter(|| {
            detector.detect_sentences(black_box(LONG_TEXT)).unwrap();
        })
    });

    group.finish();
}

fn bench_gutenberg_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Discover and read files
    let all_text = rt.block_on(async {
        let mirror_dir = match std::env::var("GUTENBERG_MIRROR_DIR") {
            Ok(dir) => dir,
            Err(_) => {
                eprintln!("GUTENBERG_MIRROR_DIR not set, skipping throughput benchmark.");
                return String::new();
            }
        };
        let root_dir = PathBuf::from(mirror_dir);

        if !root_dir.exists() {
            eprintln!("GUTENberg_MIRROR_DIR does not exist, skipping throughput benchmark.");
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

    let detector = SentenceDetector::with_default_rules().unwrap();

    group.bench_function("detection_chars_per_sec", |b| {
        b.iter(|| {
            detector.detect_sentences(black_box(&all_text)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sentence_detection, bench_gutenberg_throughput);
criterion_main!(benches);
