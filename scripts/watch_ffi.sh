#!/usr/bin/env bash
#
# Watch Rust FFI sources and regenerate Dart bindings on change.
# Requires: watchexec (cargo install watchexec-cli)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DEBOUNCE_MS="${DEBOUNCE_MS:-500}"
WATCH_COMMAND="${WATCH_COMMAND:-just gen-dart-bindings}"

if ! command -v watchexec >/dev/null 2>&1; then
    echo "Error: watchexec not found. Install via 'cargo install watchexec-cli' or your package manager." >&2
    exit 1
fi

echo "Starting FFI watch (core/src/ffi) with ${DEBOUNCE_MS}ms debounce..."
echo "On change: ${WATCH_COMMAND}"

exec watchexec \
    --clear \
    --restart \
    --debounce "${DEBOUNCE_MS}ms" \
    --watch "$REPO_ROOT/core/src/ffi" \
    --exts rs \
    --env REPO_ROOT="$REPO_ROOT" \
    -- "bash" "-c" "cd \"$REPO_ROOT\" && ${WATCH_COMMAND}"
