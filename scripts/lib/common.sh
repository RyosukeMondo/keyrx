#!/bin/bash
# Common utilities for all scripts
# Provides logging, argument parsing, exit code checking, and JSON formatting

set -euo pipefail

# Global variables for configuration
QUIET_MODE=false
JSON_MODE=false
ERROR_ONLY_MODE=false
LOG_FILE=""

# Color codes for terminal output
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_BLUE='\033[0;34m'

# Logging functions
# Format: [YYYY-MM-DD HH:MM:SS] [LEVEL] message

log_info() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] [INFO] $*"
    echo -e "${COLOR_BLUE}${message}${COLOR_RESET}"
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_error() {
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] [ERROR] $*"
    echo -e "${COLOR_RED}${message}${COLOR_RESET}" >&2
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_warn() {
    if [[ "$QUIET_MODE" == "true" ]] || [[ "$ERROR_ONLY_MODE" == "true" ]]; then
        return
    fi
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] [WARN] $*"
    echo -e "${COLOR_YELLOW}${message}${COLOR_RESET}" >&2
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_debug() {
    if [[ "$QUIET_MODE" == "true" ]] || [[ "$ERROR_ONLY_MODE" == "true" ]]; then
        return
    fi
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] [DEBUG] $*"
    echo "$message"
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_success() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] [SUCCESS] $*"
    echo -e "${COLOR_GREEN}${message}${COLOR_RESET}"
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_header() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi
    echo ""
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo -e "${COLOR_BLUE}$*${COLOR_RESET}"
    echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
    echo ""
}

# Status marker functions
log_accomplished() {
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] === accomplished ==="
    echo -e "${COLOR_GREEN}${message}${COLOR_RESET}"
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_failed() {
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] === failed ==="
    echo -e "${COLOR_RED}${message}${COLOR_RESET}" >&2
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

log_warning_marker() {
    local timestamp
    timestamp=$(get_timestamp)
    local message="[$timestamp] === warning ==="
    echo -e "${COLOR_YELLOW}${message}${COLOR_RESET}" >&2
    if [[ -n "$LOG_FILE" ]]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

# Timestamp functions
get_timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

get_epoch_timestamp() {
    date +%s
}

# Exit code checker
# Usage: check_exit_code $? "operation description"
check_exit_code() {
    local exit_code=$1
    local operation="${2:-operation}"

    if [[ $exit_code -eq 0 ]]; then
        log_info "$operation completed successfully"
        return 0
    else
        log_error "$operation failed with exit code $exit_code"
        return $exit_code
    fi
}

# Log file manager
# Usage: setup_log_file "prefix" (e.g., "build", "verify")
setup_log_file() {
    local prefix="${1:-script}"
    local epoch
    epoch=$(get_epoch_timestamp)

    # Get absolute path to scripts/logs directory
    # COMMON_SH_DIR is set when this file is sourced
    local script_base="${COMMON_SH_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
    local log_dir="${script_base}/../logs"

    # Ensure logs directory exists (use absolute path)
    mkdir -p "$log_dir"

    # Set global LOG_FILE variable with absolute path
    LOG_FILE="$(cd "$log_dir" && pwd)/${prefix}_${epoch}.log"

    log_info "Log file: $LOG_FILE"
}

# Argument parsing helper
# Usage:
#   REMAINING_ARGS=()
#   parse_common_flags "$@"
# Sets global variables: QUIET_MODE, JSON_MODE, ERROR_ONLY_MODE, LOG_FILE, REMAINING_ARGS
parse_common_flags() {
    REMAINING_ARGS=()

    while [[ $# -gt 0 ]]; do
        case $1 in
            --quiet)
                QUIET_MODE=true
                shift
                ;;
            --json)
                JSON_MODE=true
                QUIET_MODE=true  # JSON mode implies quiet
                shift
                ;;
            --error)
                ERROR_ONLY_MODE=true
                shift
                ;;
            --log-file)
                if [[ -z "${2:-}" ]]; then
                    log_error "--log-file requires a path argument"
                    return 1
                fi
                LOG_FILE="$2"
                shift 2
                ;;
            *)
                # Not a common flag, save for caller
                REMAINING_ARGS+=("$1")
                shift
                ;;
        esac
    done
}

# JSON output formatter
# Usage: output_json "key1" "value1" "key2" "value2" ...
output_json() {
    if [[ "$JSON_MODE" != "true" ]]; then
        return
    fi

    echo -n "{"
    local first=true

    while [[ $# -gt 0 ]]; do
        local key="$1"
        local value="${2:-}"

        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo -n ","
        fi

        # Escape quotes in value
        value="${value//\"/\\\"}"

        echo -n "\"$key\":\"$value\""
        shift 2
    done

    echo "}"
}

# JSON object builder for complex structures
# Usage: json_object "key1" "value1" "key2" "value2" ...
json_object() {
    echo -n "{"
    local first=true

    while [[ $# -gt 0 ]]; do
        local key="$1"
        local value="${2:-}"

        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo -n ","
        fi

        # Check if value looks like JSON (starts with { or [)
        if [[ "$value" =~ ^[\{\[] ]]; then
            echo -n "\"$key\":$value"
        else
            # Escape quotes in value
            value="${value//\"/\\\"}"
            echo -n "\"$key\":\"$value\""
        fi
        shift 2
    done

    echo "}"
}

# JSON array builder
# Usage: json_array "item1" "item2" "item3" ...
json_array() {
    echo -n "["
    local first=true

    for item in "$@"; do
        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo -n ","
        fi

        # Check if item looks like JSON object
        if [[ "$item" =~ ^[\{\[] ]]; then
            echo -n "$item"
        else
            # Escape quotes in item
            item="${item//\"/\\\"}"
            echo -n "\"$item\""
        fi
    done

    echo "]"
}

# Check if a command exists
# Usage: command_exists "command_name"
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Verify required tool is installed
# Usage: require_tool "cargo" "Install Rust from https://rustup.rs"
require_tool() {
    local tool="$1"
    local install_hint="${2:-Install $tool}"

    if ! command_exists "$tool"; then
        log_error "Required tool '$tool' not found"
        log_error "$install_hint"
        return 1
    fi

    log_debug "Tool '$tool' found"
    return 0
}

# Print a separator line
separator() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return
    fi
    echo "────────────────────────────────────────────────────────────────"
}

# Export functions for use in sourcing scripts
export -f log_info
export -f log_error
export -f log_warn
export -f log_debug
export -f log_accomplished
export -f log_failed
export -f log_warning_marker
export -f get_timestamp
export -f get_epoch_timestamp
export -f check_exit_code
export -f setup_log_file
export -f parse_common_flags
export -f output_json
export -f json_object
export -f json_array
export -f command_exists
export -f require_tool
export -f separator
