#!/usr/bin/env python3
"""
Find cases where seams (seams2.txt) disagrees with pysbd and/or nupunkt.

Specifically targets:
1. Cases where pysbd and nupunkt AGREE but seams differs (high priority)
2. Cases where seams differs from either pysbd or nupunkt (medium priority)

This helps identify potential issues with the seams dialog detector by comparing
against established sentence segmentation libraries.

Usage:
    python build_seams_disagreement_set.py --max-files 20 --target-size 300 --output seams_disagreements.json

Requires:
    - nupunkt: pip install nupunkt  
    - pysbd: pip install pysbd
"""

import os
import re
import random
import json
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Set, Optional
from dataclasses import dataclass
from collections import defaultdict
from difflib import SequenceMatcher

# Sentence segmentation imports
try:
    import nupunkt
    NUPUNKT_AVAILABLE = True
except ImportError:
    NUPUNKT_AVAILABLE = False
    print("ERROR: nupunkt not available. Install with: pip install nupunkt")

try:
    import pysbd
    PYSBD_AVAILABLE = True
    # Initialize pysbd
    pysbd_segmenter = pysbd.Segmenter(language="en", clean=False)
except ImportError:
    PYSBD_AVAILABLE = False
    print("ERROR: pysbd not available. Install with: pip install pysbd")

if not (NUPUNKT_AVAILABLE and PYSBD_AVAILABLE):
    print("Both nupunkt and pysbd are required for this script.")
    exit(1)


@dataclass
class SeamsDisagreement:
    """Represents a case where seams disagrees with other segmenters"""
    original_text: str
    seams_sentences: List[str]
    pysbd_sentences: List[str] 
    nupunkt_sentences: List[str]
    disagreement_type: str  # "seams_vs_both", "seams_vs_pysbd", "seams_vs_nupunkt"
    source_file: str
    text_region_start: int
    text_region_end: int
    complexity_indicators: List[str]


