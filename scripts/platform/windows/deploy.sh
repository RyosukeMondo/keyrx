#!/bin/bash
set -e

# Script to build Windows binaries and deploy to remote Windows PC via SSH/SCP
# Usage: ./scripts/deploy_windows.sh [OPTIONS]

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../../lib/common.sh
source "$SCRIPT_DIR/../../lib/common.sh"

# Configuration (can be overridden by environment variables or flags)
WINDOWS_CLIENT_HOST="${WINDOWS_CLIENT_HOST:-}"
WINDOWS_CLIENT_USER="${WINDOWS_CLIENT_USER:-}"
DEPLOY_DIR="${DEPLOY_DIR:-C:/keyrx_deploy}"
BUILD_IN_VM="${BUILD_IN_VM:-true}"
RUN_INSTALLER="${RUN_INSTALLER:-false}"
USE_RSYNC="${USE_RSYNC:-false}"

# Script-specific flags
RELEASE_MODE=true
SKIP_BUILD=false
INSTALLER_ONLY=false

show_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build Windows binaries and deploy to remote Windows PC via SSH/SCP.

OPTIONS:
  --host HOST              Windows client hostname or IP (required)
  --user USER              Windows client SSH username (required)
  --deploy-dir DIR         Deployment directory on Windows (default: C:/keyrx_deploy)
  --skip-build             Skip building, deploy existing artifacts only
  --installer-only         Deploy only installer package (not full build artifacts)
  --run-installer          Run installer after deployment
  --use-rsync              Use rsync instead of scp for transfer (requires rsync on Windows)
  --no-vm-build            Build directly on Linux (not in Vagrant VM)

  Common flags:
  --quiet                  Suppress non-error output
  --json                   Output in JSON format
  --error                  Show only errors
  --log-file PATH          Custom log file path

ENVIRONMENT VARIABLES:
  WINDOWS_CLIENT_HOST      Same as --host
  WINDOWS_CLIENT_USER      Same as --user
  DEPLOY_DIR               Same as --deploy-dir

EXAMPLES:
  # Basic deployment
  $(basename "$0") --host 192.168.1.100 --user developer

  # Deploy with custom directory
  $(basename "$0") --host win-client --user admin --deploy-dir D:/keyrx

  # Deploy existing build (skip building)
  $(basename "$0") --host win-client --user admin --skip-build

  # Deploy and install
  $(basename "$0") --host win-client --user admin --run-installer

  # Use rsync for faster repeated deployments
  $(basename "$0") --host win-client --user admin --use-rsync

EOF
}

# Parse arguments
parse_arguments() {
    REMAINING_ARGS=()
    parse_common_flags "$@"

    set -- "${REMAINING_ARGS[@]}"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --host)
                WINDOWS_CLIENT_HOST="$2"
                shift 2
                ;;
            --user)
                WINDOWS_CLIENT_USER="$2"
                shift 2
                ;;
            --deploy-dir)
                DEPLOY_DIR="$2"
                shift 2
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --installer-only)
                INSTALLER_ONLY=true
                shift
                ;;
            --run-installer)
                RUN_INSTALLER=true
                shift
                ;;
            --use-rsync)
                USE_RSYNC=true
                shift
                ;;
            --no-vm-build)
                BUILD_IN_VM=false
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Validate configuration
validate_config() {
    if [[ -z "$WINDOWS_CLIENT_HOST" ]]; then
        log_error "Windows client host not specified. Use --host or set WINDOWS_CLIENT_HOST"
        show_usage
        exit 1
    fi

    if [[ -z "$WINDOWS_CLIENT_USER" ]]; then
        log_error "Windows client user not specified. Use --user or set WINDOWS_CLIENT_USER"
        show_usage
        exit 1
    fi

    log_info "Target: $WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST:$DEPLOY_DIR"
}

# Test SSH connection
test_ssh_connection() {
    log_info "Testing SSH connection to $WINDOWS_CLIENT_HOST..."

    if ! ssh -o BatchMode=yes -o ConnectTimeout=5 "$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST" "echo Connection successful" >/dev/null 2>&1; then
        log_error "Cannot connect to $WINDOWS_CLIENT_HOST via SSH"
        log_error "Ensure SSH is configured and keys are set up (ssh-copy-id may help)"
        return 1
    fi

    log_info "SSH connection successful"
}

# Build Windows binaries in Vagrant VM
build_in_vagrant() {
    log_info "Building Windows binaries in Vagrant VM..."

    local vagrant_dir="vagrant/windows"

    if [[ ! -d "$vagrant_dir" ]]; then
        log_error "Vagrant directory not found: $vagrant_dir"
        return 1
    fi

    cd "$vagrant_dir"

    # Check VM status
    local status
    status=$(vagrant status --machine-readable | grep ",state," | cut -d, -f4)

    if [[ "$status" != "running" ]]; then
        log_info "Starting Vagrant VM..."
        vagrant up
    fi

    # Sync files
    log_info "Syncing files to VM..."
    vagrant rsync

    # Build release binary
    log_info "Building release binary..."
    vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'

    # Return to project root
    cd - >/dev/null

    log_info "Build completed"
}

# Build Windows binaries directly on Linux (cross-compilation)
build_on_linux() {
    log_info "Building Windows binaries on Linux (cross-compilation)..."

    # Check for Windows target
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        log_info "Installing Windows target..."
        rustup target add x86_64-pc-windows-gnu
    fi

    # Build
    cargo build --release --target x86_64-pc-windows-gnu --features windows

    log_info "Build completed"
}

