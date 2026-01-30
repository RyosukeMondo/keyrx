#!/usr/bin/env bash
# Portable Incremental Performance Optimization Worker
# No external dependencies (jq-free)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STATE_DIR="$SCRIPT_DIR/../state"
RESULTS_DIR="$SCRIPT_DIR/../metrics"
STATE_FILE="$STATE_DIR/optimize-progress.txt"
RESULTS_FILE="$RESULTS_DIR/optimize-results.jsonl"

# Ensure directories exist
mkdir -p "$STATE_DIR" "$RESULTS_DIR"

# Modules to analyze (in priority order)
MODULES=(
    "keyrx_core"
    "keyrx_daemon"
    "keyrx_compiler"
    "keyrx_ui/src"
)

# Get timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u)

# Read state
if [ ! -f "$STATE_FILE" ]; then
    echo "0" > "$STATE_FILE"
fi
CURRENT_INDEX=$(cat "$STATE_FILE")

# Wrap around if we've finished all modules
if [ "$CURRENT_INDEX" -ge "${#MODULES[@]}" ]; then
    CURRENT_INDEX=0
fi

CURRENT_MODULE="${MODULES[$CURRENT_INDEX]}"

echo "Analyzing module: $CURRENT_MODULE (${CURRENT_INDEX}/${#MODULES[@]})"

# Change to project root
cd "$PROJECT_ROOT"

# Initialize results
SUGGESTION_COUNT=0
SUGGESTIONS=""

# Check if module exists
if [ ! -d "$CURRENT_MODULE" ]; then
    echo "Warning: Module $CURRENT_MODULE not found, skipping"
    NEXT_INDEX=$((CURRENT_INDEX + 1))
    echo "$NEXT_INDEX" > "$STATE_FILE"
    echo "{\"success\": false, \"module\": \"$CURRENT_MODULE\", \"error\": \"Module not found\"}"
    exit 0
fi

# 1. Check for N+1 query patterns
echo "  Checking for N+1 query patterns..."
REPEATED_QUERIES=$(grep -r "\.query\|\.execute" "$CURRENT_MODULE" 2>/dev/null | grep -c "for\|while" || echo "0")
if [ "$REPEATED_QUERIES" -gt 0 ]; then
    SUGGESTIONS="${SUGGESTIONS}- N+1 Query Pattern: Found $REPEATED_QUERIES potential instances\n"
    SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
fi

# 2. Check for excessive clones
echo "  Checking for excessive clones..."
CLONE_COUNT=$(grep -r "\.clone()" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')
if [ "$CLONE_COUNT" -gt 50 ]; then
    SUGGESTIONS="${SUGGESTIONS}- Excessive Clones: Found $CLONE_COUNT clone() calls\n"
    SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
fi

# 3. Check for string allocations
echo "  Checking for string allocations..."
STRING_ALLOC=$(grep -r "String::from\|to_string()" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')
if [ "$STRING_ALLOC" -gt 100 ]; then
    SUGGESTIONS="${SUGGESTIONS}- String Allocations: Found $STRING_ALLOC string allocations\n"
    SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
fi

# 4. React-specific checks
if [[ "$CURRENT_MODULE" == *"ui"* ]]; then
    echo "  Checking React optimizations..."

    # Check for missing memoization
    CALLBACKS=$(grep -r "function\|const.*=" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')
    MEMOIZED=$(grep -r "useCallback\|useMemo\|React.memo" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')

    if [ "$CALLBACKS" -gt 20 ] && [ "$MEMOIZED" -lt "$((CALLBACKS / 4))" ]; then
        SUGGESTIONS="${SUGGESTIONS}- React Memoization: Only $MEMOIZED memoized out of $CALLBACKS functions\n"
        SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
    fi

    # Check for inline object/array creation in JSX
    INLINE_OBJECTS=$(grep -r "={\|=\[" "$CURRENT_MODULE" 2>/dev/null | grep -c "\.tsx\|\.jsx" || echo "0")
    if [ "$INLINE_OBJECTS" -gt 50 ]; then
        SUGGESTIONS="${SUGGESTIONS}- Inline Objects in JSX: Found $INLINE_OBJECTS potential re-render triggers\n"
        SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
    fi
fi

# 5. Async/await patterns
echo "  Checking async patterns..."
ASYNC_COUNT=$(grep -r "async fn\|async function" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')
AWAIT_COUNT=$(grep -r "await" "$CURRENT_MODULE" 2>/dev/null | wc -l | tr -d ' ')
if [ "$ASYNC_COUNT" -gt 10 ] && [ "$AWAIT_COUNT" -gt "$((ASYNC_COUNT * 5))" ]; then
    SUGGESTIONS="${SUGGESTIONS}- Potential Sequential Awaits: Consider Promise.all() for parallel operations\n"
    SUGGESTION_COUNT=$((SUGGESTION_COUNT + 1))
fi

# Write results as JSONL (one JSON per line)
cat >> "$RESULTS_FILE" <<EOF
{"timestamp":"$TIMESTAMP","module":"$CURRENT_MODULE","suggestionCount":$SUGGESTION_COUNT,"clones":$CLONE_COUNT,"stringAllocs":$STRING_ALLOC}
EOF

# Update state to next module
NEXT_INDEX=$((CURRENT_INDEX + 1))
echo "$NEXT_INDEX" > "$STATE_FILE"

# Output summary
echo "✓ Analysis complete: $CURRENT_MODULE"
echo "  Found $SUGGESTION_COUNT optimization opportunities"
if [ -n "$SUGGESTIONS" ]; then
    echo ""
    echo "Suggestions:"
    echo -e "$SUGGESTIONS"
fi

# JSON output for worker system
cat <<EOF
{
  "success": true,
  "module": "$CURRENT_MODULE",
  "moduleIndex": $CURRENT_INDEX,
  "totalModules": ${#MODULES[@]},
  "suggestionCount": $SUGGESTION_COUNT,
  "metrics": {
    "clones": $CLONE_COUNT,
    "stringAllocations": $STRING_ALLOC,
    "asyncFunctions": $ASYNC_COUNT
  },
  "message": "Analyzed $CURRENT_MODULE: found $SUGGESTION_COUNT optimization opportunities"
}
EOF

exit 0
