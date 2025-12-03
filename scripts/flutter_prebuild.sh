#!/usr/bin/env bash
# Flutter pre-build hook to verify FFI bindings are in sync
#
# This script is intended to be run before Flutter builds to ensure
# that Dart FFI bindings match the current Rust exports.
#
# Usage: ./scripts/flutter_prebuild.sh
#
# Exit codes:
#   0 - Bindings verified, build can proceed
#   1 - Bindings out of sync, build should fail

set -e

echo "==================================================================="
echo "Flutter Pre-Build: Verifying FFI Bindings"
echo "==================================================================="

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Change to project root
cd "$PROJECT_ROOT"

# Run verification
if python3 scripts/verify_bindings.py; then
    echo ""
    echo "✓ FFI bindings verified - build can proceed"
    exit 0
else
    echo ""
    echo "✗ FFI bindings are out of sync!"
    echo ""
    echo "To fix this issue:"
    echo "  1. Run: just gen-bindings"
    echo "  2. Commit the updated bindings file"
    echo "  3. Try building again"
    echo ""
    exit 1
fi
