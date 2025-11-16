#!/bin/bash
# Quick fuzzing for rapid iteration during development
# Runs for 5 minutes across all targets

set -euo pipefail

FUZZ_DIR="tests/fuzz"
DURATION=300  # 5 minutes

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Quick Fuzzing (5 minutes per target)${NC}\n"

cd "$FUZZ_DIR"

# Install cargo-fuzz if needed
if ! cargo +nightly fuzz --version &> /dev/null; then
    echo "Installing cargo-fuzz..."
    cargo +nightly install cargo-fuzz
fi

TARGETS=(fuzz_parser fuzz_compiler fuzz_ssa fuzz_optimizer fuzz_codegen)

for target in "${TARGETS[@]}"; do
    echo -e "${GREEN}Fuzzing ${target}...${NC}"
    cargo +nightly fuzz run "$target" -- -max_total_time="$DURATION" -print_final_stats=1
    echo ""
done

echo -e "${GREEN}Quick fuzz complete! Check artifacts/ for any crashes.${NC}"

# Run crash analysis if any crashes found
if [ -d "artifacts" ] && [ "$(find artifacts -type f | wc -l)" -gt 0 ]; then
    echo ""
    ../../scripts/analyze_crashes.sh artifacts
fi
