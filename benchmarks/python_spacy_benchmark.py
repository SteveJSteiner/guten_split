"""
Python benchmark using spaCy sentencizer for sentence segmentation comparison with seams.
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
    import spacy
    from spacy.lang.en import English
except ImportError:
    print("ERROR: spaCy not installed. Run: pip install spacy")
    sys.exit(1)

def discover_files(root_dir: str) -> List[str]:
    """Discover all *-0.txt files, matching seams file discovery pattern."""
    pattern = os.path.join(root_dir, "**/*-0.txt")
    return glob.glob(pattern, recursive=True)

def process_file_with_spacy(file_path: str, nlp) -> Dict[str, Any]:
    """Process a single file with spaCy sentencizer."""
    start_time = time.time()
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Process with spaCy
        doc = nlp(content)
        
        # Extract sentences
        sentences = [sent.text for sent in doc.sents]
        
        # Calculate basic stats
        chars_processed = len(content)
        sentence_count = len(sentences)
        processing_time = time.time() - start_time
        
        return {
            "file_path": file_path,
            "chars_processed": chars_processed,
            "sentence_count": sentence_count,
            "processing_time_ms": processing_time * 1000,
            "throughput_chars_per_sec": chars_processed / processing_time if processing_time > 0 else 0,
            "success": True,
            "sentences": sentences[:5] if len(sentences) > 5 else sentences  # Sample for accuracy comparison
        }
    except Exception as e:
        return {
            "file_path": file_path,
            "error": str(e),
            "success": False,
            "processing_time_ms": (time.time() - start_time) * 1000
        }

def main():
    parser = argparse.ArgumentParser(description="Python spaCy benchmark for sentence segmentation")
    parser.add_argument("root_dir", help="Root directory to process")
    parser.add_argument("--stats_out", default="python_spacy_stats.json", help="Output stats file")
    parser.add_argument("--max_files", type=int, help="Maximum number of files to process (for testing)")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.root_dir):
        print(f"ERROR: Directory {args.root_dir} does not exist")
        sys.exit(1)
    
    # Initialize spaCy with just sentencizer (no full NLP pipeline)
    print("Initializing spaCy sentencizer...")
    nlp = English()
    nlp.add_pipe("sentencizer")
    # Increase max_length to handle large Project Gutenberg files
    nlp.max_length = 5000000  # 5MB limit
    
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
        
        result = process_file_with_spacy(file_path, nlp)
        results.append(result)
        
        if result["success"]:
            total_chars += result["chars_processed"]
            total_sentences += result["sentence_count"]
            successful_files += 1
        else:
            failed_files += 1
            print(f"  ERROR: {result['error']}")
    
    total_time = time.time() - benchmark_start
    
    # Calculate aggregate stats
    stats = {
        "tool": "spacy_sentencizer",
        "version": spacy.__version__,
        "total_files": len(files),
        "successful_files": successful_files,
        "failed_files": failed_files,
        "total_chars_processed": total_chars,
        "total_sentences": total_sentences,
        "total_time_ms": total_time * 1000,
        "aggregate_throughput_chars_per_sec": total_chars / total_time if total_time > 0 else 0,
        "aggregate_throughput_mb_per_sec": (total_chars / total_time) / (1024 * 1024) if total_time > 0 else 0,
        "results": results
    }
    
    # Write stats
    with open(args.stats_out, 'w') as f:
        json.dump(stats, f, indent=2)
    
    print(f"\n=== spaCy Sentencizer Benchmark Results ===")
    print(f"Files processed: {successful_files}/{len(files)}")
    print(f"Total characters: {total_chars:,}")
    print(f"Total sentences: {total_sentences:,}")
    print(f"Total time: {total_time:.2f}s")
    print(f"Throughput: {stats['aggregate_throughput_chars_per_sec']:.1f} chars/sec")
    print(f"Throughput: {stats['aggregate_throughput_mb_per_sec']:.2f} MB/sec")
    print(f"Stats written to: {args.stats_out}")

if __name__ == "__main__":
    main()