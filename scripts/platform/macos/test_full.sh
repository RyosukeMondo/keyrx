#!/usr/bin/env bash
# macOS comprehensive test runner
# Orchestrates mock tests, E2E tests, and benchmarks with permission checking

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Track test results
MOCK_TESTS_PASSED=false
E2E_TESTS_PASSED=false
E2E_TESTS_SKIPPED=false
BENCHMARKS_PASSED=false
MANUAL_TESTS_RUN=false
OVERALL_EXIT_CODE=0

# Track test metrics
MOCK_TEST_COUNT=0
E2E_TEST_COUNT=0
COVERAGE_PERCENT=""
BENCHMARK_RESULT=""

# Color codes
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_BLUE='\033[0;34m'

# Cleanup handler
cleanup() {
    # Additional cleanup can be added here if needed
    :
}

# Trap Ctrl+C and errors
trap cleanup EXIT
trap 'echo -e "\n${COLOR_YELLOW}⚠️  Test run interrupted${COLOR_RESET}"; exit 130' INT

# Print section header
print_section() {
    echo ""
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo -e "${COLOR_BLUE}$1${COLOR_RESET}"
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo ""
}

# Print step indicator
print_step() {
    echo -e "${COLOR_BLUE}▶ $1${COLOR_RESET}"
}

# Print success
print_success() {
    echo -e "${COLOR_GREEN}✅ $1${COLOR_RESET}"
}

# Print warning
print_warning() {
    echo -e "${COLOR_YELLOW}⚠️  $1${COLOR_RESET}"
}

# Print error
print_error() {
    echo -e "${COLOR_RED}❌ $1${COLOR_RESET}"
}

# Check if terminal supports color
supports_color() {
    if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
        if [ "$(tput colors)" -ge 8 ]; then
            return 0
        fi
    fi
    return 1
}

# Change to project root
cd "$PROJECT_ROOT"

print_section "macOS Test Suite"

#
# Phase 1: Mock Tests (Required)
#
print_section "Phase 1: Mock Tests"
print_step "Running mock tests (no permissions required)..."

# Capture test output to count tests
MOCK_TEST_OUTPUT=$(mktemp)
if cargo test --package keyrx_daemon --test macos_mock_tests -- --test-threads=1 2>&1 | tee "$MOCK_TEST_OUTPUT"; then
    print_success "Mock tests passed"
    MOCK_TESTS_PASSED=true
    # Extract test count from output (e.g., "test result: ok. 15 passed")
    MOCK_TEST_COUNT=$(grep -E "test result: ok\." "$MOCK_TEST_OUTPUT" | sed -n 's/.* \([0-9][0-9]*\) passed.*/\1/p' | head -n1)
    MOCK_TEST_COUNT=${MOCK_TEST_COUNT:-0}
else
    print_error "Mock tests failed"
    OVERALL_EXIT_CODE=1
    rm -f "$MOCK_TEST_OUTPUT"
    # Mock test failure is critical - stop execution
    echo ""
    echo -e "${COLOR_RED}========================================${COLOR_RESET}"
    echo -e "${COLOR_RED}CRITICAL: Mock tests failed${COLOR_RESET}"
    echo -e "${COLOR_RED}========================================${COLOR_RESET}"
    echo ""
    exit 1
fi
rm -f "$MOCK_TEST_OUTPUT"

#
# Phase 2: Permission Check
#
print_section "Phase 2: Accessibility Permission Check"
print_step "Checking Accessibility permission..."

if "$SCRIPT_DIR/check_permission.sh"; then
    print_success "Accessibility permission granted"
    HAS_PERMISSION=true
else
    print_warning "Accessibility permission not granted"
    HAS_PERMISSION=false
    echo ""
    echo "E2E tests will be skipped. To run E2E tests:"
    echo "  1. Open System Settings > Privacy & Security > Accessibility"
    echo "  2. Grant permission to Terminal (or your IDE)"
    echo "  3. Re-run this script"
    echo ""
fi

#
# Phase 3: E2E Tests (Permission Required)
#
print_section "Phase 3: E2E Tests"

if [ "$HAS_PERMISSION" = true ]; then
    print_step "Running E2E tests (permission granted)..."

    # Capture test output to count tests
    E2E_TEST_OUTPUT=$(mktemp)
    if cargo test --package keyrx_daemon --test e2e_macos_basic --test e2e_macos_multidevice -- --test-threads=1 2>&1 | tee "$E2E_TEST_OUTPUT"; then
        print_success "E2E tests passed"
        E2E_TESTS_PASSED=true
        # Extract test count from output
        E2E_TEST_COUNT=$(grep -E "test result: ok\." "$E2E_TEST_OUTPUT" | sed -n 's/.* \([0-9][0-9]*\) passed.*/\1/p' | head -n1)
        E2E_TEST_COUNT=${E2E_TEST_COUNT:-0}
    else
        print_error "E2E tests failed"
        E2E_TESTS_PASSED=false
        OVERALL_EXIT_CODE=1
    fi
    rm -f "$E2E_TEST_OUTPUT"
