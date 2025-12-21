#!/usr/bin/env bash
# Fast CLI test runner for daemon-related tests
#
# Usage:
#   ./scripts/test/test_daemon.sh              # Run all daemon tests
#   ./scripts/test/test_daemon.sh service      # Run only service tests
#   ./scripts/test/test_daemon.sh providers    # Run only provider tests
#   ./scripts/test/test_daemon.sh widget       # Run only widget tests
#   ./scripts/test/test_daemon.sh --watch      # Watch mode
#   ./scripts/test/test_daemon.sh --coverage   # With coverage

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to UI directory
cd "$(dirname "$0")/../../ui" || exit 1

# Parse arguments
MODE="all"
WATCH=false
COVERAGE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    service|providers|widget)
      MODE="$1"
      shift
      ;;
    --watch|-w)
      WATCH=true
      shift
      ;;
    --coverage|-c)
      COVERAGE=true
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [service|providers|widget] [--watch] [--coverage]"
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

# Determine test files to run
TEST_FILES=""
case $MODE in
  service)
    TEST_FILES="test/services/daemon_service_test.dart"
    ;;
  providers)
    TEST_FILES="test/state/daemon_providers_test.dart"
    ;;
  widget)
    TEST_FILES="test/pages/monitor_daemon_page_test.dart"
    ;;
  all)
    TEST_FILES="test/services/daemon_service_test.dart test/state/daemon_providers_test.dart test/pages/monitor_daemon_page_test.dart"
    ;;
esac

# Build test command
TEST_CMD="flutter test"

if [ "$COVERAGE" = true ]; then
  TEST_CMD="$TEST_CMD --coverage"
fi

TEST_CMD="$TEST_CMD $TEST_FILES"

# Print header
echo -e "${YELLOW}================================${NC}"
echo -e "${YELLOW}Daemon Tests - Mode: $MODE${NC}"
echo -e "${YELLOW}================================${NC}"
echo ""

# Run tests
if [ "$WATCH" = true ]; then
  echo -e "${GREEN}Running in watch mode...${NC}"
  echo ""

  # Use watchexec if available, otherwise fall back to simple loop
  if command -v watchexec &> /dev/null; then
    watchexec -w lib -w test -e dart -- $TEST_CMD
  else
    echo -e "${YELLOW}watchexec not found, install with: cargo install watchexec-cli${NC}"
    echo -e "${YELLOW}Falling back to simple loop (Ctrl+C to stop)${NC}"
    echo ""

    while true; do
      clear
      echo -e "${YELLOW}Running tests...${NC}"
      $TEST_CMD || true
      echo ""
      echo -e "${YELLOW}Waiting for changes... (Ctrl+C to stop)${NC}"
      sleep 5
    done
  fi
else
  # Run once
  if $TEST_CMD; then
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"

    if [ "$COVERAGE" = true ]; then
      echo ""
      echo -e "${YELLOW}Coverage report generated:${NC}"
      echo "  coverage/lcov.info"

      # Generate HTML report if genhtml is available
      if command -v genhtml &> /dev/null; then
        echo ""
        echo -e "${YELLOW}Generating HTML coverage report...${NC}"
        genhtml coverage/lcov.info -o coverage/html
        echo -e "${GREEN}HTML report: coverage/html/index.html${NC}"
      fi
    fi

    exit 0
  else
    echo ""
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
  fi
fi
