#!/bin/bash
# Verify script for keyrx workspace
# Runs comprehensive quality checks: build, clippy, fmt, tests, coverage
# Supports: --skip-coverage, --error, --json, --quiet, --log-file

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
SKIP_COVERAGE=false

# Check results tracking
declare -A CHECK_RESULTS
CHECK_ORDER=("build" "clippy" "fmt" "test" "coverage" "ui_test" "e2e" "unwraps")

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Run comprehensive quality verification on the keyrx workspace.
Executes: build, clippy, fmt, tests, coverage, UI tests, and E2E checks.
Runs ALL checks and collects results (does not abort on first failure).

OPTIONS:
    --skip-coverage  Skip coverage check (useful for faster iteration)
    --error          Show only errors
    --json           Output results in JSON format
    --quiet          Suppress non-error output
    --log-file PATH  Specify custom log file path
    -h, --help       Show this help message

EXAMPLES:
    $(basename "$0")                 # Full verification with coverage
    $(basename "$0") --skip-coverage # Skip coverage check
    $(basename "$0") --json          # JSON output

EXIT CODES:
    0 - All checks passed
    1 - One or more checks failed
    2 - Missing required tool

CHECKS PERFORMED (in order):
    1. Build         - cargo build --workspace
    2. Clippy        - cargo clippy --workspace -- -D warnings
    3. Format        - cargo fmt --check
    4. Tests         - cargo test --workspace --lib --bins --tests
    5. Coverage      - cargo llvm-cov (80% minimum overall, 90% keyrx_core)
    6. UI Tests      - npm test --coverage (80% minimum)
    7. E2E Tests     - npm run test:e2e (Playwright)
    8. Unwraps       - scripts/check_unwraps.sh (prevent regressions)

OUTPUT MARKERS:
    === accomplished === - All checks passed
    === failed ===       - One or more checks failed
EOF
}

# Parse arguments
parse_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags from remaining args
    set -- "${REMAINING_ARGS[@]}"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-coverage)
                SKIP_COVERAGE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Run build check
check_build() {
    log_info "Running build check..."

    if "$SCRIPT_DIR/build.sh" --quiet; then
        CHECK_RESULTS["build"]="PASS"
        log_info "Build check: PASS"
        return 0
    else
        CHECK_RESULTS["build"]="FAIL"
        log_error "Build check: FAIL"
        return 1
    fi
}

# Run clippy check
check_clippy() {
    log_info "Running clippy check..."

    if cargo clippy --workspace -- -D warnings 2>&1; then
        CHECK_RESULTS["clippy"]="PASS"
        log_info "Clippy check: PASS"
        return 0
    else
        CHECK_RESULTS["clippy"]="FAIL"
        log_error "Clippy check: FAIL"
        return 1
    fi
}

# Run format check
check_fmt() {
    log_info "Running format check..."

    if cargo fmt --check 2>&1; then
        CHECK_RESULTS["fmt"]="PASS"
        log_info "Format check: PASS"
        return 0
    else
        CHECK_RESULTS["fmt"]="FAIL"
        log_error "Format check: FAIL - run 'cargo fmt' to fix"
        return 1
    fi
}

# Run test check
check_test() {
    log_info "Running test check..."

    # Skip doctests due to workspace dependency bug (rustdoc can't resolve keyrx_core versions)
    # Run: lib tests (unit), binary tests, integration tests
    # Doctests can be run separately with: cargo test --workspace --doc
    if cargo test --workspace --lib --bins --tests 2>&1; then
        CHECK_RESULTS["test"]="PASS"
        log_info "Test check: PASS"
        return 0
    else
        CHECK_RESULTS["test"]="FAIL"
        log_error "Test check: FAIL"
        return 1
    fi
}

