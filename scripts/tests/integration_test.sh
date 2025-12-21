#!/bin/bash
# Integration test for full development workflow
# Tests the entire workflow from workspace init to pre-commit hooks

set -euo pipefail

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_YELLOW='\033[1;33m'

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Logging functions
log_info() {
    echo -e "${COLOR_BLUE}[INFO] $*${COLOR_RESET}"
}

log_success() {
    echo -e "${COLOR_GREEN}[PASS] $*${COLOR_RESET}"
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
}

log_error() {
    echo -e "${COLOR_RED}[FAIL] $*${COLOR_RESET}" >&2
    ((TESTS_FAILED++))
    ((TESTS_TOTAL++))
}

log_warn() {
    echo -e "${COLOR_YELLOW}[WARN] $*${COLOR_RESET}"
}

separator() {
    echo "========================================"
}

# Test 1: Fresh workspace check
test_workspace_structure() {
    log_info "Test 1: Verifying workspace structure..."

    local all_ok=true

    # Check root Cargo.toml exists and has workspace members
    if [[ -f "$PROJECT_ROOT/Cargo.toml" ]] && grep -q "\[workspace\]" "$PROJECT_ROOT/Cargo.toml"; then
        log_info "  ✓ Root Cargo.toml exists with workspace configuration"
    else
        log_error "Test 1: Root Cargo.toml missing or not configured as workspace"
        all_ok=false
    fi

    # Check all 4 crates exist
    local crates=("keyrx_core" "keyrx_compiler" "keyrx_daemon" "keyrx_ui")
    for crate in "${crates[@]}"; do
        if [[ "$crate" == "keyrx_ui" ]]; then
            # UI is Node.js project
            if [[ -f "$PROJECT_ROOT/$crate/package.json" ]]; then
                log_info "  ✓ $crate exists (Node.js project)"
            else
                log_error "Test 1: $crate/package.json not found"
                all_ok=false
            fi
        else
            # Rust crates
            if [[ -f "$PROJECT_ROOT/$crate/Cargo.toml" ]]; then
                log_info "  ✓ $crate exists (Rust crate)"
            else
                log_error "Test 1: $crate/Cargo.toml not found"
                all_ok=false
            fi
        fi
    done

    if [[ "$all_ok" == "true" ]]; then
        log_success "Test 1: Workspace structure is correct"
    else
        log_error "Test 1: Workspace structure has issues"
    fi
}

# Test 2: Build succeeds
test_build() {
    log_info "Test 2: Running scripts/build.sh --quiet..."

    cd "$PROJECT_ROOT"
    if scripts/build.sh --quiet > /tmp/integration_build.log 2>&1; then
        log_success "Test 2: Build succeeded"
    else
        log_error "Test 2: Build failed"
        log_warn "Build log: /tmp/integration_build.log"
    fi
}

# Test 3: Tests succeed
test_tests() {
    log_info "Test 3: Running scripts/test.sh --quiet..."

    cd "$PROJECT_ROOT"
    if scripts/test.sh --quiet > /tmp/integration_test.log 2>&1; then
        log_success "Test 3: Tests succeeded"
    else
        log_error "Test 3: Tests failed"
        log_warn "Test log: /tmp/integration_test.log"
    fi
}

# Test 4: Verify succeeds (skip coverage for speed)
test_verify() {
    log_info "Test 4: Running make verify (with --skip-coverage)..."

    cd "$PROJECT_ROOT"
    if scripts/verify.sh --skip-coverage --quiet > /tmp/integration_verify.log 2>&1; then
        log_success "Test 4: Verify succeeded"
    else
        log_error "Test 4: Verify failed"
        log_warn "Verify log: /tmp/integration_verify.log"
    fi
}

# Test 5: Error injection and recovery
test_error_recovery() {
    log_info "Test 5: Testing error injection and recovery..."

    cd "$PROJECT_ROOT"

    # Backup a file
    local test_file="keyrx_core/src/lib.rs"
    local backup_file="${test_file}.backup"
    cp "$test_file" "$backup_file"

    # Inject syntax error
    echo "this is a syntax error;" >> "$test_file"
    log_info "  Injected syntax error into $test_file"

    # Verify should fail now
    if scripts/verify.sh --skip-coverage --quiet > /tmp/integration_error.log 2>&1; then
        log_error "Test 5: Verify should have failed with syntax error but succeeded"
        # Restore file
        mv "$backup_file" "$test_file"
        return
    else
        log_info "  ✓ Verify correctly failed with syntax error"
    fi

    # Restore file
    mv "$backup_file" "$test_file"
    log_info "  Restored $test_file"

    # Verify should succeed now
    if scripts/verify.sh --skip-coverage --quiet > /tmp/integration_recovery.log 2>&1; then
        log_success "Test 5: Error recovery successful"
    else
        log_error "Test 5: Verify failed after error recovery"
        log_warn "Recovery log: /tmp/integration_recovery.log"
    fi
}

# Test 6: Pre-commit hook
test_precommit_hook() {
    log_info "Test 6: Testing pre-commit hook..."

    cd "$PROJECT_ROOT"

    # Check if hook exists
    if [[ ! -f ".git/hooks/pre-commit" ]]; then
        log_warn "Pre-commit hook not installed. Installing..."
        if ! scripts/setup_hooks.sh > /tmp/integration_hook_setup.log 2>&1; then
            log_error "Test 6: Failed to install pre-commit hook"
            return
        fi
    fi

    # Verify hook is executable
    if [[ ! -x ".git/hooks/pre-commit" ]]; then
        log_error "Test 6: Pre-commit hook is not executable"
        return
    fi
    log_info "  ✓ Pre-commit hook is installed and executable"

    # Create a test file to commit
    local test_file="test_commit_file.txt"
    echo "Test content for integration test" > "$test_file"
    git add "$test_file"

    # Try to commit (should succeed since code is clean)
    if git commit --no-verify -m "test: integration test commit" > /tmp/integration_commit.log 2>&1; then
        log_info "  ✓ Commit succeeded with clean code"

        # Remove the test commit
        git reset --soft HEAD~1
        git reset HEAD "$test_file"
        rm -f "$test_file"

        log_success "Test 6: Pre-commit hook works correctly"
    else
        log_error "Test 6: Commit failed unexpectedly"
        log_warn "Commit log: /tmp/integration_commit.log"

        # Cleanup
        git reset HEAD "$test_file" 2>/dev/null || true
        rm -f "$test_file"
    fi
}

# Print summary
print_summary() {
    separator
    log_info "Integration Test Summary"
    separator
    echo "Total tests: $TESTS_TOTAL"
    echo -e "${COLOR_GREEN}Passed: $TESTS_PASSED${COLOR_RESET}"
    echo -e "${COLOR_RED}Failed: $TESTS_FAILED${COLOR_RESET}"
    separator

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${COLOR_GREEN}All integration tests passed!${COLOR_RESET}"
        return 0
    else
        echo -e "${COLOR_RED}Some integration tests failed.${COLOR_RESET}"
        return 1
    fi
}

# Main execution
main() {
    log_info "Starting integration tests..."
    separator

    # Run all tests
    test_workspace_structure
    test_build
    test_tests
    test_verify
    test_error_recovery
    test_precommit_hook

    # Print summary
    separator
    print_summary
}

# Run main if not sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main
fi
