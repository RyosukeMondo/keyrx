#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/log_ffi_implementation.sh --task-id <id> [--title "summary"] [--output <path>] [--base-ref <git ref>] [--template <path>]

Creates a new FFI implementation log entry using the spec template and pre-populates
metadata (timestamp, log id, git status, diff stat). Designed to be fast (<2 min)
and AI-parseable.

Examples:
  scripts/log_ffi_implementation.sh --task-id 3 --title "Logging infra"
  scripts/log_ffi_implementation.sh --task-id 8 --base-ref main
EOF
}

TASK_ID=""
TITLE=""
OUTPUT=""
BASE_REF=""
TEMPLATE_OVERRIDE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --task-id)
            TASK_ID="$2"
            shift 2
            ;;
        --title)
            TITLE="$2"
            shift 2
            ;;
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --base-ref)
            BASE_REF="$2"
            shift 2
            ;;
        --template)
            TEMPLATE_OVERRIDE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR/.." rev-parse --show-toplevel 2>/dev/null || (cd "$SCRIPT_DIR/.." && pwd))"
SPEC_ROOT="${REPO_ROOT}/.spec-workflow/specs/rust-first-ai-native-ffi"
LOG_DIR="${SPEC_ROOT}/Implementation Logs"
DEFAULT_TEMPLATE="${REPO_ROOT}/.spec-workflow/templates/ffi-implementation-log-template.md"
TEMPLATE="${TEMPLATE_OVERRIDE:-$DEFAULT_TEMPLATE}"

if [[ -z "$TASK_ID" ]]; then
    echo "Error: --task-id is required (matches tasks.md entry)" >&2
    usage
    exit 1
fi

if [[ ! -f "$TEMPLATE" ]]; then
    echo "Error: Template not found at $TEMPLATE" >&2
    exit 1
fi

mkdir -p "$LOG_DIR"

TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
LOG_ID="$(python3 - <<'PY'
import uuid
print(uuid.uuid4())
PY
)"

SAFE_TASK="$(echo "${TASK_ID}" | tr -cs '[:alnum:]' '-' | sed 's/^-*//;s/-*$//' )"
SAFE_TASK="${SAFE_TASK:-manual}"

if [[ -n "$OUTPUT" ]]; then
    OUTPUT_PATH="$OUTPUT"
else
    OUTPUT_PATH="${LOG_DIR}/task-${SAFE_TASK}_${TIMESTAMP//:/-}_${LOG_ID%%-*}.md"
fi

FILES_CHANGED="$(git -C "$REPO_ROOT" status --short 2>/dev/null || true)"
if [[ -z "$FILES_CHANGED" ]]; then
    FILES_CHANGED="(clean working tree)"
fi

DIFF_CMD=(git -C "$REPO_ROOT" diff --stat)
if [[ -n "$BASE_REF" ]]; then
    DIFF_CMD=(git -C "$REPO_ROOT" diff "$BASE_REF"... --stat)
fi

if DIFF_OUTPUT="$("${DIFF_CMD[@]}" 2>/dev/null)"; then
    :
else
    DIFF_OUTPUT="(diff unavailable)"
fi
if [[ -z "$DIFF_OUTPUT" ]]; then
    DIFF_OUTPUT="(no diff)"
fi

export TASK_ID TIMESTAMP LOG_ID FILES_CHANGED DIFF_OUTPUT

python3 - "$TEMPLATE" "$OUTPUT_PATH" <<'PY'
import os
import pathlib
import sys

template_path = pathlib.Path(sys.argv[1])
output_path = pathlib.Path(sys.argv[2])

content = template_path.read_text()
replacements = {
    "TASK_ID": os.environ.get("TASK_ID", "TBD"),
    "TIMESTAMP": os.environ.get("TIMESTAMP", ""),
    "LOG_ID": os.environ.get("LOG_ID", ""),
    "FILES_CHANGED": os.environ.get("FILES_CHANGED", "").strip(),
    "GIT_DIFF_STAT": os.environ.get("DIFF_OUTPUT", "").strip(),
}

for key, value in replacements.items():
    content = content.replace(f"{{{{{key}}}}}", value or "(not provided)")

output_path.write_text(content)
print(output_path)
PY

echo "Created log entry at: $OUTPUT_PATH"
echo "Next: open the file, fill remaining sections, and commit/push when ready."
if [[ -n "$TITLE" ]]; then
    echo "Title hint: $TITLE"
fi