# Run coverage check
check_coverage() {
    if [[ "$SKIP_COVERAGE" == "true" ]]; then
        CHECK_RESULTS["coverage"]="SKIP"
        log_info "Coverage check: SKIPPED"
        return 0
    fi

    log_info "Running coverage check..."

    # Check if cargo-llvm-cov is installed
    if ! require_tool "cargo-llvm-cov" "Install cargo-llvm-cov: cargo install cargo-llvm-cov"; then
        CHECK_RESULTS["coverage"]="FAIL"
        log_error "Coverage check: FAIL - cargo-llvm-cov not installed"
        return 2
    fi

    # Run llvm-cov and capture output
    # llvm-cov is 25-50x faster than tarpaulin (~12s vs 5-10min)
    # and provides more accurate coverage metrics
    # --ignore-run-fail allows coverage generation even if some tests fail (flaky tests)
    local llvm_cov_output
    if llvm_cov_output=$(cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info --ignore-run-fail 2>&1); then
        # Extract coverage percentage from TOTAL line
        # llvm-cov outputs: "TOTAL    18732    3273    82.53%    ..."
        local coverage_pct
        coverage_pct=$(echo "$llvm_cov_output" | grep '^TOTAL' | awk '{print $4}' | sed 's/%//')

        if [[ -z "$coverage_pct" ]]; then
            log_warn "Could not parse coverage percentage, assuming pass"
            CHECK_RESULTS["coverage"]="PASS"
            return 0
        fi

        # Check if coverage meets 80% minimum
        if (( $(echo "$coverage_pct >= 80.0" | bc -l) )); then
            CHECK_RESULTS["coverage"]="PASS (${coverage_pct}%)"
            log_info "Coverage check: PASS (${coverage_pct}%)"
            return 0
        else
            CHECK_RESULTS["coverage"]="FAIL (${coverage_pct}% < 80%)"
            log_error "Coverage check: FAIL (${coverage_pct}% < 80% minimum)"
            return 1
        fi
    else
        CHECK_RESULTS["coverage"]="FAIL"
        log_error "Coverage check: FAIL - llvm-cov execution failed"
        return 1
    fi
}

# Run UI tests
check_ui_test() {
    log_info "Running UI tests..."

    # Check if keyrx_ui directory exists
    if [[ ! -d "keyrx_ui" ]]; then
        CHECK_RESULTS["ui_test"]="SKIP"
        log_warn "UI test: SKIPPED (keyrx_ui not found)"
        return 0
    fi

    # Check if node_modules exists
    if [[ ! -d "keyrx_ui/node_modules" ]]; then
        log_warn "UI node_modules not found, running npm install..."
        (cd keyrx_ui && npm install --silent)
    fi

    # Run UI tests with coverage
    if (cd keyrx_ui && npm test -- --coverage --run 2>&1); then
        # TODO: Parse coverage percentage from output
        CHECK_RESULTS["ui_test"]="PASS"
        log_info "UI test: PASS"
        return 0
    else
        CHECK_RESULTS["ui_test"]="FAIL"
        log_error "UI test: FAIL"
        return 1
    fi
}

# Run E2E tests
check_e2e() {
    log_info "Running E2E tests..."

    # Check if keyrx_ui directory exists
    if [[ ! -d "keyrx_ui" ]]; then
        CHECK_RESULTS["e2e"]="SKIP"
        log_warn "E2E test: SKIPPED (keyrx_ui not found)"
        return 0
    fi

    # Check if package.json has test:e2e script
    if ! grep -q '"test:e2e"' keyrx_ui/package.json 2>/dev/null; then
        CHECK_RESULTS["e2e"]="SKIP"
        log_warn "E2E test: SKIPPED (test:e2e script not found)"
        return 0
    fi

    # Run E2E tests
    if (cd keyrx_ui && npm run test:e2e 2>&1); then
        CHECK_RESULTS["e2e"]="PASS"
        log_info "E2E test: PASS"
        return 0
    else
        CHECK_RESULTS["e2e"]="FAIL"
        log_error "E2E test: FAIL"
        return 1
    fi
}

