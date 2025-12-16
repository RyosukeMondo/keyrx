#!/usr/bin/env bash
# Integrated development workflow: Rust watch + Flutter hot reload in parallel
# - Runs cargo-watch for Rust FFI changes (compile → codegen → reload)
# - Runs Flutter hot reload for UI changes
# - Coordinates logs with [RUST] and [FLUTTER] prefixes
# - Handles Ctrl+C gracefully (kills both processes)
# - Detects ABI changes requiring full restart vs Dart-only hot reload
# - Supports --mock flag to run with mock FFI bridge

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
USE_MOCK=0
FLUTTER_DEVICE="${DEVICE:-linux}"
RUST_PID=""
FLUTTER_PID=""
ABI_HASH_FILE="$REPO_ROOT/.cache/integrated_dev/abi_hash"

# Color codes for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_help() {
    cat <<'EOF'
integrated_dev.sh - unified development workflow for KeyRx

Usage: scripts/integrated_dev.sh [OPTIONS]

Options:
  --mock        Run Flutter with mock FFI bridge (no Rust compilation)
  --device=X    Flutter device target (default: linux)
  -h, --help    Show this help message

Description:
  Runs Rust watch mode and Flutter hot reload in parallel:
  - Rust changes trigger: compile → FFI codegen → Flutter reload
  - Flutter-only changes trigger: hot reload only
  - ABI changes trigger: full restart
  - All logs prefixed with [RUST] or [FLUTTER]
  - Ctrl+C gracefully shuts down both processes

Examples:
  scripts/integrated_dev.sh              # Normal mode with Rust + Flutter
  scripts/integrated_dev.sh --mock       # Mock FFI mode (UI-only development)
  scripts/integrated_dev.sh --device=chrome  # Run Flutter on Chrome
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mock)
            USE_MOCK=1
            shift
            ;;
        --device=*)
            FLUTTER_DEVICE="${1#*=}"
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

# Ensure cache directory exists
mkdir -p "$(dirname "$ABI_HASH_FILE")"

# Cleanup handler for graceful shutdown
cleanup() {
    local exit_code=$?
    echo -e "\n${YELLOW}[SHUTDOWN]${NC} Received interrupt, shutting down processes..."

    if [[ -n "$RUST_PID" ]] && kill -0 "$RUST_PID" 2>/dev/null; then
        echo -e "${BLUE}[RUST]${NC} Stopping Rust watch mode (PID: $RUST_PID)..."
        kill -TERM "$RUST_PID" 2>/dev/null || true
        wait "$RUST_PID" 2>/dev/null || true
    fi

    if [[ -n "$FLUTTER_PID" ]] && kill -0 "$FLUTTER_PID" 2>/dev/null; then
        echo -e "${CYAN}[FLUTTER]${NC} Stopping Flutter hot reload (PID: $FLUTTER_PID)..."
        kill -TERM "$FLUTTER_PID" 2>/dev/null || true
        wait "$FLUTTER_PID" 2>/dev/null || true
    fi

    echo -e "${GREEN}[SHUTDOWN]${NC} All processes stopped gracefully."
    exit $exit_code
}

# Register cleanup handler for Ctrl+C and other signals
trap cleanup SIGINT SIGTERM EXIT

# Compute ABI hash from Rust FFI source files
compute_abi_hash() {
    find "$REPO_ROOT/core/src/ffi" -name "*.rs" -type f -exec sha256sum {} + | \
        sort | sha256sum | awk '{print $1}'
}

# Check if ABI changed (requires full restart vs hot reload)
detect_abi_change() {
    local current_hash
    current_hash="$(compute_abi_hash)"
    local previous_hash
    previous_hash="$(cat "$ABI_HASH_FILE" 2>/dev/null || echo "")"

    if [[ "$current_hash" != "$previous_hash" ]]; then
        echo "$current_hash" > "$ABI_HASH_FILE"
        return 0  # ABI changed
    else
        return 1  # ABI unchanged
    fi
}

# Prefix logs with colored tags
prefix_logs() {
    local prefix="$1"
    local color="$2"
    while IFS= read -r line; do
        echo -e "${color}[${prefix}]${NC} $line"
    done
}

