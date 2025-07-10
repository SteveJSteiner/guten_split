"""
Comprehensive benchmark comparison runner for seams vs Python alternatives.
Runs all benchmarks and generates comparative analysis.
"""

import os
import sys
import json
import time
import subprocess
import platform
from pathlib import Path
from typing import Dict, Any, List
import argparse

def get_system_info() -> Dict[str, Any]:
    """Collect system information for hardware context."""
    import psutil
    
    return {
        "platform": platform.platform(),
        "processor": platform.processor(),
        "python_version": platform.python_version(),
        "cpu_count": psutil.cpu_count(),
        "memory_gb": round(psutil.virtual_memory().total / (1024**3), 2),
        "architecture": platform.machine()
    }

def run_seams_benchmark(root_dir: str, stats_file: str, max_files: int = None) -> Dict[str, Any]:
    """Run seams benchmark and extract results."""
    benchmark_start = time.time()
    
    print("🔨 Building seams...")
    if max_files:
        print(f"   Note: seams will process ALL files (--max-files not supported yet)")
    
    # Build seams first
    build_result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=Path(__file__).parent.parent,
        capture_output=True,
        text=True
    )
    
    if build_result.returncode != 0:
        print("❌ Build failed!")
        return {
            "success": False,
            "error": f"Build failed: {build_result.stderr}"
        }
    
    print("🏃 Running seams benchmark...")
    
    # Run seams benchmark
    start_time = time.time()
    cmd = ["./target/release/seams", root_dir, "--stats-out", stats_file, "--overwrite-all"]
    # Note: seams doesn't support --max-files yet, so it processes all files
    
    result = subprocess.run(
        cmd,
        cwd=Path(__file__).parent.parent,
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print("❌ Seams benchmark failed!")
        return {
            "success": False,
            "error": f"Seams failed: {result.stderr}"
        }
    
    # Load stats
    try:
        with open(Path(__file__).parent.parent / stats_file, 'r') as f:
            stats = json.load(f)
        
        # Calculate sentence statistics
        total_sentences = stats.get("total_sentences_detected", 0)
        files_failed = stats.get("files_failed", 0)
        
        # Extract sentence counts from file_stats for detailed analysis
        sentence_counts = []
        successful_files = []
        for file_stat in stats.get("file_stats", []):
            if file_stat.get("status") == "success":
                sentence_counts.append(file_stat.get("sentences_detected", 0))
                successful_files.append(file_stat)
        
        sentence_stats = {}
        if sentence_counts:
            sorted_counts = sorted(sentence_counts)
            n = len(sorted_counts)
            
            sentence_stats = {
                "min": min(sentence_counts),
                "max": max(sentence_counts),
                "average": sum(sentence_counts) / len(sentence_counts),
                "median": sorted_counts[n // 2] if n % 2 == 1 else (sorted_counts[n // 2 - 1] + sorted_counts[n // 2]) / 2,
                "q25": sorted_counts[n // 4],
                "q75": sorted_counts[3 * n // 4],
                "total": sum(sentence_counts)
            }
        
        benchmark_time = time.time() - benchmark_start
        
        # Print preliminary results
        throughput = stats.get("overall_chars_per_sec", 0)
        chars_processed = stats.get("total_chars_processed", 0)
        files_processed = stats.get("files_processed", 0)
        
        print(f"✅ seams completed:")
        print(f"   Total e2e time: {benchmark_time:.2f}s")
        print(f"   Files: {files_processed}/{files_processed + files_failed}")
        print(f"   Chars: {chars_processed:,}")
        print(f"   Sentences: {total_sentences:,} (min: {sentence_stats.get('min', 0)}, Q25: {sentence_stats.get('q25', 0)}, avg: {sentence_stats.get('average', 0):.1f}, median: {sentence_stats.get('median', 0):.1f}, Q75: {sentence_stats.get('q75', 0)}, max: {sentence_stats.get('max', 0)})")
        
        # Show sentence detection throughput if available
        sentence_detection_throughput = stats.get("sentence_detection_chars_per_sec", 0)
        if sentence_detection_throughput > 0:
            print(f"   Sentence detection throughput: {sentence_detection_throughput:,.0f} chars/sec ({sentence_detection_throughput/(1024*1024):.2f} MB/sec)")
        print(f"   Total e2e throughput: {throughput:,.0f} chars/sec ({throughput/(1024*1024):.2f} MB/sec)")
        if files_failed > 0:
            print(f"   ⚠️  {files_failed} file(s) failed:")
            for file_stat in stats.get("file_stats", []):
                if file_stat.get("status") == "failed":
                    file_path = file_stat.get("path", "unknown")
                    error = file_stat.get("error", "No error details")
                    if file_path == "unknown":
                        print(f"      • Unknown file: {error}")
                    else:
                        print(f"      • {Path(file_path).name}: {error}")
        print()
        
        # Add benchmark timing and sentence stats to stats
        stats["benchmark_e2e_time_s"] = benchmark_time
        stats["sentence_stats"] = sentence_stats
        
        return {
            "success": True,
            "tool": "seams",
            "stats": stats,
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except Exception as e:
        print("❌ Failed to load seams stats!")
        return {
            "success": False,
            "error": f"Failed to load seams stats: {e}"
        }

def run_python_benchmark_with_venv(script_path: str, root_dir: str, stats_file: str, python_cmd: str, max_files: int = None) -> Dict[str, Any]:
    """Run a Python benchmark script with virtual environment."""
    benchmark_start = time.time()
    
    script_name = Path(script_path).stem
    tool_name = script_name.replace("python_", "").replace("_benchmark", "")
    print(f"🐍 Running {tool_name} benchmark...")
    
    cmd = [python_cmd, script_path, root_dir, "--stats_out", stats_file]
    if max_files:
        cmd.extend(["--max_files", str(max_files)])
    
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"❌ {tool_name} benchmark failed!")
        return {
            "success": False,
            "error": f"{script_name} failed: {result.stderr}",
            "stdout": result.stdout
        }
    
    # Load stats
    try:
        with open(stats_file, 'r') as f:
            stats = json.load(f)
        
        # Calculate sentence statistics from results
        total_sentences = stats.get("total_sentences", 0)
        sentence_counts = []
        for file_result in stats.get("results", []):
            if file_result.get("status") == "success" or file_result.get("success"):
                sentence_counts.append(file_result.get("sentences_detected") or file_result.get("sentence_count", 0))
        
        sentence_stats = {}
        if sentence_counts:
            sorted_counts = sorted(sentence_counts)
            n = len(sorted_counts)
            
            sentence_stats = {
                "min": min(sentence_counts),
                "max": max(sentence_counts),
                "average": sum(sentence_counts) / len(sentence_counts),
                "median": sorted_counts[n // 2] if n % 2 == 1 else (sorted_counts[n // 2 - 1] + sorted_counts[n // 2]) / 2,
                "q25": sorted_counts[n // 4],
                "q75": sorted_counts[3 * n // 4],
                "total": sum(sentence_counts)
            }
        
        benchmark_time = time.time() - benchmark_start
        
        # Print preliminary results
        throughput = stats.get("aggregate_throughput_chars_per_sec", 0)
        chars_processed = stats.get("total_chars_processed", 0)
        files_processed = stats.get("successful_files", 0)
        failed_files = stats.get("failed_files", 0)
        
        # Calculate total e2e throughput
        total_e2e_throughput = chars_processed / benchmark_time if benchmark_time > 0 else 0
        
        print(f"✅ {tool_name} completed:")
        print(f"   Total e2e time: {benchmark_time:.2f}s")
        print(f"   Files: {files_processed}/{files_processed + failed_files}")
        print(f"   Chars: {chars_processed:,}")
        print(f"   Sentences: {total_sentences:,} (min: {sentence_stats.get('min', 0)}, Q25: {sentence_stats.get('q25', 0)}, avg: {sentence_stats.get('average', 0):.1f}, median: {sentence_stats.get('median', 0):.1f}, Q75: {sentence_stats.get('q75', 0)}, max: {sentence_stats.get('max', 0)})")
        print(f"   Sentence detection throughput: {throughput:,.0f} chars/sec ({throughput/(1024*1024):.2f} MB/sec)")
        print(f"   Total e2e throughput: {total_e2e_throughput:,.0f} chars/sec ({total_e2e_throughput/(1024*1024):.2f} MB/sec)")
        if failed_files > 0:
            print(f"   ⚠️  {failed_files} file(s) failed:")
            for file_result in stats.get("results", []):
                if not file_result.get("success", True):
                    file_path = file_result.get("file_path", "unknown")
                    error = file_result.get("error", "No error details")
                    print(f"      • {Path(file_path).name}: {error}")
        print()
        
        # Add benchmark timing, sentence stats, and total e2e throughput to stats
        stats["benchmark_e2e_time_s"] = benchmark_time
        stats["sentence_stats"] = sentence_stats
        stats["total_e2e_throughput_chars_per_sec"] = total_e2e_throughput
        
        return {
            "success": True,
            "tool": stats.get("tool", script_name),
            "stats": stats,
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except Exception as e:
        print(f"❌ Failed to load {tool_name} stats!")
        return {
            "success": False,
            "error": f"Failed to load {script_name} stats: {e}"
        }

def find_common_successful_files(results: List[Dict[str, Any]]) -> set:
    """Find files that were successfully processed by ALL tools."""
    successful_results = [r for r in results if r["success"]]
    
    if not successful_results:
        return set()
    
    # Get successful file paths for each tool
    all_file_sets = []
    for result in successful_results:
        if "results" in result["stats"]:
            # Python format
            successful_files = {
                r.get("file_path") or r.get("path") for r in result["stats"]["results"] 
                if r.get("success", True) or r.get("status") == "success"
            }
            all_file_sets.append(successful_files)
        elif "file_stats" in result["stats"]:
            # Seams format
            successful_files = {
                r["path"] for r in result["stats"]["file_stats"] 
                if r.get("status") == "success"
            }
            all_file_sets.append(successful_files)
    
    # Find intersection (files successful in ALL tools)
    if all_file_sets:
        return set.intersection(*all_file_sets)
    return set()

def recalculate_stats_for_common_files(result: Dict[str, Any], common_files: set) -> Dict[str, Any]:
    """Recalculate stats using only the common successful files."""
    if not result["success"]:
        return result
    
    stats = result["stats"]
    
    # Handle different result formats
    if "results" in stats:
        # Python format
        file_results = stats["results"]
    elif "file_stats" in stats:
        # Seams format - just return as-is since we can't easily recalculate seams stats
        return result
    else:
        return result
    
    # Filter to only common files
    common_results = [
        r for r in file_results 
        if (r.get("file_path") in common_files or r.get("path") in common_files) and (r.get("success", True) or r.get("status") == "success")
    ]
    
    if not common_results:
        return result
    
    # Recalculate totals
    total_chars = sum(r.get("chars_processed", 0) for r in common_results)
    total_sentences = sum(r.get("sentence_count", 0) or r.get("sentences_detected", 0) for r in common_results)
    total_time = sum(r.get("processing_time_ms", 0) for r in common_results) / 1000.0  # Convert to seconds
    
    # Create new stats
    new_stats = stats.copy()
    new_stats.update({
        "total_files": len(common_results),
        "successful_files": len(common_results),
        "failed_files": 0,
        "total_chars_processed": total_chars,
        "total_sentences": total_sentences,
        "total_time_ms": total_time * 1000,
        "aggregate_throughput_chars_per_sec": total_chars / total_time if total_time > 0 else 0,
        "aggregate_throughput_mb_per_sec": (total_chars / total_time) / (1024 * 1024) if total_time > 0 else 0,
        "common_files_only": True,
        "common_files_count": len(common_results)
    })
    
    result_copy = result.copy()
    result_copy["stats"] = new_stats
    return result_copy

def show_current_leaderboard(results: List[Dict[str, Any]]) -> None:
    """Show current leaderboard of completed benchmarks."""
    successful_results = [r for r in results if r["success"]]
    
    if len(successful_results) < 2:
        return
    
    # Extract throughput and timing for ranking
    rankings = []
    for result in successful_results:
        stats = result["stats"]
        if "overall_chars_per_sec" in stats:
            throughput = stats["overall_chars_per_sec"]
        elif "aggregate_chars_per_sec" in stats:
            throughput = stats["aggregate_chars_per_sec"]
        else:
            throughput = stats.get("aggregate_throughput_chars_per_sec", 0)
        
        # Get e2e time
        e2e_time = stats.get("benchmark_e2e_time_s", 0)
        
        rankings.append({
            "tool": result["tool"],
            "throughput": throughput,
            "e2e_time": e2e_time
        })
    
    # Sort by throughput
    rankings.sort(key=lambda x: x["throughput"], reverse=True)
    
    print("📊 Current leaderboard:")
    for i, item in enumerate(rankings):
        relative = item["throughput"] / rankings[0]["throughput"] if rankings[0]["throughput"] > 0 else 0
        e2e_time = item.get("e2e_time", 0)
        time_str = f" ({e2e_time:.1f}s)" if e2e_time > 0 else ""
        print(f"   {i+1}. {item['tool']}: {item['throughput']:,.0f} chars/sec [{relative:.2f}x]{time_str}")
    print()

def generate_comparison_report(results: List[Dict[str, Any]], system_info: Dict[str, Any]) -> Dict[str, Any]:
    """Generate comprehensive comparison report."""
    successful_results = [r for r in results if r["success"]]
    
    if not successful_results:
        return {
            "error": "No successful benchmark results",
            "system_info": system_info,
            "results": results
        }
    
    # Find common files processed by all tools
    common_files = find_common_successful_files(results)
    
    if not common_files:
        return {
            "error": "No files were successfully processed by all tools",
            "system_info": system_info,
            "results": results
        }
    
    # Recalculate stats for common files only
    normalized_results = []
    for result in successful_results:
        normalized_result = recalculate_stats_for_common_files(result, common_files)
        normalized_results.append(normalized_result)
    
    # Extract key metrics
    performance_comparison = []
    for result in normalized_results:
        stats = result["stats"]
        
        # Handle both seams and Python stats formats
        if "overall_chars_per_sec" in stats:
            # Seams format
            throughput = stats["overall_chars_per_sec"]
            chars_processed = stats["total_chars_processed"]
            files_processed = stats["files_processed"]
        elif "aggregate_chars_per_sec" in stats:
            # Legacy seams format
            throughput = stats["aggregate_chars_per_sec"]
            chars_processed = stats["total_chars_processed"]
            files_processed = stats["total_files_processed"]
        else:
            # Python format
            throughput = stats.get("aggregate_throughput_chars_per_sec", 0)
            chars_processed = stats.get("total_chars_processed", 0)
            files_processed = stats.get("successful_files", 0)
        
        performance_comparison.append({
            "tool": result["tool"],
            "throughput_chars_per_sec": throughput,
            "throughput_mb_per_sec": throughput / (1024 * 1024) if throughput > 0 else 0,
            "chars_processed": chars_processed,
            "files_processed": files_processed,
            "total_time_ms": stats.get("total_time_ms", 0)
        })
    
    # Sort by throughput
    performance_comparison.sort(key=lambda x: x["throughput_chars_per_sec"], reverse=True)
    
    # Calculate relative performance
    if performance_comparison:
        baseline = performance_comparison[0]["throughput_chars_per_sec"]
        for item in performance_comparison:
            item["relative_performance"] = item["throughput_chars_per_sec"] / baseline if baseline > 0 else 0
    
    return {
        "system_info": system_info,
        "performance_comparison": performance_comparison,
        "common_files_processed": len(common_files),
        "methodology_note": "Performance comparison based on files successfully processed by ALL tools",
        "raw_results": results,
        "normalized_results": normalized_results,
        "benchmark_timestamp": time.time()
    }

def main():
    parser = argparse.ArgumentParser(description="Run comprehensive sentence segmentation benchmark comparison")
    parser.add_argument("root_dir", help="Root directory containing *-0.txt files")
    parser.add_argument("--output", default="benchmark_comparison.json", help="Output comparison file")
    parser.add_argument("--max_files", type=int, help="Maximum files to process (for testing)")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.root_dir):
        print(f"ERROR: Directory {args.root_dir} does not exist")
        sys.exit(1)
    
    # Get system info
    try:
        system_info = get_system_info()
    except ImportError:
        print("WARNING: psutil not installed, system info limited")
        system_info = {
            "platform": platform.platform(),
            "python_version": platform.python_version(),
            "architecture": platform.machine()
        }
    
    benchmarks_dir = Path(__file__).parent
    results = []
    
    print("🚀 Starting sentence segmentation benchmark comparison...")
    print(f"📁 Processing files from: {args.root_dir}")
    print(f"💻 System: {system_info.get('platform', 'Unknown')}")
    print(f"🧠 Memory: {system_info.get('memory_gb', 'Unknown')} GB")
    print()
    
    # Run seams benchmark
    seams_result = run_seams_benchmark(args.root_dir, "seams_comparison_stats.json", args.max_files)
    results.append(seams_result)
    
    # Run Python benchmarks (using venv python)
    python_benchmarks = [
        #("python_pysbd_benchmark.py", "python_pysbd_stats.json"),
        #("python_spacy_benchmark.py", "python_spacy_stats.json"),
        ("python_nupunkt_benchmark.py", "python_nupunkt_stats.json")
    ]
    
    # Check for virtual environment
    venv_python = benchmarks_dir / ".venv" / "bin" / "python"
    python_cmd = str(venv_python) if venv_python.exists() else "python3"
    
    for script_name, stats_file in python_benchmarks:
        script_path = benchmarks_dir / script_name
        if script_path.exists():
            result = run_python_benchmark_with_venv(str(script_path), args.root_dir, stats_file, python_cmd, args.max_files)
            results.append(result)
            
            # Show current leaderboard
            show_current_leaderboard(results)
        else:
            print(f"WARNING: {script_name} not found, skipping")
    
    # Generate comparison report
    comparison = generate_comparison_report(results, system_info)
    
    # Write comparison results
    with open(args.output, 'w') as f:
        json.dump(comparison, f, indent=2)
    
    # Print summary table
    print(f"\n=== Benchmark Comparison Summary ===")
    print(f"| Benchmark (version) | Cores | End-to-end time | Speed-up vs nupunkt | Sentences / s | Sentence detection throughput | Total e2e throughput | Note |")
    print(f"|---------------------|:----:|---------------:|--------------------:|--------------:|-----------------------------:|--------------------:|------|")

    # Get nupunkt as baseline
    baseline_results = comparison.get("raw_results", []) if comparison.get("raw_results") else results
    nupunkt_results = next((r for r in baseline_results if r.get("tool") == "nupunkt"), None)
    nupunkt_e2e_time = nupunkt_results.get("stats", {}).get("benchmark_e2e_time_s", 1) if nupunkt_results else 1

    # Process results in specific order: seams first, then others
    # Use raw_results if available, otherwise fall back to results list
    if comparison.get("raw_results"):
        all_results = comparison.get("raw_results", [])
    else:
        all_results = results
    
    successful_results = [r for r in all_results if r.get("success")]
    
    # Sort to put seams first, then others alphabetically
    successful_results.sort(key=lambda x: (0 if x.get("tool") == "seams" else 1, x.get("tool", "")))

    for result in successful_results:
        tool = result.get("tool")
        stats = result.get("stats")
        version = stats.get("version", "")
        cores = system_info.get('cpu_count', 1) if tool == "seams" else 1
        e2e_time = stats.get("benchmark_e2e_time_s", 0)
        speed_up = nupunkt_e2e_time / e2e_time if e2e_time > 0 else 0
        total_sentences = stats.get("total_sentences_detected") or stats.get("total_sentences", 0)
        sentences_per_sec = total_sentences / e2e_time if e2e_time > 0 else 0
        
        # Format tool name and version
        if version:
            tool_display = f"{tool} ({version})"
        else:
            tool_display = tool
            
        # Apply bold formatting for seams
        if tool == "seams":
            tool_display = f"**{tool_display}**"
            
        # Format cores
        cores_str = f"**{cores}**" if tool == "seams" else str(cores)
        
        # Format time
        if tool == "seams":
            if e2e_time < 60:
                time_str = f"**{e2e_time:.0f} s**"
            else:
                minutes = int(e2e_time // 60)
                seconds = int(e2e_time % 60)
                time_str = f"**{minutes} m {seconds} s**"
        else:
            if e2e_time < 60:
                time_str = f"{e2e_time:.0f} s"
            else:
                minutes = int(e2e_time // 60)
                seconds = int(e2e_time % 60)
                time_str = f"{minutes} m {seconds} s"
        
        # Format speed-up
        if tool == "seams":
            speed_up_str = f"**{speed_up:.0f} ×**"
        else:
            if speed_up < 1:
                speed_up_str = f"{speed_up:.2f} ×"
            else:
                speed_up_str = f"{speed_up:.0f} ×"
        
        # Format sentences per second
        if tool == "seams":
            sentences_per_sec_str = f"**{sentences_per_sec / 1000000:.1f} M**"
        else:
            if sentences_per_sec >= 1000:
                sentences_per_sec_str = f"{sentences_per_sec / 1000:.0f} k"
            else:
                sentences_per_sec_str = f"{sentences_per_sec:.0f}"
        
        # Get sentence detection throughput and total throughput
        if tool == "seams":
            sentence_detection_throughput = stats.get("sentence_detection_chars_per_sec", 0)
            total_throughput = stats.get("overall_chars_per_sec", 0)
            
            # Format sentence detection throughput
            if sentence_detection_throughput > 0:
                sentence_detection_str = f"**{sentence_detection_throughput / 1000000:.1f} MB/s**"
            else:
                sentence_detection_str = "**N/A**"
            
            # Format total throughput
            total_throughput_str = f"**{total_throughput / 1000000:.1f} MB/s**"
        else:
            # For Python tools, get both throughputs
            sentence_detection_throughput = stats.get("aggregate_throughput_chars_per_sec", 0)
            total_throughput = stats.get("total_e2e_throughput_chars_per_sec", sentence_detection_throughput)
            
            # Format throughputs
            if sentence_detection_throughput > 0:
                sentence_detection_str = f"{sentence_detection_throughput / 1000000:.1f} MB/s"
            else:
                sentence_detection_str = "N/A"
                
            if total_throughput > 0:
                total_throughput_str = f"{total_throughput / 1000000:.1f} MB/s"
            else:
                total_throughput_str = "N/A"
        
        # Set note
        note = "line offsets included" if tool == "seams" else "pure-Python"

        print(f"| {tool_display} | {cores_str} | {time_str} | {speed_up_str} | {sentences_per_sec_str} | {sentence_detection_str} | {total_throughput_str} | {note} |")
    
    print(f"\nDetailed results written to: {args.output}")

if __name__ == "__main__":
    main()