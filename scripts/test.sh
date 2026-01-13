#!/bin/bash
# Test script with flexible test execution modes
# Supports unit tests, integration tests, fuzzing, and benchmarks

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

# Test mode flags
UNIT_MODE=false
INTEGRATION_MODE=false
FUZZ_DURATION=""
BENCH_MODE=false
COVERAGE_MODE=false
USE_NEXTEST=false

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Execute tests with flexible filtering options.

OPTIONS:
    --unit              Run only unit tests (lib tests)
    --integration       Run only integration tests (tests/ directory)
    --fuzz DURATION     Run fuzz tests for DURATION seconds (requires cargo-fuzz)
    --bench             Run benchmarks (requires nightly toolchain)
    --coverage          Run tests with coverage analysis (requires cargo-tarpaulin)
    --nextest           Use cargo-nextest for faster test execution (recommended)
    --error             Show only errors
    --json              Output results in JSON format
    --quiet             Suppress non-error output
    --log-file PATH     Specify custom log file path
    -h, --help          Show this help message

EXAMPLES:
    $(basename "$0")                    # Run all tests
    $(basename "$0") --unit             # Unit tests only
    $(basename "$0") --integration      # Integration tests only
    $(basename "$0") --nextest          # Use nextest for faster execution
    $(basename "$0") --nextest --unit   # Fast unit tests with nextest
    $(basename "$0") --fuzz 60          # Fuzz for 60 seconds
    $(basename "$0") --bench            # Run benchmarks
    $(basename "$0") --coverage         # Run with coverage analysis
    $(basename "$0") --json             # JSON output

EXIT CODES:
    0 - Tests passed
    1 - Tests failed
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - Tests passed
    === failed ===       - Tests failed
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
        --unit)
            UNIT_MODE=true
            shift
            ;;
        --integration)
            INTEGRATION_MODE=true
            shift
            ;;
        --fuzz)
            if [[ -z "${2:-}" ]]; then
                log_error "--fuzz requires a duration argument (in seconds)"
                exit 1
            fi
            FUZZ_DURATION="$2"
            shift 2
            ;;
        --bench)
            BENCH_MODE=true
            shift
            ;;
        --coverage)
            COVERAGE_MODE=true
            shift
            ;;
        --nextest)
            USE_NEXTEST=true
            shift
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
    setup_log_file "test"
fi

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0
TEST_MODE="all"
TEST_COMMAND=""

# Verify required tools based on mode
if [[ -n "$FUZZ_DURATION" ]]; then
    require_tool "cargo" "Install Rust from https://rustup.rs"
    if ! cargo fuzz --version >/dev/null 2>&1; then
        log_error "cargo-fuzz not found"
        log_error "Install with: cargo install cargo-fuzz"
        exit 1
    fi
    TEST_MODE="fuzz"
elif [[ "$BENCH_MODE" == "true" ]]; then
    require_tool "cargo" "Install Rust from https://rustup.rs"
    # Check if nightly toolchain is available
    if ! cargo +nightly --version >/dev/null 2>&1; then
        log_error "Nightly Rust toolchain not found"
        log_error "Install with: rustup install nightly"
        exit 1
    fi
    TEST_MODE="bench"
elif [[ "$COVERAGE_MODE" == "true" ]]; then
    require_tool "cargo" "Install Rust from https://rustup.rs"
    if ! cargo tarpaulin --version >/dev/null 2>&1; then
        log_error "cargo-tarpaulin not found"
        log_error "Install with: cargo install cargo-tarpaulin"
        exit 2
    fi
    TEST_MODE="coverage"
else
    require_tool "cargo" "Install Rust from https://rustup.rs"

    # Check if nextest should be used
    if [[ "$USE_NEXTEST" == "true" ]]; then
        if ! cargo nextest --version >/dev/null 2>&1; then
            log_error "cargo-nextest not found"
            log_error "Install with: cargo install cargo-nextest"
            exit 2
        fi
        TEST_MODE="nextest"
    fi
fi

# Main execution
separator
log_info "Starting test execution (mode: $TEST_MODE)"
separator