# Start Rust watch mode (cargo-watch with FFI codegen)
start_rust_watch() {
    if [[ $USE_MOCK -eq 1 ]]; then
        echo -e "${YELLOW}[INFO]${NC} Skipping Rust watch mode (--mock flag enabled)"
        return
    fi

    echo -e "${BLUE}[RUST]${NC} Starting Rust watch mode..."
    echo -e "${BLUE}[RUST]${NC} Monitoring: core/src/ffi/ and core/src/api.rs"

    # Check if cargo-watch is installed
    if ! command -v cargo-watch >/dev/null 2>&1; then
        echo -e "${RED}[ERROR]${NC} cargo-watch not found. Install via:"
        echo -e "  ${GREEN}cargo install cargo-watch${NC}"
        exit 1
    fi

    # Run cargo-watch with FFI watch script
    (cd "$REPO_ROOT/core" && \
        cargo watch -c -w src/ffi -w src/api.rs \
        -s '../scripts/ffi_watch.sh' --delay 0.5 2>&1) | \
        prefix_logs "RUST" "$BLUE" &

    RUST_PID=$!
    echo -e "${GREEN}[RUST]${NC} Watch mode started (PID: $RUST_PID)"
}

# Start Flutter hot reload
start_flutter_watch() {
    echo -e "${CYAN}[FLUTTER]${NC} Starting Flutter hot reload..."
    echo -e "${CYAN}[FLUTTER]${NC} Device: $FLUTTER_DEVICE"

    # Check if Flutter is installed
    if ! command -v flutter >/dev/null 2>&1; then
        echo -e "${RED}[ERROR]${NC} Flutter not found. Please install Flutter."
        exit 1
    fi

    # Build Flutter run command
    local flutter_cmd="flutter run -d $FLUTTER_DEVICE"

    if [[ $USE_MOCK -eq 1 ]]; then
        echo -e "${YELLOW}[FLUTTER]${NC} Running with mock FFI bridge"
        flutter_cmd="$flutter_cmd --dart-define=MOCK_FFI=true"
    fi

    # Run Flutter in watch mode
    (cd "$REPO_ROOT/ui" && $flutter_cmd 2>&1) | \
        prefix_logs "FLUTTER" "$CYAN" &

    FLUTTER_PID=$!
    echo -e "${GREEN}[FLUTTER]${NC} Hot reload started (PID: $FLUTTER_PID)"
}

# Monitor for ABI changes and notify
monitor_abi_changes() {
    if [[ $USE_MOCK -eq 1 ]]; then
        return  # Skip ABI monitoring in mock mode
    fi

    echo -e "${MAGENTA}[MONITOR]${NC} Starting ABI change detection..."

    while true; do
        sleep 2  # Check every 2 seconds

        if detect_abi_change; then
            echo -e "${YELLOW}[MONITOR]${NC} ABI change detected!"
            echo -e "${YELLOW}[MONITOR]${NC} Rust FFI interface changed - full restart recommended"
            echo -e "${YELLOW}[MONITOR]${NC} Press 'R' in Flutter terminal to perform hot restart"
        fi
    done
}

# Print startup banner
print_banner() {
    cat <<'BANNER'
╔═══════════════════════════════════════════════════════════╗
║          KeyRx Integrated Development Workflow            ║
╚═══════════════════════════════════════════════════════════╝
BANNER

    if [[ $USE_MOCK -eq 1 ]]; then
        echo -e "${YELLOW}Mode:${NC} Mock FFI (UI-only development)"
        echo -e "${YELLOW}Device:${NC} $FLUTTER_DEVICE"
    else
        echo -e "${GREEN}Mode:${NC} Full development (Rust + Flutter)"
        echo -e "${GREEN}Device:${NC} $FLUTTER_DEVICE"
    fi

    echo ""
    echo -e "${GREEN}Controls:${NC}"
    echo -e "  • Press ${CYAN}r${NC} in Flutter terminal for hot reload"
    echo -e "  • Press ${CYAN}R${NC} in Flutter terminal for hot restart"
    echo -e "  • Press ${CYAN}Ctrl+C${NC} to stop all processes"
    echo ""
    echo -e "${GREEN}Timeline:${NC}"
    echo -e "  Edit → Compile → Generate → Reload"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo ""
}

# Main execution
main() {
    print_banner

    # Initialize ABI hash
    if [[ $USE_MOCK -eq 0 ]]; then
        compute_abi_hash > "$ABI_HASH_FILE"
    fi

    # Start both processes
    start_rust_watch
    sleep 2  # Give Rust watch a head start
    start_flutter_watch

    # Start ABI monitoring in background (non-mock mode only)
    if [[ $USE_MOCK -eq 0 ]]; then
        monitor_abi_changes &
    fi

    # Wait for both processes to complete
    # (they should run until interrupted)
    if [[ -n "$RUST_PID" ]]; then
        wait "$RUST_PID" 2>/dev/null || true
    fi

    if [[ -n "$FLUTTER_PID" ]]; then
        wait "$FLUTTER_PID" 2>/dev/null || true
    fi
}

# Run main function
main
