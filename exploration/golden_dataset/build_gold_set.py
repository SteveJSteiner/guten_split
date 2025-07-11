#!/usr/bin/env python3
"""
Gold-set sentence extractor for comparative analysis.

Analyzes discrepancies between _seams.txt and _sentences_fast.txt files
to create a curated test set for comparing against punkt, psybd, etc.
"""

import os
import re
import random
import json
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Set
from dataclasses import dataclass
from collections import defaultdict


@dataclass
class SentenceExample:
    """A sentence with metadata for gold-set analysis."""
    text: str
    source_file: str
    method: str  # 'seams', 'fast', 'both', 'neither'
    complexity: str  # 'dialog', 'normal', 'complex', 'abbreviation'
    context_before: str = ""
    context_after: str = ""
    line_number: int = 0


class GoldSetBuilder:
    """Builds a curated gold-set of sentences from gutenberg corpus discrepancies."""
    
    def __init__(self, gutenberg_root: str):
        self.gutenberg_root = Path(gutenberg_root)
        self.examples: List[SentenceExample] = []
        
        # Patterns for complexity classification
        self.dialog_patterns = [
            r'"[^"]*"',  # Quoted speech
            r"'[^']*'",  # Single quoted speech
            r'\b(?:said|asked|replied|answered|exclaimed|whispered|shouted)\b',
            r'^\s*"',    # Lines starting with quotes
            r'--'        # Em-dashes often used in dialog
        ]
        
        self.abbreviation_patterns = [
            r'\b(?:Mr|Mrs|Dr|Prof|St|Ave|Rd|etc|vs|i\.e|e\.g)\.',
            r'\b[A-Z]\.',  # Single letter abbreviations
            r'\b\w+\.\w+\.',  # Multiple abbreviations
        ]
        
        self.complex_patterns = [
            r'[;:]',     # Complex punctuation
            r'\([^)]*\)', # Parenthetical expressions
            r'--',       # Em-dashes
            r'\b(?:however|therefore|nevertheless|furthermore|moreover)\b',
        ]

    def find_file_pairs(self) -> List[Tuple[Path, Path]]:
        """Find matching _seams.txt and _sentences_fast.txt file pairs."""
        seams_files = list(self.gutenberg_root.glob("**/*_seams.txt"))
        pairs = []
        
        for seams_file in seams_files:
            # Convert seams file to corresponding sentences_fast file
            fast_file = seams_file.parent / seams_file.name.replace('_seams.txt', '_sentences_fast.txt')
            if fast_file.exists():
                pairs.append((seams_file, fast_file))
        
        return pairs

    def parse_sentences_with_coordinates(self, file_path: Path) -> List[tuple]:
        """Parse sentences from seams or fast sentence file with coordinates."""
        sentences = []
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        # Extract sentence text and coordinates
                        # Format: sentence_id TAB text [TAB (start_line,start_col,end_line,end_col)]
                        parts = line.split('\t')
                        if len(parts) >= 3:
                            # Seams format with coordinates
                            sentence_id = parts[0]
                            text_part = parts[1]
                            coord_part = parts[2]
                            
                            # Extract coordinates
                            coord_match = re.search(r'\((\d+),(\d+),(\d+),(\d+)\)', coord_part)
                            if coord_match:
                                start_line = int(coord_match.group(1))
                                start_col = int(coord_match.group(2))
                                end_line = int(coord_match.group(3))
                                end_col = int(coord_match.group(4))
                                
                                sentences.append((text_part.strip(), start_line, start_col, end_line, end_col))
                        elif len(parts) >= 2:
                            # Fast format without coordinates (just sentence_id TAB text)
                            sentence_id = parts[0]
                            text_part = parts[1]
                            
                            # For fast files without coordinates, we'll use dummy coordinates
                            # These will be skipped in the analysis since we need coordinates to extract original text
                            sentences.append((text_part.strip(), -1, -1, -1, -1))
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
        
        return sentences

    def extract_original_text(self, original_file: Path, start_line: int, start_col: int, 
                            end_line: int, end_col: int) -> str:
        """Extract original text from coordinates."""
        try:
            with open(original_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            if start_line > len(lines) or end_line > len(lines):
                return ""
            
            # Convert to 0-based indexing (coordinates appear to be 1-based)
            start_line_idx = start_line - 1
            end_line_idx = end_line - 1
            start_col_idx = start_col - 1
            end_col_idx = end_col - 1
            
            if start_line_idx == end_line_idx:
                # Single line
                line = lines[start_line_idx].rstrip('\n\r')
                if end_col_idx <= len(line):
                    return line[start_col_idx:end_col_idx]
                else:
                    return line[start_col_idx:]
            else:
                # Multiple lines
                text_parts = []
                
                # First line (from start_col to end)
                first_line = lines[start_line_idx].rstrip('\n\r')
                if start_col_idx < len(first_line):
                    text_parts.append(first_line[start_col_idx:])
                
                # Middle lines (complete lines)
                for line_idx in range(start_line_idx + 1, end_line_idx):
                    text_parts.append(lines[line_idx].rstrip('\n\r'))
                
                # Last line (from start to end_col)
                if end_line_idx < len(lines):
                    last_line = lines[end_line_idx].rstrip('\n\r')
                    if end_col_idx <= len(last_line):
                        text_parts.append(last_line[:end_col_idx])
                    else:
                        text_parts.append(last_line)
                
                return ' '.join(text_parts)
                
        except Exception as e:
            print(f"Error extracting text from {original_file}: {e}")
            return ""

    def classify_sentence_complexity(self, sentence: str) -> str:
        """Classify sentence complexity based on patterns."""
        sentence_lower = sentence.lower()
        
        # Check for dialog patterns
        for pattern in self.dialog_patterns:
            if re.search(pattern, sentence, re.IGNORECASE):
                return 'dialog'
        
        # Check for abbreviations
        for pattern in self.abbreviation_patterns:
            if re.search(pattern, sentence):
                return 'abbreviation'
        
        # Check for complex structures
        for pattern in self.complex_patterns:
            if re.search(pattern, sentence):
                return 'complex'
        
        return 'normal'

    def analyze_discrepancies(self, seams_file: Path, fast_file: Path) -> List[SentenceExample]:
        """Analyze discrepancies between seams and fast sentence extraction."""
        # Get the original text file
        # Convert 69826-0_seams.txt -> 69826-0.txt
        original_file = seams_file.parent / seams_file.name.replace('_seams.txt', '.txt')
        if not original_file.exists():
            print(f"Warning: Original file not found: {original_file}")
            return []
        
        seams_sentences = self.parse_sentences_with_coordinates(seams_file)
        fast_sentences = self.parse_sentences_with_coordinates(fast_file)
        
        examples = []
        
        # Only use seams sentences (which have coordinates) for extraction
        seams_by_text = {sent[0]: sent for sent in seams_sentences if sent[1] != -1}
        fast_texts = {sent[0] for sent in fast_sentences}
        
        seams_texts = set(seams_by_text.keys())
        
        # Find sentences only in seams (missed by fast) - these are interesting cases
        seams_only = seams_texts - fast_texts
        for norm_text in seams_only:
            if len(norm_text) > 20:  # Filter out very short fragments
                sent_data = seams_by_text[norm_text]
                original_text = self.extract_original_text(original_file, 
                                                         sent_data[1], sent_data[2], 
                                                         sent_data[3], sent_data[4])
                
                if original_text.strip() and len(original_text.strip()) > 20:  # Valid original text
                    example = SentenceExample(
                        text=original_text.strip(),
                        source_file=str(original_file.relative_to(self.gutenberg_root)),
                        method='seams_only',
                        complexity=self.classify_sentence_complexity(original_text),
                        line_number=sent_data[1]
                    )
                    examples.append(example)
        
        # Find sentences in both (for baseline comparison) - use seams coordinates
        both_texts = seams_texts & fast_texts
        both_sample = random.sample(list(both_texts), min(50, len(both_texts)))
        for norm_text in both_sample:
            if len(norm_text) > 20:
                sent_data = seams_by_text[norm_text]
                original_text = self.extract_original_text(original_file, 
                                                         sent_data[1], sent_data[2], 
                                                         sent_data[3], sent_data[4])
                
                if original_text.strip() and len(original_text.strip()) > 20:
                    example = SentenceExample(
                        text=original_text.strip(),
                        source_file=str(original_file.relative_to(self.gutenberg_root)),
                        method='both',
                        complexity=self.classify_sentence_complexity(original_text),
                        line_number=sent_data[1]
                    )
                    examples.append(example)
        
        # Sample some general seams sentences for variety
        all_seams_sample = random.sample(list(seams_texts), min(100, len(seams_texts)))
        for norm_text in all_seams_sample:
            if len(norm_text) > 20 and norm_text not in seams_only and norm_text not in both_sample:
                sent_data = seams_by_text[norm_text]
                original_text = self.extract_original_text(original_file, 
                                                         sent_data[1], sent_data[2], 
                                                         sent_data[3], sent_data[4])
                
                if original_text.strip() and len(original_text.strip()) > 20:
                    example = SentenceExample(
                        text=original_text.strip(),
                        source_file=str(original_file.relative_to(self.gutenberg_root)),
                        method='seams',
                        complexity=self.classify_sentence_complexity(original_text),
                        line_number=sent_data[1]
                    )
                    examples.append(example)
        
        return examples

    def build_gold_set(self, max_files: int = 50, target_size: int = 1000) -> List[SentenceExample]:
        """Build gold-set by analyzing discrepancies across files."""
        file_pairs = self.find_file_pairs()
        
        if max_files and len(file_pairs) > max_files:
            file_pairs = random.sample(file_pairs, max_files)
        
        all_examples = []
        
        print(f"Analyzing {len(file_pairs)} file pairs...")
        
        for i, (seams_file, fast_file) in enumerate(file_pairs):
            if i % 10 == 0:
                print(f"Processed {i}/{len(file_pairs)} files...")
            
            examples = self.analyze_discrepancies(seams_file, fast_file)
            all_examples.extend(examples)
        
        print(f"Found {len(all_examples)} potential examples")
        
        # Stratified sampling to get balanced representation
        examples_by_complexity = defaultdict(list)
        for example in all_examples:
            examples_by_complexity[example.complexity].append(example)
        
        # Target distribution (adjust as needed)
        target_distribution = {
            'dialog': 0.3,
            'normal': 0.4,
            'complex': 0.2,
            'abbreviation': 0.1
        }
        
        selected_examples = []
        for complexity, ratio in target_distribution.items():
            target_count = int(target_size * ratio)
            available = examples_by_complexity[complexity]
            
            if len(available) > target_count:
                selected = random.sample(available, target_count)
            else:
                selected = available
            
            selected_examples.extend(selected)
            print(f"Selected {len(selected)} {complexity} examples (target: {target_count})")
        
        # If we're under target, fill with random examples
        if len(selected_examples) < target_size:
            remaining_examples = [ex for ex in all_examples if ex not in selected_examples]
            additional_needed = target_size - len(selected_examples)
            if remaining_examples:
                additional = random.sample(remaining_examples, 
                                         min(additional_needed, len(remaining_examples)))
                selected_examples.extend(additional)
        
        # Shuffle final set
        random.shuffle(selected_examples)
        
        return selected_examples[:target_size]

    def export_gold_set(self, examples: List[SentenceExample], output_file: str):
        """Export gold-set to JSON for punkt/psybd comparison."""
        export_data = {
            'metadata': {
                'total_examples': len(examples),
                'complexity_distribution': {},
                'method_distribution': {},
                'description': 'Gold-set sentences from Gutenberg corpus discrepancy analysis'
            },
            'sentences': []
        }
        
        # Calculate distributions
        complexity_counts = defaultdict(int)
        method_counts = defaultdict(int)
        
        for example in examples:
            complexity_counts[example.complexity] += 1
            method_counts[example.method] += 1
            
            export_data['sentences'].append({
                'text': example.text,
                'source_file': example.source_file,
                'method': example.method,
                'complexity': example.complexity,
                'length': len(example.text),
                'line_number': example.line_number,
                'context_before': example.context_before,
                'context_after': example.context_after
            })
        
        export_data['metadata']['complexity_distribution'] = dict(complexity_counts)
        export_data['metadata']['method_distribution'] = dict(method_counts)
        
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(export_data, f, indent=2, ensure_ascii=False)
        
        print(f"Exported {len(examples)} examples to {output_file}")
        print("Complexity distribution:", dict(complexity_counts))
        print("Method distribution:", dict(method_counts))


def main():
    parser = argparse.ArgumentParser(description='Build gold-set from Gutenberg sentence discrepancies')
    parser.add_argument('--gutenberg-root', default='/home/steve/gutenberg',
                       help='Root directory of Gutenberg corpus')
    parser.add_argument('--output', default='gold_set_sentences.json',
                       help='Output JSON file')
    parser.add_argument('--max-files', type=int, default=50,
                       help='Maximum number of file pairs to analyze')
    parser.add_argument('--target-size', type=int, default=1000,
                       help='Target number of sentences in gold-set')
    parser.add_argument('--seed', type=int, default=42,
                       help='Random seed for reproducibility')
    
    args = parser.parse_args()
    
    random.seed(args.seed)
    
    builder = GoldSetBuilder(args.gutenberg_root)
    examples = builder.build_gold_set(args.max_files, args.target_size)
    builder.export_gold_set(examples, args.output)


if __name__ == '__main__':
    main()