#!/bin/bash
# Comprehensive overnight fuzzing script for Fast Forth
# Runs multiple fuzzing strategies in parallel for extended periods

set -euo pipefail

# Configuration
FUZZ_DIR="$(cd "$(dirname "$0")/../tests/fuzz" && pwd)"
SCRIPTS_DIR="$(dirname "$0")"
DURATION_HOURS="${FUZZ_DURATION_HOURS:-8}"
DURATION_SECONDS=$((DURATION_HOURS * 3600))
REPORT_DIR="${FUZZ_DIR}/overnight_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/fuzz_report_${TIMESTAMP}.html"
CRASHES_DIR="${FUZZ_DIR}/crashes/${TIMESTAMP}"
CORPUS_DIR="${FUZZ_DIR}/corpus/${TIMESTAMP}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Fast Forth Overnight Fuzzing${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Duration: ${GREEN}${DURATION_HOURS} hours${NC}"
echo -e "Start time: ${GREEN}$(date)${NC}"
echo -e "Report will be saved to: ${YELLOW}${REPORT_FILE}${NC}"
echo ""

# Create output directories
mkdir -p "${REPORT_DIR}" "${CRASHES_DIR}" "${CORPUS_DIR}"

# Track PIDs for cleanup
FUZZ_PIDS=()
trap cleanup EXIT INT TERM

cleanup() {
    echo -e "\n${YELLOW}Cleaning up fuzzing processes...${NC}"
    for pid in "${FUZZ_PIDS[@]}"; do
        kill -TERM "$pid" 2>/dev/null || true
    done
    wait
    generate_final_report
}

# ============================================================================
# FUZZING STRATEGIES
# ============================================================================

# Strategy 1: LibFuzzer - Parser
run_libfuzzer_parser() {
    echo -e "${BLUE}[LibFuzzer Parser]${NC} Starting coverage-guided fuzzing..."
    cd "${FUZZ_DIR}"
    cargo +nightly fuzz run fuzz_parser -- \
        -max_total_time="${DURATION_SECONDS}" \
        -print_final_stats=1 \
        -artifact_prefix="${CRASHES_DIR}/parser_" \
        -timeout=10 \
        &> "${REPORT_DIR}/libfuzzer_parser_${TIMESTAMP}.log" &
    FUZZ_PIDS+=($!)
}

# Strategy 2: LibFuzzer - Compiler (end-to-end)
run_libfuzzer_compiler() {
    echo -e "${BLUE}[LibFuzzer Compiler]${NC} Starting end-to-end compilation fuzzing..."
    cd "${FUZZ_DIR}"
    cargo +nightly fuzz run fuzz_compiler -- \
        -max_total_time="${DURATION_SECONDS}" \
        -print_final_stats=1 \
        -artifact_prefix="${CRASHES_DIR}/compiler_" \
        -timeout=30 \
        &> "${REPORT_DIR}/libfuzzer_compiler_${TIMESTAMP}.log" &
    FUZZ_PIDS+=($!)
}

# Strategy 3: LibFuzzer - SSA Construction
run_libfuzzer_ssa() {
    echo -e "${BLUE}[LibFuzzer SSA]${NC} Starting SSA construction fuzzing..."
    cd "${FUZZ_DIR}"
    cargo +nightly fuzz run fuzz_ssa -- \
        -max_total_time="${DURATION_SECONDS}" \
        -print_final_stats=1 \
        -artifact_prefix="${CRASHES_DIR}/ssa_" \
        -timeout=20 \
        &> "${REPORT_DIR}/libfuzzer_ssa_${TIMESTAMP}.log" &
    FUZZ_PIDS+=($!)
}

# Strategy 4: LibFuzzer - Optimizer
run_libfuzzer_optimizer() {
    echo -e "${BLUE}[LibFuzzer Optimizer]${NC} Starting optimization passes fuzzing..."
    cd "${FUZZ_DIR}"
    cargo +nightly fuzz run fuzz_optimizer -- \
        -max_total_time="${DURATION_SECONDS}" \
        -print_final_stats=1 \
        -artifact_prefix="${CRASHES_DIR}/optimizer_" \
        -timeout=30 \
        &> "${REPORT_DIR}/libfuzzer_optimizer_${TIMESTAMP}.log" &
    FUZZ_PIDS+=($!)
}

