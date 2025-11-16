#!/bin/bash
# Quick test runner for property-based fuzzing

set -e

echo "Fast Forth Property-Based Fuzzing Test Runner"
echo "=============================================="
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from tests/fuzz directory${NC}"
    exit 1
fi

# Parse arguments
MODE="${1:-quick}"
CASES="${PROPTEST_CASES:-1000}"

case "$MODE" in
    quick)
        echo -e "${GREEN}Running quick validation (corpus tests only)${NC}"
        echo ""
        cargo test corpus_tests --lib
        ;;

    standard)
        echo -e "${GREEN}Running standard property tests (${CASES} cases per property)${NC}"
        echo ""
        export PROPTEST_CASES=$CASES
        cargo test --lib -- --test-threads=1
        ;;

    deep)
        echo -e "${YELLOW}Running deep exploration (10000 cases per property)${NC}"
        echo -e "${YELLOW}This may take 10-15 minutes...${NC}"
        echo ""
        export PROPTEST_CASES=10000
        cargo test --lib -- --test-threads=1
        ;;

    differential)
        if ! command -v gforth &> /dev/null; then
            echo -e "${RED}GForth not found. Install with: brew install gforth${NC}"
            exit 1
        fi
        echo -e "${GREEN}Running differential tests against GForth${NC}"
        echo ""
        export PROPTEST_CASES=${CASES}
        cargo test differential_tests --lib
        ;;

    oracle)
        echo -e "${GREEN}Testing GForth differential oracle${NC}"
        echo ""
        ./test_gforth_oracle.sh
        ;;

    stats)
        echo "Test Statistics:"
        echo "================"
        echo ""
        echo "Property Test Suites:"
        grep -E "fn prop_|fn diff_" src/property_tests.rs | wc -l | xargs echo "  Test functions:"
        echo ""
        echo "Corpus Cases:"
        grep -E '^\s+"' src/property_tests.rs | wc -l | xargs echo "  Edge cases:"
        echo ""
        echo "Default test cases per run:"
        echo "  Property tests: ~6000"
        echo "  Corpus tests: 40+"
        echo "  Differential tests: 200"
        echo ""
        echo "Expected runtime (standard mode):"
        echo "  Property tests: 2-5 minutes"
        echo "  Corpus tests: < 1 second"
        echo "  Differential tests: 1-2 minutes"
        ;;

    *)
        echo "Usage: $0 [mode]"
        echo ""
        echo "Modes:"
        echo "  quick        - Run corpus tests only (< 1 second)"
        echo "  standard     - Run all property tests with 1000 cases (default)"
        echo "  deep         - Run all property tests with 10000 cases (~15 min)"
        echo "  differential - Run differential tests against GForth"
        echo "  oracle       - Test GForth oracle functionality"
        echo "  stats        - Show test statistics"
        echo ""
        echo "Environment variables:"
        echo "  PROPTEST_CASES - Number of test cases per property (default: 1000)"
        echo ""
        echo "Examples:"
        echo "  $0 quick"
        echo "  $0 standard"
        echo "  PROPTEST_CASES=5000 $0 standard"
        echo "  $0 deep"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✓ Tests complete!${NC}"
