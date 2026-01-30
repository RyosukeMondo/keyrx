#!/usr/bin/env bash
# Incremental Performance Optimization Worker
# Analyzes codebase in chunks to avoid timeouts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STATE_FILE="$SCRIPT_DIR/../state/optimize-progress.json"
RESULTS_FILE="$SCRIPT_DIR/../metrics/optimize-results.json"

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
  "completedModules": [],
  "currentModule": "",
  "startedAt": "$(get_timestamp)"
}
EOF
fi

# Define modules to analyze (in priority order)
MODULES=(
    "keyrx_core"
    "keyrx_daemon"
    "keyrx_compiler"
    "keyrx_ui/src"
)

# Get next module to analyze
CURRENT_MODULE=""
for module in "${MODULES[@]}"; do
    if ! jq -e ".completedModules | index(\"$module\")" "$STATE_FILE" > /dev/null 2>&1; then
        CURRENT_MODULE="$module"
        break
    fi
done

# If all modules done, reset and start over
if [ -z "$CURRENT_MODULE" ]; then
    echo "All modules analyzed, resetting cycle"
    jq '.completedModules = []' "$STATE_FILE" > "$STATE_FILE.tmp"
    mv "$STATE_FILE.tmp" "$STATE_FILE"
    CURRENT_MODULE="${MODULES[0]}"
fi

echo "Analyzing module: $CURRENT_MODULE"

# Update state
jq ".currentModule = \"$CURRENT_MODULE\" | .lastAnalyzed = \"$(get_timestamp)\"" "$STATE_FILE" > "$STATE_FILE.tmp"
mv "$STATE_FILE.tmp" "$STATE_FILE"

# Run focused analysis on this module only
ANALYSIS_OUTPUT=$(cat <<EOF
{
  "module": "$CURRENT_MODULE",
  "timestamp": "$(get_timestamp)",
  "suggestions": []
}
EOF
)

# Change to project root for analysis
cd "$PROJECT_ROOT"

# Check for common performance issues in this module
echo "Checking for performance issues in $CURRENT_MODULE..."

# 1. Check for N+1 queries (database access patterns)
if [ -d "$CURRENT_MODULE" ]; then
    # Look for repeated database calls in loops
    REPEATED_QUERIES=$(grep -rn "\.query\|\.execute" "$CURRENT_MODULE" 2>/dev/null | grep -c "for\|while" || true)
    if [ "$REPEATED_QUERIES" -gt 0 ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.suggestions += [{
            "type": "n+1-query",
            "severity": "high",
            "count": '"$REPEATED_QUERIES"',
            "suggestion": "Found potential N+1 query patterns. Consider using batch queries or joins."
        }]')
    fi

    # 2. Check for unnecessary clones
    CLONE_COUNT=$(grep -rn "\.clone()" "$CURRENT_MODULE" 2>/dev/null | wc -l || true)
    if [ "$CLONE_COUNT" -gt 50 ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.suggestions += [{
            "type": "excessive-clones",
            "severity": "medium",
            "count": '"$CLONE_COUNT"',
            "suggestion": "Found '"$CLONE_COUNT"' clone() calls. Review for unnecessary memory allocations."
        }]')
    fi

    # 3. Check for string allocations in hot paths
    STRING_ALLOC=$(grep -rn "String::from\|to_string()" "$CURRENT_MODULE" 2>/dev/null | wc -l || true)
    if [ "$STRING_ALLOC" -gt 100 ]; then
        ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.suggestions += [{
            "type": "string-allocations",
            "severity": "low",
            "count": '"$STRING_ALLOC"',
            "suggestion": "Consider using &str or Cow<str> for frequently allocated strings."
        }]')
    fi

    # 4. React-specific checks (if UI module)
    if [[ "$CURRENT_MODULE" == *"ui"* ]]; then
        # Check for missing useCallback/useMemo
        MISSING_MEMO=$(grep -rn "function\|const.*=" "$CURRENT_MODULE" 2>/dev/null | grep -v "useCallback\|useMemo" | wc -l || true)
        if [ "$MISSING_MEMO" -gt 20 ]; then
            ANALYSIS_OUTPUT=$(echo "$ANALYSIS_OUTPUT" | jq '.suggestions += [{
                "type": "missing-memoization",
                "severity": "medium",
                "suggestion": "Consider using useCallback/useMemo for expensive computations and handlers."
            }]')
        fi
    fi
fi

# Append to results file
if [ ! -f "$RESULTS_FILE" ]; then
    echo "[]" > "$RESULTS_FILE"
fi

jq ". += [$ANALYSIS_OUTPUT]" "$RESULTS_FILE" > "$RESULTS_FILE.tmp"
mv "$RESULTS_FILE.tmp" "$RESULTS_FILE"

# Mark module as complete
jq ".completedModules += [\"$CURRENT_MODULE\"]" "$STATE_FILE" > "$STATE_FILE.tmp"
mv "$STATE_FILE.tmp" "$STATE_FILE"

# Output summary
SUGGESTION_COUNT=$(echo "$ANALYSIS_OUTPUT" | jq '.suggestions | length')
echo "✓ Analysis complete: $CURRENT_MODULE"
echo "  Found $SUGGESTION_COUNT optimization suggestions"

# Output for worker system
cat <<EOF
{
  "success": true,
  "module": "$CURRENT_MODULE",
  "suggestionCount": $SUGGESTION_COUNT,
  "message": "Analyzed $CURRENT_MODULE: found $SUGGESTION_COUNT optimization opportunities"
}
EOF

exit 0
