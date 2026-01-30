#!/usr/bin/env bash
# Incremental Test Coverage Analysis Worker
# Analyzes test coverage gaps in chunks to avoid timeouts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STATE_FILE="$SCRIPT_DIR/../state/testgaps-progress.json"
RESULTS_FILE="$SCRIPT_DIR/../metrics/testgaps-results.json"

# Portable timestamp function
get_timestamp() {
    date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u +"%Y-%m-%dT%H:%M:%S" 2>/dev/null || echo "$(date -u)"
}

# Ensure directories exist
mkdir -p "$(dirname "$STATE_FILE")"
mkdir -p "$(dirname "$RESULTS_FILE")"

# Initialize state if needed
if [ ! -f "$STATE_FILE" ]; then
    cat > "$STATE_FILE" <<EOF
{
  "lastAnalyzed": "",
  "completedCrates": [],
  "currentCrate": "",
  "startedAt": "$(get_timestamp)"
}
EOF
fi

# Define crates to analyze (in priority order: critical paths first)
CRATES=(
    "keyrx_core"
    "keyrx_daemon"
    "keyrx_compiler"
)

# Get next crate to analyze
CURRENT_CRATE=""
for crate in "${CRATES[@]}"; do
    if ! jq -e ".completedCrates | index(\"$crate\")" "$STATE_FILE" > /dev/null 2>&1; then
        CURRENT_CRATE="$crate"
        break
    fi
done

# If all crates done, reset and start over
if [ -z "$CURRENT_CRATE" ]; then
    echo "All crates analyzed, resetting cycle"
    jq '.completedCrates = []' "$STATE_FILE" > "$STATE_FILE.tmp"
    mv "$STATE_FILE.tmp" "$STATE_FILE"
    CURRENT_CRATE="${CRATES[0]}"
fi

echo "Analyzing test coverage for: $CURRENT_CRATE"

# Update state
jq ".currentCrate = \"$CURRENT_CRATE\" | .lastAnalyzed = \"$(get_timestamp)\"" "$STATE_FILE" > "$STATE_FILE.tmp"
mv "$STATE_FILE.tmp" "$STATE_FILE"

# Run coverage analysis for this crate
ANALYSIS_OUTPUT=$(cat <<EOF
{
  "crate": "$CURRENT_CRATE",
  "timestamp": "$(get_timestamp)",
  "gaps": [],
  "coverage": 0
}
EOF
)

# Change to project root for analysis
cd "$PROJECT_ROOT"

# Check if crate exists
if [ ! -d "$CURRENT_CRATE" ]; then
    echo "Warning: Crate $CURRENT_CRATE not found"
    ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.error = "Crate not found"')
else
    echo "Running coverage analysis..."

    # Run cargo tarpaulin for this crate only (with short timeout)
    if command -v cargo &> /dev/null && cargo tarpaulin --version &> /dev/null; then
        COVERAGE_OUTPUT=$(timeout 120s cargo tarpaulin -p "$CURRENT_CRATE" --skip-clean --quiet --out Json 2>/dev/null || echo "{}")

        if [ -n "$COVERAGE_OUTPUT" ] && [ "$COVERAGE_OUTPUT" != "{}" ]; then
            # Extract coverage percentage
            COVERAGE=$(echo "$COVERAGE_OUTPUT" | jq '.coverage // 0' 2>/dev/null || echo "0")
            ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq ".coverage = $COVERAGE")
        fi
    else
        echo "cargo-tarpaulin not available, using basic analysis"
    fi

    # Find untested functions
    echo "Checking for untested code..."

    # 1. Find public functions without tests
    UNTESTED_PUB_FNS=$(find "$CURRENT_CRATE/src" -name "*.rs" -exec grep -l "pub fn" {} \; 2>/dev/null | while read -r file; do
        basename "$file" .rs
    done | sort -u | while read -r module; do
        if ! find "$CURRENT_CRATE/tests" -name "*${module}*.rs" 2>/dev/null | grep -q .; then
            echo "$module"
        fi
    done | wc -l || true)

    if [ "$UNTESTED_PUB_FNS" -gt 0 ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.gaps += [{
            "type": "untested-public-functions",
            "severity": "high",
            "count": '"$UNTESTED_PUB_FNS"',
            "suggestion": "Found '"$UNTESTED_PUB_FNS"' modules with public functions lacking dedicated tests"
        }]')
    fi

    # 2. Check for error handling test gaps
    ERROR_HANDLERS=$(grep -r "Result<" "$CURRENT_CRATE/src" 2>/dev/null | wc -l || true)
    ERROR_TESTS=$(grep -r "#\[should_panic\]\|expect_err\|is_err" "$CURRENT_CRATE/tests" 2>/dev/null | wc -l || true)

    if [ "$ERROR_HANDLERS" -gt 0 ] && [ "$ERROR_TESTS" -lt "$((ERROR_HANDLERS / 2))" ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.gaps += [{
            "type": "missing-error-tests",
            "severity": "high",
            "errorHandlers": '"$ERROR_HANDLERS"',
            "errorTests": '"$ERROR_TESTS"',
            "suggestion": "Only '"$ERROR_TESTS"' error tests for '"$ERROR_HANDLERS"' Result types. Add error path tests."
        }]')
    fi

    # 3. Check for integration test gaps
    INTEGRATION_TESTS=$(find "$CURRENT_CRATE/tests" -name "*.rs" 2>/dev/null | wc -l || true)
    if [ "$INTEGRATION_TESTS" -lt 3 ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.gaps += [{
            "type": "missing-integration-tests",
            "severity": "medium",
            "count": '"$INTEGRATION_TESTS"',
            "suggestion": "Only '"$INTEGRATION_TESTS"' integration test files. Consider adding more end-to-end tests."
        }]')
    fi
fi

# Append to results file
if [ ! -f "$RESULTS_FILE" ]; then
    echo "[]" > "$RESULTS_FILE"
fi

jq ". += [$ANALYSIS_OUTPUT]" "$RESULTS_FILE" > "$RESULTS_FILE.tmp"
mv "$RESULTS_FILE.tmp" "$RESULTS_FILE"

# Mark crate as complete
jq ".completedCrates += [\"$CURRENT_CRATE\"]" "$STATE_FILE" > "$STATE_FILE.tmp"
mv "$STATE_FILE.tmp" "$STATE_FILE"

# Output summary
GAP_COUNT=$(echo "$ANALYSIS_OUTPUT" | jq '.gaps | length')
COVERAGE=$(echo "$ANALYSIS_OUTPUT" | jq '.coverage')
echo "✓ Analysis complete: $CURRENT_CRATE"
echo "  Coverage: ${COVERAGE}%"
echo "  Found $GAP_COUNT test gaps"

# Output for worker system
cat <<EOF
{
  "success": true,
  "crate": "$CURRENT_CRATE",
  "coverage": $COVERAGE,
  "gapCount": $GAP_COUNT,
  "message": "Analyzed $CURRENT_CRATE: ${COVERAGE}% coverage, found $GAP_COUNT gaps"
}
EOF

exit 0
