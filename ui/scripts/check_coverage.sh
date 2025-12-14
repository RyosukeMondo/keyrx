#!/usr/bin/env bash
# Run Flutter tests with coverage and optionally enforce a minimum threshold.
#
# Environment variables:
#   CONCURRENCY          - test concurrency (default: 4)
#   EXCLUDE_TAGS         - comma-separated test tags to skip (default: slow,integration)
#   TEST_RANDOM_SEED     - seed for test randomization (default: random)
#   COVERAGE_DIR         - directory where coverage output is written (default: coverage)
#   LCOV_FILE            - override path to the generated lcov info file
#   COVERAGE_THRESHOLD   - if set, fail when line coverage is below this percentage

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

COVERAGE_DIR="${COVERAGE_DIR:-coverage}"
LCOV_FILE="${LCOV_FILE:-$COVERAGE_DIR/lcov.info}"
THRESHOLD="${COVERAGE_THRESHOLD:-}"
COLLECT_COVERAGE="${COLLECT_COVERAGE:-1}"

if [[ "$COLLECT_COVERAGE" == "1" ]]; then
    "$SCRIPT_DIR/collect_coverage.sh"
else
    cd "$UI_DIR"
    if [[ ! -f "$LCOV_FILE" ]]; then
        echo "Coverage file not found: $LCOV_FILE (set COLLECT_COVERAGE=1 to generate)"
        exit 1
    fi
fi

if [[ -n "$THRESHOLD" ]]; then
    coverage_stats="$(awk -F',' '/^DA:/{total++; if ($2>0) covered++} END{if (total==0){print "0 0"} else {print covered " " total}}' "$LCOV_FILE")"
    read -r covered total <<<"${coverage_stats:-"0 0"}"
    percent="$(awk -v c="$covered" -v t="$total" 'BEGIN{if (t==0){print 0} else {printf "%.2f", (c/t)*100}}')"
    echo "Line coverage: $percent% ($covered/$total lines)"
    awk -v pct="$percent" -v thr="$THRESHOLD" 'BEGIN{exit !(pct+0 >= thr+0)}' || {
        echo "Coverage ${percent}% is below required threshold ${THRESHOLD}%"
        exit 1
    }
else
    echo "Line coverage collected at $LCOV_FILE (no threshold enforced)."
fi

echo "==> Coverage check complete"
