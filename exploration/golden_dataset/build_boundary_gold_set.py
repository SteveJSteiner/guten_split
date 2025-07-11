#!/usr/bin/env python3
"""
Enhanced gold-set builder focused on sentence boundary disagreements.

Finds cases where the same original text region is segmented differently
by seams vs fast extraction methods, and includes segmentations from 
nupunkt and pysbd for comprehensive algorithm comparison.

Features:
- Extracts original text using coordinate mappings from seams files
- Compares segmentations across multiple methods (seams, fast/sentences, nupunkt, pysbd)
- Applies filtering rules to focus on meaningful disagreements
- Generates rich datasets for sentence segmentation evaluation

Usage:
    python build_boundary_gold_set.py --max-files 25 --target-size 500 --output golden_dataset.json

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

# Sentence segmentation imports for filtering
try:
    import nupunkt
    NUPUNKT_AVAILABLE = True
except ImportError:
    NUPUNKT_AVAILABLE = False
    print("Warning: nupunkt not available, filtering will be limited")

try:
    import pysbd
    PYSBD_AVAILABLE = True
except ImportError:
    PYSBD_AVAILABLE = False
    print("Warning: pysbd not available, filtering will be limited")


@dataclass
class BoundaryExample:
    """A sentence boundary disagreement with original text context."""
    original_text: str
    seams_sentences: List[str]
    comparison_sentences: List[str]
    comparison_method: str  # 'fast' or 'sentences'
    nupunkt_sentences: List[str]  # Added: nupunkt segmentation
    pysbd_sentences: List[str]  # Added: pysbd segmentation
    source_file: str
    start_line: int
    end_line: int
    complexity: str
    disagreement_type: str  # 'over_split', 'under_split', 'different_boundaries'


class BoundaryGoldSetBuilder:
    """Builds gold-set focused on sentence boundary disagreements."""
    
    def __init__(self, gutenberg_root: str):
        self.gutenberg_root = Path(gutenberg_root)
        self.examples: List[BoundaryExample] = []
    
    def find_file_pairs(self) -> List[Tuple[Path, Path, str]]:
        """Find matching _seams.txt and sentence files (_sentences_fast.txt and _sentences.txt)."""
        seams_files = list(self.gutenberg_root.glob("**/*_seams.txt"))
        pairs = []
        
        for seams_file in seams_files:
            # Check for both _sentences_fast.txt and _sentences.txt
            fast_file = seams_file.parent / seams_file.name.replace('_seams.txt', '_sentences_fast.txt')
            sentences_file = seams_file.parent / seams_file.name.replace('_seams.txt', '_sentences.txt')
            
            if fast_file.exists():
                pairs.append((seams_file, fast_file, 'fast'))
            if sentences_file.exists():
                pairs.append((seams_file, sentences_file, 'sentences'))
        
        return pairs
    
    def parse_seams_with_coordinates(self, file_path: Path) -> List[tuple]:
        """Parse seams file with coordinates."""
        sentences = []
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        parts = line.split('\t')
                        if len(parts) >= 3:
                            sentence_id = parts[0]
                            text_part = parts[1]
                            coord_part = parts[2]
                            
                            coord_match = re.search(r'\((\d+),(\d+),(\d+),(\d+)\)', coord_part)
                            if coord_match:
                                start_line = int(coord_match.group(1))
                                start_col = int(coord_match.group(2))
                                end_line = int(coord_match.group(3))
                                end_col = int(coord_match.group(4))
                                
                                sentences.append((text_part.strip(), start_line, start_col, end_line, end_col))
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
        
        return sentences
    
    def parse_comparison_sentences(self, file_path: Path) -> List[str]:
        """Parse comparison sentences (fast or regular sentences, no coordinates)."""
        sentences = []
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        parts = line.split('\t')
                        if len(parts) >= 2:
                            sentences.append(parts[1].strip())
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
        
        return sentences
    
    def load_original_lines(self, original_file: Path) -> List[str]:
        """Load original text file as lines."""
        try:
            with open(original_file, 'r', encoding='utf-8') as f:
                return f.readlines()
        except Exception as e:
            print(f"Error reading {original_file}: {e}")
            return []
    
    def extract_original_region(self, original_lines: List[str], start_line: int, start_col: int, 
                              end_line: int, end_col: int) -> str:
        """Extract original text from coordinates."""
        if start_line > len(original_lines) or end_line > len(original_lines):
            return ""
        
        start_line_idx = start_line - 1
        end_line_idx = end_line - 1
        start_col_idx = start_col - 1
        end_col_idx = end_col - 1
        
        if start_line_idx == end_line_idx:
            line = original_lines[start_line_idx].rstrip('\n\r')
            if end_col_idx <= len(line):
                return line[start_col_idx:end_col_idx]
            else:
                return line[start_col_idx:]
        else:
            text_parts = []
            
            first_line = original_lines[start_line_idx].rstrip('\n\r')
            if start_col_idx < len(first_line):
                text_parts.append(first_line[start_col_idx:])
            
            for line_idx in range(start_line_idx + 1, end_line_idx):
                text_parts.append(original_lines[line_idx].rstrip('\n\r'))
            
            if end_line_idx < len(original_lines):
                last_line = original_lines[end_line_idx].rstrip('\n\r')
                if end_col_idx <= len(last_line):
                    text_parts.append(last_line[:end_col_idx])
                else:
                    text_parts.append(last_line)
            
            return ' '.join(text_parts)
    
    def find_overlapping_regions(self, seams_sentences: List[tuple], comparison_sentences: List[str], 
                               original_lines: List[str], comparison_method: str) -> List[BoundaryExample]:
        """Find regions where seams and fast disagree on boundaries using efficient sorted traversal."""
        examples = []
        fast_idx = 0
        
        # Process seams sentences in groups, efficiently finding overlapping fast sentences
        for seams_start in range(0, len(seams_sentences) - 1, 2):
            seams_group = seams_sentences[seams_start:seams_start + 3]  # Take 3 consecutive seams
            
            if not seams_group:
                continue
            
            # Get the overall region coordinates
            first_sent = seams_group[0]
            last_sent = seams_group[-1]
            
            region_start_line = first_sent[1]
            region_start_col = first_sent[2]
            region_end_line = last_sent[3]
            region_end_col = last_sent[4]
            
            # Extract original text for this region
            original_region = self.extract_original_region(
                original_lines, region_start_line, region_start_col,
                region_end_line, region_end_col
            )
            
            if not original_region or len(original_region) < 50:
                continue
            
            # Find overlapping comparison sentences efficiently by advancing the fast_idx
            seams_texts = [sent[0] for sent in seams_group]
            overlapping_comparison = []
            
            # Look ahead from current fast_idx to find matches, but don't go too far
            search_start = max(0, fast_idx - 5)  # Look back a bit for safety
            search_end = min(len(comparison_sentences), fast_idx + 20)  # Limited lookahead
            
            consumed_count = 0
            for i in range(search_start, search_end):
                comparison_sent = comparison_sentences[i]
                
                # Quick checks for overlap
                if self.sentences_overlap(comparison_sent, seams_texts, original_region):
                    overlapping_comparison.append(comparison_sent)
                    if i >= fast_idx:
                        consumed_count += 1
            
            # Advance fast_idx by the number of sentences we consumed
            if consumed_count > 0:
                fast_idx += consumed_count
            
            # Only include if we found a meaningful disagreement
            if len(overlapping_comparison) != len(seams_texts) and overlapping_comparison:
                disagreement_type = self.classify_disagreement(seams_texts, overlapping_comparison)
                complexity = self.classify_complexity(original_region)
                
                # Get nupunkt and pysbd segmentations
                nupunkt_sentences, pysbd_sentences = self.get_nupunkt_pysbd_segmentations(original_region.strip())
                
                example = BoundaryExample(
                    original_text=original_region.strip(),
                    seams_sentences=seams_texts,
                    comparison_sentences=overlapping_comparison,
                    comparison_method=comparison_method,
                    nupunkt_sentences=nupunkt_sentences,
                    pysbd_sentences=pysbd_sentences,
                    source_file="",  # Will be set by caller
                    start_line=region_start_line,
                    end_line=region_end_line,
                    complexity=complexity,
                    disagreement_type=disagreement_type
                )
                
                # Apply filtering rules
                if not self.should_filter_example(example):
                    examples.append(example)
        
        return examples
    
    def sentences_overlap(self, comparison_sent: str, seams_texts: List[str], original_region: str) -> bool:
        """Efficiently check if a comparison sentence overlaps with the seams region."""
        comparison_lower = comparison_sent.lower()
        original_lower = original_region.lower()
        
        # Quick substring check
        if comparison_lower in original_lower:
            return True
        
        # Check similarity with any seams sentence (but limit to avoid O(n^2))
        for seams_text in seams_texts:
            if SequenceMatcher(None, comparison_lower, seams_text.lower()).ratio() > 0.7:
                return True
        
        # Check if significant words overlap
        comparison_words = {word for word in comparison_lower.split() if len(word) > 4}
        if len(comparison_words) > 0:
            original_words = set(original_lower.split())
            overlap_ratio = len(comparison_words & original_words) / len(comparison_words)
            if overlap_ratio > 0.5:
                return True
        
        return False
    
    def classify_disagreement(self, seams_sentences: List[str], comparison_sentences: List[str]) -> str:
        """Classify the type of boundary disagreement."""
        seams_count = len(seams_sentences)
        comparison_count = len(comparison_sentences)
        
        if comparison_count > seams_count:
            return 'over_split'
        elif comparison_count < seams_count:
            return 'under_split'
        else:
            return 'different_boundaries'
    
    def classify_complexity(self, text: str) -> str:
        """Classify text complexity."""
        # Dialog patterns
        if ('"' in text or "'" in text or 
            re.search(r'\b(?:said|asked|replied)\b', text, re.IGNORECASE)):
            return 'dialog'
        
        # Abbreviations
        if re.search(r'\b(?:Mr|Mrs|Dr|etc|vs|i\.e|e\.g)\.|[A-Z]\.', text):
            return 'abbreviation'
        
        # Complex structures
        if re.search(r'[;:()\-\-]|however|therefore', text, re.IGNORECASE):
            return 'complex'
        
        return 'normal'
    
    def get_nupunkt_pysbd_segmentations(self, original_text: str) -> Tuple[List[str], List[str]]:
        """Get nupunkt and pysbd segmentations for the original text."""
        nupunkt_sentences = []
        pysbd_sentences = []
        
        # Get nupunkt segmentation
        if NUPUNKT_AVAILABLE:
            try:
                nupunkt_sentences = [sent.strip() for sent in nupunkt.sent_tokenize(original_text) if sent.strip()]
            except Exception:
                nupunkt_sentences = []
        
        # Get pysbd segmentation
        if PYSBD_AVAILABLE:
            try:
                segmenter = pysbd.Segmenter(language="en", clean=False)
                pysbd_sentences = [sent.strip() for sent in segmenter.segment(original_text) if sent.strip()]
            except Exception:
                pysbd_sentences = []
        
        return nupunkt_sentences, pysbd_sentences
    
    def should_filter_example(self, example: BoundaryExample) -> bool:
        """Apply filtering rules to reject unwanted examples.
        
        Returns True if example should be filtered out (rejected).
        """
        original_text = example.original_text
        
        # Rule 1: Reject if no lowercase letters
        if not re.search(r'[a-z]', original_text):
            return True
        
        # Rule 2: Reject if all three methods (seams, nupunkt, pysbd) agree
        if NUPUNKT_AVAILABLE and PYSBD_AVAILABLE:
            # Get seams segmentation (already known)
            seams_sentences = [sent.strip() for sent in example.seams_sentences if sent.strip()]
            nupunkt_sentences = example.nupunkt_sentences
            pysbd_sentences = example.pysbd_sentences
            
            # If all three methods produce the same number of sentences and similar content, reject
            if (len(seams_sentences) == len(nupunkt_sentences) == len(pysbd_sentences) and 
                len(seams_sentences) > 0):
                
                # Quick check: if sentence counts match, compare normalized content
                seams_normalized = " ".join(seams_sentences).lower().replace(" ", "")
                nupunkt_normalized = " ".join(nupunkt_sentences).lower().replace(" ", "")
                pysbd_normalized = " ".join(pysbd_sentences).lower().replace(" ", "")
                
                # If content is very similar (allowing for minor differences), reject
                if (seams_normalized == nupunkt_normalized == pysbd_normalized):
                    return True
        
        return False
    
    def analyze_boundary_disagreements(self, seams_file: Path, comparison_file: Path, comparison_method: str) -> List[BoundaryExample]:
        """Find boundary disagreements between seams and comparison extraction."""
        original_file = seams_file.parent / seams_file.name.replace('_seams.txt', '.txt')
        if not original_file.exists():
            return []
        
        seams_sentences = self.parse_seams_with_coordinates(seams_file)
        comparison_sentences = self.parse_comparison_sentences(comparison_file)
        original_lines = self.load_original_lines(original_file)
        
        if not seams_sentences or not comparison_sentences or not original_lines:
            return []
        
        examples = self.find_overlapping_regions(seams_sentences, comparison_sentences, original_lines, comparison_method)
        
        # Set source file for all examples
        for example in examples:
            example.source_file = str(original_file.relative_to(self.gutenberg_root))
        
        return examples
    
    def build_boundary_gold_set(self, max_files: int = 20, target_size: int = 500) -> List[BoundaryExample]:
        """Build gold-set focused on boundary disagreements."""
        file_pairs = self.find_file_pairs()
        
        if max_files and len(file_pairs) > max_files:
            file_pairs = random.sample(file_pairs, max_files)
        
        all_examples = []
        
        print(f"Analyzing {len(file_pairs)} file pairs for boundary disagreements...")
        
        for i, (seams_file, comparison_file, comparison_method) in enumerate(file_pairs):
            if i % 5 == 0:
                print(f"Processed {i}/{len(file_pairs)} files...")
            
            examples = self.analyze_boundary_disagreements(seams_file, comparison_file, comparison_method)
            all_examples.extend(examples)
        
        print(f"Found {len(all_examples)} boundary disagreement examples")
        
        # Stratified sampling by disagreement type, complexity, and comparison method
        examples_by_type = defaultdict(list)
        for example in all_examples:
            key = f"{example.comparison_method}_{example.disagreement_type}_{example.complexity}"
            examples_by_type[key].append(example)
        
        selected_examples = []
        for category, examples in examples_by_type.items():
            # Take a sample from each category
            sample_size = min(len(examples), target_size // len(examples_by_type))
            if examples:
                selected = random.sample(examples, min(sample_size, len(examples)))
                selected_examples.extend(selected)
                print(f"Selected {len(selected)} {category} examples")
        
        # Fill remaining slots if under target
        if len(selected_examples) < target_size:
            remaining = [ex for ex in all_examples if ex not in selected_examples]
            additional_needed = target_size - len(selected_examples)
            if remaining:
                additional = random.sample(remaining, min(additional_needed, len(remaining)))
                selected_examples.extend(additional)
        
        random.shuffle(selected_examples)
        return selected_examples[:target_size]
    
    def export_boundary_gold_set(self, examples: List[BoundaryExample], output_file: str):
        """Export boundary disagreement gold-set."""
        export_data = {
            'metadata': {
                'total_examples': len(examples),
                'disagreement_distribution': {},
                'complexity_distribution': {},
                'comparison_method_distribution': {},
                'description': 'Gold-set focused on sentence boundary disagreements between seams and other methods'
            },
            'examples': []
        }
        
        disagreement_counts = defaultdict(int)
        complexity_counts = defaultdict(int)
        method_counts = defaultdict(int)
        
        for example in examples:
            disagreement_counts[example.disagreement_type] += 1
            complexity_counts[example.complexity] += 1
            method_counts[example.comparison_method] += 1
            
            export_data['examples'].append({
                'original_text': example.original_text,
                'seams_sentences': example.seams_sentences,
                'comparison_sentences': example.comparison_sentences,
                'comparison_method': example.comparison_method,
                'nupunkt_sentences': example.nupunkt_sentences,
                'pysbd_sentences': example.pysbd_sentences,
                'source_file': example.source_file,
                'start_line': example.start_line,
                'end_line': example.end_line,
                'complexity': example.complexity,
                'disagreement_type': example.disagreement_type,
                'original_length': len(example.original_text),
                'seams_count': len(example.seams_sentences),
                'comparison_count': len(example.comparison_sentences),
                'nupunkt_count': len(example.nupunkt_sentences),
                'pysbd_count': len(example.pysbd_sentences)
            })
        
        export_data['metadata']['disagreement_distribution'] = dict(disagreement_counts)
        export_data['metadata']['complexity_distribution'] = dict(complexity_counts)
        export_data['metadata']['comparison_method_distribution'] = dict(method_counts)
        
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(export_data, f, indent=2, ensure_ascii=False)
        
        print(f"Exported {len(examples)} boundary examples to {output_file}")
        print("Disagreement distribution:", dict(disagreement_counts))
        print("Complexity distribution:", dict(complexity_counts))
        print("Comparison method distribution:", dict(method_counts))


def main():
    parser = argparse.ArgumentParser(description='Build boundary disagreement gold-set')
    parser.add_argument('--gutenberg-root', default='/home/steve/gutenberg',
                       help='Root directory of Gutenberg corpus')
    parser.add_argument('--output', default='boundary_gold_set.json',
                       help='Output JSON file')
    parser.add_argument('--max-files', type=int, default=20,
                       help='Maximum number of file pairs to analyze')
    parser.add_argument('--target-size', type=int, default=500,
                       help='Target number of boundary examples')
    parser.add_argument('--seed', type=int, default=42,
                       help='Random seed for reproducibility')
    
    args = parser.parse_args()
    
    random.seed(args.seed)
    
    builder = BoundaryGoldSetBuilder(args.gutenberg_root)
    examples = builder.build_boundary_gold_set(args.max_files, args.target_size)
    builder.export_boundary_gold_set(examples, args.output)


if __name__ == '__main__':
    main()