else
    print_warning "E2E tests skipped (no permission)"
    E2E_TESTS_SKIPPED=true
fi

#
# Phase 4: Benchmarks
#
print_section "Phase 4: Benchmarks"
print_step "Running benchmarks..."

# Capture benchmark output to extract results
BENCH_OUTPUT=$(mktemp)
if cargo bench --package keyrx_daemon --bench macos_latency 2>&1 | tee "$BENCH_OUTPUT"; then
    print_success "Benchmarks completed"
    BENCHMARKS_PASSED=true
    # Extract benchmark result (e.g., "time: [123.45 ns 125.67 ns 127.89 ns]")
    # Look for lines with timing data
    BENCHMARK_RESULT=$(grep -E "time:.*\[.*ns.*\]" "$BENCH_OUTPUT" | head -n1 | sed 's/.*time:.*\[\([^]]*\)\].*/\1/' || echo "")
else
    print_warning "Benchmarks failed or not available"
    BENCHMARKS_PASSED=false
    # Benchmark failure is not critical
fi
rm -f "$BENCH_OUTPUT"

#
# Phase 4.5: Coverage Analysis (Optional)
#
# Only run coverage if tarpaulin is available and tests passed
if command -v cargo-tarpaulin >/dev/null 2>&1 && [ "$MOCK_TESTS_PASSED" = true ]; then
    print_section "Coverage Analysis"
    print_step "Analyzing test coverage (this may take a moment)..."

    # Run tarpaulin only on platform/macos modules
    COVERAGE_OUTPUT=$(mktemp)
    if cargo tarpaulin --package keyrx_daemon --target-dir target/tarpaulin \
        --include-tests --exclude-files 'tests/*' \
        --out Stdout 2>&1 | tee "$COVERAGE_OUTPUT" >/dev/null; then

        # Extract coverage percentage (e.g., "82.45% coverage")
        COVERAGE_PERCENT=$(grep -E "[0-9]+\.[0-9]+% coverage" "$COVERAGE_OUTPUT" | tail -n1 | sed -n 's/\([0-9]\+\.[0-9]\+\)% coverage.*/\1/p')

        if [ -n "$COVERAGE_PERCENT" ]; then
            print_success "Coverage analysis complete: ${COVERAGE_PERCENT}%"
        else
            print_warning "Coverage percentage could not be extracted"
        fi
    else
        print_warning "Coverage analysis failed"
    fi
    rm -f "$COVERAGE_OUTPUT"
else
    if [ "$MOCK_TESTS_PASSED" = false ]; then
        print_warning "Coverage analysis skipped (tests failed)"
    else
        print_warning "Coverage analysis skipped (cargo-tarpaulin not installed)"
        echo "    Install with: cargo install cargo-tarpaulin"
    fi
fi

#
# Phase 5: Manual Test Prompt (Interactive Only)
#
# Only prompt if:
# - stdin is a terminal (interactive mode)
# - Accessibility permission is granted
if [ -t 0 ] && [ "$HAS_PERMISSION" = true ]; then
    print_section "Phase 5: Manual Testing (Optional)"

    echo "Automated tests complete. Would you like to run manual tests?"
    echo "Manual tests require physical keyboard interaction to verify:"
    echo "  • Key remapping accuracy"
    echo "  • Tap-hold timing behavior"
    echo "  • Multi-device discrimination"
    echo ""

    # Prompt with default to No (safety)
    read -r -p "Run manual tests? [y/N] " response || {
        # Handle EOF gracefully (e.g., Ctrl+D)
        echo ""
        print_warning "Manual tests skipped (EOF received)"
        response="n"
    }

    echo ""

    case "$response" in
        [yY][eE][sS]|[yY])
            print_step "Manual testing instructions:"
            echo ""
            echo "1. Ensure keyrx daemon is running with a test config"
            echo "2. Test basic remapping (e.g., A → B)"
            echo "3. Test tap-hold behavior with timing variations"
            echo "4. If multi-device, test device-specific configs"
            echo "5. Verify latency is imperceptible (<1ms)"
            echo ""
            echo "Refer to docs/development/MACOS_TESTING_GUIDE.md for detailed procedures"
            echo ""
            read -r -p "Press Enter when manual testing is complete..." || true
            echo ""
            print_success "Manual tests completed"
            MANUAL_TESTS_RUN=true
            ;;
        *)
            print_warning "Manual tests skipped by user"
            ;;
    esac