def extract_original_text_from_coordinates(original_file: Path, start_line: int, start_col: int, 
                                         end_line: int, end_col: int) -> str:
    """Extract original text using 1-based coordinates from seams file"""
    try:
        with open(original_file, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        # Convert to 0-based indexing
        start_line_idx = start_line - 1
        end_line_idx = end_line - 1
        start_col_idx = start_col - 1
        end_col_idx = end_col - 1
        
        if start_line_idx == end_line_idx:
            # Single line
            return lines[start_line_idx][start_col_idx:end_col_idx]
        else:
            # Multi-line: start of first line + middle lines + end of last line
            result = lines[start_line_idx][start_col_idx:]
            for line_idx in range(start_line_idx + 1, end_line_idx):
                result += lines[line_idx]
            result += lines[end_line_idx][:end_col_idx]
            return result
            
    except Exception as e:
        print(f"Error extracting text from {original_file}: {e}")
        return ""


def parse_seams_file(seams_file: Path) -> List[Tuple[str, int, int, int, int]]:
    """Parse seams2.txt file and return sentences with coordinates"""
    sentences_with_coords = []
    
    try:
        with open(seams_file, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                    
                parts = line.split('\t')
                if len(parts) >= 3:
                    sentence_text = parts[1]
                    coords_str = parts[2]
                    
                    # Parse coordinates: (start_line,start_col,end_line,end_col)
                    coord_match = re.match(r'\((\d+),(\d+),(\d+),(\d+)\)', coords_str)
                    if coord_match:
                        start_line, start_col, end_line, end_col = map(int, coord_match.groups())
                        sentences_with_coords.append((sentence_text, start_line, start_col, end_line, end_col))
                        
    except Exception as e:
        print(f"Error parsing seams file {seams_file}: {e}")
        
    return sentences_with_coords


def segment_with_pysbd(text: str) -> List[str]:
    """Segment text using pysbd"""
    try:
        sentences = pysbd_segmenter.segment(text)
        return [s.strip() for s in sentences if s.strip()]
    except Exception as e:
        print(f"pysbd segmentation error: {e}")
        return []


def segment_with_nupunkt(text: str) -> List[str]:
    """Segment text using nupunkt"""  
    try:
        sentences = nupunkt.sent_tokenize(text)
        return [s.strip() for s in sentences if s.strip()]
    except Exception as e:
        print(f"nupunkt segmentation error: {e}")
        return []


def normalize_for_comparison(text: str) -> str:
    """Normalize text for fair comparison between segmenters"""
    # Remove extra whitespace and normalize
    text = ' '.join(text.split())
    return text.strip()


def sentences_are_similar(seams_sentences: List[str], other_sentences: List[str], threshold: float = 0.9, debug: bool = False) -> bool:
    """Check if two sentence lists are similar enough to be considered the same segmentation"""
    # First check: different sentence counts = definitely different
    if len(seams_sentences) != len(other_sentences):
        if debug:
            print(f"      Different sentence counts: {len(seams_sentences)} vs {len(other_sentences)} -> DIFFERENT")
        return False
    
    # Second check: compare normalized text similarity
    seams_text = ' '.join(normalize_for_comparison(s) for s in seams_sentences)
    other_text = ' '.join(normalize_for_comparison(s) for s in other_sentences)
    
    similarity = SequenceMatcher(None, seams_text, other_text).ratio()
    
    if debug:
        print(f"      Text similarity: {similarity:.3f} (threshold: {threshold})")
        print(f"      Seams ({len(seams_sentences)}): {seams_sentences}")
        print(f"      Other ({len(other_sentences)}): {other_sentences}")
    
    return similarity >= threshold


def analyze_text_complexity(text: str) -> List[str]:
    """Identify complexity indicators in the text"""
    indicators = []
    
    # Dialog indicators
    if '"' in text:
        indicators.append("double_quotes")
    if "'" in text and re.search(r"'\w", text):  # Avoid apostrophes
        indicators.append("single_quotes")
    if '(' in text and ')' in text:
        indicators.append("parenthetical")
    
    # Dialog attribution patterns
    if re.search(r'"\s*[,;]\s*\w+\s+(said|asked|replied|shouted)', text.lower()):
        indicators.append("dialog_attribution")
    
    # Hard separators
    if '\n\n' in text:
        indicators.append("hard_separator")
    
    # Abbreviations
    if re.search(r'\b[A-Z]\.\s*[A-Z]\.', text):
        indicators.append("abbreviations")
    
    # Complex punctuation
    if '...' in text or '—' in text or '–' in text:
        indicators.append("complex_punctuation")
    
    # Quote with parenthetical (pattern we just fixed)
    if re.search(r'"\s*\([^)]*\)', text):
        indicators.append("quote_parenthetical")
    
    return indicators


def classify_disagreement(seams_sentences: List[str], pysbd_sentences: List[str], 
                         nupunkt_sentences: List[str], debug: bool = False) -> str:
    """Classify the type of disagreement"""
    
    seams_vs_pysbd = not sentences_are_similar(seams_sentences, pysbd_sentences, debug=debug)
    seams_vs_nupunkt = not sentences_are_similar(seams_sentences, nupunkt_sentences, debug=debug)
    pysbd_vs_nupunkt = not sentences_are_similar(pysbd_sentences, nupunkt_sentences, debug=debug)
    
    if seams_vs_pysbd and seams_vs_nupunkt and not pysbd_vs_nupunkt:
        return "seams_vs_both"  # HIGH PRIORITY: pysbd and nupunkt agree, seams differs
    elif seams_vs_pysbd and not seams_vs_nupunkt:
        return "seams_vs_pysbd"
    elif seams_vs_nupunkt and not seams_vs_pysbd:
        return "seams_vs_nupunkt"
    elif seams_vs_pysbd and seams_vs_nupunkt and pysbd_vs_nupunkt:
        return "all_disagree"
    else:
        return "no_disagreement"


def find_disagreements_in_file_pair(original_file: Path, seams_file: Path) -> List[SeamsDisagreement]:
    """Find disagreements between seams and other segmenters for a file pair"""
    disagreements = []
    
    print(f"Analyzing {original_file.name}...")
    
    # Parse seams file
    seams_data = parse_seams_file(seams_file)
    if not seams_data:
        print(f"  ❌ No seams data found in {seams_file}")
        return disagreements
    
    print(f"  📄 Found {len(seams_data)} sentences in seams file")
    
    # Group consecutive sentences to create larger text regions for comparison
    region_size = 3  # Look at 3-sentence windows
    
    for i in range(0, len(seams_data) - region_size + 1, region_size):
        region_sentences = seams_data[i:i + region_size]
        
        # Extract the original text for this region
        first_sentence = region_sentences[0]
        last_sentence = region_sentences[-1]
        
        # Get the full text span from first sentence start to last sentence end
        original_text = extract_original_text_from_coordinates(
            original_file, 
            first_sentence[1], first_sentence[2],  # start line, start col
            last_sentence[3], last_sentence[4]     # end line, end col
        )
        
        if not original_text or len(original_text.strip()) < 20:
            continue
            
        # Get seams sentences for this region
        seams_sentences = [sent[0] for sent in region_sentences]
        
        # Segment the original text with other methods
        pysbd_sentences = segment_with_pysbd(original_text)
        nupunkt_sentences = segment_with_nupunkt(original_text)
        
        if not pysbd_sentences or not nupunkt_sentences:
            continue
            
        # Debug output for first few regions
        if i < 2:
            print(f"    Region {i}: {len(seams_sentences)} seams, {len(pysbd_sentences)} pysbd, {len(nupunkt_sentences)} nupunkt")
            print(f"    Text sample: {original_text[:60]}...")
            
        # Classify disagreement
        disagreement_type = classify_disagreement(seams_sentences, pysbd_sentences, nupunkt_sentences, debug=(i < 2))
        
        # Debug disagreement classification
        if i < 2:
            print(f"    Disagreement type: {disagreement_type}")
        
        if disagreement_type in ["seams_vs_both", "seams_vs_pysbd", "seams_vs_nupunkt"]:
            complexity = analyze_text_complexity(original_text)
            
            disagreement = SeamsDisagreement(
                original_text=original_text,
                seams_sentences=seams_sentences,
                pysbd_sentences=pysbd_sentences,
                nupunkt_sentences=nupunkt_sentences,
                disagreement_type=disagreement_type,
                source_file=str(original_file),
                text_region_start=i,
                text_region_end=i + region_size,
                complexity_indicators=complexity
            )
            
            disagreements.append(disagreement)
            
            # Debug output for high-priority cases
            if disagreement_type == "seams_vs_both":
                print(f"  🔍 HIGH PRIORITY: pysbd+nupunkt agree, seams differs")
                print(f"     Complexity: {complexity}")
                print(f"     Original: {original_text[:100]}...")
    
    return disagreements


def find_all_disagreements(gutenberg_root: str, max_files: int, target_size: int) -> List[SeamsDisagreement]:
    """Find disagreements across multiple file pairs"""
    
    disagreements = []
    files_processed = 0
    
    # Find all original/seams file pairs
    gutenberg_path = Path(gutenberg_root)
    
    # Collect all valid file pairs first
    valid_file_pairs = []
    for original_file in gutenberg_path.rglob("*-0.txt"):
        seams_file = original_file.with_name(original_file.stem + "_seams2.txt")
        if seams_file.exists():
            valid_file_pairs.append(original_file)
    
    print(f"Found {len(valid_file_pairs)} valid file pairs")
    
    # Randomize the order using the seed (already set in main())
    random.shuffle(valid_file_pairs)
    
    # Process up to max_files in randomized order
    for original_file in valid_file_pairs[:max_files]:
        seams_file = original_file.with_name(original_file.stem + "_seams2.txt")
        
        file_disagreements = find_disagreements_in_file_pair(original_file, seams_file)
        disagreements.extend(file_disagreements)
        files_processed += 1
        
        # Stop if we have enough examples
        if len(disagreements) >= target_size:
            break
    
    print(f"\nProcessed {files_processed} file pairs")
    print(f"Found {len(disagreements)} disagreements")
    
    # Prioritize disagreements
    high_priority = [d for d in disagreements if d.disagreement_type == "seams_vs_both"]
    medium_priority = [d for d in disagreements if d.disagreement_type in ["seams_vs_pysbd", "seams_vs_nupunkt"]]
    
    print(f"  - {len(high_priority)} HIGH PRIORITY (seams vs both)")
    print(f"  - {len(medium_priority)} MEDIUM PRIORITY (seams vs one)")
    
    # Return prioritized list
    prioritized = high_priority + medium_priority
    return prioritized[:target_size]


def export_disagreements(disagreements: List[SeamsDisagreement], output_file: str):
    """Export disagreements to JSON format"""
    
    export_data = {
        "metadata": {
            "total_disagreements": len(disagreements),
            "high_priority_count": len([d for d in disagreements if d.disagreement_type == "seams_vs_both"]),
            "generation_method": "seams_disagreement_analysis",
            "segmenters_compared": ["seams", "pysbd", "nupunkt"]
        },
        "disagreements": []
    }
    
    for disagreement in disagreements:
        export_data["disagreements"].append({
            "original_text": disagreement.original_text,
            "seams_sentences": disagreement.seams_sentences,
            "pysbd_sentences": disagreement.pysbd_sentences,
            "nupunkt_sentences": disagreement.nupunkt_sentences,
            "disagreement_type": disagreement.disagreement_type,
            "source_file": disagreement.source_file,
            "complexity_indicators": disagreement.complexity_indicators,
            "priority": "HIGH" if disagreement.disagreement_type == "seams_vs_both" else "MEDIUM"
        })
    
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(export_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nExported {len(disagreements)} disagreements to {output_file}")


def main():
    parser = argparse.ArgumentParser(description="Find cases where seams disagrees with pysbd/nupunkt")
    parser.add_argument('--gutenberg-root', default='/data/gutenberg', 
                       help='Root directory of Gutenberg corpus')
    parser.add_argument('--output', default='seams_disagreements.json',
                       help='Output JSON file')
    parser.add_argument('--max-files', type=int, default=20,
                       help='Maximum number of file pairs to analyze')
    parser.add_argument('--target-size', type=int, default=300,
                       help='Target number of disagreement examples')
    parser.add_argument('--seed', type=int, default=42,
                       help='Random seed for reproducibility')
    
    args = parser.parse_args()
    
    # Set random seed
    random.seed(args.seed)
    
    print("🔍 Finding cases where seams disagrees with pysbd/nupunkt...")
    print(f"📁 Gutenberg root: {args.gutenberg_root}")
    print(f"📊 Target: {args.target_size} examples from max {args.max_files} files")
    print(f"📄 Output: {args.output}")
    print()
    
    # Find disagreements
    disagreements = find_all_disagreements(args.gutenberg_root, args.max_files, args.target_size)
    
    if disagreements:
        export_disagreements(disagreements, args.output)
        
        # Print summary
        print("\n📊 SUMMARY:")
        print(f"HIGH PRIORITY (seams vs both): {len([d for d in disagreements if d.disagreement_type == 'seams_vs_both'])}")
        print(f"MEDIUM PRIORITY (seams vs one): {len([d for d in disagreements if d.disagreement_type != 'seams_vs_both'])}")
        
        # Show complexity distribution
        all_complexity = []
        for d in disagreements:
            all_complexity.extend(d.complexity_indicators)
        
        complexity_counts = {}
        for indicator in all_complexity:
            complexity_counts[indicator] = complexity_counts.get(indicator, 0) + 1
            
        print("\n🎯 Most common complexity patterns:")
        for indicator, count in sorted(complexity_counts.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"  {indicator}: {count}")
            
    else:
        print("No disagreements found!")


if __name__ == "__main__":
    main()