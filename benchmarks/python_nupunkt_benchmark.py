"""
Python benchmark using nupunkt for sentence segmentation comparison with seams.
Follows equivalent processing pipeline for fair comparison.
"""

import os
import sys
import time
import json
import glob
from pathlib import Path
from typing import List, Dict, Any
import argparse

try:
    import nupunkt
except ImportError:
    print("ERROR: nupunkt not installed. Run: pip install nupunkt")
    sys.exit(1)

def discover_files(root_dir: str) -> List[str]:
    """Discover all *-0.txt files, matching seams file discovery pattern."""
    pattern = os.path.join(root_dir, "**/*-0.txt")
    return glob.glob(pattern, recursive=True)

def process_file_with_nupunkt(file_path: str, segmenter) -> Dict[str, Any]:
    """Process a single file with nupunkt sentence segmentation."""
    start_time = time.time()
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Time the sentence detection separately
        sentence_detection_start = time.time()
        sentences = segmenter.tokenize(content)
        sentence_detection_time = time.time() - sentence_detection_start
        
        # Calculate basic stats
        chars_processed = len(content)
        sentence_count = len(sentences)
        processing_time = time.time() - start_time
        
        # Calculate sentence length statistics
        sentence_length_stats = None
        if sentences:
            sentence_lengths = [len(sentence.strip()) for sentence in sentences if sentence.strip()]
            if sentence_lengths:
                sentence_lengths.sort()
                n = len(sentence_lengths)
                min_length = min(sentence_lengths)
                max_length = max(sentence_lengths)
                mean_length = sum(sentence_lengths) / n
                median_length = sentence_lengths[n // 2] if n % 2 == 1 else (sentence_lengths[n // 2 - 1] + sentence_lengths[n // 2]) / 2
                p25_idx = int(n * 0.25)
                p75_idx = int(n * 0.75)
                p90_idx = int(n * 0.90)
                p25_length = sentence_lengths[min(p25_idx, n - 1)]
                p75_length = sentence_lengths[min(p75_idx, n - 1)]
                p90_length = sentence_lengths[min(p90_idx, n - 1)]
                
                # Calculate standard deviation
                variance = sum((x - mean_length) ** 2 for x in sentence_lengths) / n
                std_dev = variance ** 0.5
                
                sentence_length_stats = {
                    "min_length": min_length,
                    "max_length": max_length,
                    "mean_length": mean_length,
                    "median_length": median_length,
                    "p25_length": p25_length,
                    "p75_length": p75_length,
                    "p90_length": p90_length,
                    "std_dev": std_dev
                }
        
        return {
            "path": file_path,
            "chars_processed": chars_processed,
            "sentences_detected": sentence_count,
            "sentence_length_stats": sentence_length_stats,
            "processing_time_ms": processing_time * 1000,
            "sentence_detection_time_ms": sentence_detection_time * 1000,
            "chars_per_sec": chars_processed / processing_time if processing_time > 0 else 0,
            "status": "success",
            "error": None
        }
    except Exception as e:
        return {
            "path": file_path,
            "chars_processed": 0,
            "sentences_detected": 0,
            "sentence_length_stats": None,
            "processing_time_ms": (time.time() - start_time) * 1000,
            "sentence_detection_time_ms": 0,
            "chars_per_sec": 0,
            "status": "failed",
            "error": str(e),
        }

def main():
    parser = argparse.ArgumentParser(description="Python nupunkt benchmark for sentence segmentation")
    parser.add_argument("root_dir", help="Root directory to process")
    parser.add_argument("--stats_out", default="python_nupunkt_stats.json", help="Output stats file")
    parser.add_argument("--max_files", type=int, help="Maximum number of files to process (for testing)")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.root_dir):
        print(f"ERROR: Directory {args.root_dir} does not exist")
        sys.exit(1)
    
    # Initialize nupunkt segmenter
    print("Initializing nupunkt segmenter...")
    segmenter = nupunkt.PunktSentenceTokenizer()
    
    # Discover files
    print(f"Discovering files in {args.root_dir}...")
    files = discover_files(args.root_dir)
    
    if args.max_files:
        files = files[:args.max_files]
    
    print(f"Found {len(files)} files to process")
    
    if not files:
        print("No files found matching pattern **/*-0.txt")
        sys.exit(1)
    
    # Process files
    results = []
    total_chars = 0
    total_sentences = 0
    successful_files = 0
    failed_files = 0
    
    benchmark_start = time.time()
    
    for i, file_path in enumerate(files):
        print(f"Processing [{i+1}/{len(files)}]: {os.path.basename(file_path)}")
        
        result = process_file_with_nupunkt(file_path, segmenter)
        results.append(result)
        
        if result["status"] == "success":
            total_chars += result["chars_processed"]
            total_sentences += result["sentences_detected"]
            successful_files += 1
        else:
            failed_files += 1
            print(f"  ERROR: {result['error']}")
    
    total_time = time.time() - benchmark_start
    
    # Calculate aggregate sentence length statistics
    all_sentence_lengths = []
    for result in results:
        if result["status"] == "success" and result["sentence_length_stats"]:
            # Approximate individual lengths using mean and count (simplified approach)
            sentence_count = result["sentences_detected"]
            mean_length = result["sentence_length_stats"]["mean_length"]
            for _ in range(sentence_count):
                all_sentence_lengths.append(mean_length)
    
    aggregate_sentence_length_stats = None
    if all_sentence_lengths:
        all_sentence_lengths.sort()
        n = len(all_sentence_lengths)
        min_length = min(all_sentence_lengths)
        max_length = max(all_sentence_lengths)
        mean_length = sum(all_sentence_lengths) / n
        median_length = all_sentence_lengths[n // 2] if n % 2 == 1 else (all_sentence_lengths[n // 2 - 1] + all_sentence_lengths[n // 2]) / 2
        p25_idx = int(n * 0.25)
        p75_idx = int(n * 0.75)
        p90_idx = int(n * 0.90)
        p25_length = all_sentence_lengths[min(p25_idx, n - 1)]
        p75_length = all_sentence_lengths[min(p75_idx, n - 1)]
        p90_length = all_sentence_lengths[min(p90_idx, n - 1)]
        
        # Calculate standard deviation
        variance = sum((x - mean_length) ** 2 for x in all_sentence_lengths) / n
        std_dev = variance ** 0.5
        
        aggregate_sentence_length_stats = {
            "min_length": min_length,
            "max_length": max_length,
            "mean_length": mean_length,
            "median_length": median_length,
            "p25_length": p25_length,
            "p75_length": p75_length,
            "p90_length": p90_length,
            "std_dev": std_dev
        }
    
    # Calculate aggregate stats
    stats = {
        "tool": "nupunkt",
        "version": nupunkt.__version__,
        "total_files": len(files),
        "successful_files": successful_files,
        "failed_files": failed_files,
        "total_chars_processed": total_chars,
        "total_sentences": total_sentences,
        "aggregate_sentence_length_stats": aggregate_sentence_length_stats,
        "total_time_ms": total_time * 1000,
        "aggregate_throughput_chars_per_sec": total_chars / total_time if total_time > 0 else 0,
        "aggregate_throughput_mb_per_sec": (total_chars / total_time) / (1024 * 1024) if total_time > 0 else 0,
        "results": results
    }
    
    # Write stats
    with open(args.stats_out, 'w') as f:
        json.dump(stats, f, indent=2)
    
    print(f"\n=== nupunkt Benchmark Results ===")
    print(f"Files processed: {successful_files}/{len(files)}")
    print(f"Total characters: {total_chars:,}")
    print(f"Total sentences: {total_sentences:,}")
    print(f"Total time: {total_time:.2f}s")
    print(f"Throughput: {stats['aggregate_throughput_chars_per_sec']:.1f} chars/sec")
    print(f"Throughput: {stats['aggregate_throughput_mb_per_sec']:.2f} MB/sec")
    print(f"Stats written to: {args.stats_out}")

if __name__ == "__main__":
    main()