# Copy binaries from Vagrant VM to local
copy_from_vagrant() {
    log_info "Copying binaries from Vagrant VM..."

    local vagrant_dir="vagrant/windows"
    local temp_dir="/tmp/keyrx_windows_build_$$"

    mkdir -p "$temp_dir"

    cd "$vagrant_dir"

    # Copy built artifacts
    vagrant winrm -c "powershell -Command \"Compress-Archive -Path C:\\vagrant_project\\target\\release\\*.exe -DestinationPath C:\\tmp\\keyrx_build.zip -Force\""
    vagrant download C:/tmp/keyrx_build.zip "$temp_dir/keyrx_build.zip"

    cd - >/dev/null

    # Extract
    unzip -o "$temp_dir/keyrx_build.zip" -d "$temp_dir"

    echo "$temp_dir"
}

# Get build artifacts location
get_build_artifacts() {
    if [[ "$BUILD_IN_VM" == "true" ]]; then
        copy_from_vagrant
    else
        # Local cross-compiled build
        echo "target/x86_64-pc-windows-gnu/release"
    fi
}

# Create remote directory
create_remote_dir() {
    log_info "Creating deployment directory on remote: $DEPLOY_DIR"

    # Convert Unix path to Windows path for PowerShell
    local win_path="${DEPLOY_DIR//\//\\}"

    ssh "$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST" \
        "powershell -Command \"New-Item -ItemType Directory -Path '$win_path' -Force\" | Out-Null"
}

# Deploy using SCP
deploy_scp() {
    local artifacts_dir="$1"

    log_info "Deploying files via SCP..."

    if [[ "$INSTALLER_ONLY" == "true" ]]; then
        # Copy only installer package
        scp "$artifacts_dir/keyrx_setup.exe" "$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST:$DEPLOY_DIR/"
    else
        # Copy all executables
        scp "$artifacts_dir"/*.exe "$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST:$DEPLOY_DIR/"
    fi

    log_info "SCP transfer completed"
}

# Deploy using rsync
deploy_rsync() {
    local artifacts_dir="$1"

    log_info "Deploying files via rsync..."

    # Check if rsync is available
    if ! command_exists rsync; then
        log_error "rsync not found on local machine"
        return 1
    fi

    local rsync_dest="$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST:$(echo "$DEPLOY_DIR" | sed 's|\\|/|g')"

    if [[ "$INSTALLER_ONLY" == "true" ]]; then
        rsync -avz -e ssh "$artifacts_dir/keyrx_setup.exe" "$rsync_dest/"
    else
        rsync -avz -e ssh "$artifacts_dir"/*.exe "$rsync_dest/"
    fi

    log_info "Rsync transfer completed"
}

# Run installer on remote
run_remote_installer() {
    log_info "Running installer on remote Windows PC..."

    local win_path="${DEPLOY_DIR//\//\\}"

    ssh "$WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST" \
        "powershell -Command \"Start-Process -FilePath '$win_path\\keyrx_setup.exe' -ArgumentList '/silent' -Wait\""

    log_info "Installer completed"
}

# Main deployment flow
main() {
    local start_time
    start_time=$(date +%s)

    # Setup logging
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "deploy_windows"
    fi

    log_info "=== Windows Deployment Script ==="

    # Parse arguments
    parse_arguments "$@"

    # Validate
    validate_config

    # Test connection
    if ! test_ssh_connection; then
        log_failed
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "SSH connection failed" "exit_code" "1"
        fi
        exit 1
    fi

    # Build (unless skipped)
    local artifacts_dir
    if [[ "$SKIP_BUILD" == "true" ]]; then
        log_info "Skipping build (using existing artifacts)"
        artifacts_dir=$(get_build_artifacts)
    else
        if [[ "$BUILD_IN_VM" == "true" ]]; then
            if ! build_in_vagrant; then
                log_failed
                if [[ "$JSON_MODE" == "true" ]]; then
                    output_json "status" "failed" "error" "Build failed" "exit_code" "1"
                fi
                exit 1
            fi
        else
            if ! build_on_linux; then
                log_failed
                if [[ "$JSON_MODE" == "true" ]]; then
                    output_json "status" "failed" "error" "Build failed" "exit_code" "1"
                fi
                exit 1
            fi
        fi

        artifacts_dir=$(get_build_artifacts)
    fi

    # Create remote directory
    create_remote_dir

    # Deploy
    if [[ "$USE_RSYNC" == "true" ]]; then
        if ! deploy_rsync "$artifacts_dir"; then
            log_failed
            if [[ "$JSON_MODE" == "true" ]]; then
                output_json "status" "failed" "error" "Rsync transfer failed" "exit_code" "1"
            fi
            exit 1
        fi
    else
        if ! deploy_scp "$artifacts_dir"; then
            log_failed
            if [[ "$JSON_MODE" == "true" ]]; then
                output_json "status" "failed" "error" "SCP transfer failed" "exit_code" "1"
            fi
            exit 1
        fi
    fi

    # Run installer if requested
    if [[ "$RUN_INSTALLER" == "true" ]]; then
        if ! run_remote_installer; then
            log_warning_marker
            log_warn "Installer execution failed or completed with warnings"
        fi
    fi

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    log_accomplished
    log_info "Deployment completed in ${duration}s"

    if [[ "$JSON_MODE" == "true" ]]; then
        output_json \
            "status" "success" \
            "target_host" "$WINDOWS_CLIENT_HOST" \
            "deploy_dir" "$DEPLOY_DIR" \
            "duration_seconds" "$duration" \
            "exit_code" "0"
    fi
}

# Run main function
main "$@"