fi

#
# Summary
#
print_section "Test Summary"

# Calculate total tests run
TOTAL_TESTS=$((MOCK_TEST_COUNT + E2E_TEST_COUNT))

echo "Test Results:"
echo ""

# Mock tests
if [ "$MOCK_TESTS_PASSED" = true ]; then
    print_success "Mock tests: PASSED ($MOCK_TEST_COUNT tests)"
else
    print_error "Mock tests: FAILED"
fi

# E2E tests
if [ "$E2E_TESTS_SKIPPED" = true ]; then
    print_warning "E2E tests: SKIPPED (no permission)"
elif [ "$E2E_TESTS_PASSED" = true ]; then
    print_success "E2E tests: PASSED ($E2E_TEST_COUNT tests)"
else
    print_error "E2E tests: FAILED"
fi

# Coverage
if [ -n "$COVERAGE_PERCENT" ]; then
    # Check if coverage meets threshold
    COVERAGE_INT=$(echo "$COVERAGE_PERCENT" | cut -d'.' -f1)
    if [ "$COVERAGE_INT" -ge 80 ]; then
        print_success "Coverage: ${COVERAGE_PERCENT}% (threshold: 80%)"
    else
        print_warning "Coverage: ${COVERAGE_PERCENT}% (threshold: 80%)"
    fi
else
    echo -e "${COLOR_BLUE}ℹ️  Coverage: Not measured${COLOR_RESET}"
fi

# Benchmarks
if [ "$BENCHMARKS_PASSED" = true ]; then
    if [ -n "$BENCHMARK_RESULT" ]; then
        print_success "Benchmarks: COMPLETED ($BENCHMARK_RESULT)"
    else
        print_success "Benchmarks: COMPLETED"
    fi
else
    print_warning "Benchmarks: FAILED/UNAVAILABLE"
fi

# Manual tests
if [ "$MANUAL_TESTS_RUN" = true ]; then
    print_success "Manual tests: COMPLETED"
elif [ -t 0 ] && [ "$HAS_PERMISSION" = true ]; then
    print_warning "Manual tests: SKIPPED BY USER"
else
    echo -e "${COLOR_BLUE}ℹ️  Manual tests: NOT PROMPTED${COLOR_RESET}"
fi

echo ""
echo "Summary:"
echo "  • Total automated tests: $TOTAL_TESTS"

if [ -n "$COVERAGE_PERCENT" ]; then
    echo "  • Test coverage: ${COVERAGE_PERCENT}%"
fi

if [ "$BENCHMARKS_PASSED" = true ] && [ -n "$BENCHMARK_RESULT" ]; then
    echo "  • Benchmark result: $BENCHMARK_RESULT"
fi

# Recommendations section
echo ""
echo "Recommendations:"

RECOMMENDATIONS_SHOWN=false

if [ "$E2E_TESTS_SKIPPED" = true ]; then
    echo "  • Grant Accessibility permission to run E2E tests:"
    echo "    System Settings > Privacy & Security > Accessibility"
    RECOMMENDATIONS_SHOWN=true
fi

if [ -n "$COVERAGE_PERCENT" ]; then
    COVERAGE_INT=$(echo "$COVERAGE_PERCENT" | cut -d'.' -f1)
    if [ "$COVERAGE_INT" -lt 80 ]; then
        echo "  • Increase test coverage to meet 80% threshold"
        RECOMMENDATIONS_SHOWN=true
    fi
fi

if [ "$BENCHMARKS_PASSED" = false ]; then
    echo "  • Fix benchmark failures or install required dependencies"
    RECOMMENDATIONS_SHOWN=true
fi

if [ -t 0 ] && [ "$HAS_PERMISSION" = true ] && [ "$MANUAL_TESTS_RUN" = false ]; then
    echo "  • Run manual tests to verify user-facing behavior"
    echo "    Re-run with 'y' when prompted for manual testing"
    RECOMMENDATIONS_SHOWN=true
fi

if [ "$RECOMMENDATIONS_SHOWN" = false ]; then
    echo "  • None - all tests passed successfully!"
fi

if [ -n "$COVERAGE_PERCENT" ]; then
    COVERAGE_INT=$(echo "$COVERAGE_PERCENT" | cut -d'.' -f1)
    if [ "$COVERAGE_INT" -ge 90 ]; then
        echo "  • Excellent coverage! Consider this a benchmark for other modules."
    fi
fi

echo ""

if [ $OVERALL_EXIT_CODE -eq 0 ]; then
    print_section "✅ Test suite completed successfully"
else
    print_section "❌ Test suite completed with failures"
fi

echo ""
echo "For more details, see: docs/development/MACOS_TESTING_GUIDE.md"
echo ""

exit $OVERALL_EXIT_CODE
