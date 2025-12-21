#!/usr/bin/env bats
# Unit tests for scripts/build.sh

# Get the directory where the tests are located
BATS_TEST_DIRNAME="$(cd "$(dirname "$BATS_TEST_FILENAME")" && pwd)"
PROJECT_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
BUILD_SCRIPT="$PROJECT_ROOT/scripts/build.sh"

# Setup function - runs before each test
setup() {
    # Create a temporary directory for test artifacts
    TEST_TEMP_DIR="$(mktemp -d)"
    export TEST_LOG_FILE="$TEST_TEMP_DIR/test_build.log"
}

# Teardown function - runs after each test
teardown() {
    # Clean up temporary directory
    if [[ -n "$TEST_TEMP_DIR" ]] && [[ -d "$TEST_TEMP_DIR" ]]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}

# Test: Script exists and is executable
@test "build.sh exists and is executable" {
    [[ -f "$BUILD_SCRIPT" ]]
    [[ -x "$BUILD_SCRIPT" ]]
}

# Test: Help flag displays usage information
@test "build.sh --help displays usage information" {
    run "$BUILD_SCRIPT" --help
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "Usage:" ]]
    [[ "$output" =~ "--release" ]]
    [[ "$output" =~ "--watch" ]]
    [[ "$output" =~ "--json" ]]
}

# Test: Successful build (debug mode)
@test "build.sh succeeds with debug build" {
    run "$BUILD_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]
}

# Test: Successful build (release mode)
@test "build.sh --release succeeds with release build" {
    run "$BUILD_SCRIPT" --release --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"
    [[ "$status" -eq 0 ]]
    [[ "$output" =~ "=== accomplished ===" ]]
}

# Test: JSON output mode
@test "build.sh --json produces valid JSON output" {
    run "$BUILD_SCRIPT" --json --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"
    [[ "$status" -eq 0 ]]

    # Extract the JSON line (last line that starts with {)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Validate JSON structure using jq
    echo "$json_output" | jq -e '.status' > /dev/null
    echo "$json_output" | jq -e '.build_type' > /dev/null
    echo "$json_output" | jq -e '.exit_code' > /dev/null

    # Check values
    [[ "$(echo "$json_output" | jq -r '.status')" == "success" ]]
    [[ "$(echo "$json_output" | jq -r '.build_type')" == "debug" ]]
    [[ "$(echo "$json_output" | jq -r '.exit_code')" == "0" ]]
}

# Test: JSON output with release flag
@test "build.sh --release --json shows release in JSON" {
    run "$BUILD_SCRIPT" --release --json --quiet --log-file "$TEST_LOG_FILE"
    echo "Exit code: $status"
    echo "Output: $output"
    [[ "$status" -eq 0 ]]

    # Extract the JSON line (last line that starts with {)
    json_output=$(echo "$output" | grep -o '{.*}' | tail -n1)
    echo "JSON: $json_output"

    # Validate build_type is "release"
    [[ "$(echo "$json_output" | jq -r '.build_type')" == "release" ]]
}

# Test: Log file creation
@test "build.sh creates log file when --log-file is specified" {
    run "$BUILD_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]
    [[ -f "$TEST_LOG_FILE" ]]

    # Verify log file has content
    [[ -s "$TEST_LOG_FILE" ]]
}

# Test: Invalid option handling
@test "build.sh fails with invalid option" {
    run "$BUILD_SCRIPT" --invalid-option
    [[ "$status" -ne 0 ]]
    [[ "$output" =~ "Unknown option" ]]
    [[ "$output" =~ "Usage:" ]]
}

# Test: Cargo dependency check
@test "build.sh checks for cargo availability" {
    # This test verifies the script handles missing cargo
    # We can't actually uninstall cargo, so we'll just verify the check exists
    # by examining the script content
    run grep -q "require_tool.*cargo" "$BUILD_SCRIPT"
    [[ "$status" -eq 0 ]]
}

# Test: Quiet flag suppresses non-error output
@test "build.sh --quiet suppresses informational messages" {
    run "$BUILD_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]

    # In quiet mode, should only see status markers
    [[ "$output" =~ "=== accomplished ===" ]]

    # Should NOT see log_info messages in stdout (they go to log file only)
    ! [[ "$output" =~ "Building in debug mode" ]]
}

# Test: Build artifacts are created
@test "build.sh creates target directory with artifacts" {
    # Clean target directory first
    rm -rf "$PROJECT_ROOT/target"

    run "$BUILD_SCRIPT" --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]

    # Verify target directory was created
    [[ -d "$PROJECT_ROOT/target" ]]
    [[ -d "$PROJECT_ROOT/target/debug" ]]
}

# Test: Release build creates release artifacts
@test "build.sh --release creates release artifacts" {
    run "$BUILD_SCRIPT" --release --quiet --log-file "$TEST_LOG_FILE"
    [[ "$status" -eq 0 ]]

    # Verify release directory exists
    [[ -d "$PROJECT_ROOT/target/release" ]]
}
