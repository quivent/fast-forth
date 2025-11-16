#!/bin/bash
# Verify fuzzing infrastructure is correctly set up

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ERRORS=0
WARNINGS=0

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Fast Forth Fuzzing Setup Verification${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

check_ok() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    ((ERRORS++))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

# Check Rust installation
echo -e "${BLUE}Checking Rust installation...${NC}"
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    check_ok "Rust installed: $RUST_VERSION"
else
    check_fail "Rust not installed"
fi

# Check nightly toolchain
if command -v cargo &> /dev/null && cargo +nightly --version &> /dev/null; then
    NIGHTLY_VERSION=$(cargo +nightly --version)
    check_ok "Nightly toolchain: $NIGHTLY_VERSION"
else
    check_fail "Nightly toolchain not installed (run: rustup install nightly)"
fi

# Check cargo-fuzz
if cargo +nightly fuzz --version &> /dev/null; then
    FUZZ_VERSION=$(cargo +nightly fuzz --version 2>&1 | head -1)
    check_ok "cargo-fuzz installed: $FUZZ_VERSION"
else
    check_warn "cargo-fuzz not installed (run: cargo +nightly install cargo-fuzz)"
fi

# Check GForth (optional)
echo ""
echo -e "${BLUE}Checking optional dependencies...${NC}"
if command -v gforth &> /dev/null; then
    GFORTH_VERSION=$(gforth --version 2>&1 | head -1)
    check_ok "GForth installed: $GFORTH_VERSION (differential testing available)"
else
    check_warn "GForth not installed (differential testing disabled)"
    echo -e "  Install with: ${YELLOW}brew install gforth${NC} (macOS) or ${YELLOW}apt install gforth${NC} (Linux)"
fi

# Check directory structure
echo ""
echo -e "${BLUE}Checking directory structure...${NC}"

REQUIRED_DIRS=(
    "tests/fuzz"
    "tests/fuzz/fuzz_targets"
    "tests/fuzz/src"
    "scripts"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        check_ok "Directory exists: $dir"
    else
        check_fail "Directory missing: $dir"
    fi
done

# Check fuzz targets
echo ""
echo -e "${BLUE}Checking fuzz targets...${NC}"

FUZZ_TARGETS=(
    "tests/fuzz/fuzz_targets/fuzz_parser.rs"
    "tests/fuzz/fuzz_targets/fuzz_compiler.rs"
    "tests/fuzz/fuzz_targets/fuzz_ssa.rs"
    "tests/fuzz/fuzz_targets/fuzz_optimizer.rs"
    "tests/fuzz/fuzz_targets/fuzz_codegen.rs"
)

for target in "${FUZZ_TARGETS[@]}"; do
    if [ -f "$target" ]; then
        check_ok "Fuzz target: $(basename "$target")"
    else
        check_fail "Missing fuzz target: $target"
    fi
done

# Check scripts
echo ""
echo -e "${BLUE}Checking fuzzing scripts...${NC}"

SCRIPTS=(
    "scripts/fuzz_overnight.sh"
    "scripts/quick_fuzz.sh"
    "scripts/analyze_crashes.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            check_ok "Script: $(basename "$script") (executable)"
        else
            check_warn "Script: $(basename "$script") (not executable - run: chmod +x $script)"
        fi
    else
        check_fail "Missing script: $script"
    fi
done

# Check Cargo configuration
echo ""
echo -e "${BLUE}Checking Cargo configuration...${NC}"

if [ -f "tests/fuzz/Cargo.toml" ]; then
    check_ok "Fuzz Cargo.toml exists"

    # Check for fuzz targets in Cargo.toml
    if grep -q "fuzz_parser" tests/fuzz/Cargo.toml; then
        check_ok "Parser target registered in Cargo.toml"
    else
        check_fail "Parser target not registered in Cargo.toml"
    fi

    if grep -q "fuzz_compiler" tests/fuzz/Cargo.toml; then
        check_ok "Compiler target registered in Cargo.toml"
    else
        check_fail "Compiler target not registered in Cargo.toml"
    fi
else
    check_fail "tests/fuzz/Cargo.toml missing"
fi

# Try to build fuzz targets
echo ""
echo -e "${BLUE}Testing fuzz build...${NC}"

cd tests/fuzz
if cargo +nightly build --bins &> /tmp/fuzz_build.log; then
    check_ok "Fuzz targets build successfully"
else
    check_fail "Fuzz targets failed to build (see /tmp/fuzz_build.log)"
    echo ""
    echo -e "${YELLOW}Build errors:${NC}"
    tail -20 /tmp/fuzz_build.log
fi
cd - > /dev/null

# Try to build property tests
echo ""
echo -e "${BLUE}Testing property tests build...${NC}"

cd tests/fuzz
if cargo test --lib --no-run &> /tmp/proptest_build.log; then
    check_ok "Property tests build successfully"
else
    check_fail "Property tests failed to build (see /tmp/proptest_build.log)"
    echo ""
    echo -e "${YELLOW}Build errors:${NC}"
    tail -20 /tmp/proptest_build.log
fi
cd - > /dev/null

# Summary
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! Fuzzing infrastructure is ready.${NC}"
    echo ""
    echo -e "${BLUE}Quick start:${NC}"
    echo "  ./scripts/quick_fuzz.sh              # 5-minute quick fuzz"
    echo "  ./scripts/fuzz_overnight.sh          # 8-hour overnight fuzz"
    echo ""
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ Setup complete with ${WARNINGS} warnings${NC}"
    echo -e "Optional features are missing but core fuzzing works."
    echo ""
    echo -e "${BLUE}Quick start:${NC}"
    echo "  ./scripts/quick_fuzz.sh              # 5-minute quick fuzz"
    echo "  ./scripts/fuzz_overnight.sh          # 8-hour overnight fuzz"
    echo ""
else
    echo -e "${RED}✗ Setup incomplete: ${ERRORS} errors, ${WARNINGS} warnings${NC}"
    echo ""
    echo -e "${YELLOW}Fix errors before running fuzzing.${NC}"
    echo ""
    exit 1
fi

echo -e "${BLUE}Documentation:${NC}"
echo "  tests/fuzz/README.md                 # Fuzzing guide"
echo "  fuzz/README.md                       # Comprehensive documentation"
echo ""
