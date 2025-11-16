# Fast Forth Fuzzing - Quick Start

## 30-Second Setup

```bash
# 1. Verify setup
./scripts/verify_fuzz_setup.sh

# 2. Run overnight fuzzing
./scripts/fuzz_overnight.sh

# 3. Check results in the morning
open tests/fuzz/overnight_reports/fuzz_report_*.html
```

## What It Does

Runs for **8 hours** (configurable) testing:
- 🔍 **Coverage-guided fuzzing** - 5 targets, ~7M executions
- ✅ **Property testing** - 160k structured test cases
- 🔄 **Differential testing** - 50k comparisons vs GForth
- 💪 **Stress testing** - Extreme values and edge cases

## Expected Results

**If no crashes:**
```
✅ No crashes found! Compiler is robust.
  - Review corpus for interesting cases
  - Consider longer fuzzing duration
```

**If crashes found:**
```
⚠ Found 5 crashes!
  - Crashes saved to: tests/fuzz/overnight_reports/crashes/
  - Run: ./scripts/analyze_crashes.sh
```

## Commands

```bash
# Quick 5-minute test
./scripts/quick_fuzz.sh

# Overnight (8 hours)
./scripts/fuzz_overnight.sh

# Weekend run (24 hours)
FUZZ_DURATION_HOURS=24 ./scripts/fuzz_overnight.sh

# Analyze crashes
./scripts/analyze_crashes.sh tests/fuzz/artifacts/
```

## Output Files

```
tests/fuzz/overnight_reports/
├── fuzz_report_20241115_220000.html  # Main report (open this!)
├── crashes/                           # Crash artifacts
│   ├── parser_crash-abc123
│   └── compiler_crash-def456
├── corpus/                            # Interesting test cases
└── *.log                              # Detailed logs
```

## When Bugs Are Found

```bash
# 1. Analyze
./scripts/analyze_crashes.sh tests/fuzz/artifacts/

# 2. Reproduce
cd tests/fuzz
cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123

# 3. Debug
RUST_BACKTRACE=1 cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123

# 4. Minimize
cargo +nightly fuzz cmin fuzz_parser

# 5. Fix bug and add regression test
```

## Installation Requirements

```bash
# Rust nightly (required)
rustup install nightly

# cargo-fuzz (auto-installed by script)
cargo +nightly install cargo-fuzz

# GForth (optional - for differential testing)
brew install gforth  # macOS
apt install gforth   # Linux
```

## Performance

| Duration | Parser Execs | Compiler Execs | Test Cases | Use Case |
|----------|--------------|----------------|------------|----------|
| 5 min | ~50k | ~5k | ~60k | Pre-commit |
| 1 hour | ~600k | ~60k | ~160k | Quick CI |
| 8 hours | ~5M | ~500k | ~210k | Overnight |
| 24 hours | ~15M | ~1.5M | ~600k | Weekend |

## Troubleshooting

**"cargo-fuzz not found"**
```bash
cargo +nightly install cargo-fuzz
```

**Out of memory**
```bash
# Reduce parallel fuzzers in scripts/fuzz_overnight.sh
# Or add: -rss_limit_mb=2048
```

**Takes too long**
```bash
# Set shorter duration
FUZZ_DURATION_HOURS=1 ./scripts/fuzz_overnight.sh
```

## Full Documentation

- Comprehensive guide: `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/FUZZING_SETUP.md`
- Detailed reference: `tests/fuzz/README.md`
- This quick start: `tests/fuzz/QUICK_START.md`
