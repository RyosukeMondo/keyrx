#!/usr/bin/env bash
# Collect Flutter test coverage and normalize output for reporting.
#
# Environment overrides:
#   CONCURRENCY        - test concurrency (default: 4)
#   EXCLUDE_TAGS       - comma-separated tags to skip (default: slow,integration)
#   TEST_RANDOM_SEED   - seed for test randomization (default: random)
#   TEST_FILES         - test target path/glob (default: test)
#   COVERAGE_DIR       - directory for coverage artifacts (default: coverage)
#   RAW_LCOV           - path for the unfiltered lcov file
#   LCOV_FILE          - path for the filtered lcov file
#   FILTER_PATTERNS    - space-separated lcov filter patterns

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

CONCURRENCY="${CONCURRENCY:-4}"
EXCLUDE_TAGS="${EXCLUDE_TAGS:-slow,integration}"
TEST_RANDOM_SEED="${TEST_RANDOM_SEED:-random}"
TEST_FILES="${TEST_FILES:-test}"
COVERAGE_DIR="${COVERAGE_DIR:-coverage}"
SOURCE_LCOV="${SOURCE_LCOV:-$UI_DIR/coverage/lcov.info}"
RAW_LCOV="${RAW_LCOV:-$COVERAGE_DIR/lcov.info.raw}"
LCOV_FILE="${LCOV_FILE:-$COVERAGE_DIR/lcov.info}"

# Default filters drop generated code and boilerplate from reports.
DEFAULT_FILTERS=(
  '**/*.g.dart'
  '**/*.freezed.dart'
  '**/generated/**'
  '**/*.mocks.dart'
  '**/l10n/*.dart'
)

if [[ -n "${FILTER_PATTERNS:-}" ]]; then
  # Allow space-separated overrides, e.g. FILTER_PATTERNS="**/*.g.dart **/generated/**"
  read -r -a FILTERS <<<"$FILTER_PATTERNS"
else
  FILTERS=("${DEFAULT_FILTERS[@]}")
fi

cd "$UI_DIR"
mkdir -p "$COVERAGE_DIR"
rm -f "$SOURCE_LCOV" "$RAW_LCOV" "$LCOV_FILE"

echo "==> Collecting Flutter test coverage into $COVERAGE_DIR"
flutter test --coverage \
  --concurrency="$CONCURRENCY" \
  --exclude-tags="$EXCLUDE_TAGS" \
  --test-randomize-ordering-seed "$TEST_RANDOM_SEED" \
  $TEST_FILES

if [[ ! -f "$SOURCE_LCOV" ]]; then
  echo "Coverage collection failed: $SOURCE_LCOV not found"
  exit 1
fi

mv "$SOURCE_LCOV" "$RAW_LCOV"

if command -v lcov >/dev/null 2>&1; then
  echo "==> Filtering coverage output (removing generated files)"
  lcov --remove "$RAW_LCOV" "${FILTERS[@]}" --output-file "$LCOV_FILE"
  echo "==> Coverage summary"
  lcov --summary "$LCOV_FILE" || true
else
  echo "lcov not found; keeping raw coverage at $RAW_LCOV"
  cp "$RAW_LCOV" "$LCOV_FILE"
fi

echo "==> Coverage written to $LCOV_FILE"
