#!/usr/bin/env bash
# Generate an HTML coverage report from Flutter lcov output.
#
# Environment variables:
#   COVERAGE_DIR       - directory where coverage artifacts live (default: coverage)
#   LCOV_FILE          - path to the lcov input file (default: $COVERAGE_DIR/lcov.info)
#   HTML_OUTPUT_DIR    - directory for the generated HTML report (default: $COVERAGE_DIR/html)
#   COLLECT_COVERAGE   - if 1, run collect_coverage.sh first (default: 1)
#   GENHTML_BIN        - override genhtml binary (default: genhtml)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

COVERAGE_DIR="${COVERAGE_DIR:-coverage}"
LCOV_FILE="${LCOV_FILE:-$COVERAGE_DIR/lcov.info}"
HTML_OUTPUT_DIR="${HTML_OUTPUT_DIR:-$COVERAGE_DIR/html}"
COLLECT_COVERAGE="${COLLECT_COVERAGE:-1}"
GENHTML_BIN="${GENHTML_BIN:-genhtml}"

if [[ -z "$HTML_OUTPUT_DIR" || "$HTML_OUTPUT_DIR" == "/" ]]; then
  echo "Refusing to write coverage output to an empty or root directory."
  exit 1
fi

export COVERAGE_DIR LCOV_FILE

if [[ "$COLLECT_COVERAGE" == "1" ]]; then
  "$SCRIPT_DIR/collect_coverage.sh"
else
  cd "$UI_DIR"
  if [[ ! -f "$LCOV_FILE" ]]; then
    echo "Coverage input not found: $LCOV_FILE (set COLLECT_COVERAGE=1 to generate)"
    exit 1
  fi
fi

if ! command -v "$GENHTML_BIN" >/dev/null 2>&1; then
  echo "genhtml not found. Install lcov or set GENHTML_BIN to an available genhtml path."
  exit 1
fi

cd "$UI_DIR"
rm -rf "$HTML_OUTPUT_DIR"
mkdir -p "$HTML_OUTPUT_DIR"

echo "==> Generating HTML coverage report to $HTML_OUTPUT_DIR"
"$GENHTML_BIN" "$LCOV_FILE" \
  --output-directory "$HTML_OUTPUT_DIR" \
  --legend \
  --prefix "$UI_DIR" \
  --branch-coverage

echo "HTML coverage report ready at $HTML_OUTPUT_DIR/index.html"
