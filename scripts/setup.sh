#!/bin/bash
# setup.sh - Unified Environment Setup
#
# Consolidates: setup_dev_environment.sh, setup_linux.sh, setup_hooks.sh,
#               setup_desktop_integration.sh, setup_keyrx_windows_vm.sh
#
# Usage:
#   ./scripts/setup.sh              # Full setup (dev tools + Linux + hooks)
#   ./scripts/setup.sh --check      # Check current setup status
#   ./scripts/setup.sh --dev-tools  # Install development tools only
#   ./scripts/setup.sh --linux      # Linux environment setup only
#   ./scripts/setup.sh --hooks      # Git hooks setup only
#   ./scripts/setup.sh --desktop    # Desktop integration only
#   ./scripts/setup.sh --windows-vm # Windows VM setup only
#
# This script sets up everything needed for KeyRX development.

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
CHECK_ONLY=false
DEV_TOOLS_ONLY=false
LINUX_ONLY=false
MACOS_ONLY=false
HOOKS_ONLY=false
DESKTOP_ONLY=false
WINDOWS_VM_ONLY=false

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Unified environment setup for KeyRX development.

OPTIONS:
    --check         Check current setup status (don't install anything)
    --dev-tools     Install development tools only (cargo tools, npm)
    --linux         Linux environment setup only (groups, udev, uinput)
    --macos         macOS environment setup only (Accessibility permission check)
    --hooks         Git hooks setup only
    --desktop       Desktop integration only (.desktop file, icons)
    --windows-vm    Windows VM setup only (Vagrant + libvirt)
    --error         Show only errors
    --json          Output results in JSON format
    --quiet         Suppress non-error output
    --log-file PATH Specify custom log file path
    -h, --help      Show this help message

COMPONENTS:
    1. Dev Tools   - cargo-watch, cargo-llvm-cov, wasm-pack, npm packages
    2. Linux       - User groups (input, uinput), udev rules, kernel modules (Linux only)
    3. macOS       - Accessibility permission check, dev environment (macOS only)
    4. Git Hooks   - Pre-commit hook for verification
    5. Desktop     - .desktop file, application icons (Linux only)
    6. Windows VM  - Vagrant + libvirt for Windows testing (Linux only)

EXAMPLES:
    $(basename "$0")                 # Full setup
    $(basename "$0") --check         # Check status
    $(basename "$0") --dev-tools     # Dev tools only
    $(basename "$0") --linux         # Linux setup only

EXIT CODES:
    0 - Setup completed successfully
    1 - Setup failed
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - Setup succeeded
    === failed ===       - Setup failed
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
            --check)
                CHECK_ONLY=true
                shift
                ;;
            --dev-tools)
                DEV_TOOLS_ONLY=true
                shift
                ;;
            --linux)
                LINUX_ONLY=true
                shift
                ;;
            --macos)
                MACOS_ONLY=true
                shift
                ;;
            --hooks)
                HOOKS_ONLY=true
                shift
                ;;
            --desktop)
                DESKTOP_ONLY=true
                shift
                ;;
            --windows-vm)
                WINDOWS_VM_ONLY=true
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

# ============================================================================
# Status Checking
# ============================================================================

check_dev_tools_status() {
    log_info "Checking development tools..."

    local all_ok=true

    # Rust toolchain
    if command_exists rustc; then
        log_info "  Rust: $(rustc --version)"
    else
        log_warn "  Rust: NOT INSTALLED"
        all_ok=false
    fi

    # Cargo tools
    for tool in cargo-watch cargo-llvm-cov wasm-pack; do
        if cargo install --list 2>/dev/null | grep -q "^$tool"; then
            log_info "  $tool: installed"
        else
            log_warn "  $tool: NOT INSTALLED"
            all_ok=false
        fi
    done

    # WASM target
    if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        log_info "  wasm32-unknown-unknown: installed"
    else
        log_warn "  wasm32-unknown-unknown: NOT INSTALLED"
        all_ok=false
    fi

    # Node.js
    if command_exists node; then
        log_info "  Node.js: $(node --version)"
    else
        log_warn "  Node.js: NOT INSTALLED"
        all_ok=false
    fi

    # npm
    if command_exists npm; then
        log_info "  npm: $(npm --version)"
    else
        log_warn "  npm: NOT INSTALLED"
        all_ok=false
    fi

    if [[ "$all_ok" == "true" ]]; then
        log_info "Dev tools: OK"
        return 0
    else
        log_warn "Dev tools: INCOMPLETE"
        return 1
    fi
}

