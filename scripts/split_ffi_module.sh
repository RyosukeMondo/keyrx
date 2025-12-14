#!/usr/bin/env bash
# Split large FFI modules into smaller, focused files
# Usage: scripts/split_ffi_module.sh <command> <module-file> [options]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TOOL_PATH="$PROJECT_ROOT/core/tools/module_splitter"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    cat <<EOF
Usage: $0 <command> <module-file> [options]

Commands:
    analyze         Analyze a module and show statistics
    preview         Preview the split plan without applying changes
    apply           Apply the split and create new module files

Options:
    --max-size N    Maximum lines per output module (default: 500)
    -y, --yes       Skip confirmation prompt (for apply command)
    --format FMT    Output format for analyze: text, json (default: text)

Examples:
    # Analyze a module
    $0 analyze core/src/ffi/exports_engine.rs

    # Preview split plan
    $0 preview core/src/ffi/exports_engine.rs --max-size 400

    # Apply split without confirmation
    $0 apply core/src/ffi/exports_engine.rs -y

EOF
}

# Check if module_splitter is built
ensure_tool_built() {
    if [ ! -f "$TOOL_PATH/target/release/module_splitter" ] && [ ! -f "$TOOL_PATH/target/debug/module_splitter" ]; then
        echo -e "${YELLOW}Building module_splitter tool...${NC}"
        cd "$PROJECT_ROOT"
        cargo build --package module_splitter
        echo -e "${GREEN}✓ Tool built successfully${NC}"
    fi
}

# Run the module_splitter tool
run_tool() {
    cd "$PROJECT_ROOT"

    # Use release build if available, otherwise debug
    if [ -f "$TOOL_PATH/target/release/module_splitter" ]; then
        "$TOOL_PATH/target/release/module_splitter" "$@"
    else
        cargo run --package module_splitter -- "$@"
    fi
}

main() {
    if [ $# -lt 1 ]; then
        print_usage
        exit 1
    fi

    local command="$1"
    shift

    case "$command" in
        analyze|preview|apply)
            ensure_tool_built
            run_tool "$command" "$@"
            ;;
        --help|-h|help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown command '$command'${NC}"
            echo
            print_usage
            exit 1
            ;;
    esac
}

main "$@"
