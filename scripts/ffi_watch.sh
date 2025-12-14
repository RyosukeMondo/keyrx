#!/usr/bin/env bash
# Watch helper invoked by cargo-watch to regenerate Dart bindings.
# - Tracks SHA256 of contract JSON files to skip unnecessary generation.
# - Emits human-friendly notifications and never crashes the watcher on failure.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACT_DIR="$REPO_ROOT/core/src/ffi/contracts"
CACHE_DIR="$REPO_ROOT/.cache/ffi_watch"
HASH_FILE="$CACHE_DIR/contracts.sha256"

FORCE_REGEN=0
STRICT_FAILURE=0

print_help() {
    cat <<'EOF'
ffi_watch.sh - incremental Dart binding generator for cargo-watch

Usage: scripts/ffi_watch.sh [--force] [--strict]
  --force   Always run codegen, even if contract hashes are unchanged
  --strict  Exit non-zero on codegen failure (by default errors are logged and treated as non-fatal)
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --force)
            FORCE_REGEN=1
            shift
            ;;
        --strict)
            STRICT_FAILURE=1
            shift
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            print_help
            exit 1
            ;;
    esac
done

mkdir -p "$CACHE_DIR"

log() {
    printf '[ffi-watch] %s\n' "$*"
}

notify_desktop() {
    local title="$1" body="$2" level="${3:-info}"
    if command -v notify-send >/dev/null 2>&1; then
        notify-send --urgency="${level}" "$title" "$body" || true
    elif command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"${body}\" with title \"${title}\"" >/dev/null 2>&1 || true
    fi
}

codegen_command() {
    if [[ -n "${FFI_WATCH_COMMAND:-}" ]]; then
        printf '%s\n' "$FFI_WATCH_COMMAND"
    elif command -v just >/dev/null 2>&1; then
        printf '%s\n' "just gen-dart-bindings"
    else
        printf '%s\n' "cargo run --release --manifest-path \"$REPO_ROOT/core/tools/generate_dart_bindings/Cargo.toml\" -- --verbose"
    fi
}

compute_contract_hash() {
    CONTRACT_DIR="$CONTRACT_DIR" python - <<'PY'
import hashlib, os, sys
root = os.environ.get("CONTRACT_DIR")
if not root or not os.path.isdir(root):
    print("missing")
    sys.exit(0)

h = hashlib.sha256()
has_files = False
for dirpath, _, files in os.walk(root):
    for name in sorted(f for f in files if f.endswith(".json")):
        has_files = True
        path = os.path.join(dirpath, name)
        rel = os.path.relpath(path, root).encode()
        h.update(rel)
        with open(path, "rb") as fh:
            h.update(fh.read())

print(h.hexdigest() if has_files else "empty")
PY
}

run_codegen() {
    local cmd
    cmd="$(codegen_command)"
    log "Running codegen: ${cmd}"
    set +e
    eval "$cmd"
    local status=$?
    set -e
    return $status
}

current_hash="$(compute_contract_hash)"
previous_hash="$(cat "$HASH_FILE" 2>/dev/null || echo "")"

if [[ $FORCE_REGEN -ne 1 && "$current_hash" == "$previous_hash" ]]; then
    log "Contracts unchanged (hash: ${current_hash}). Skipping code generation."
    exit 0
fi

log "Change detected (prev: ${previous_hash:-none}, new: ${current_hash}). Triggering Dart binding generation..."
if run_codegen; then
    echo "$current_hash" > "$HASH_FILE"
    log "Bindings regenerated successfully."
    notify_desktop "FFI watch" "Bindings refreshed (hash: ${current_hash})" "low"
else
    log "Code generation failed. See output above; watcher will continue running."
    notify_desktop "FFI watch failed" "Bindings generation failed. Check terminal output." "critical"
    if [[ $STRICT_FAILURE -eq 1 ]]; then
        exit 1
    fi
fi
