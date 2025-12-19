#!/bin/bash
# KeyRX Headless E2E Test Script
# Verifies the Dart CLI harness and Rust core integration.

set -e

# Determine script directory to resolve paths correctly
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( dirname "$SCRIPT_DIR" )"
cd "$PROJECT_ROOT"

# Configuration
CLI_DIR="apps/cli_tool"
LIB_PATH="target/debug/libkeyrx_core.so"
TEST_LOG="e2e_test.log"

echo "=== KeyRX Headless E2E Test ==="

# 1. Build Rust core if needed
if [ ! -f "$LIB_PATH" ]; then
    echo "Building Rust core..."
    cargo build -p keyrx_core
fi

# 2. Run multiple commands in a single session
echo "Running multi-command session..."

cat <<EOF > e2e_commands.json
{"command": "version", "id": "t1"}
{"command": "echo", "id": "t2", "payload": "multi-test"}
{"command": "initialize", "id": "t3"}
{"command": "isInitialized", "id": "t4"}
{"command": "listDevices", "id": "t5"}
{"command": "exit", "id": "t6"}
EOF

rm -f "$TEST_LOG"
response=$(cat e2e_commands.json | dart run "$CLI_DIR/bin/keyrx_cli.dart" --lib-path "$LIB_PATH" 2>"$TEST_LOG")

# 3. Verify responses
verify() {
    local expected="$1"
    local name="$2"
    if echo "$response" | grep -q "$expected"; then
        echo "✓ $name PASSED"
    else
        echo "✗ $name FAILED (Expected: $expected)"
        echo "Full response subset:"
        echo "$response" | grep "id"
        exit 1
    fi
}

verify '"id":"t1","result":"0.1.0"' "Version Check"
verify '"id":"t2","result":"multi-test"' "Echo Command"
verify '"id":"t3","result":true' "Initialize Engine"
verify '"id":"t4","result":true' "Verify Initialization (Stateful)"
verify '"id":"t5","result":\[' "List Devices"
verify '"id":"t6","result":"Goodbye"' "Exit Command"

rm e2e_commands.json
echo "=== All Headless E2E Tests Passed ==="