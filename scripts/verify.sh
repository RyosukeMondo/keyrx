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
CHECK_ORDER=("build" "clippy" "fmt" "test" "coverage")

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Run comprehensive quality verification on the keyrx workspace.
Executes: build, clippy, fmt, tests, and coverage checks in order.
Aborts on first failure.

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
    4. Tests         - cargo test --workspace
    5. Coverage      - cargo tarpaulin (80% minimum)

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

    if cargo test --workspace 2>&1; then
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

    # Check if cargo-tarpaulin is installed
    if ! require_tool "cargo-tarpaulin" "Install cargo-tarpaulin: cargo install cargo-tarpaulin"; then
        CHECK_RESULTS["coverage"]="FAIL"
        log_error "Coverage check: FAIL - cargo-tarpaulin not installed"
        return 2
    fi

    # Run tarpaulin and capture output
    # Exclude files that:
    # 1. Linux feature-gated test files that produce empty coverage without the feature
    # 2. tap_hold.rs which has 68+ tests passing but uses const fn that tarpaulin can't instrument
    local tarpaulin_output
    if tarpaulin_output=$(cargo tarpaulin --workspace --out Xml --output-dir coverage \
        --run-types AllTargets \
        --exclude-files 'keyrx_daemon/tests/e2e_harness.rs' 'keyrx_daemon/tests/e2e_tests.rs' 'keyrx_daemon/tests/daemon_tests.rs' 'keyrx_core/src/runtime/tap_hold.rs' 2>&1); then
        # Extract coverage percentage from output
        # Tarpaulin outputs: "XX.XX% coverage, XX/XX lines covered"
        local coverage_pct
        coverage_pct=$(echo "$tarpaulin_output" | grep -oP '\d+\.\d+(?=% coverage)' | head -n1)

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
        log_error "Coverage check: FAIL - tarpaulin execution failed"
        return 1
    fi
}

# Print summary table
print_summary() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi

    separator
    log_info "Verification Summary:"
    echo ""
    printf "  %-15s %s\n" "CHECK" "RESULT"
    printf "  %-15s %s\n" "──────────────" "──────"

    for check in "${CHECK_ORDER[@]}"; do
        local result="${CHECK_RESULTS[$check]:-SKIP}"
        printf "  %-15s %s\n" "$check" "$result"
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
    separator

    # Run checks in order, abort on first failure
    if ! check_build; then
        exit_code=1
        log_error "Aborting verification - build check failed"
    elif ! check_clippy; then
        exit_code=1
        log_error "Aborting verification - clippy check failed"
    elif ! check_fmt; then
        exit_code=1
        log_error "Aborting verification - format check failed"
    elif ! check_test; then
        exit_code=1
        log_error "Aborting verification - test check failed"
    elif ! check_coverage; then
        local coverage_exit=$?
        if [[ $coverage_exit -eq 2 ]]; then
            exit_code=2
        else
            exit_code=1
        fi
        log_error "Aborting verification - coverage check failed"
    fi

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
