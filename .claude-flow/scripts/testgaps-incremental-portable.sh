#!/usr/bin/env bash
# Portable Incremental Test Coverage Analysis Worker
# No external dependencies (jq-free)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STATE_DIR="$SCRIPT_DIR/../state"
RESULTS_DIR="$SCRIPT_DIR/../metrics"
STATE_FILE="$STATE_DIR/testgaps-progress.txt"
RESULTS_FILE="$RESULTS_DIR/testgaps-results.jsonl"

# Ensure directories exist
mkdir -p "$STATE_DIR" "$RESULTS_DIR"

# Crates to analyze (in priority order)
CRATES=(
    "keyrx_core"
    "keyrx_daemon"
    "keyrx_compiler"
)

# Get timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u)

# Read state
if [ ! -f "$STATE_FILE" ]; then
    echo "0" > "$STATE_FILE"
fi
CURRENT_INDEX=$(cat "$STATE_FILE")

# Wrap around if we've finished all crates
if [ "$CURRENT_INDEX" -ge "${#CRATES[@]}" ]; then
    CURRENT_INDEX=0
fi

CURRENT_CRATE="${CRATES[$CURRENT_INDEX]}"

echo "Analyzing test coverage for: $CURRENT_CRATE (${CURRENT_INDEX}/${#CRATES[@]})"

# Change to project root
cd "$PROJECT_ROOT"

# Initialize metrics
GAP_COUNT=0
GAPS=""

# Check if crate exists
if [ ! -d "$CURRENT_CRATE" ]; then
    echo "Warning: Crate $CURRENT_CRATE not found, skipping"
    NEXT_INDEX=$((CURRENT_INDEX + 1))
    echo "$NEXT_INDEX" > "$STATE_FILE"
    echo "{\"success\": false, \"crate\": \"$CURRENT_CRATE\", \"error\": \"Crate not found\"}"
    exit 0
fi

# 1. Find untested public functions
echo "  Checking for untested public functions..."
PUB_FN_COUNT=$(find "$CURRENT_CRATE/src" -name "*.rs" -exec grep -c "pub fn" {} + 2>/dev/null | awk '{s+=$1} END {print s}' || echo "0")
TEST_FN_COUNT=$(find "$CURRENT_CRATE" -path "*/tests/*.rs" -exec grep -c "#\[test\]" {} + 2>/dev/null | awk '{s+=$1} END {print s}' || echo "0")

if [ "$PUB_FN_COUNT" -gt 0 ] && [ "$TEST_FN_COUNT" -lt "$((PUB_FN_COUNT / 2))" ]; then
    GAPS="${GAPS}- Untested Functions: $PUB_FN_COUNT public functions but only $TEST_FN_COUNT tests\n"
    GAP_COUNT=$((GAP_COUNT + 1))
fi

# 2. Check for error handling test gaps
echo "  Checking error handling coverage..."
RESULT_COUNT=$(grep -r "Result<" "$CURRENT_CRATE/src" 2>/dev/null | wc -l | tr -d ' ')
ERROR_TEST_COUNT=$(grep -r "should_panic\|expect_err\|is_err" "$CURRENT_CRATE" 2>/dev/null | wc -l | tr -d ' ')

if [ "$RESULT_COUNT" -gt 0 ] && [ "$ERROR_TEST_COUNT" -lt "$((RESULT_COUNT / 3))" ]; then
    GAPS="${GAPS}- Error Handling Tests: $RESULT_COUNT Result types but only $ERROR_TEST_COUNT error tests\n"
    GAP_COUNT=$((GAP_COUNT + 1))
fi

# 3. Check for integration tests
echo "  Checking integration test coverage..."
INTEGRATION_TESTS=$(find "$CURRENT_CRATE/tests" -name "*.rs" 2>/dev/null | wc -l | tr -d ' ')
if [ "$INTEGRATION_TESTS" -lt 3 ]; then
    GAPS="${GAPS}- Integration Tests: Only $INTEGRATION_TESTS integration test files found\n"
    GAP_COUNT=$((GAP_COUNT + 1))
fi

# 4. Check for doc tests
echo "  Checking documentation test coverage..."
DOC_TESTS=$(grep -r "```rust\|```" "$CURRENT_CRATE/src" 2>/dev/null | wc -l | tr -d ' ')
if [ "$DOC_TESTS" -lt 5 ]; then
    GAPS="${GAPS}- Documentation Tests: Only $DOC_TESTS doc tests found\n"
    GAP_COUNT=$((GAP_COUNT + 1))
fi

# 5. Estimate coverage (simple heuristic)
SOURCE_LINES=$(find "$CURRENT_CRATE/src" -name "*.rs" -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')
TEST_LINES=$(find "$CURRENT_CRATE/tests" -name "*.rs" -exec cat {} + 2>/dev/null | wc -l | tr -d ' ')

if [ "$SOURCE_LINES" -gt 0 ]; then
    COVERAGE=$((TEST_LINES * 100 / SOURCE_LINES))
else
    COVERAGE=0
fi

# Write results as JSONL
cat >> "$RESULTS_FILE" <<EOF
{"timestamp":"$TIMESTAMP","crate":"$CURRENT_CRATE","gapCount":$GAP_COUNT,"coverage":$COVERAGE,"pubFunctions":$PUB_FN_COUNT,"tests":$TEST_FN_COUNT}
EOF

# Update state to next crate
NEXT_INDEX=$((CURRENT_INDEX + 1))
echo "$NEXT_INDEX" > "$STATE_FILE"

# Output summary
echo "✓ Analysis complete: $CURRENT_CRATE"
echo "  Estimated coverage: ~${COVERAGE}% (test lines / source lines)"
echo "  Found $GAP_COUNT test gaps"
if [ -n "$GAPS" ]; then
    echo ""
    echo "Gaps identified:"
    echo -e "$GAPS"
fi

# JSON output for worker system
cat <<EOF
{
  "success": true,
  "crate": "$CURRENT_CRATE",
  "crateIndex": $CURRENT_INDEX,
  "totalCrates": ${#CRATES[@]},
  "gapCount": $GAP_COUNT,
  "estimatedCoverage": $COVERAGE,
  "metrics": {
    "publicFunctions": $PUB_FN_COUNT,
    "testFunctions": $TEST_FN_COUNT,
    "integrationTests": $INTEGRATION_TESTS,
    "docTests": $DOC_TESTS
  },
  "message": "Analyzed $CURRENT_CRATE: ~${COVERAGE}% coverage, found $GAP_COUNT gaps"
}
EOF

exit 0
