use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rs_sft_sentences::sentence_detector::{SentenceBoundaryRules, SentenceDetector, SentenceDetectorDFA};

const TEST_SIZES: &[(usize, &str)] = &[
    (100, "small"),
    (1000, "medium"), 
    (10000, "large"),
];

fn generate_test_text(char_count: usize) -> String {
    let base_sentences = [
        "Hello world.",
        "This is a test sentence.",
        "How are you doing today?",
        "The quick brown fox jumps over the lazy dog.",
        "Rust is a systems programming language!",
        "Performance benchmarks are important.",
        "Regular expressions can be very fast.",
        "DFA implementations provide O(n) guarantees.",
    ];
    
    let mut text = String::new();
    let mut current_len = 0;
    let mut sentence_idx = 0;
    
    while current_len < char_count {
        let sentence = base_sentences[sentence_idx % base_sentences.len()];
        text.push_str(sentence);
        text.push(' ');
        current_len += sentence.len() + 1;
        sentence_idx += 1;
    }
    
    text.truncate(char_count);
    text
}

fn bench_dfa_vs_manual_comparison(c: &mut Criterion) {
    let manual_detector = SentenceDetector::with_default_rules().unwrap();
    let dfa_detector = SentenceDetectorDFA::new().unwrap();
    
    for &(size, size_name) in TEST_SIZES {
        let test_text = generate_test_text(size);
        
        let mut group = c.benchmark_group(format!("sentence_detection_{}", size_name));
        group.throughput(Throughput::Bytes(test_text.len() as u64));
        
        group.bench_function("manual", |b| {
            b.iter(|| {
                manual_detector.detect_sentences(black_box(&test_text)).unwrap();
            })
        });
        
        group.bench_function("dfa", |b| {
            b.iter(|| {
                dfa_detector.detect_sentences(black_box(&test_text)).unwrap();
            })
        });
        
        group.finish();
    }
}

fn bench_compilation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");
    
    group.bench_function("manual_fst", |b| {
        b.iter(|| {
            let rules = SentenceBoundaryRules::default();
            black_box(SentenceDetector::new(rules).unwrap());
        })
    });
    
    group.bench_function("dfa", |b| {
        b.iter(|| {
            black_box(SentenceDetectorDFA::new().unwrap());
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_compilation_overhead, bench_dfa_vs_manual_comparison);
criterion_main!(benches);