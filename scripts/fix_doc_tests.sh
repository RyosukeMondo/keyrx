#!/bin/bash
# Doc test fix script
# Automates cargo clean, workspace build, and doc test execution
# Resolves crate version mismatch issues in documentation tests

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Check if TTY is available for interactive output
HAS_TTY=false
if [[ -t 1 ]] && [[ -w /dev/tty ]]; then
    HAS_TTY=true
fi

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Fix and execute Rust documentation tests by cleaning build artifacts
and rebuilding the workspace to resolve crate version mismatches.

OPTIONS:
    --error             Show only errors
    --json              Output results in JSON format
    --quiet             Suppress non-error output
    --log-file PATH     Specify custom log file path
    -h, --help          Show this help message

EXAMPLES:
    $(basename "$0")                # Fix and run doc tests
    $(basename "$0") --json         # JSON output
    $(basename "$0") --quiet        # Minimal output

EXIT CODES:
    0 - Doc tests passed
    1 - Doc tests failed
    2 - Missing required tool
    3 - Build failed

OUTPUT MARKERS:
    === accomplished === - Doc tests passed
    === failed ===       - Doc tests failed
EOF
}

# Parse arguments
parse_common_flags "$@"
set -- "${REMAINING_ARGS[@]}"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo ""
            usage
            exit 1
            ;;
    esac
done

# Setup log file if not already set
if [[ -z "$LOG_FILE" ]]; then
    setup_log_file "fix_doc_tests"
fi

# Verify required tools
require_tool "cargo" "Install Rust from https://rustup.rs" || exit 2

# Track results
DOC_TESTS_PASSED=0
DOC_TESTS_FAILED=0

# Main execution
separator
log_info "Starting doc test fix workflow"
separator

# Step 1: Clean build artifacts
log_info "Step 1/3: Cleaning build artifacts..."
if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
    CLEAN_OUTPUT=$(cargo clean 2>&1)
else
    CLEAN_OUTPUT=$(cargo clean 2>&1 | tee /dev/tty)
fi
CLEAN_EXIT_CODE=$?

if [[ -n "$LOG_FILE" ]]; then
    echo "$CLEAN_OUTPUT" >> "$LOG_FILE"
fi

if [[ $CLEAN_EXIT_CODE -ne 0 ]]; then
    log_error "cargo clean failed with exit code $CLEAN_EXIT_CODE"
    log_failed

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "failed" \
            "step" "clean" \
            "exit_code" "$CLEAN_EXIT_CODE"
    fi

    exit 3
fi

log_info "Build artifacts cleaned successfully"

# Step 2: Build workspace
log_info "Step 2/3: Building workspace..."
if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
    BUILD_OUTPUT=$(cargo build --workspace 2>&1)
else
    BUILD_OUTPUT=$(cargo build --workspace 2>&1 | tee /dev/tty)
fi
BUILD_EXIT_CODE=$?

if [[ -n "$LOG_FILE" ]]; then
    echo "$BUILD_OUTPUT" >> "$LOG_FILE"
fi

if [[ $BUILD_EXIT_CODE -ne 0 ]]; then
    log_error "cargo build failed with exit code $BUILD_EXIT_CODE"
    log_failed

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "failed" \
            "step" "build" \
            "exit_code" "$BUILD_EXIT_CODE"
    fi

    exit 3
fi

log_info "Workspace built successfully"

# Step 3: Run doc tests
log_info "Step 3/3: Running doc tests..."
if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
    DOC_TEST_OUTPUT=$(cargo test --doc 2>&1)
else
    DOC_TEST_OUTPUT=$(cargo test --doc 2>&1 | tee /dev/tty)
fi
DOC_TEST_EXIT_CODE=$?

if [[ -n "$LOG_FILE" ]]; then
    echo "$DOC_TEST_OUTPUT" >> "$LOG_FILE"
fi

# Parse test results from output
# Format: "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
if echo "$DOC_TEST_OUTPUT" | grep -q "test result:"; then
    PASSED_LINE=$(echo "$DOC_TEST_OUTPUT" | grep "test result:" | tail -n 1)

    if echo "$PASSED_LINE" | grep -q "passed"; then
        DOC_TESTS_PASSED=$(echo "$PASSED_LINE" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
    fi

    if echo "$PASSED_LINE" | grep -q "failed"; then
        DOC_TESTS_FAILED=$(echo "$PASSED_LINE" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
    fi
fi

# Output results
separator

if [[ $DOC_TEST_EXIT_CODE -eq 0 ]]; then
    log_info "Doc tests completed: $DOC_TESTS_PASSED passed, $DOC_TESTS_FAILED failed"
    log_accomplished

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "success" \
            "doc_tests_passed" "$DOC_TESTS_PASSED" \
            "doc_tests_failed" "$DOC_TESTS_FAILED" \
            "exit_code" "0"
    fi

    exit 0
else
    log_error "Doc tests failed: $DOC_TESTS_PASSED passed, $DOC_TESTS_FAILED failed"
    log_failed

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "failed" \
            "doc_tests_passed" "$DOC_TESTS_PASSED" \
            "doc_tests_failed" "$DOC_TESTS_FAILED" \
            "exit_code" "$DOC_TEST_EXIT_CODE"
    fi

    exit 1
fi
