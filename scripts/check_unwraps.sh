#!/bin/bash
# DEPRECATED: This script is deprecated as of error-handling-migration Task 15
#
# Unwrap policy is now enforced by Clippy lints (unwrap_used = "deny") in Cargo.toml.
# Use `cargo clippy --workspace` instead of this script.
#
# This script remains for reference but is no longer part of the verification pipeline.
# See .spec-workflow/specs/error-handling-migration/ for details.

echo "WARNING: check_unwraps.sh is DEPRECATED"
echo "Unwrap policy is now enforced by Clippy lints: unwrap_used = 'deny'"
echo "Run 'cargo clippy --workspace' for unwrap/expect checking"
echo ""

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

# Maximum allowed (current baseline after reducing unwraps)
# Buffer allows for small increases while maintaining quality
MAX_UNWRAPS=410

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
