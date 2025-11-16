#!/bin/bash
# Analyze and minimize crash artifacts from fuzzing
# Usage: ./analyze_crashes.sh [crash_directory]

set -euo pipefail

CRASHES_DIR="${1:-tests/fuzz/artifacts}"
FUZZ_DIR="tests/fuzz"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Fast Forth Crash Analysis Tool${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ ! -d "$CRASHES_DIR" ]; then
    echo -e "${RED}Error: Crash directory not found: ${CRASHES_DIR}${NC}"
    exit 1
fi

# Find all crash artifacts
CRASHES=$(find "$CRASHES_DIR" -type f -name "crash-*" -o -name "*crash*" 2>/dev/null)
CRASH_COUNT=$(echo "$CRASHES" | grep -c . || echo 0)

if [ "$CRASH_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✅ No crashes found!${NC}"
    exit 0
fi

echo -e "${RED}Found ${CRASH_COUNT} crash artifacts${NC}\n"

# Analyze each crash
for crash in $CRASHES; do
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Crash: $(basename "$crash")${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Determine which fuzzer produced this crash
    FUZZER="unknown"
    if [[ "$crash" == *"parser"* ]]; then
        FUZZER="fuzz_parser"
    elif [[ "$crash" == *"compiler"* ]]; then
        FUZZER="fuzz_compiler"
    elif [[ "$crash" == *"ssa"* ]]; then
        FUZZER="fuzz_ssa"
    elif [[ "$crash" == *"optimizer"* ]]; then
        FUZZER="fuzz_optimizer"
    elif [[ "$crash" == *"codegen"* ]]; then
        FUZZER="fuzz_codegen"
    fi

    echo -e "Fuzzer: ${BLUE}${FUZZER}${NC}"
    echo -e "Size: $(wc -c < "$crash") bytes"
    echo -e "Location: ${crash}"
    echo ""

    # Show crash content if it's text
    if file "$crash" | grep -q text; then
        echo -e "${BLUE}Content:${NC}"
        echo "----------------------------------------"
        head -20 "$crash" | cat -v
        echo "----------------------------------------"
        echo ""
    else
        echo -e "${YELLOW}Binary crash artifact (not displaying)${NC}"
        echo ""
    fi

    # Try to minimize the crash
    if [ "$FUZZER" != "unknown" ]; then
        echo -e "${GREEN}Attempting to minimize crash...${NC}"
        MINIMIZED="${crash}.minimized"

        cd "$FUZZ_DIR"
        if timeout 60 cargo +nightly fuzz cmin "$FUZZER" "$crash" > "$MINIMIZED" 2>&1; then
            ORIGINAL_SIZE=$(wc -c < "$crash")
            MINIMIZED_SIZE=$(wc -c < "$MINIMIZED")
            REDUCTION=$((100 - (MINIMIZED_SIZE * 100 / ORIGINAL_SIZE)))

            echo -e "${GREEN}✅ Minimized successfully!${NC}"
            echo -e "Original size: ${ORIGINAL_SIZE} bytes"
            echo -e "Minimized size: ${MINIMIZED_SIZE} bytes"
            echo -e "Reduction: ${REDUCTION}%"

            if file "$MINIMIZED" | grep -q text; then
                echo -e "\n${BLUE}Minimized content:${NC}"
                echo "----------------------------------------"
                cat "$MINIMIZED" | cat -v
                echo "----------------------------------------"
            fi
        else
            echo -e "${YELLOW}⚠ Could not minimize crash${NC}"
            rm -f "$MINIMIZED"
        fi
        cd - > /dev/null
    fi

    echo ""
done

# Generate summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Crash Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Count by fuzzer
echo -e "${YELLOW}Crashes by fuzzer:${NC}"
for fuzzer in fuzz_parser fuzz_compiler fuzz_ssa fuzz_optimizer fuzz_codegen; do
    count=$(echo "$CRASHES" | grep -c "$fuzzer" || echo 0)
    if [ "$count" -gt 0 ]; then
        echo -e "  ${fuzzer}: ${RED}${count}${NC}"
    fi
done
echo ""

# Recommendations
echo -e "${BLUE}Recommendations:${NC}"
echo ""
echo -e "1. ${GREEN}Reproduce crashes:${NC}"
echo "   cd tests/fuzz"
echo "   cargo +nightly fuzz run <fuzzer_name> <crash_file>"
echo ""
echo -e "2. ${GREEN}Debug with stack trace:${NC}"
echo "   RUST_BACKTRACE=1 cargo +nightly fuzz run <fuzzer_name> <crash_file>"
echo ""
echo -e "3. ${GREEN}Run in debugger:${NC}"
echo "   cargo +nightly fuzz run --debug <fuzzer_name> <crash_file>"
echo "   lldb target/debug/<fuzzer_name>"
echo ""
echo -e "4. ${GREEN}Add regression test:${NC}"
echo "   - Copy minimized case to tests/regression/"
echo "   - Add test case to prevent re-occurrence"
echo ""
echo -e "5. ${GREEN}View coverage:${NC}"
echo "   cargo +nightly fuzz coverage <fuzzer_name>"
echo ""