check_linux_status() {
    log_info "Checking Linux environment..."

    local all_ok=true

    # Check input group
    if groups | grep -q "input"; then
        log_info "  User in 'input' group: YES"
    else
        log_warn "  User in 'input' group: NO"
        all_ok=false
    fi

    # Check uinput group
    if getent group uinput > /dev/null 2>&1; then
        if groups | grep -q "uinput"; then
            log_info "  User in 'uinput' group: YES"
        else
            log_warn "  User in 'uinput' group: NO"
            all_ok=false
        fi
    else
        log_warn "  'uinput' group: DOES NOT EXIST"
        all_ok=false
    fi

    # Check udev rules
    if [[ -f "/etc/udev/rules.d/99-keyrx.rules" ]]; then
        log_info "  udev rules: INSTALLED"
    else
        log_warn "  udev rules: NOT INSTALLED"
        all_ok=false
    fi

    # Check uinput module
    if lsmod | grep -q "uinput"; then
        log_info "  uinput module: LOADED"
    else
        log_warn "  uinput module: NOT LOADED"
        all_ok=false
    fi

    if [[ "$all_ok" == "true" ]]; then
        log_info "Linux environment: OK"
        return 0
    else
        log_warn "Linux environment: INCOMPLETE"
        return 1
    fi
}

check_macos_status() {
    log_info "Checking macOS environment..."

    local all_ok=true

    # Check Accessibility permission
    if "$SCRIPT_DIR/platform/macos/check_permission.sh" 2>/dev/null; then
        log_info "  Accessibility permission: GRANTED"
    else
        log_warn "  Accessibility permission: NOT GRANTED"
        log_info "    Grant permission: System Settings > Privacy & Security > Accessibility"
        all_ok=false
    fi

    # Check if dev tools are macOS compatible
    if command_exists cargo && command_exists rustc; then
        log_info "  Rust toolchain: INSTALLED"
    else
        log_warn "  Rust toolchain: NOT INSTALLED"
        all_ok=false
    fi

    if [[ "$all_ok" == "true" ]]; then
        log_info "macOS environment: OK"
        return 0
    else
        log_warn "macOS environment: INCOMPLETE"
        return 1
    fi
}

check_hooks_status() {
    log_info "Checking Git hooks..."

    if [[ -f "$PROJECT_ROOT/.git/hooks/pre-commit" ]]; then
        log_info "  Pre-commit hook: INSTALLED"
        return 0
    else
        log_warn "  Pre-commit hook: NOT INSTALLED"
        return 1
    fi
}

check_desktop_status() {
    log_info "Checking desktop integration..."

    local all_ok=true

    # Check .desktop file
    local desktop_file="$HOME/.local/share/applications/keyrx.desktop"
    if [[ -f "$desktop_file" ]]; then
        log_info "  Desktop file: INSTALLED"
    else
        log_warn "  Desktop file: NOT INSTALLED"
        all_ok=false
    fi

    # Check icon
    local icon_dir="$HOME/.local/share/icons/hicolor/256x256/apps"
    if [[ -f "$icon_dir/keyrx.png" ]]; then
        log_info "  Application icon: INSTALLED"
    else
        log_warn "  Application icon: NOT INSTALLED"
        all_ok=false
    fi

    if [[ "$all_ok" == "true" ]]; then
        log_info "Desktop integration: OK"
        return 0
    else
        log_warn "Desktop integration: INCOMPLETE"
        return 1
    fi
}

