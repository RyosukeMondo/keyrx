#!/usr/bin/env bash
# Autonomous daemon test runner with auto-fix
#
# This script:
# 1. Pre-checks for common issues
# 2. Runs daemon tests
# 3. Auto-fixes discovered bugs where possible
# 4. Re-runs tests after fixes
# 5. Reports results

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Change to UI directory
cd "$(dirname "$0")/../../ui" || exit 1

ISSUES_FOUND=0
FIXES_APPLIED=0
MAX_ITERATIONS=3
ITERATION=1

# Logging functions
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

# Pre-flight checks
preflight_check() {
  log_info "Running pre-flight checks..."

  # Check if dependencies are installed
  if ! flutter pub get &> /dev/null; then
    log_error "flutter pub get failed"
    return 1
  fi

  # Check if freezed code is generated
  if ! grep -q "daemon_protocol.freezed.dart" lib/models/daemon_protocol.dart 2>/dev/null; then
    log_warning "Freezed imports might be missing, running code generation..."
    if flutter pub run build_runner build --delete-conflicting-outputs &> /dev/null; then
      log_success "Code generation completed"
      ((FIXES_APPLIED++))
    else
      log_error "Code generation failed"
    fi
  fi

  # Check for common import issues
  if grep -r "import 'dart:async'" lib/services/daemon_service.dart &> /dev/null; then
    if grep -q "ConnectionState" lib/services/daemon_service.dart && \
       ! grep -q "DaemonConnectionState" lib/services/daemon_service.dart; then
      log_error "ConnectionState naming conflict detected"
      ((ISSUES_FOUND++))
    fi
  fi

  log_success "Pre-flight checks completed"
}

# Run tests and capture output
run_tests() {
  local test_output
  local exit_code

  log_info "Running daemon tests (iteration $ITERATION/$MAX_ITERATIONS)..."

  test_output=$(flutter test \
    test/services/daemon_service_test.dart \
    test/state/daemon_providers_test.dart \
    test/pages/monitor_daemon_page_test.dart \
    2>&1) || exit_code=$?

  echo "$test_output"

  return ${exit_code:-0}
}

# Analyze test failures and attempt fixes
analyze_and_fix() {
  local test_output="$1"

  log_info "Analyzing test failures..."

  # Check for missing dependencies
  if echo "$test_output" | grep -q "Target of URI doesn't exist"; then
    log_warning "Missing dependencies detected"

    if echo "$test_output" | grep -q "package:intl"; then
      log_info "Adding intl package..."
      # This would be caught in preflight, but defensive check
      ((ISSUES_FOUND++))
    fi

    if echo "$test_output" | grep -q "package:mocktail"; then
      log_info "Adding mocktail package..."
      # Already in dev_dependencies, run pub get
      flutter pub get &> /dev/null
      ((FIXES_APPLIED++))
    fi
  fi

  # Check for import conflicts
  if echo "$test_output" | grep -q "Ambiguous import"; then
    log_error "Ambiguous import detected - manual fix required"
    ((ISSUES_FOUND++))
    return 1
  fi

  # Check for missing generated files
  if echo "$test_output" | grep -q "\.g\.dart.*doesn't exist"; then
    log_warning "Generated files missing, running build_runner..."
    if flutter pub run build_runner build --delete-conflicting-outputs; then
      log_success "Code generation completed"
      ((FIXES_APPLIED++))
      return 0
    else
      log_error "Code generation failed"
      ((ISSUES_FOUND++))
      return 1
    fi
  fi

  # Check for timeout issues
  if echo "$test_output" | grep -q "Test timed out"; then
    log_warning "Test timeout detected - may indicate deadlock or slow test"
    ((ISSUES_FOUND++))
  fi

  # Check for null safety issues
  if echo "$test_output" | grep -q "Null check operator"; then
    log_error "Null safety issue detected - manual fix required"
    ((ISSUES_FOUND++))
    return 1
  fi

  return 0
}

# Generate bug report
generate_bug_report() {
  local test_output="$1"

  log_info "Generating bug report..."

  cat > /tmp/daemon_test_report.txt << EOF
Daemon Test Report
==================

Date: $(date)
Iteration: $ITERATION
Issues Found: $ISSUES_FOUND
Fixes Applied: $FIXES_APPLIED

Test Output:
------------
$test_output

Common Issues Detected:
-----------------------
EOF

  # Analyze for specific issues
  if echo "$test_output" | grep -q "Connection"; then
    echo "- Connection-related failures (check socket mocking)" >> /tmp/daemon_test_report.txt
  fi

  if echo "$test_output" | grep -q "timeout"; then
    echo "- Timeout issues (check async handling)" >> /tmp/daemon_test_report.txt
  fi

  if echo "$test_output" | grep -q "Expected:.*Actual:"; then
    echo "- Assertion failures (check test expectations)" >> /tmp/daemon_test_report.txt
  fi

  log_success "Bug report saved to /tmp/daemon_test_report.txt"
}

# Main execution
main() {
  echo -e "${YELLOW}================================${NC}"
  echo -e "${YELLOW}Autonomous Daemon Test Runner${NC}"
  echo -e "${YELLOW}================================${NC}"
  echo ""

  # Pre-flight checks
  if ! preflight_check; then
    log_error "Pre-flight checks failed"
    exit 1
  fi

  echo ""

  # Test loop with auto-fix
  while [ $ITERATION -le $MAX_ITERATIONS ]; do
    test_output=$(run_tests 2>&1) || test_exit=$?

    if [ ${test_exit:-0} -eq 0 ]; then
      echo ""
      log_success "All daemon tests passed!"

      if [ $FIXES_APPLIED -gt 0 ]; then
        log_info "Applied $FIXES_APPLIED fix(es) during test run"
      fi

      exit 0
    fi

    echo ""
    log_warning "Tests failed on iteration $ITERATION"

    # Analyze and attempt fixes
    if analyze_and_fix "$test_output"; then
      if [ $FIXES_APPLIED -eq 0 ]; then
        # No fixes available, stop trying
        log_error "No automatic fixes available"
        break
      fi

      # Fixes applied, try again
      ((ITERATION++))
      echo ""
      log_info "Fixes applied, retrying..."
      echo ""
    else
      # Analysis failed or manual fix needed
      break
    fi
  done

  # Generate bug report
  echo ""
  generate_bug_report "$test_output"

  echo ""
  log_error "Tests failed after $ITERATION iteration(s)"
  log_info "Review bug report: /tmp/daemon_test_report.txt"

  # Show summary
  echo ""
  echo -e "${YELLOW}Summary:${NC}"
  echo "  Issues Found: $ISSUES_FOUND"
  echo "  Fixes Applied: $FIXES_APPLIED"
  echo "  Iterations: $ITERATION"

  exit 1
}

# Handle Ctrl+C
trap 'echo ""; log_warning "Test run interrupted"; exit 130' INT

# Run main
main "$@"