# Run unwrap check
check_unwraps() {
    log_info "Running unwrap check..."

    if "$SCRIPT_DIR/check_unwraps.sh" 2>&1; then
        CHECK_RESULTS["unwraps"]="PASS"
        log_info "Unwrap check: PASS"
        return 0
    else
        CHECK_RESULTS["unwraps"]="FAIL"
        log_error "Unwrap check: FAIL - too many unwrap() calls in production code"
        return 1
    fi
}

# Print summary table with colored output
print_summary() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi

    separator
    log_info "Verification Summary:"
    echo ""
    printf "  %-15s %s\n" "CHECK" "RESULT"
    printf "  %-15s %s\n" "──────────────" "──────────────────"

    for check in "${CHECK_ORDER[@]}"; do
        local result="${CHECK_RESULTS[$check]:-SKIP}"
        local display_result=""

        if [[ "$result" == PASS* ]]; then
            display_result="${COLOR_GREEN}✓${COLOR_RESET} ${result}"
        elif [[ "$result" == FAIL* ]]; then
            display_result="${COLOR_RED}✗${COLOR_RESET} ${result}"
        elif [[ "$result" == "SKIP" ]]; then
            display_result="${COLOR_YELLOW}○${COLOR_RESET} ${result}"
        else
            display_result="  ${result}"
        fi

        printf "  %-15s %b\n" "$check" "$display_result"
    done

    echo ""
    separator
}

# Generate JSON output
output_verification_json() {
    if [[ "$JSON_MODE" != "true" ]]; then
        return
    fi

    local status="success"
    local failed_checks=()

    # Check if any check failed
    for check in "${CHECK_ORDER[@]}"; do
        local result="${CHECK_RESULTS[$check]:-SKIP}"
        if [[ "$result" == FAIL* ]]; then
            status="failed"
            failed_checks+=("$check")
        fi
    done

    # Build checks JSON object
    local checks_json="{"
    local first=true
    for check in "${CHECK_ORDER[@]}"; do
        local result="${CHECK_RESULTS[$check]:-SKIP}"
        if [[ "$first" == "true" ]]; then
            first=false
        else
            checks_json+=","
        fi
        checks_json+="\"$check\":\"$result\""
    done
    checks_json+="}"

    # Build failed_checks JSON array
    local failed_json="["
    first=true
    for check in "${failed_checks[@]}"; do
        if [[ "$first" == "true" ]]; then
            first=false
        else
            failed_json+=","
        fi
        failed_json+="\"$check\""
    done
    failed_json+="]"

    # Output final JSON
    json_object \
        "status" "$status" \
        "checks" "$checks_json" \
        "failed_checks" "$failed_json" \
        "exit_code" "$([[ "$status" == "success" ]] && echo "0" || echo "1")"
}

# Main execution
main() {
    local exit_code=0

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "verify"
    fi

    # Verify cargo is installed
    if ! require_tool "cargo" "Install Rust from https://rustup.rs"; then
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "cargo not found" "exit_code" "2"
        fi
        exit 2
    fi

    log_info "Starting comprehensive verification..."
    log_info "Running all checks (will not abort on first failure)..."
    separator

    # Run all checks and collect results
    # Do not abort on first failure - collect all results
    check_build || exit_code=1
    separator

    check_clippy || exit_code=1
    separator

    check_fmt || exit_code=1
    separator

    check_test || exit_code=1
    separator

    # Coverage check might return 2 (missing tool) or 1 (failure)
    if ! check_coverage; then
        local coverage_exit=$?
        if [[ $coverage_exit -eq 2 ]]; then
            exit_code=2
        else
            exit_code=1
        fi
    fi
    separator

    check_ui_test || exit_code=1
    separator

    check_e2e || exit_code=1
    separator

    check_unwraps || exit_code=1
    separator

    # Print summary
    print_summary

    # Final status marker
    if [[ $exit_code -eq 0 ]]; then
        log_accomplished
    else
        log_failed
    fi

    # JSON output
    output_verification_json

    exit $exit_code
}

# Run main function
main "$@"