# Execute appropriate test mode
EXIT_CODE=0

if [[ "$UNIT_MODE" == "true" ]]; then
    log_info "Running unit tests..."

    if [[ "$USE_NEXTEST" == "true" ]]; then
        TEST_COMMAND="cargo nextest run --lib --workspace"
    else
        TEST_COMMAND="cargo test --lib --workspace"
    fi

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$($TEST_COMMAND 2>&1)
    else
        TEST_OUTPUT=$($TEST_COMMAND 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

elif [[ "$INTEGRATION_MODE" == "true" ]]; then
    log_info "Running integration tests..."

    if [[ "$USE_NEXTEST" == "true" ]]; then
        # Nextest filters integration tests with --test flag
        TEST_COMMAND="cargo nextest run --workspace --tests"
    else
        TEST_COMMAND="cargo test --test '*' --workspace"
    fi

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$($TEST_COMMAND 2>&1)
    else
        TEST_OUTPUT=$($TEST_COMMAND 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

elif [[ -n "$FUZZ_DURATION" ]]; then
    log_info "Running fuzz tests for ${FUZZ_DURATION}s..."

    # Check if fuzz directory exists
    if [[ ! -d "keyrx_core/fuzz" ]]; then
        log_error "Fuzz directory not found: keyrx_core/fuzz"
        log_error "Initialize fuzzing with: cd keyrx_core && cargo fuzz init"
        exit 1
    fi

    TEST_COMMAND="cargo fuzz run fuzz_target_1 -- -max_total_time=$FUZZ_DURATION"

    cd keyrx_core/fuzz || {
        log_error "Failed to change to fuzz directory"
        exit 1
    }

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$(cargo fuzz run fuzz_target_1 -- -max_total_time="$FUZZ_DURATION" 2>&1)
    else
        TEST_OUTPUT=$(cargo fuzz run fuzz_target_1 -- -max_total_time="$FUZZ_DURATION" 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    cd ../.. || exit 1

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

elif [[ "$BENCH_MODE" == "true" ]]; then
    log_info "Running benchmarks..."
    TEST_COMMAND="cargo +nightly bench --workspace"

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$(cargo +nightly bench --workspace 2>&1)
    else
        TEST_OUTPUT=$(cargo +nightly bench --workspace 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

elif [[ "$COVERAGE_MODE" == "true" ]]; then
    log_info "Running tests with coverage analysis..."

    # Run tarpaulin with HTML and lcov output
    # Test keyrx_core and keyrx_compiler only (80% threshold)
    # Exclude daemon due to platform-specific code that can't be tested in all CI environments
    TEST_COMMAND="cargo tarpaulin -p keyrx_core -p keyrx_compiler --out Html --out Lcov --output-dir coverage --line --ignore-tests --timeout 300"

    log_info "Analyzing coverage for keyrx_core and keyrx_compiler..."
    log_info "Coverage threshold: 80%"

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$(cargo tarpaulin -p keyrx_core -p keyrx_compiler --out Html --out Lcov --output-dir coverage --line --ignore-tests --timeout 300 2>&1)
    else
        TEST_OUTPUT=$(cargo tarpaulin -p keyrx_core -p keyrx_compiler --out Html --out Lcov --output-dir coverage --line --ignore-tests --timeout 300 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

    # Parse coverage percentage from output
    if [[ $EXIT_CODE -eq 0 ]]; then
        # Extract overall coverage percentage (last occurrence)
        COVERAGE_PCT=$(echo "$TEST_OUTPUT" | grep -oP '\d+\.\d+%' | tail -1 | sed 's/%//')

        if [[ -n "$COVERAGE_PCT" ]]; then
            log_info "Overall coverage: ${COVERAGE_PCT}%"

            # Check if coverage meets 80% threshold
            if (( $(echo "$COVERAGE_PCT < 80.0" | bc -l) )); then
                log_error "Coverage ${COVERAGE_PCT}% is below 80% threshold"
                log_info "Note: Current baseline is ~68%. Additional tests needed to reach 80%."
                EXIT_CODE=1
            else
                log_info "Coverage ${COVERAGE_PCT}% meets 80% threshold ✓"
            fi
        fi

        log_info "Coverage reports generated in coverage/ directory"
        log_info "  - HTML report: coverage/index.html"
        log_info "  - LCOV report: coverage/lcov.info"
    fi

else
    # Run all tests (default)
    log_info "Running all tests..."

    if [[ "$USE_NEXTEST" == "true" ]]; then
        TEST_COMMAND="cargo nextest run --workspace"
    else
        TEST_COMMAND="cargo test --workspace"
    fi

    if [[ "$QUIET_MODE" == "true" ]] || [[ "$HAS_TTY" == "false" ]]; then
        TEST_OUTPUT=$($TEST_COMMAND 2>&1)
    else
        TEST_OUTPUT=$($TEST_COMMAND 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi
fi

# Parse test results from output
if [[ "$TEST_MODE" != "fuzz" ]] && [[ "$TEST_MODE" != "bench" ]] && [[ "$TEST_MODE" != "coverage" ]]; then
    if [[ "$TEST_MODE" == "nextest" ]]; then
        # Parse nextest output
        # Format: "Summary [ 10.517s] 674 tests run: 674 passed, 11 skipped"
        if echo "$TEST_OUTPUT" | grep -q "Summary"; then
            SUMMARY_LINE=$(echo "$TEST_OUTPUT" | grep "Summary" | tail -n 1)

            if echo "$SUMMARY_LINE" | grep -q "passed"; then
                TESTS_PASSED=$(echo "$SUMMARY_LINE" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
            fi

            if echo "$SUMMARY_LINE" | grep -q "failed"; then
                TESTS_FAILED=$(echo "$SUMMARY_LINE" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
            fi
        fi
    else
        # Extract passed/failed counts from cargo test output
        # Format: "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        if echo "$TEST_OUTPUT" | grep -q "test result:"; then
            PASSED_LINE=$(echo "$TEST_OUTPUT" | grep "test result:" | tail -n 1)

            if echo "$PASSED_LINE" | grep -q "passed"; then
                TESTS_PASSED=$(echo "$PASSED_LINE" | sed -n 's/.*\([0-9]\+\) passed.*/\1/p')
            fi

            if echo "$PASSED_LINE" | grep -q "failed"; then
                TESTS_FAILED=$(echo "$PASSED_LINE" | sed -n 's/.*\([0-9]\+\) failed.*/\1/p')
            fi
        fi
    fi
fi

# Output results
separator

if [[ $EXIT_CODE -eq 0 ]]; then
    if [[ "$TEST_MODE" == "fuzz" ]]; then
        log_info "Fuzz testing completed (${FUZZ_DURATION}s)"
    elif [[ "$TEST_MODE" == "bench" ]]; then
        log_info "Benchmarks completed"
    elif [[ "$TEST_MODE" == "coverage" ]]; then
        log_info "Coverage analysis completed"
    else
        log_info "Tests completed: $TESTS_PASSED passed, $TESTS_FAILED failed"
    fi

    log_accomplished

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "success" \
            "mode" "$TEST_MODE" \
            "tests_passed" "$TESTS_PASSED" \
            "tests_failed" "$TESTS_FAILED" \
            "exit_code" "0"
    fi
else
    if [[ "$TEST_MODE" == "fuzz" ]]; then
        log_error "Fuzz testing failed"
    elif [[ "$TEST_MODE" == "bench" ]]; then
        log_error "Benchmarks failed"
    elif [[ "$TEST_MODE" == "coverage" ]]; then
        log_error "Coverage analysis failed or coverage below threshold"
    else
        log_error "Tests failed: $TESTS_PASSED passed, $TESTS_FAILED failed"
    fi

    log_failed

    if [[ "$JSON_MODE" == "true" ]]; then
        json_object \
            "status" "failed" \
            "mode" "$TEST_MODE" \
            "tests_passed" "$TESTS_PASSED" \
            "tests_failed" "$TESTS_FAILED" \
            "exit_code" "$EXIT_CODE"
    fi
fi

exit $EXIT_CODE
