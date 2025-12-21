#!/usr/bin/env bats
# Unit tests for scripts/verify.sh

# Get the directory where the tests are located
BATS_TEST_DIRNAME="$(cd "$(dirname "$BATS_TEST_FILENAME")" && pwd)"
PROJECT_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
VERIFY_SCRIPT="$PROJECT_ROOT/scripts/verify.sh"

# Setup function - runs before each test
setup() {
    # Create a temporary directory for test artifacts
    TEST_TEMP_DIR="$(mktemp -d)"
    export TEST_LOG_FILE="$TEST_TEMP_DIR/test_verify.log"
}

# Teardown function - runs after each test
teardown() {
    # Clean up temporary directory
    if [[ -n "$TEST_TEMP_DIR" ]] && [[ -d "$TEST_TEMP_DIR" ]]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}

# Test: Script exists and is executable
@test "verify.sh exists and is executable" {
    [[ -f "$VERIFY_SCRIPT" ]]
    [[ -x "$VERIFY_SCRIPT" ]]
}

# Test: Help flag displays usage information
@test "verify.sh --help displays usage information" {
    run "$VERIFY_SCRIPT" --help
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "Usage:" ]]
    [[ "$output" =~ "--skip-coverage" ]]
    [[ "$output" =~ "--json" ]]
    [[ "$output" =~ "CHECKS PERFORMED" ]]
}

# Test: All checks pass on clean code
@test "verify.sh succeeds on clean codebase" {
    # Skip this test if it takes too long in CI
    skip "This is a slow integration test - run manually"

    run timeout 300 "$VERIFY_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should complete successfully (exit 0)
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]
}

# Test: Skip coverage flag works
@test "verify.sh --skip-coverage skips coverage check" {
    # This is faster than full verification
    run timeout 180 "$VERIFY_SCRIPT" --skip-coverage --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should complete successfully
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]
}

# Test: JSON output mode
@test "verify.sh --json produces valid JSON output" {
    run timeout 180 "$VERIFY_SCRIPT" --skip-coverage --json --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"

    # Should complete successfully
    [[ "$status" -eq 0 ]]

    # Extract the JSON line (last line that starts with {)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Validate JSON structure using jq
    echo "$json_output" | jq -e '.status' > /dev/null
    echo "$json_output" | jq -e '.checks' > /dev/null
    echo "$json_output" | jq -e '.exit_code' > /dev/null

    # Check values
    [[ "$(echo "$json_output" | jq -r '.status')" == "success" ]]
    [[ "$(echo "$json_output" | jq -r '.exit_code')" == "0" ]]
}

# Test: JSON output contains check results
@test "verify.sh --json includes all check results" {
    run timeout 180 "$VERIFY_SCRIPT" --skip-coverage --json --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"

    # Extract the JSON line (last line that starts with {)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Verify checks object contains expected fields
    echo "$json_output" | jq -e '.checks.build' > /dev/null
    echo "$json_output" | jq -e '.checks.clippy' > /dev/null
    echo "$json_output" | jq -e '.checks.fmt' > /dev/null
    echo "$json_output" | jq -e '.checks.test' > /dev/null

    # Check that checks have PASS status
    [[ "$(echo "$json_output" | jq -r '.checks.build')" == "PASS" ]]
}

# Test: Log file creation
@test "verify.sh creates log file when --log-file is specified" {
    run timeout 180 "$VERIFY_SCRIPT" --skip-coverage --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]
    [[ -f "$TEST_LOG_FILE" ]]

    # Verify log file has content
    [[ -s "$TEST_LOG_FILE" ]]
}

# Test: Invalid option handling
@test "verify.sh fails with invalid option" {
    run "$VERIFY_SCRIPT" --invalid-option
    [[ "$status" -ne 0 ]]
    [[ "$output" =~ "Unknown option" ]]
    [[ "$output" =~ "Usage:" ]]
}

# Test: Cargo dependency check
@test "verify.sh checks for cargo availability" {
    # Verify the script checks for cargo
    run grep -q "require_tool.*cargo" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Verification runs checks in order
@test "verify.sh script defines check order" {
    # Verify CHECK_ORDER array exists and contains expected checks
    run grep -q 'CHECK_ORDER=.*build.*clippy.*fmt.*test.*coverage' "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Coverage tool check
@test "verify.sh checks for cargo-tarpaulin when not skipping coverage" {
    # Verify the script checks for cargo-tarpaulin
    run grep -q "require_tool.*cargo-tarpaulin" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Quiet mode suppresses output
@test "verify.sh --quiet suppresses verbose output" {
    run timeout 180 "$VERIFY_SCRIPT" --skip-coverage --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]

    # In quiet mode, should only see status markers
    [[ "$output" =~ "=== accomplished ===" ]]

    # Should NOT see detailed log_info messages (they go to log file only)
    ! [[ "$output" =~ "Running build check" ]]
}

# Test: Build check function exists
@test "verify.sh defines check_build function" {
    run grep -q "^check_build()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Clippy check function exists
@test "verify.sh defines check_clippy function" {
    run grep -q "^check_clippy()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Format check function exists
@test "verify.sh defines check_fmt function" {
    run grep -q "^check_fmt()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Test check function exists
@test "verify.sh defines check_test function" {
    run grep -q "^check_test()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Coverage check function exists
@test "verify.sh defines check_coverage function" {
    run grep -q "^check_coverage()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Summary table function exists
@test "verify.sh defines print_summary function" {
    run grep -q "^print_summary()" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Clippy runs with -D warnings flag
@test "verify.sh runs clippy with warnings as errors" {
    run grep -q "clippy.*--.*-D warnings" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Format check runs with --check flag
@test "verify.sh runs format check without modifying files" {
    run grep -q "cargo fmt --check" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Coverage enforces 80% minimum
@test "verify.sh enforces 80% minimum coverage" {
    run grep -q "80" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]

    # More specifically, check for the coverage comparison
    run grep -q "coverage_pct >= 80.0" "$VERIFY_SCRIPT"
    [[ "$status" -eq 0 ]]
}
