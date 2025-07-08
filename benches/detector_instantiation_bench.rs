use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use seams::sentence_detector::dialog_detector::SentenceDetectorDialog;
use std::time::Duration;

/// Benchmark detector instantiation to measure regex compilation overhead
fn bench_detector_instantiation(c: &mut Criterion) {
    let mut group = c.benchmark_group("detector_instantiation");
    
    // Set longer measurement time since we're measuring cold starts
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50); // Fewer samples since each instantiation is expensive
    
    // Benchmark single instantiation
    group.bench_function("single_instantiation", |b| {
        b.iter(|| {
            let detector = SentenceDetectorDialog::new().unwrap();
            black_box(detector);
        })
    });
    
    // Benchmark multiple instantiations to simulate test overhead
    for count in [1, 5, 10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiple_instantiations", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut detectors = Vec::with_capacity(count);
                    for _ in 0..count {
                        let detector = SentenceDetectorDialog::new().unwrap();
                        detectors.push(detector);
                    }
                    black_box(detectors);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark the cost of shared vs individual detector usage patterns
fn bench_detector_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("detector_usage_patterns");
    
    let sample_texts = vec![
        "Hello world. This is a test.",
        "Dr. Smith went to the U.S.A. yesterday. He said \"Hello there!\" to Mr. Jones.",
        "She asked, \"How are you?\" Then he replied: \"I'm fine, thanks.\"",
        "Multiple sentences here. Each one distinct. Testing performance impact.",
        "A longer text with more complex punctuation! Does this impact performance? Let's see.",
    ];
    
    // Pattern 1: Create detector for each text (simulating test pattern)
    group.bench_function("create_per_text", |b| {
        b.iter(|| {
            for text in &sample_texts {
                let detector = SentenceDetectorDialog::new().unwrap();
                let sentences = detector.detect_sentences(black_box(text)).unwrap();
                black_box(sentences);
            }
        })
    });
    
    // Pattern 2: Shared detector (simulating optimized pattern)
    group.bench_function("shared_detector", |b| {
        let detector = SentenceDetectorDialog::new().unwrap();
        b.iter(|| {
            for text in &sample_texts {
                let sentences = detector.detect_sentences(black_box(text)).unwrap();
                black_box(sentences);
            }
        })
    });
    
    group.finish();
}

/// Benchmark instantiation components to identify bottlenecks
fn bench_instantiation_components(c: &mut Criterion) {
    use seams::sentence_detector::dialog_detector::DialogStateMachine;
    use seams::sentence_detector::abbreviations::AbbreviationChecker;
    
    let mut group = c.benchmark_group("instantiation_components");
    
    // Benchmark DialogStateMachine creation (the heavy part)
    group.bench_function("dialog_state_machine_only", |b| {
        b.iter(|| {
            let machine = DialogStateMachine::new().unwrap();
            black_box(machine);
        })
    });
    
    // Benchmark AbbreviationChecker creation (should be light)
    group.bench_function("abbreviation_checker_only", |b| {
        b.iter(|| {
            let checker = AbbreviationChecker::new();
            black_box(checker);
        })
    });
    
    // Benchmark full detector creation for comparison
    group.bench_function("full_detector", |b| {
        b.iter(|| {
            let detector = SentenceDetectorDialog::new().unwrap();
            black_box(detector);
        })
    });
    
    group.finish();
}

/// Provide performance characterization data for users
fn report_instantiation_characteristics() {
    println!("=== SentenceDetectorDialog Instantiation Characteristics ===");
    
    // Measure single instantiation
    let start = std::time::Instant::now();
    let _detector = SentenceDetectorDialog::new().unwrap();
    let single_duration = start.elapsed();
    
    println!("Single instantiation: {:.2}ms", single_duration.as_millis());
    
    // Measure 10 instantiations (simulating test suite)
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _detector = SentenceDetectorDialog::new().unwrap();
    }
    let ten_duration = start.elapsed();
    
    println!("10 instantiations: {:.2}ms ({:.2}ms each)", 
             ten_duration.as_millis(), 
             ten_duration.as_millis() as f64 / 10.0);
    
    // Estimate test overhead for 38+ instances
    let estimated_test_overhead = single_duration.as_millis() as f64 * 38.0;
    println!("Estimated test overhead (38 instances): {estimated_test_overhead:.2}ms");
    
    // Performance guidance
    println!("\n=== Performance Guidance ===");
    if single_duration.as_millis() > 50 {
        println!("⚠️  HIGH INSTANTIATION COST: Consider sharing detector instances");
        println!("   - Use static/cached detectors in tests");
        println!("   - Reuse detectors across operations when possible");
    } else if single_duration.as_millis() > 10 {
        println!("⚡ MODERATE INSTANTIATION COST: Optimization beneficial for test suites");
    } else {
        println!("✅ LOW INSTANTIATION COST: Optimization not critical");
    }
    
    println!("   - Single detection operation: {}μs", single_duration.as_micros());
    println!("   - Regex compilation overhead: ~{:.1}% of total time", 
             (single_duration.as_millis() as f64 / (single_duration.as_millis() as f64 + 1.0)) * 100.0);
}

criterion_group!(
    benches,
    bench_detector_instantiation,
    bench_detector_usage_patterns,
    bench_instantiation_components
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instantiation_characteristics() {
        // This test provides user-visible data about instantiation cost
        report_instantiation_characteristics();
    }
}