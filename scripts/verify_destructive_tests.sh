#!/usr/bin/env bash
# Verification script for destructive testing infrastructure
# Checks that all components are in place and configured correctly

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Destructive Testing Infrastructure Verification"
echo "================================================"
echo ""

# Check 1: Docker availability
check_info "Checking Docker availability..."
if command -v docker &> /dev/null; then
    if docker info &> /dev/null; then
        check_pass "Docker is installed and running"
    else
        check_fail "Docker is installed but not running"
        exit 1
    fi
else
    check_fail "Docker is not installed"
    exit 1
fi

# Check 2: Required files exist
check_info "Checking required files..."
REQUIRED_FILES=(
    "tests/destructive/Dockerfile"
    "tests/destructive/mod.rs"
    "tests/destructive/safety.rs"
    "tests/destructive/test_oom.rs"
    "tests/destructive/test_disk_full.rs"
    "tests/destructive/test_stack_overflow.rs"
    "tests/destructive/test_fd_exhaustion.rs"
    "tests/destructive/README.md"
    "scripts/run_destructive_tests.sh"
    ".github/workflows/destructive-tests.yml"
)

all_files_present=true
for file in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$PROJECT_ROOT/$file" ]]; then
        check_pass "$file exists"
    else
        check_fail "$file missing"
        all_files_present=false
    fi
done

if ! $all_files_present; then
    exit 1
fi

# Check 3: Cargo.toml feature flag
check_info "Checking Cargo.toml configuration..."
if grep -q "destructive_tests = \[\]" "$PROJECT_ROOT/Cargo.toml"; then
    check_pass "destructive_tests feature flag present"
else
    check_fail "destructive_tests feature flag missing in Cargo.toml"
    exit 1
fi

# Check 4: Test runner is executable
check_info "Checking test runner permissions..."
if [[ -x "$PROJECT_ROOT/scripts/run_destructive_tests.sh" ]]; then
    check_pass "Test runner is executable"
else
    check_warn "Test runner is not executable (fixing...)"
    chmod +x "$PROJECT_ROOT/scripts/run_destructive_tests.sh"
    check_pass "Fixed test runner permissions"
fi

# Check 5: Count tests
check_info "Counting destructive tests..."
test_counts=(
    "$(grep -c "^#\[test\]" "$PROJECT_ROOT/tests/destructive/test_oom.rs" || echo 0)"
    "$(grep -c "^#\[test\]" "$PROJECT_ROOT/tests/destructive/test_disk_full.rs" || echo 0)"
    "$(grep -c "^#\[test\]" "$PROJECT_ROOT/tests/destructive/test_stack_overflow.rs" || echo 0)"
    "$(grep -c "^#\[test\]" "$PROJECT_ROOT/tests/destructive/test_fd_exhaustion.rs" || echo 0)"
)

total_tests=0
for count in "${test_counts[@]}"; do
    total_tests=$((total_tests + count))
done

check_pass "Found $total_tests destructive tests"
echo "  - OOM: ${test_counts[0]} tests"
echo "  - Disk Full: ${test_counts[1]} tests"
echo "  - Stack Overflow: ${test_counts[2]} tests"
echo "  - FD Exhaustion: ${test_counts[3]} tests"

# Check 6: Code statistics
check_info "Code statistics..."
total_lines=$(wc -l "$PROJECT_ROOT"/tests/destructive/*.rs | tail -1 | awk '{print $1}')
check_pass "$total_lines lines of test code"

# Check 7: Safety guards
check_info "Verifying safety guards..."
if grep -q "ensure_containerized()" "$PROJECT_ROOT/tests/destructive/test_oom.rs" &&
   grep -q "is_in_container()" "$PROJECT_ROOT/tests/destructive/safety.rs"; then
    check_pass "Safety guards implemented"
else
    check_fail "Safety guards missing"
    exit 1
fi

# Check 8: Dockerfile configuration
check_info "Checking Dockerfile configuration..."
if grep -q "destructive_tests" "$PROJECT_ROOT/tests/destructive/Dockerfile"; then
    check_pass "Dockerfile configured for destructive tests"
else
    check_warn "Dockerfile may need destructive_tests feature"
fi

# Check 9: CI workflow
check_info "Checking CI workflow..."
if grep -q "destructive_tests" "$PROJECT_ROOT/.github/workflows/destructive-tests.yml"; then
    check_pass "CI workflow configured"
else
    check_fail "CI workflow not properly configured"
    exit 1
fi

# Check 10: Compilation check
check_info "Verifying tests compile..."
cd "$PROJECT_ROOT"
if cargo check --features destructive_tests --quiet 2>&1 | grep -q "error"; then
    check_fail "Compilation errors detected"
    echo "Run: cargo check --features destructive_tests"
    exit 1
else
    check_pass "Tests compile successfully"
fi

# Summary
echo ""
echo "================================================"
echo "Verification Summary"
echo "================================================"
check_pass "All checks passed!"
echo ""
echo "Destructive testing infrastructure is ready:"
echo "  - $total_tests tests across 4 categories"
echo "  - $total_lines lines of test code"
echo "  - Docker containerization configured"
echo "  - Safety guards in place"
echo "  - CI/CD integration ready"
echo ""
echo "To run tests:"
echo "  ./scripts/run_destructive_tests.sh"
echo ""
echo "To run specific category:"
echo "  ./scripts/run_destructive_tests.sh [oom|disk|stack|fd]"
echo ""
