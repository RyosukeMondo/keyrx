#!/usr/bin/env bash
#
# Test Failure Audit Script
# Runs frontend tests and categorizes failures by root cause
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_FILE="${1:-$PROJECT_ROOT/test-failure-audit.json}"
TEMP_FILE="/tmp/test_output_$$.txt"

# Run tests and capture output
echo "[INFO] Running frontend test suite..."
cd "$PROJECT_ROOT/keyrx_ui"

# Run tests, continue on failure
npm test > "$TEMP_FILE" 2>&1 || true

# Extract summary
echo "[INFO] Analyzing test results..."

# Count passed/failed tests from summary line
SUMMARY_LINE=$(grep "Test Files.*failed.*passed" "$TEMP_FILE" | tail -1 || echo "")
if [ -n "$SUMMARY_LINE" ]; then
    echo "[INFO] Test summary: $SUMMARY_LINE"
fi

# Find test count line (e.g., "Tests  168 failed | 521 passed (758)")
TEST_LINE=$(grep -P "^\s+Tests\s+" "$TEMP_FILE" | tail -1 || echo "Tests  0 failed | 0 passed (0)")
PASSED=$(echo "$TEST_LINE" | grep -oP '\d+(?= passed)' || echo "0")
TOTAL=$(echo "$TEST_LINE" | grep -oP '\d+\)' | grep -oP '\d+' || echo "0")
FAILED=$(echo "$TEST_LINE" | grep -oP '\d+(?= failed)' || echo "0")

if [ "$TOTAL" -eq 0 ]; then
    TOTAL=$((PASSED + FAILED))
fi

if [ "$TOTAL" -gt 0 ]; then
    PASS_RATE=$(echo "scale=2; $PASSED * 100 / $TOTAL" | bc -l)
else
    PASS_RATE="0"
fi

echo "[INFO] Tests: $PASSED passed / $FAILED failed / $TOTAL total"
echo "[INFO] Pass rate: ${PASS_RATE}%"

# Categorize errors
CONTEXT_ERRORS=$(grep -c "No QueryClient set\|useContext\|Provider" "$TEMP_FILE" || echo "0")
WEBSOCKET_ERRORS=$(grep -c "assertIsWebSocket\|WebSocket" "$TEMP_FILE" || echo "0")
ASYNC_ERRORS=$(grep -c "act(\|waitFor\|timeout" "$TEMP_FILE" || echo "0")
DOM_ERRORS=$(grep -c "scrollIntoView\|not a function" "$TEMP_FILE" || echo "0")

echo ""
echo "[INFO] === Failure Categories ==="
echo "Context errors (missing providers):  $CONTEXT_ERRORS occurrences"
echo "WebSocket errors:                     $WEBSOCKET_ERRORS occurrences"
echo "Async timing errors:                  $ASYNC_ERRORS occurrences"
echo "DOM API errors (jsdom limitations):  $DOM_ERRORS occurrences"

# Generate JSON report
echo "[INFO] Generating JSON report..."

cat > "$OUTPUT_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "summary": {
    "total_tests": $TOTAL,
    "passed_tests": $PASSED,
    "failed_tests": $FAILED,
    "pass_rate": $PASS_RATE,
    "target_pass_rate": 95.0,
    "meets_requirement": $([ $(echo "$PASS_RATE >= 95" | bc -l) -eq 1 ] && echo "true" || echo "false")
  },
  "error_categories": {
    "context_errors": $CONTEXT_ERRORS,
    "websocket_errors": $WEBSOCKET_ERRORS,
    "async_errors": $ASYNC_ERRORS,
    "dom_errors": $DOM_ERRORS
  },
  "recommendations": {
    "context_errors": "Fix by wrapping tests with renderWithProviders() from tests/testUtils.tsx",
    "websocket_errors": "Mock WebSocket in test setup or skip WebSocket-dependent tests",
    "async_errors": "Use waitFor() and proper async handling from @testing-library/react",
    "dom_errors": "Mock DOM APIs like scrollIntoView in test setup (src/test/setup.ts)"
  }
}
EOF

echo "[INFO] Report saved to: $OUTPUT_FILE"
echo "[INFO] Audit complete!"

# Clean up
rm -f "$TEMP_FILE"

exit 0
