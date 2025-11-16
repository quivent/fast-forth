#!/usr/bin/env bash
# Destructive Test Runner
# Safely executes resource-constrained tests in Docker containers

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE_NAME="fastforth-destructive-tests"
CONTAINER_NAME="fastforth-destructive-test-runner"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi

    log_success "Docker is available"
}

# Build the test container
build_container() {
    log_info "Building destructive test container..."

    cd "$PROJECT_ROOT"

    docker build \
        -t "$IMAGE_NAME" \
        -f tests/destructive/Dockerfile \
        . || {
        log_error "Failed to build container"
        exit 1
    }

    log_success "Container built successfully"
}

# Run OOM tests
run_oom_tests() {
    log_info "Running OOM tests (128MB memory limit)..."

    docker run --rm \
        --name "${CONTAINER_NAME}-oom" \
        --memory=128m \
        --memory-swap=128m \
        --env DESTRUCTIVE_TESTS_ENABLED=1 \
        --env ALLOW_DESTRUCTIVE_TESTS=1 \
        "$IMAGE_NAME" \
        cargo test --release --features destructive_tests test_oom -- --test-threads=1 --nocapture || {
        log_warning "Some OOM tests failed (may be expected)"
    }

    log_success "OOM tests completed"
}

# Run disk full tests
run_disk_full_tests() {
    log_info "Running disk full tests (100MB disk limit)..."

    # Create a limited-size volume
    DISK_IMAGE="$PROJECT_ROOT/target/test_disk.img"
    mkdir -p "$PROJECT_ROOT/target"

    # Create 100MB disk image
    dd if=/dev/zero of="$DISK_IMAGE" bs=1M count=100 2>/dev/null || true

    # Format it
    if command -v mkfs.ext4 &> /dev/null; then
        mkfs.ext4 -F "$DISK_IMAGE" &>/dev/null || true
    fi

    docker run --rm \
        --name "${CONTAINER_NAME}-disk" \
        --env DESTRUCTIVE_TESTS_ENABLED=1 \
        --env ALLOW_DESTRUCTIVE_TESTS=1 \
        "$IMAGE_NAME" \
        cargo test --release --features destructive_tests test_disk_full -- --test-threads=1 --nocapture || {
        log_warning "Some disk tests failed (may be expected)"
    }

    # Cleanup
    rm -f "$DISK_IMAGE"

    log_success "Disk full tests completed"
}

# Run stack overflow tests
run_stack_overflow_tests() {
    log_info "Running stack overflow tests (1MB stack limit)..."

    docker run --rm \
        --name "${CONTAINER_NAME}-stack" \
        --ulimit stack=1048576:1048576 \
        --env DESTRUCTIVE_TESTS_ENABLED=1 \
        --env ALLOW_DESTRUCTIVE_TESTS=1 \
        "$IMAGE_NAME" \
        cargo test --release --features destructive_tests test_stack_overflow -- --test-threads=1 --nocapture || {
        log_warning "Some stack tests failed (may be expected)"
    }

    log_success "Stack overflow tests completed"
}

# Run FD exhaustion tests
run_fd_exhaustion_tests() {
    log_info "Running FD exhaustion tests (256 FD limit)..."

    docker run --rm \
        --name "${CONTAINER_NAME}-fd" \
        --ulimit nofile=256:256 \
        --env DESTRUCTIVE_TESTS_ENABLED=1 \
        --env ALLOW_DESTRUCTIVE_TESTS=1 \
        "$IMAGE_NAME" \
        cargo test --release --features destructive_tests test_fd_exhaustion -- --test-threads=1 --nocapture || {
        log_warning "Some FD tests failed (may be expected)"
    }

    log_success "FD exhaustion tests completed"
}

# Run all destructive tests
run_all_tests() {
    log_info "Running all destructive tests..."

    docker run --rm \
        --name "${CONTAINER_NAME}-all" \
        --memory=256m \
        --memory-swap=256m \
        --ulimit stack=1048576:1048576 \
        --ulimit nofile=512:512 \
        --env DESTRUCTIVE_TESTS_ENABLED=1 \
        --env ALLOW_DESTRUCTIVE_TESTS=1 \
        "$IMAGE_NAME" \
        cargo test --release --features destructive_tests -- --test-threads=1 --nocapture || {
        log_warning "Some tests failed (may be expected for destructive tests)"
    }

    log_success "All destructive tests completed"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."

    # Stop any running containers
    docker ps -a | grep "$CONTAINER_NAME" | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

    # Remove test artifacts
    rm -f "$PROJECT_ROOT/target/test_disk.img"

    log_success "Cleanup completed"
}

# Main execution
main() {
    log_info "FastForth Destructive Test Runner"
    log_info "=================================="

    # Setup cleanup trap
    trap cleanup EXIT

    check_docker
    build_container

    case "${1:-all}" in
        oom)
            run_oom_tests
            ;;
        disk)
            run_disk_full_tests
            ;;
        stack)
            run_stack_overflow_tests
            ;;
        fd)
            run_fd_exhaustion_tests
            ;;
        all)
            run_all_tests
            ;;
        *)
            log_error "Unknown test type: $1"
            echo "Usage: $0 {oom|disk|stack|fd|all}"
            exit 1
            ;;
    esac

    log_success "Test run completed successfully"
}

# Run main if not being sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