check_all_status() {
    log_info "Checking all setup components..."
    separator

    local dev_ok=true
    local platform_ok=true
    local hooks_ok=true
    local desktop_ok=true

    check_dev_tools_status || dev_ok=false
    separator

    # Check platform-specific components
    case "$OSTYPE" in
        darwin*)
            check_macos_status || platform_ok=false
            ;;
        linux*)
            check_linux_status || platform_ok=false
            separator
            check_desktop_status || desktop_ok=false
            ;;
        *)
            log_warn "Unsupported OS: $OSTYPE"
            platform_ok=false
            ;;
    esac
    separator

    check_hooks_status || hooks_ok=false
    separator

    # Summary
    log_info "Setup Status Summary:"
    [[ "$dev_ok" == "true" ]] && log_info "  Dev Tools: OK" || log_warn "  Dev Tools: INCOMPLETE"

    case "$OSTYPE" in
        darwin*)
            [[ "$platform_ok" == "true" ]] && log_info "  macOS: OK" || log_warn "  macOS: INCOMPLETE"
            ;;
        linux*)
            [[ "$platform_ok" == "true" ]] && log_info "  Linux: OK" || log_warn "  Linux: INCOMPLETE"
            [[ "$desktop_ok" == "true" ]] && log_info "  Desktop: OK" || log_warn "  Desktop: INCOMPLETE"
            ;;
    esac

    [[ "$hooks_ok" == "true" ]] && log_info "  Git Hooks: OK" || log_warn "  Git Hooks: INCOMPLETE"

    # Determine overall success
    local all_ok=true
    [[ "$dev_ok" == "false" ]] && all_ok=false
    [[ "$platform_ok" == "false" ]] && all_ok=false
    [[ "$hooks_ok" == "false" ]] && all_ok=false
    if [[ "$OSTYPE" == "linux"* ]] && [[ "$desktop_ok" == "false" ]]; then
        all_ok=false
    fi

    if [[ "$all_ok" == "true" ]]; then
        log_accomplished
        return 0
    else
        log_warning_marker
        return 1
    fi
}

# ============================================================================
# Installation Functions
# ============================================================================

install_dev_tools() {
    log_info "Installing development tools..."

    # Check Rust
    if ! command_exists rustc; then
        log_error "Rust not found. Install from: https://rustup.rs"
        return 1
    fi

    # Install cargo tools
    local tools=("cargo-watch" "cargo-llvm-cov" "wasm-pack")
    for tool in "${tools[@]}"; do
        if ! cargo install --list 2>/dev/null | grep -q "^$tool"; then
            log_info "Installing $tool..."
            cargo install "$tool" || log_warn "Failed to install $tool"
        else
            log_info "$tool already installed"
        fi
    done

    # Install wasm32 target for WASM compilation
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        log_info "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown || log_warn "Failed to install wasm32 target"
    else
        log_info "wasm32-unknown-unknown target already installed"
    fi

    # Check Node.js and npm
    if ! command_exists node; then
        log_error "Node.js not found. Install from: https://nodejs.org"
        return 1
    fi

    # Install npm dependencies for keyrx_ui
    if [[ -d "$PROJECT_ROOT/keyrx_ui" ]]; then
        log_info "Installing npm dependencies..."
        cd "$PROJECT_ROOT/keyrx_ui"
        npm install --silent
        cd "$PROJECT_ROOT"
    fi

    log_info "Dev tools setup complete"
    return 0
}

setup_linux() {
    log_info "Setting up Linux environment..."

    # Add user to input group
    if ! groups | grep -q "input"; then
        log_info "Adding user to 'input' group..."
        sudo usermod -aG input "$USER"
        log_warn "You need to log out and back in for group changes to take effect"
    fi

    # Create and add user to uinput group
    if ! getent group uinput > /dev/null 2>&1; then
        log_info "Creating 'uinput' group..."
        sudo groupadd -f uinput
    fi

    if ! groups | grep -q "uinput"; then
        log_info "Adding user to 'uinput' group..."
        sudo usermod -aG uinput "$USER"
        log_warn "You need to log out and back in for group changes to take effect"
    fi

    # Install udev rules
    local udev_src="$PROJECT_ROOT/keyrx_daemon/udev/99-keyrx.rules"
    local udev_dst="/etc/udev/rules.d/99-keyrx.rules"
    if [[ -f "$udev_src" ]] && [[ ! -f "$udev_dst" ]]; then
        log_info "Installing udev rules..."
        sudo cp "$udev_src" "$udev_dst"
        sudo udevadm control --reload-rules
        sudo udevadm trigger
    fi

    # Load uinput module
    if ! lsmod | grep -q "uinput"; then
        log_info "Loading uinput module..."
        sudo modprobe uinput
    fi

    # Ensure uinput loads on boot
    if [[ ! -f "/etc/modules-load.d/uinput.conf" ]]; then
        log_info "Configuring uinput to load on boot..."
        echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf > /dev/null
    fi

    log_info "Linux setup complete"
    return 0
}

