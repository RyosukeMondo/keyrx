#!/bin/bash
# Launch script for keyrx daemon
# Supports: --headless, --debug, --config PATH, --release, --error, --json, --quiet, --log-file

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
HEADLESS_MODE=false
DEBUG_MODE=false
CONFIG_PATH=""
RELEASE_MODE=false
DAEMON_PID=""
DAEMON_PORT=""

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Launch the keyrx daemon with specified configuration.

OPTIONS:
    --headless      Suppress browser launch (headless mode)
    --debug         Enable debug logging (log-level: debug)
    --config PATH   Specify custom configuration file path
    --release       Build and run release binary (optimized)
    --error         Show only errors
    --json          Output results in JSON format (includes PID and ports)
    --quiet         Suppress non-error output
    --log-file PATH Specify custom log file path
    -h, --help      Show this help message

EXAMPLES:
    $(basename "$0")                         # Launch daemon (debug build, info logging)
    $(basename "$0") --release               # Launch daemon (release build)
    $(basename "$0") --debug                 # Launch with debug logging
    $(basename "$0") --headless              # Launch without opening browser
    $(basename "$0") --config custom.toml    # Launch with custom config
    $(basename "$0") --json                  # JSON output with PID and ports

EXIT CODES:
    0 - Daemon launched successfully
    1 - Launch failed (build or startup error)
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - Daemon launched successfully
    === failed ===       - Daemon launch failed
EOF
}

# Parse arguments
parse_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags from remaining args
    set -- "${REMAINING_ARGS[@]}"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --headless)
                HEADLESS_MODE=true
                shift
                ;;
            --debug)
                DEBUG_MODE=true
                shift
                ;;
            --config)
                if [[ -z "${2:-}" ]]; then
                    log_error "--config requires a path argument"
                    exit 1
                fi
                CONFIG_PATH="$2"
                shift 2
                ;;
            --release)
                RELEASE_MODE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Build daemon binary
build_daemon() {
    local build_cmd="cargo build --bin keyrx_daemon"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_cmd="$build_cmd --release"
        log_info "Building keyrx_daemon in release mode..."
    else
        log_info "Building keyrx_daemon in debug mode..."
    fi

    # Execute build
    if $build_cmd 2>&1 | while IFS= read -r line; do
        if [[ "$ERROR_ONLY_MODE" == "true" ]]; then
            # Only show errors
            if echo "$line" | grep -qi "error"; then
                log_error "$line"
            fi
        else
            # Show all output in quiet mode suppressed by log_info
            if [[ "$QUIET_MODE" != "true" ]]; then
                echo "$line"
            fi
        fi
    done; then
        log_info "Build completed successfully"
        return 0
    else
        log_error "Build failed"
        return 1
    fi
}

