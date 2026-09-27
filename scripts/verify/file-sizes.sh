#!/bin/bash
# File size gate: max 500 code lines (non-blank, non-comment) per source file.
#
# Files that were already over the limit are listed in file-size-baseline.list
# with their size at the time; they may shrink but never grow, and every new
# file must fit. Shrink the baseline as files are split:
#   scripts/verify/file-sizes.sh --update
set -euo pipefail

LIMIT=500
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="$ROOT/scripts/verify/file-size-baseline.list"
cd "$ROOT"

# Source files subject to the limit (tests, generated code and vendored WASM excluded).
source_files() {
    git ls-files '*.rs' | grep -vE '(^|/)tests/|_tests?\.rs$|/tests\.rs$|/fuzz/|/benches/'
    git ls-files 'keyrx_ui/src/*.ts' 'keyrx_ui/src/*.tsx' |
        grep -vE '\.(test|spec)\.tsx?$|/test/|/__tests__/|generated\.ts$|/wasm/pkg/'
}

code_lines() {
    grep -cvE '^[[:space:]]*($|//|/\*|\*)' "$1" || true
}

measure() {
    source_files | while read -r f; do
        n=$(code_lines "$f")
        if (( n > LIMIT )); then echo "$n $f"; fi
    done | sort -k2
}

if [[ "${1:-}" == "--update" ]]; then
    measure > "$BASELINE"
    echo "baseline: $(wc -l < "$BASELINE") file(s) over $LIMIT lines"
    exit 0
fi

declare -A allowed=()
if [[ -f "$BASELINE" ]]; then
    while read -r n f; do allowed["$f"]=$n; done < "$BASELINE"
fi

status=0
while read -r n f; do
    [[ -z "$f" ]] && continue
    cap=${allowed["$f"]:-$LIMIT}
    if (( n > cap )); then
        if [[ -n "${allowed["$f"]:-}" ]]; then
            echo "FAIL $f: $n code lines (baseline $cap, limit $LIMIT) - it grew; split it"
        else
            echo "FAIL $f: $n code lines (limit $LIMIT)"
        fi
        status=1
    fi
done < <(measure)

if (( status == 0 )); then
    echo "file sizes OK (${#allowed[@]} baselined file(s) over $LIMIT; shrink them)"
fi
exit $status