setup_macos() {
    log_info "Setting up macOS environment..."

    # Check Accessibility permission
    if "$SCRIPT_DIR/platform/macos/check_permission.sh" 2>/dev/null; then
        log_info "Accessibility permission already granted"
    else
        log_warn "Accessibility permission not granted"
        log_info ""
        log_info "To grant Accessibility permission:"
        log_info "  1. Open System Settings"
        log_info "  2. Go to Privacy & Security > Accessibility"
        log_info "  3. Click the lock icon and enter your password"
        log_info "  4. Add Terminal (or your IDE) to the list"
        log_info "  5. Toggle the switch ON"
        log_info ""
        log_info "You can also use the interactive setup script:"
        log_info "  $SCRIPT_DIR/platform/macos/setup_accessibility.sh"
        log_info ""
    fi

    # Install dev tools (already cross-platform)
    install_dev_tools

    # Setup git hooks (already cross-platform)
    setup_hooks

    log_info "macOS setup complete"
    return 0
}

setup_hooks() {
    log_info "Setting up Git hooks..."

    local hook_dir="$PROJECT_ROOT/.git/hooks"
    local hook_file="$hook_dir/pre-commit"

    # Check if in git repo
    if [[ ! -d "$PROJECT_ROOT/.git" ]]; then
        log_error "Not a git repository"
        return 1
    fi

    # Create hooks directory if needed
    mkdir -p "$hook_dir"

    # Create pre-commit hook
    cat > "$hook_file" << 'EOF'
#!/bin/bash
# Pre-commit hook for keyrx
# Runs verification before allowing commits

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Running pre-commit verification..."

# Run quick verification (skip coverage for speed)
if ! "$PROJECT_ROOT/scripts/verify.sh" --skip-coverage --quiet; then
    echo ""
    echo "Pre-commit checks failed. Commit aborted."
    echo "Run './scripts/verify.sh' to see detailed errors."
    exit 1
fi

echo "Pre-commit checks passed."
exit 0
EOF

    chmod +x "$hook_file"
    log_info "Pre-commit hook installed"
    return 0
}

setup_desktop() {
    log_info "Setting up desktop integration..."

    # Create directories
    local app_dir="$HOME/.local/share/applications"
    local icon_dir="$HOME/.local/share/icons/hicolor/256x256/apps"
    mkdir -p "$app_dir" "$icon_dir"

    # Copy icon
    local icon_src="$PROJECT_ROOT/keyrx_daemon/assets/icon.png"
    if [[ -f "$icon_src" ]]; then
        cp "$icon_src" "$icon_dir/keyrx.png"
        log_info "Icon installed"
    else
        log_warn "Icon not found at $icon_src"
    fi

    # Create .desktop file
    local desktop_file="$app_dir/keyrx.desktop"
    cat > "$desktop_file" << EOF
[Desktop Entry]
Name=KeyRX
Comment=Keyboard Remapping Daemon
Exec=$HOME/.local/bin/keyrx_daemon run
Icon=keyrx
Terminal=false
Type=Application
Categories=Utility;System;
StartupNotify=false
EOF

    chmod +x "$desktop_file"
    log_info "Desktop file created"

    # Update icon cache
    if command_exists gtk-update-icon-cache; then
        gtk-update-icon-cache "$HOME/.local/share/icons/hicolor/" -f 2>/dev/null || true
        log_info "Icon cache updated"
    fi

    log_info "Desktop integration complete"
    return 0
}

setup_windows_vm() {
    log_info "Setting up Windows VM for testing..."

    # Check Vagrant
    if ! command_exists vagrant; then
        log_error "Vagrant not found. Install from: https://www.vagrantup.com"
        return 1
    fi

    # Check libvirt
    if ! command_exists virsh; then
        log_error "libvirt not found. Install with: sudo apt install libvirt-daemon-system"
        return 1
    fi

    # Check vagrant-libvirt plugin
    if ! vagrant plugin list | grep -q "vagrant-libvirt"; then
        log_info "Installing vagrant-libvirt plugin..."
        vagrant plugin install vagrant-libvirt
    fi

    # Check user is in libvirt group
    if ! groups | grep -q "libvirt"; then
        log_info "Adding user to 'libvirt' group..."
        sudo usermod -aG libvirt "$USER"
        log_warn "You need to log out and back in for group changes to take effect"
    fi

    # Initialize VM if Vagrantfile exists
    local vagrant_dir="$PROJECT_ROOT/vagrant/windows"
    if [[ -d "$vagrant_dir" ]]; then
        log_info "Windows VM configuration found at: $vagrant_dir"
        log_info "To start VM: cd $vagrant_dir && vagrant up"
    else
        log_warn "Windows VM configuration not found"
    fi

    log_info "Windows VM setup complete"
    return 0
}

