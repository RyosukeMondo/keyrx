#!/bin/bash
# Test all feature combinations for build-optimization spec
# This ensures all feature flags work correctly in isolation and combination

set -e  # Exit on first error

echo "====================================="
echo "Testing Feature Combinations"
echo "====================================="
echo ""

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

test_count=0
pass_count=0
fail_count=0

# Function to test a feature combination
test_features() {
    local description="$1"
    local features="$2"
    local build_cmd="cargo build -p keyrx_core"

    test_count=$((test_count + 1))
    echo "[$test_count] Testing: $description"

    if [ -z "$features" ]; then
        build_cmd="$build_cmd --no-default-features"
        echo "    Command: cargo build -p keyrx_core --no-default-features"
    elif [ "$features" = "default" ]; then
        echo "    Command: cargo build -p keyrx_core"
    else
        build_cmd="$build_cmd --no-default-features --features $features"
        echo "    Command: cargo build -p keyrx_core --no-default-features --features $features"
    fi

    # Capture the exit code
    if $build_cmd > /tmp/build_output_$test_count.log 2>&1; then
        echo -e "    ${GREEN}✓ PASS${NC}"
        pass_count=$((pass_count + 1))
    else
        echo -e "    ${RED}✗ FAIL${NC}"
        echo "    Error output:"
        tail -10 /tmp/build_output_$test_count.log | sed 's/^/    /'
        fail_count=$((fail_count + 1))
        return 1
    fi
    echo ""
}

# Clean build to ensure fresh state
echo "Cleaning build artifacts..."
cargo clean
echo ""

# Test 1: Minimal build (no features)
test_features "Minimal build (no features)" ""

# Test 2: Default features
test_features "Default features" "default"

# Test 3: Windows driver only
test_features "Windows driver only" "windows-driver"

# Test 4: Linux driver only
test_features "Linux driver only" "linux-driver"

# Test 5: OpenTelemetry tracing only
test_features "OpenTelemetry tracing only" "otel-tracing"

# Test 6: Windows driver + otel-tracing
test_features "Windows driver + otel-tracing" "windows-driver,otel-tracing"

# Test 7: Linux driver + otel-tracing
test_features "Linux driver + otel-tracing" "linux-driver,otel-tracing"

# Test 8: Both platform drivers (without otel)
test_features "Both platform drivers" "windows-driver,linux-driver"

# Test 9: All features
test_features "All features" "windows-driver,linux-driver,otel-tracing"

# Summary
echo "====================================="
echo "Summary"
echo "====================================="
echo "Total tests: $test_count"
echo -e "Passed: ${GREEN}$pass_count${NC}"
echo -e "Failed: ${RED}$fail_count${NC}"
echo ""

if [ $fail_count -eq 0 ]; then
    echo -e "${GREEN}All feature combinations build successfully!${NC}"
    exit 0
else
    echo -e "${RED}Some feature combinations failed to build.${NC}"
    exit 1
fi
