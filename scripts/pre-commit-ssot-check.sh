#!/usr/bin/env bash
#
# SSOT Violation Check Script
# Runs the SSOT checker and reports violations with actionable errors
# Exit codes: 0 = success, 1 = violations found or error
#
# Usage:
#   ./scripts/pre-commit-ssot-check.sh
#   ./scripts/pre-commit-ssot-check.sh --verbose
#

set -e

# Determine repository root
find_repo_root() {
    local dir="$1"
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/.git" ]]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(find_repo_root "$SCRIPT_DIR")"

if [[ -z "$REPO_ROOT" ]]; then
    echo "Error: Could not find repository root" >&2
    exit 1
fi

SSOT_CHECKER_DIR="$REPO_ROOT/tools/ssot_checker"

# Check if SSOT checker exists
if [[ ! -d "$SSOT_CHECKER_DIR" ]]; then
    echo "Error: SSOT checker not found at $SSOT_CHECKER_DIR" >&2
    exit 1
fi

# Check if Dart is available
if ! command -v dart >/dev/null 2>&1; then
    echo "Error: Dart is not installed or not in PATH" >&2
    echo "Install Flutter/Dart from: https://docs.flutter.dev/get-started/install" >&2
    exit 1
fi

# Parse arguments
VERBOSE_FLAG=""
if [[ "$1" == "--verbose" ]] || [[ "$1" == "-v" ]]; then
    VERBOSE_FLAG="--verbose"
fi

# Run SSOT checker from tools directory
cd "$SSOT_CHECKER_DIR"

# Run the checker with project root path
if [[ -n "$VERBOSE_FLAG" ]]; then
    dart run bin/check.dart --path "$REPO_ROOT" --verbose
else
    dart run bin/check.dart --path "$REPO_ROOT"
fi

# Exit code is propagated from dart run command
