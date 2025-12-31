#!/bin/bash
# Check for new unwrap() calls in production code
# Part of unwrap-remediation spec (Task 16)

set -e

# Count unwraps in production code (exclude tests, benches, and testing directories)
# We use line count instead of -c flag for more reliable results
UNWRAP_COUNT=$(rg '\.unwrap\(\)' --type rust \
    --glob '!tests/' \
    --glob '!**/test_*.rs' \
    --glob '!**/*_test.rs' \
    --glob '!**/*_tests.rs' \
    --glob '!**/benches/' \
    --glob '!**/testing/' \
    2>/dev/null | wc -l || echo "0")

# Maximum allowed (current baseline - will be set after Task 15 documentation)
# For now, using current count as baseline
MAX_UNWRAPS=400

if [ "$UNWRAP_COUNT" -gt "$MAX_UNWRAPS" ]; then
    echo "ERROR: Too many unwrap() calls in production code"
    echo "Found: $UNWRAP_COUNT, Maximum: $MAX_UNWRAPS"
    echo ""
    echo "Files with unwraps:"
    rg '\.unwrap\(\)' --type rust \
        --glob '!tests/' \
        --glob '!**/test_*.rs' \
        --glob '!**/*_test.rs' \
        --glob '!**/*_tests.rs' \
        --glob '!**/benches/' \
        --glob '!**/testing/' \
        --files-with-matches
    echo ""
    echo "Please replace unwraps with proper error handling or add SAFETY comments"
    exit 1
fi

echo "✓ unwrap() count: $UNWRAP_COUNT / $MAX_UNWRAPS (OK)"
exit 0