# Strategy 5: Extended Property Testing
run_property_tests() {
    echo -e "${BLUE}[Property Tests]${NC} Running extended property-based tests..."
    cd "${FUZZ_DIR}"

    # Run with increasing test case counts
    for cases in 10000 50000 100000; do
        echo -e "  ${GREEN}Running ${cases} test cases...${NC}"
        PROPTEST_CASES="${cases}" \
        PROPTEST_MAX_SHRINK_ITERS=10000 \
        cargo test --lib --release \
            &> "${REPORT_DIR}/proptest_${cases}_${TIMESTAMP}.log"

        # Check for failures
        if [ $? -ne 0 ]; then
            echo -e "  ${RED}Found failures with ${cases} cases!${NC}"
            cp -r "${FUZZ_DIR}/proptest-regressions" "${CRASHES_DIR}/proptest_${cases}/" || true
        fi
    done
}

# Strategy 6: Differential Fuzzing against GForth
run_differential_fuzzing() {
    if ! command -v gforth &> /dev/null; then
        echo -e "${YELLOW}[Differential]${NC} GForth not found, skipping differential fuzzing"
        return
    fi

    echo -e "${BLUE}[Differential]${NC} Running differential fuzzing against GForth..."
    cd "${FUZZ_DIR}"

    PROPTEST_CASES=50000 cargo test differential_tests --release \
        &> "${REPORT_DIR}/differential_${TIMESTAMP}.log"

    if [ $? -ne 0 ]; then
        echo -e "  ${RED}Found divergences from GForth!${NC}"
        cp -r "${FUZZ_DIR}/proptest-regressions/differential_tests" \
            "${CRASHES_DIR}/differential/" || true
    fi
}

# Strategy 7: Stress Testing with Extreme Values
run_stress_tests() {
    echo -e "${BLUE}[Stress Tests]${NC} Running stress tests with extreme values..."
    cd "${FUZZ_DIR}"

    # Create stress test programs
    cat > /tmp/stress_test.fth <<EOF
: stress-max-int 9223372036854775807 . ;
: stress-min-int -9223372036854775808 . ;
: stress-deep-recursion 1000 0 do i 2 * loop ;
: stress-large-stack 10000 0 do i loop ;
: stress-nested-loops 100 0 do 100 0 do 100 0 do i j k + + loop loop loop ;
EOF

    # Run stress tests through the compiler
    timeout 300 cargo run --release --bin fastforth -- /tmp/stress_test.fth \
        &> "${REPORT_DIR}/stress_${TIMESTAMP}.log" || true
}

# ============================================================================
# EXECUTION
# ============================================================================

# Check for cargo-fuzz installation
if ! cargo +nightly fuzz --version &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-fuzz...${NC}"
    cargo +nightly install cargo-fuzz
fi

# Start all fuzzing strategies
echo -e "\n${GREEN}Starting fuzzing strategies...${NC}\n"

run_libfuzzer_parser
sleep 2  # Stagger starts

run_libfuzzer_compiler
sleep 2

run_libfuzzer_ssa
sleep 2

run_libfuzzer_optimizer
sleep 2

# Run property tests in foreground (they're finite)
run_property_tests

# Run differential fuzzing
run_differential_fuzzing

# Run stress tests
run_stress_tests

# Monitor progress
echo -e "\n${GREEN}Fuzzing in progress...${NC}"
echo -e "Background processes: ${#FUZZ_PIDS[@]}"
echo -e "Monitoring for ${DURATION_HOURS} hours...\n"

# Periodic status updates
START_TIME=$(date +%s)
while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    REMAINING=$((DURATION_SECONDS - ELAPSED))

    if [ $REMAINING -le 0 ]; then
        echo -e "\n${GREEN}Fuzzing duration complete!${NC}"
        break
    fi

    # Print status every 30 minutes
    HOURS=$((ELAPSED / 3600))
    MINUTES=$(((ELAPSED % 3600) / 60))

    echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} Elapsed: ${HOURS}h ${MINUTES}m | Remaining: $((REMAINING / 3600))h $(((REMAINING % 3600) / 60))m"

    # Check for crash files
    CRASH_COUNT=$(find "${CRASHES_DIR}" -type f 2>/dev/null | wc -l)
    if [ "$CRASH_COUNT" -gt 0 ]; then
        echo -e "  ${RED}⚠ Found ${CRASH_COUNT} crashes!${NC}"
    fi

    sleep 1800  # 30 minutes
done

