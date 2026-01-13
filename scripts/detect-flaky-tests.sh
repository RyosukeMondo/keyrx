#!/usr/bin/env bash
#
# Flaky Test Detector
#
# Analyzes Vitest JSON output to identify tests that pass on retry,
# indicating flakiness. Suggests tests for quarantine.
#
# Usage:
#   ./scripts/detect-flaky-tests.sh [vitest-results.json]
#   npm run test -- --reporter=json --outputFile=test-results.json && ./scripts/detect-flaky-tests.sh test-results.json
#
# Exit codes:
#   0 - No flaky tests detected
#   1 - Flaky tests detected (list printed)
#   2 - Error (invalid input, missing file, etc.)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Input validation
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Missing required argument${NC}"
    echo "Usage: $0 <vitest-results.json>"
    echo ""
    echo "Example:"
    echo "  cd keyrx_ui"
    echo "  npm run test -- --reporter=json --outputFile=test-results.json"
    echo "  ../scripts/detect-flaky-tests.sh test-results.json"
    exit 2
fi

RESULTS_FILE="$1"

# Check if file exists
if [ ! -f "$RESULTS_FILE" ]; then
    echo -e "${RED}Error: File not found: $RESULTS_FILE${NC}"
    exit 2
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}Warning: jq not installed. Install with: sudo apt install jq${NC}"
    echo "Falling back to basic grep/sed parsing..."
    USE_JQ=false
else
    USE_JQ=true
fi

echo -e "${BLUE}=== Flaky Test Detection ===${NC}\n"

# Function to detect flaky tests using jq
detect_with_jq() {
    local results_file="$1"

    # Extract tests that have retry information
    # Vitest JSON format includes test.state and test.retryCount
    local flaky_tests=$(jq -r '
        .testResults[]? |
        select(.assertionResults != null) |
        .assertionResults[] |
        select(.status == "passed" and .retryCount != null and .retryCount > 0) |
        {
            name: .fullName,
            file: .ancestorTitles[0],
            retries: .retryCount,
            duration: .duration
        } |
        @json
    ' "$results_file" 2>/dev/null || echo "[]")

    if [ "$flaky_tests" == "[]" ] || [ -z "$flaky_tests" ]; then
        echo -e "${GREEN}✓ No flaky tests detected${NC}"
        echo ""
        return 0
    fi

    # Parse and display flaky tests
    local count=0
    while IFS= read -r test; do
        if [ -n "$test" ]; then
            count=$((count + 1))
            local name=$(echo "$test" | jq -r '.name')
            local file=$(echo "$test" | jq -r '.file')
            local retries=$(echo "$test" | jq -r '.retries')
            local duration=$(echo "$test" | jq -r '.duration')

            echo -e "${YELLOW}⚠️  Flaky test #$count:${NC}"
            echo "   Test: $name"
            echo "   File: $file"
            echo "   Retries needed: $retries"
            echo "   Duration: ${duration}ms"
            echo ""
        fi
    done <<< "$flaky_tests"

    if [ $count -gt 0 ]; then
        echo -e "${RED}Found $count flaky test(s)${NC}"
        echo ""
        echo "Recommendation:"
        echo "  1. Add these tests to keyrx_ui/tests/quarantine.json"
        echo "  2. Create GitHub issues to track fixes"
        echo "  3. Run quarantined tests separately: npm run test:quarantine"
        echo ""
        return 1
    fi

    return 0
}

# Function to detect flaky tests without jq (basic fallback)
detect_without_jq() {
    local results_file="$1"

    # Look for retry patterns in JSON
    # This is a basic fallback - not as accurate as jq parsing
    if grep -q '"retryCount"' "$results_file" && grep -q '"status":"passed"' "$results_file"; then
        echo -e "${YELLOW}⚠️  Possible flaky tests detected (basic detection)${NC}"
        echo "   Install jq for detailed analysis: sudo apt install jq"
        echo ""

        # Count retry occurrences
        local retry_count=$(grep -o '"retryCount":[1-9]' "$results_file" | wc -l)

        if [ "$retry_count" -gt 0 ]; then
            echo "   Tests with retries: $retry_count"
            echo ""
            echo "Run with jq installed for detailed test names and paths."
            return 1
        fi
    fi

    echo -e "${GREEN}✓ No obvious flaky tests detected${NC}"
    echo ""
    return 0
}

# Detect flaky tests
if [ "$USE_JQ" = true ]; then
    detect_with_jq "$RESULTS_FILE"
    exit_code=$?
else
    detect_without_jq "$RESULTS_FILE"
    exit_code=$?
fi

# Print summary
if [ $exit_code -eq 0 ]; then
    echo -e "${GREEN}Test stability: GOOD${NC}"
else
    echo -e "${RED}Test stability: NEEDS ATTENTION${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Review flaky tests above"
    echo "  2. Investigate root causes (timing, race conditions, async issues)"
    echo "  3. Consider adding to quarantine if fix is non-trivial"
    echo "  4. Track quarantine size: npm run test:quarantine:status"
fi

echo -e "\n${BLUE}===========================${NC}"

exit $exit_code
