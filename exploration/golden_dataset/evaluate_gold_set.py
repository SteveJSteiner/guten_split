#!/usr/bin/env python3
"""
Evaluate gold-set sentences against punkt, psybd, and other sentence segmenters.

Loads a gold-set JSON file and runs various sentence segmentation algorithms
to compare their performance.
"""

import json
import argparse
import re
from typing import List, Dict, Any
from dataclasses import dataclass
from collections import defaultdict


@dataclass
class EvaluationResult:
    """Results from evaluating a sentence segmenter."""
    algorithm: str
    complexity: str
    total_sentences: int
    correctly_segmented: int
    over_segmented: int
    under_segmented: int
    precision: float
    recall: float
    f1: float


class SentenceSegmenterEvaluator:
    """Evaluates sentence segmentation algorithms against gold-set."""
    
    def __init__(self, gold_set_file: str):
        with open(gold_set_file, 'r', encoding='utf-8') as f:
            self.gold_data = json.load(f)
        
        self.sentences = self.gold_data['sentences']
        print(f"Loaded {len(self.sentences)} gold-set sentences")
    
    def segment_with_punkt(self, text: str) -> List[str]:
        """Segment text using NLTK's punkt."""
        try:
            import nltk
            from nltk.tokenize import sent_tokenize
            
            # Download punkt if not available
            try:
                nltk.data.find('tokenizers/punkt')
            except LookupError:
                nltk.download('punkt')
            
            return sent_tokenize(text)
        except ImportError:
            print("NLTK not available, skipping punkt evaluation")
            return []
    
    def segment_with_psybd(self, text: str) -> List[str]:
        """Segment text using pysbd."""
        try:
            import pysbd
            seg = pysbd.Segmenter(language="en", clean=False)
            return seg.segment(text)
        except ImportError:
            print("pysbd not available, skipping pysbd evaluation")
            return []
    
    def segment_with_spacy(self, text: str) -> List[str]:
        """Segment text using spaCy."""
        try:
            import spacy
            
            # Try to load English model
            try:
                nlp = spacy.load("en_core_web_sm")
            except OSError:
                print("spaCy English model not available, skipping spaCy evaluation")
                return []
            
            doc = nlp(text)
            return [sent.text.strip() for sent in doc.sents]
        except ImportError:
            print("spaCy not available, skipping spaCy evaluation")
            return []
    
    def simple_regex_segmenter(self, text: str) -> List[str]:
        """Simple regex-based sentence segmenter for baseline."""
        # Split on sentence-ending punctuation followed by whitespace and capital letter
        sentences = re.split(r'(?<=[.!?])\s+(?=[A-Z])', text)
        return [s.strip() for s in sentences if s.strip()]
    
    def evaluate_segmenter(self, segmenter_func, name: str) -> Dict[str, EvaluationResult]:
        """Evaluate a sentence segmenter against the gold-set."""
        results_by_complexity = defaultdict(lambda: {
            'total': 0,
            'correct': 0,
            'over_segmented': 0,
            'under_segmented': 0
        })
        
        for sentence_data in self.sentences:
            gold_sentence = sentence_data['text']
            complexity = sentence_data['complexity']
            
            # Create context for sentence (simulate paragraph)
            context = f"Previous sentence. {gold_sentence} Next sentence."
            
            # Get segmentation from algorithm
            segments = segmenter_func(context)
            
            # Find which segment(s) contain our gold sentence
            containing_segments = []
            for segment in segments:
                if gold_sentence in segment:
                    containing_segments.append(segment)
            
            results_by_complexity[complexity]['total'] += 1
            
            if len(containing_segments) == 1:
                # Check if it's exactly the gold sentence (plus context)
                segment = containing_segments[0].strip()
                if segment == gold_sentence or segment.replace("Previous sentence. ", "").replace(" Next sentence.", "") == gold_sentence:
                    results_by_complexity[complexity]['correct'] += 1
                else:
                    # Partial match - could be over or under segmentation
                    if len(segment) < len(gold_sentence):
                        results_by_complexity[complexity]['under_segmented'] += 1
                    else:
                        results_by_complexity[complexity]['over_segmented'] += 1
            elif len(containing_segments) > 1:
                results_by_complexity[complexity]['under_segmented'] += 1
            else:
                results_by_complexity[complexity]['over_segmented'] += 1
        
        # Calculate metrics
        evaluation_results = {}
        for complexity, stats in results_by_complexity.items():
            total = stats['total']
            correct = stats['correct']
            over_seg = stats['over_segmented']
            under_seg = stats['under_segmented']
            
            precision = correct / total if total > 0 else 0
            recall = correct / total if total > 0 else 0
            f1 = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0
            
            evaluation_results[complexity] = EvaluationResult(
                algorithm=name,
                complexity=complexity,
                total_sentences=total,
                correctly_segmented=correct,
                over_segmented=over_seg,
                under_segmented=under_seg,
                precision=precision,
                recall=recall,
                f1=f1
            )
        
        return evaluation_results
    
    def run_evaluation(self) -> Dict[str, Dict[str, EvaluationResult]]:
        """Run evaluation against all available segmenters."""
        segmenters = [
            (self.segment_with_punkt, "punkt"),
            (self.segment_with_psybd, "pysbd"),
            (self.segment_with_spacy, "spacy"),
            (self.simple_regex_segmenter, "simple_regex")
        ]
        
        all_results = {}
        
        for segmenter_func, name in segmenters:
            print(f"Evaluating {name}...")
            try:
                results = self.evaluate_segmenter(segmenter_func, name)
                all_results[name] = results
            except Exception as e:
                print(f"Error evaluating {name}: {e}")
        
        return all_results
    
    def print_results(self, results: Dict[str, Dict[str, EvaluationResult]]):
        """Print evaluation results in a readable format."""
        complexities = ['normal', 'dialog', 'complex', 'abbreviation']
        
        print("\n" + "="*80)
        print("SENTENCE SEGMENTATION EVALUATION RESULTS")
        print("="*80)
        
        for complexity in complexities:
            print(f"\n{complexity.upper()} SENTENCES:")
            print("-" * 50)
            print(f"{'Algorithm':<15} {'Precision':<10} {'Recall':<10} {'F1':<10} {'Correct':<8} {'Total':<8}")
            print("-" * 50)
            
            for algorithm, complexity_results in results.items():
                if complexity in complexity_results:
                    result = complexity_results[complexity]
                    print(f"{algorithm:<15} {result.precision:<10.3f} {result.recall:<10.3f} "
                          f"{result.f1:<10.3f} {result.correctly_segmented:<8} {result.total_sentences:<8}")
        
        # Overall summary
        print(f"\nOVERALL SUMMARY:")
        print("-" * 50)
        print(f"{'Algorithm':<15} {'Avg F1':<10} {'Total Correct':<15} {'Total Sentences':<15}")
        print("-" * 50)
        
        for algorithm, complexity_results in results.items():
            total_correct = sum(r.correctly_segmented for r in complexity_results.values())
            total_sentences = sum(r.total_sentences for r in complexity_results.values())
            avg_f1 = sum(r.f1 for r in complexity_results.values()) / len(complexity_results)
            
            print(f"{algorithm:<15} {avg_f1:<10.3f} {total_correct:<15} {total_sentences:<15}")
    
    def export_results(self, results: Dict[str, Dict[str, EvaluationResult]], output_file: str):
        """Export results to JSON."""
        export_data = {
            'metadata': {
                'gold_set_size': len(self.sentences),
                'algorithms_tested': list(results.keys()),
                'complexity_types': list(set(s['complexity'] for s in self.sentences))
            },
            'results': {}
        }
        
        for algorithm, complexity_results in results.items():
            export_data['results'][algorithm] = {}
            for complexity, result in complexity_results.items():
                export_data['results'][algorithm][complexity] = {
                    'total_sentences': result.total_sentences,
                    'correctly_segmented': result.correctly_segmented,
                    'over_segmented': result.over_segmented,
                    'under_segmented': result.under_segmented,
                    'precision': result.precision,
                    'recall': result.recall,
                    'f1': result.f1
                }
        
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(export_data, f, indent=2)
        
        print(f"\nResults exported to {output_file}")


def main():
    parser = argparse.ArgumentParser(description='Evaluate sentence segmenters against gold-set')
    parser.add_argument('gold_set', help='Gold-set JSON file')
    parser.add_argument('--output', help='Output JSON file for results')
    
    args = parser.parse_args()
    
    evaluator = SentenceSegmenterEvaluator(args.gold_set)
    results = evaluator.run_evaluation()
    evaluator.print_results(results)
    
    if args.output:
        evaluator.export_results(results, args.output)


if __name__ == '__main__':
    main()