# Wait for all background processes
echo -e "\n${YELLOW}Stopping fuzzing processes...${NC}"
for pid in "${FUZZ_PIDS[@]}"; do
    kill -TERM "$pid" 2>/dev/null || true
done
wait

# ============================================================================
# REPORT GENERATION
# ============================================================================

generate_final_report() {
    echo -e "\n${BLUE}Generating final report...${NC}"

    # Collect statistics
    TOTAL_CRASHES=$(find "${CRASHES_DIR}" -type f 2>/dev/null | wc -l)
    CORPUS_SIZE=$(find "${CORPUS_DIR}" -type f 2>/dev/null | wc -l)

    # Parse libfuzzer stats
    PARSER_EXECS=$(grep "stat::number_of_executed_units:" "${REPORT_DIR}/libfuzzer_parser_${TIMESTAMP}.log" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
    COMPILER_EXECS=$(grep "stat::number_of_executed_units:" "${REPORT_DIR}/libfuzzer_compiler_${TIMESTAMP}.log" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")

    # Generate HTML report
    cat > "${REPORT_FILE}" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Fast Forth Fuzzing Report - ${TIMESTAMP}</title>
    <style>
        body { font-family: monospace; margin: 40px; background: #1e1e1e; color: #d4d4d4; }
        h1 { color: #4ec9b0; border-bottom: 2px solid #4ec9b0; }
        h2 { color: #569cd6; margin-top: 30px; }
        .metric { background: #252526; padding: 15px; margin: 10px 0; border-left: 4px solid #4ec9b0; }
        .crash { background: #3c1f1f; border-left: 4px solid #f48771; }
        .success { background: #1f3c1f; border-left: 4px solid #4ec9b0; }
        .warning { background: #3c3c1f; border-left: 4px solid #dcdcaa; }
        pre { background: #1e1e1e; padding: 10px; overflow-x: auto; border: 1px solid #3e3e42; }
        table { border-collapse: collapse; width: 100%; margin: 20px 0; }
        th, td { border: 1px solid #3e3e42; padding: 12px; text-align: left; }
        th { background: #2d2d30; color: #4ec9b0; }
        .stat-value { font-size: 24px; font-weight: bold; color: #4ec9b0; }
    </style>
</head>
<body>
    <h1>🔬 Fast Forth Overnight Fuzzing Report</h1>

    <div class="metric">
        <strong>Fuzzing Session:</strong> ${TIMESTAMP}<br>
        <strong>Duration:</strong> ${DURATION_HOURS} hours<br>
        <strong>Completed:</strong> $(date)<br>
    </div>

    <h2>📊 Summary Statistics</h2>

    <table>
        <tr>
            <th>Metric</th>
            <th>Value</th>
        </tr>
        <tr>
            <td>Total Crashes Found</td>
            <td class="stat-value" style="color: ${TOTAL_CRASHES -gt 0 && echo '#f48771' || echo '#4ec9b0'}">${TOTAL_CRASHES}</td>
        </tr>
        <tr>
            <td>Parser Executions</td>
            <td class="stat-value">${PARSER_EXECS}</td>
        </tr>
        <tr>
            <td>Compiler Executions</td>
            <td class="stat-value">${COMPILER_EXECS}</td>
        </tr>
        <tr>
            <td>Corpus Size</td>
            <td class="stat-value">${CORPUS_SIZE}</td>
        </tr>
    </table>

    <h2>🎯 Fuzzing Strategies Executed</h2>

    <div class="metric">
        <h3>1. Coverage-Guided Fuzzing (LibFuzzer)</h3>
        <ul>
            <li>Parser fuzzing: ${PARSER_EXECS} executions</li>
            <li>End-to-end compiler fuzzing: ${COMPILER_EXECS} executions</li>
            <li>SSA construction fuzzing</li>
            <li>Optimization passes fuzzing</li>
        </ul>
    </div>

    <div class="metric">
        <h3>2. Property-Based Testing (PropTest)</h3>
        <ul>
            <li>10,000 test cases (standard)</li>
            <li>50,000 test cases (extended)</li>
            <li>100,000 test cases (deep exploration)</li>
        </ul>
    </div>

    <div class="metric">
        <h3>3. Differential Testing</h3>
        <ul>
            <li>Comparison against GForth oracle</li>
            <li>50,000 differential test cases</li>
        </ul>
    </div>

    <h2>🐛 Crash Analysis</h2>

EOF

    # List crashes if any
    if [ "$TOTAL_CRASHES" -gt 0 ]; then
        cat >> "${REPORT_FILE}" <<EOF
    <div class="crash">
        <h3>⚠ Crashes Found: ${TOTAL_CRASHES}</h3>
        <p>Crash artifacts saved to: <code>${CRASHES_DIR}</code></p>
        <h4>Crash Files:</h4>
        <pre>
EOF
        find "${CRASHES_DIR}" -type f | head -20 >> "${REPORT_FILE}"
        cat >> "${REPORT_FILE}" <<EOF
        </pre>
    </div>

    <h3>📝 Reproducing Crashes</h3>
    <pre>
# Reproduce a parser crash:
cd tests/fuzz
cargo +nightly fuzz run fuzz_parser ${CRASHES_DIR}/parser_crash-xxxxx

# Reproduce a compiler crash:
cargo +nightly fuzz run fuzz_compiler ${CRASHES_DIR}/compiler_crash-xxxxx

# Minimize a crash case:
cargo +nightly fuzz cmin fuzz_parser
    </pre>
EOF
    else
        cat >> "${REPORT_FILE}" <<EOF
    <div class="success">
        <h3>✅ No Crashes Found!</h3>
        <p>All fuzzing strategies completed without finding crashes. The compiler appears robust.</p>
    </div>
EOF
    fi

    # Add log excerpts
    cat >> "${REPORT_FILE}" <<EOF

    <h2>📋 Detailed Logs</h2>

    <h3>LibFuzzer Parser</h3>
    <pre>
EOF
    tail -50 "${REPORT_DIR}/libfuzzer_parser_${TIMESTAMP}.log" 2>/dev/null >> "${REPORT_FILE}" || echo "No log available" >> "${REPORT_FILE}"

    cat >> "${REPORT_FILE}" <<EOF
    </pre>

    <h3>Property Tests (100k cases)</h3>
    <pre>
EOF
    tail -50 "${REPORT_DIR}/proptest_100000_${TIMESTAMP}.log" 2>/dev/null >> "${REPORT_FILE}" || echo "No log available" >> "${REPORT_FILE}"

    cat >> "${REPORT_FILE}" <<EOF
    </pre>

    <h2>📁 Artifacts</h2>

    <div class="metric">
        <strong>Crashes:</strong> <code>${CRASHES_DIR}</code><br>
        <strong>Corpus:</strong> <code>${CORPUS_DIR}</code><br>
        <strong>Logs:</strong> <code>${REPORT_DIR}</code><br>
    </div>

    <h2>🔄 Next Steps</h2>

    <div class="warning">
        <h3>If Crashes Found:</h3>
        <ol>
            <li>Reproduce crashes using the commands above</li>
            <li>Minimize test cases: <code>cargo +nightly fuzz cmin fuzz_parser</code></li>
            <li>Debug with: <code>cargo test --lib -- --nocapture</code></li>
            <li>Add regression tests for fixed bugs</li>
        </ol>
    </div>

    <div class="success">
        <h3>If No Crashes:</h3>
        <ol>
            <li>Review corpus for interesting test cases</li>
            <li>Increase fuzzing duration for next run</li>
            <li>Add corpus cases to regression suite</li>
            <li>Consider adding new fuzzing targets</li>
        </ol>
    </div>

    <hr>
    <p><em>Report generated: $(date)</em></p>
    <p><em>Fuzzing infrastructure: tests/fuzz/</em></p>
</body>
</html>
EOF

    echo -e "${GREEN}✅ Report generated: ${REPORT_FILE}${NC}"
    echo ""
    echo -e "${BLUE}Summary:${NC}"
    echo -e "  Total crashes: ${RED}${TOTAL_CRASHES}${NC}"
    echo -e "  Parser executions: ${GREEN}${PARSER_EXECS}${NC}"
    echo -e "  Compiler executions: ${GREEN}${COMPILER_EXECS}${NC}"
    echo ""
    echo -e "${YELLOW}View full report:${NC}"
    echo -e "  open ${REPORT_FILE}"
    echo ""

    if [ "$TOTAL_CRASHES" -gt 0 ]; then
        echo -e "${RED}⚠ CRASHES FOUND! Review artifacts in:${NC}"
        echo -e "  ${CRASHES_DIR}"
    else
        echo -e "${GREEN}✅ No crashes found! Compiler is robust.${NC}"
    fi
}

echo -e "\n${GREEN}Fuzzing complete!${NC}\n"