run_full_setup() {
    log_info "Running full environment setup..."
    log_info "Detected OS: $OSTYPE"
    separator

    local exit_code=0

    # Dev tools (cross-platform)
    log_info "Component 1: Development Tools"
    if ! install_dev_tools; then
        log_warn "Dev tools setup incomplete"
        exit_code=1
    fi
    separator

    # Platform-specific setup
    case "$OSTYPE" in
        darwin*)
            log_info "Component 2: macOS Environment"
            # Note: setup_macos calls install_dev_tools and setup_hooks internally
            # but we've already done dev tools above, so just check permission
            if "$SCRIPT_DIR/platform/macos/check_permission.sh" 2>/dev/null; then
                log_info "Accessibility permission already granted"
            else
                log_warn "Accessibility permission not granted"
                log_info ""
                log_info "To grant Accessibility permission:"
                log_info "  1. Open System Settings"
                log_info "  2. Go to Privacy & Security > Accessibility"
                log_info "  3. Add Terminal (or your IDE) to the list"
                log_info ""
                log_info "Interactive setup: $SCRIPT_DIR/platform/macos/setup_accessibility.sh"
                exit_code=1
            fi
            ;;
        linux*)
            log_info "Component 2: Linux Environment"
            if ! setup_linux; then
                log_warn "Linux setup incomplete"
                exit_code=1
            fi
            separator

            log_info "Component 3: Desktop Integration"
            if ! setup_desktop; then
                log_warn "Desktop setup incomplete"
                exit_code=1
            fi
            ;;
        *)
            log_error "Unsupported OS: $OSTYPE"
            log_error "KeyRX supports Linux and macOS only"
            return 1
            ;;
    esac
    separator

    # Hooks (cross-platform)
    log_info "Component (final): Git Hooks"
    if ! setup_hooks; then
        log_warn "Git hooks setup incomplete"
        exit_code=1
    fi
    separator

    if [[ $exit_code -eq 0 ]]; then
        log_accomplished
        log_info "Full setup complete!"
        if [[ "$OSTYPE" == "linux"* ]]; then
            log_warn "You may need to log out and back in for group changes to take effect"
        fi
    else
        log_warning_marker
        log_info "Setup completed with warnings"
    fi

    return $exit_code
}

# Main execution
main() {
    local exit_code=0

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "setup"
    fi

    separator
    log_info "KeyRX Environment Setup"
    separator

    # Determine what to do
    if [[ "$CHECK_ONLY" == "true" ]]; then
        check_all_status
        exit_code=$?
    elif [[ "$DEV_TOOLS_ONLY" == "true" ]]; then
        install_dev_tools
        exit_code=$?
    elif [[ "$LINUX_ONLY" == "true" ]]; then
        setup_linux
        exit_code=$?
    elif [[ "$MACOS_ONLY" == "true" ]]; then
        setup_macos
        exit_code=$?
    elif [[ "$HOOKS_ONLY" == "true" ]]; then
        setup_hooks
        exit_code=$?
    elif [[ "$DESKTOP_ONLY" == "true" ]]; then
        setup_desktop
        exit_code=$?
    elif [[ "$WINDOWS_VM_ONLY" == "true" ]]; then
        setup_windows_vm
        exit_code=$?
    else
        # Full setup
        run_full_setup
        exit_code=$?
    fi

    # JSON output
    if [[ "$JSON_MODE" == "true" ]]; then
        local status="success"
        if [[ $exit_code -ne 0 ]]; then
            status="failed"
        fi
        output_json "status" "$status" "exit_code" "$exit_code"
    fi

    exit $exit_code
}

# Run main function
main "$@"
