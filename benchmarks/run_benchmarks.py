#!/usr/bin/env python3
"""
Fast Forth Benchmark Runner
Comprehensive performance benchmarking and comparison tool

Usage:
    ./run_benchmarks.py --all
    ./run_benchmarks.py --c
    ./run_benchmarks.py --gforth
    ./run_benchmarks.py --rust
    ./run_benchmarks.py --report
"""

import subprocess
import json
import time
import sys
import os
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import statistics

class BenchmarkRunner:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.c_baseline_dir = self.project_root / "benchmarks" / "c_baseline"
        self.forth_dir = self.project_root / "benchmarks" / "forth"
        self.results = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "platform": self._get_platform_info(),
            "benchmarks": {}
        }

    def _get_platform_info(self) -> Dict[str, str]:
        """Get platform and compiler information"""
        try:
            gcc_version = subprocess.check_output(
                ["gcc", "--version"],
                stderr=subprocess.STDOUT
            ).decode().split('\n')[0]
        except:
            gcc_version = "Unknown"

        try:
            gforth_version = subprocess.check_output(
                ["gforth", "--version"],
                stderr=subprocess.STDOUT
            ).decode().split('\n')[0]
        except:
            gforth_version = "Not installed"

        return {
            "os": os.uname().sysname,
            "machine": os.uname().machine,
            "gcc_version": gcc_version,
            "gforth_version": gforth_version
        }

    def run_c_benchmark(self, name: str, command: List[str]) -> Dict[str, any]:
        """Run a C benchmark and extract timing"""
        print(f"  Running C {name}...")
        try:
            result = subprocess.run(
                command,
                cwd=self.c_baseline_dir,
                capture_output=True,
                text=True,
                timeout=120
            )

            output = result.stdout

            # Extract average time
            for line in output.split('\n'):
                if 'Average time:' in line:
                    time_str = line.split(':')[1].strip().split()[0]
                    avg_time = float(time_str)

                    return {
                        "status": "success",
                        "avg_time_ms": avg_time,
                        "output": output,
                        "validation": "PASS" if "PASS" in output else "FAIL" if "FAIL" in output else "N/A"
                    }

            return {
                "status": "no_timing",
                "output": output
            }
        except subprocess.TimeoutExpired:
            return {"status": "timeout"}
        except Exception as e:
            return {"status": "error", "error": str(e)}

    def run_gforth_benchmark(self, name: str, forth_file: str) -> Dict[str, any]:
        """Run a Forth benchmark with GForth"""
        print(f"  Running GForth {name}...")
        forth_path = self.forth_dir / forth_file

        if not forth_path.exists():
            return {"status": "not_found"}

        try:
            # GForth doesn't have built-in timing, so we measure externally
            start = time.time()
            result = subprocess.run(
                ["gforth", str(forth_path), "-e", "bye"],
                capture_output=True,
                text=True,
                timeout=120
            )
            elapsed = (time.time() - start) * 1000

            return {
                "status": "success",
                "avg_time_ms": elapsed,
                "output": result.stdout,
                "stderr": result.stderr
            }
        except subprocess.TimeoutExpired:
            return {"status": "timeout"}
        except Exception as e:
            return {"status": "error", "error": str(e)}

    def run_all_c_benchmarks(self):
        """Run all C baseline benchmarks"""
        print("\n" + "="*60)
        print("C BASELINE BENCHMARKS (gcc -O2)")
        print("="*60)

        benchmarks = {
            "sieve": ["./sieve", "8190", "100"],
            "fibonacci_rec": ["./fibonacci", "35", "40"],
            "matrix": ["./matrix", "100", "10"],
            "bubble_sort": ["./bubble_sort", "1000", "10"],
            "string_ops": ["./string_ops", "10000", "10000"]
        }

        for name, command in benchmarks.items():
            result = self.run_c_benchmark(name, command)
            if "c" not in self.results["benchmarks"]:
                self.results["benchmarks"]["c"] = {}
            self.results["benchmarks"]["c"][name] = result

            if result["status"] == "success":
                print(f"    ✓ {name}: {result['avg_time_ms']:.3f} ms")
            else:
                print(f"    ✗ {name}: {result['status']}")

    def run_all_gforth_benchmarks(self):
        """Run all GForth benchmarks"""
        print("\n" + "="*60)
        print("GFORTH BENCHMARKS")
        print("="*60)

        benchmarks = {
            "sieve": "sieve.fth",
            "fibonacci": "fibonacci.fth",
            "matrix": "matrix.fth",
            "bubble_sort": "bubble_sort.fth"
        }

        for name, forth_file in benchmarks.items():
            result = self.run_gforth_benchmark(name, forth_file)
            if "gforth" not in self.results["benchmarks"]:
                self.results["benchmarks"]["gforth"] = {}
            self.results["benchmarks"]["gforth"][name] = result

            if result["status"] == "success":
                print(f"    ✓ {name}: {result['avg_time_ms']:.3f} ms")
            else:
                print(f"    ✗ {name}: {result['status']}")

    def run_rust_benchmarks(self):
        """Run Rust reference benchmarks"""
        print("\n" + "="*60)
        print("RUST REFERENCE BENCHMARKS")
        print("="*60)

        try:
            result = subprocess.run(
                ["cargo", "test", "--release", "--", "--nocapture", "bench_"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=300
            )

            print(result.stdout)
            self.results["benchmarks"]["rust"] = {
                "status": "completed",
                "output": result.stdout
            }
        except Exception as e:
            print(f"    ✗ Error: {e}")
            self.results["benchmarks"]["rust"] = {
                "status": "error",
                "error": str(e)
            }

    def generate_report(self):
        """Generate comprehensive performance report"""
        print("\n" + "="*60)
        print("PERFORMANCE COMPARISON REPORT")
        print("="*60)

        print(f"\nPlatform: {self.results['platform']['os']} {self.results['platform']['machine']}")
        print(f"GCC: {self.results['platform']['gcc_version']}")
        print(f"GForth: {self.results['platform']['gforth_version']}")
        print(f"Timestamp: {self.results['timestamp']}")

        # Compare C vs targets
        if "c" in self.results["benchmarks"]:
            print("\n" + "-"*60)
            print("C BASELINE vs TARGET")
            print("-"*60)

            targets = {
                "sieve": 50.0,  # ms
                "fibonacci_rec": 35.0,
                "matrix": 80.0,
                "bubble_sort": 50.0,
                "string_ops": 1.0
            }

            for name, target in targets.items():
                if name in self.results["benchmarks"]["c"]:
                    result = self.results["benchmarks"]["c"][name]
                    if result["status"] == "success":
                        actual = result["avg_time_ms"]
                        ratio = actual / target
                        status = "✓" if ratio <= 1.5 else "⚠"
                        print(f"  {status} {name:20s}: {actual:8.3f} ms (target: {target:.1f} ms, {ratio:.2f}x)")

        # Save results to JSON
        results_file = self.project_root / "benchmarks" / "results.json"
        with open(results_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        print(f"\nResults saved to: {results_file}")

        # Save detailed report
        report_file = self.project_root / "benchmarks" / "BENCHMARK_REPORT.md"
        self._write_markdown_report(report_file)
        print(f"Report saved to: {report_file}")

    def _write_markdown_report(self, filepath: Path):
        """Write detailed markdown report"""
        with open(filepath, 'w') as f:
            f.write("# Fast Forth Benchmark Report\n\n")
            f.write(f"**Date**: {self.results['timestamp']}\n\n")
            f.write("## Platform Information\n\n")
            f.write(f"- **OS**: {self.results['platform']['os']}\n")
            f.write(f"- **Machine**: {self.results['platform']['machine']}\n")
            f.write(f"- **GCC**: {self.results['platform']['gcc_version']}\n")
            f.write(f"- **GForth**: {self.results['platform']['gforth_version']}\n\n")

            if "c" in self.results["benchmarks"]:
                f.write("## C Baseline Results (gcc -O2)\n\n")
                f.write("| Benchmark | Time (ms) | Status |\n")
                f.write("|-----------|-----------|--------|\n")

                for name, result in self.results["benchmarks"]["c"].items():
                    if result["status"] == "success":
                        f.write(f"| {name} | {result['avg_time_ms']:.3f} | {result.get('validation', 'N/A')} |\n")
                    else:
                        f.write(f"| {name} | - | {result['status']} |\n")

                f.write("\n")

            f.write("## Target Performance Comparison\n\n")
            f.write("Based on BENCHMARK_SUITE_SPECIFICATION.md targets:\n\n")
            f.write("| Benchmark | Target (ms) | Actual (ms) | Ratio | Status |\n")
            f.write("|-----------|-------------|-------------|-------|--------|\n")

            targets = {
                "sieve": (50.0, "Target for gcc -O2"),
                "fibonacci_rec": (35.0, "Recursive fib(35)"),
                "matrix": (80.0, "100x100 matrices"),
                "bubble_sort": (50.0, "1000 elements"),
                "string_ops": (1.0, "String operations")
            }

            if "c" in self.results["benchmarks"]:
                for name, (target, desc) in targets.items():
                    if name in self.results["benchmarks"]["c"]:
                        result = self.results["benchmarks"]["c"][name]
                        if result["status"] == "success":
                            actual = result["avg_time_ms"]
                            ratio = actual / target
                            status = "✓ On target" if ratio <= 1.5 else "⚠ Slower than target"
                            f.write(f"| {name} ({desc}) | {target:.1f} | {actual:.3f} | {ratio:.2f}x | {status} |\n")


def main():
    parser = argparse.ArgumentParser(description="Fast Forth Benchmark Runner")
    parser.add_argument("--all", action="store_true", help="Run all benchmarks")
    parser.add_argument("--c", action="store_true", help="Run C baseline benchmarks")
    parser.add_argument("--gforth", action="store_true", help="Run GForth benchmarks")
    parser.add_argument("--rust", action="store_true", help="Run Rust benchmarks")
    parser.add_argument("--report", action="store_true", help="Generate report")
    parser.add_argument("--project-root", default=".", help="Project root directory")

    args = parser.parse_args()

    # Default to --all if no options specified
    if not (args.all or args.c or args.gforth or args.rust or args.report):
        args.all = True

    runner = BenchmarkRunner(args.project_root)

    if args.all or args.c:
        runner.run_all_c_benchmarks()

    if args.all or args.gforth:
        runner.run_all_gforth_benchmarks()

    if args.all or args.rust:
        runner.run_rust_benchmarks()

    if args.all or args.report:
        runner.generate_report()

if __name__ == "__main__":
    main()
