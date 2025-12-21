#!/bin/bash
# Test script with flexible test execution modes
# Supports unit tests, integration tests, fuzzing, and benchmarks

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Test mode flags
UNIT_MODE=false
INTEGRATION_MODE=false
FUZZ_DURATION=""
BENCH_MODE=false

# Parse arguments
parse_common_flags "$@"
set -- "${REMAINING_ARGS[@]}"

while [[ $# -gt 0 ]]; do
    case $1 in
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
        *)
            log_error "Unknown option: $1"
            log_error "Usage: $0 [--unit|--integration|--fuzz DURATION|--bench] [--error] [--json] [--quiet] [--log-file PATH]"
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
else
    require_tool "cargo" "Install Rust from https://rustup.rs"
fi

# Main execution
separator
log_info "Starting test execution (mode: $TEST_MODE)"
separator

# Execute appropriate test mode
EXIT_CODE=0

if [[ "$UNIT_MODE" == "true" ]]; then
    log_info "Running unit tests..."
    TEST_COMMAND="cargo test --lib --workspace"

    if [[ "$QUIET_MODE" == "true" ]]; then
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
    TEST_COMMAND="cargo test --test '*' --workspace"

    if [[ "$QUIET_MODE" == "true" ]]; then
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

    if [[ "$QUIET_MODE" == "true" ]]; then
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

    if [[ "$QUIET_MODE" == "true" ]]; then
        TEST_OUTPUT=$(cargo +nightly bench --workspace 2>&1)
    else
        TEST_OUTPUT=$(cargo +nightly bench --workspace 2>&1 | tee /dev/tty)
    fi
    EXIT_CODE=$?

    if [[ -n "$LOG_FILE" ]]; then
        echo "$TEST_OUTPUT" >> "$LOG_FILE"
    fi

else
    # Run all tests (default)
    log_info "Running all tests..."
    TEST_COMMAND="cargo test --workspace"

    if [[ "$QUIET_MODE" == "true" ]]; then
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
if [[ "$TEST_MODE" != "fuzz" ]] && [[ "$TEST_MODE" != "bench" ]]; then
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

# Output results
separator

if [[ $EXIT_CODE -eq 0 ]]; then
    if [[ "$TEST_MODE" == "fuzz" ]]; then
        log_info "Fuzz testing completed (${FUZZ_DURATION}s)"
    elif [[ "$TEST_MODE" == "bench" ]]; then
        log_info "Benchmarks completed"
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