# Launch daemon
launch_daemon() {
    # Determine binary path
    local binary_path
    if [[ "$RELEASE_MODE" == "true" ]]; then
        binary_path="target/release/keyrx_daemon"
    else
        binary_path="target/debug/keyrx_daemon"
    fi

    # Verify binary exists
    if [[ ! -f "$binary_path" ]]; then
        log_error "Daemon binary not found at: $binary_path"
        return 1
    fi

    # Construct daemon arguments for the 'run' subcommand
    local daemon_args=""

    if [[ "$DEBUG_MODE" == "true" ]]; then
        daemon_args="$daemon_args --debug"
    fi

    if [[ -n "$CONFIG_PATH" ]]; then
        if [[ ! -f "$CONFIG_PATH" ]]; then
            log_error "Config file not found: $CONFIG_PATH"
            return 1
        fi
        daemon_args="$daemon_args --config $CONFIG_PATH"
    fi

    log_info "Launching daemon: $binary_path $daemon_args"

    # Launch daemon in background and capture output
    local temp_output
    temp_output=$(mktemp)

    # On Windows, launch via cmd.exe to get a proper console with message loop
    # (bash & doesn't pump Windows messages, breaking keyboard hooks)
    if command -v cmd.exe &>/dev/null; then
        local win_binary
        win_binary=$(cygpath -w "$binary_path" 2>/dev/null || echo "$binary_path")
        # Use PowerShell Start-Process to launch with proper console for Windows message loop
        local ps_args="run ${daemon_args}"
        powershell -Command "Start-Process -FilePath '${win_binary}' -ArgumentList '${ps_args}'" 2>"$temp_output"
        sleep 3
        # Find the PID of the launched daemon
        DAEMON_PID=$(powershell -Command "(Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Select-Object -First 1).Id" 2>/dev/null || echo "")
    else
        # Linux/macOS: background process works fine
        $binary_path run $daemon_args > "$temp_output" 2>&1 &
        DAEMON_PID=$!
    fi

    # Give daemon time to start
    sleep 2

    # Check if daemon is still running
    local daemon_alive=false
    if command -v powershell &>/dev/null; then
        # Windows: check via PowerShell
        if powershell -Command "Get-Process keyrx_daemon -ErrorAction SilentlyContinue" &>/dev/null; then
            daemon_alive=true
            # Get PID if we don't have it yet
            if [[ -z "$DAEMON_PID" ]]; then
                DAEMON_PID=$(powershell -Command "(Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Select-Object -First 1).Id" 2>/dev/null || echo "unknown")
            fi
        fi
    elif kill -0 "$DAEMON_PID" 2>/dev/null; then
        daemon_alive=true
    fi

    if [[ "$daemon_alive" != "true" ]]; then
        log_error "Daemon failed to start"
        log_error "Output:"
        cat "$temp_output" >&2
        rm -f "$temp_output"
        return 1
    fi

    # Try to extract port from daemon output
    # Common patterns: "Listening on", "Server started on", "port 8080", etc.
    if grep -qE "(Listening|Server|port)" "$temp_output" 2>/dev/null; then
        DAEMON_PORT=$(grep -oE "([0-9]{4,5})" "$temp_output" | head -n 1)
    fi

    # Default port if not found (common web server default)
    if [[ -z "$DAEMON_PORT" ]]; then
        DAEMON_PORT="8080"
    fi

    rm -f "$temp_output"

    log_info "Daemon started with PID: $DAEMON_PID"
    if [[ -n "$DAEMON_PORT" ]]; then
        log_info "Listening on port: $DAEMON_PORT"
    fi

    return 0
}

# Main execution
main() {
    local exit_code=0
    local build_type="debug"

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "launch"
    fi

    # Kill existing keyrx processes for stable testing
    log_info "Stopping existing keyrx processes..."
    if command -v taskkill &>/dev/null; then
        taskkill //F //IM keyrx_daemon.exe 2>/dev/null && log_info "Killed existing keyrx_daemon.exe" || true
    else
        pkill -f keyrx_daemon 2>/dev/null && log_info "Killed existing keyrx_daemon" || true
    fi
    sleep 1

    # Verify cargo is installed
    if ! require_tool "cargo" "Install Rust from https://rustup.rs"; then
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "cargo not found" "exit_code" "2"
        fi
        exit 2
    fi

    separator

    # Determine build type
    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_type="release"
    fi

    # Build daemon
    if ! build_daemon; then
        log_failed
        separator
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json \
                "status" "failed" \
                "error" "build failed" \
                "build_type" "$build_type" \
                "exit_code" "1"
        fi
        exit 1
    fi

    # Launch daemon
    if ! launch_daemon; then
        log_failed
        separator
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json \
                "status" "failed" \
                "error" "daemon startup failed" \
                "build_type" "$build_type" \
                "exit_code" "1"
        fi
        exit 1
    fi

    log_accomplished

    separator

    # JSON output
    if [[ "$JSON_MODE" == "true" ]]; then
        output_json \
            "status" "success" \
            "pid" "$DAEMON_PID" \
            "port" "$DAEMON_PORT" \
            "build_type" "$build_type" \
            "headless" "$HEADLESS_MODE" \
            "debug" "$DEBUG_MODE" \
            "exit_code" "0"
    fi

    exit 0
}

# Run main function
main "$@"
