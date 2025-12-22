#!/usr/bin/env bash
#
# setup_linux.sh - Set up Linux environment for keyrx daemon development and testing
#
# This script configures:
#   1. User groups (input, uinput) for non-root device access
#   2. udev rules for device permissions
#   3. uinput kernel module loading
#   4. System dependencies for evdev development
#
# Usage:
#   ./scripts/setup_linux.sh           # Full setup (requires sudo)
#   ./scripts/setup_linux.sh --check   # Check current setup status
#   ./scripts/setup_linux.sh --groups  # Only configure user groups
#   ./scripts/setup_linux.sh --udev    # Only install udev rules
#   ./scripts/setup_linux.sh --deps    # Only install system dependencies
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Log functions
info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Check if running on Linux
check_linux() {
    if [[ "$(uname -s)" != "Linux" ]]; then
        error "This script only works on Linux"
        exit 1
    fi
}

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/fedora-release ]; then
        echo "fedora"
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    else
        echo "unknown"
    fi
}

# Check if user is in a group
user_in_group() {
    local group="$1"
    groups "$USER" 2>/dev/null | grep -qw "$group"
}

# Check if group exists
group_exists() {
    local group="$1"
    getent group "$group" >/dev/null 2>&1
}

# Check current setup status
check_status() {
    info "Checking Linux setup status for keyrx daemon..."
    echo ""

    local all_ok=true

    # Check input group
    if user_in_group "input"; then
        success "User '$USER' is in 'input' group"
    else
        warn "User '$USER' is NOT in 'input' group (needed to read /dev/input/event*)"
        all_ok=false
    fi

    # Check uinput group
    if group_exists "uinput"; then
        if user_in_group "uinput"; then
            success "User '$USER' is in 'uinput' group"
        else
            warn "User '$USER' is NOT in 'uinput' group (needed to create virtual keyboard)"
            all_ok=false
        fi
    else
        warn "'uinput' group does not exist"
        all_ok=false
    fi

    # Check uinput module
    if lsmod | grep -q "^uinput"; then
        success "uinput kernel module is loaded"
    else
        warn "uinput kernel module is NOT loaded"
        all_ok=false
    fi

    # Check /dev/uinput
    if [ -c /dev/uinput ]; then
        success "/dev/uinput device exists"
        if [ -w /dev/uinput ]; then
            success "/dev/uinput is writable by current user"
        else
            warn "/dev/uinput is NOT writable by current user"
            all_ok=false
        fi
    else
        warn "/dev/uinput device does NOT exist"
        all_ok=false
    fi

    # Check /dev/input access
    local input_devices
    input_devices=$(ls /dev/input/event* 2>/dev/null | wc -l)
    if [ "$input_devices" -gt 0 ]; then
        success "Found $input_devices input devices in /dev/input/"

        local readable=0
        for dev in /dev/input/event*; do
            if [ -r "$dev" ]; then
                ((readable++)) || true
            fi
        done

        if [ "$readable" -eq "$input_devices" ]; then
            success "All $input_devices input devices are readable"
        else
            warn "Only $readable of $input_devices input devices are readable"
            all_ok=false
        fi
    else
        warn "No input devices found in /dev/input/"
    fi

    # Check udev rules
    if [ -f /etc/udev/rules.d/99-keyrx.rules ]; then
        success "keyrx udev rules are installed"
    else
        warn "keyrx udev rules are NOT installed"
        all_ok=false
    fi

    # Check system dependencies
    echo ""
    info "Checking system dependencies..."

    local distro
    distro=$(detect_distro)

    case "$distro" in
        ubuntu|debian|pop)
            if dpkg -l libevdev-dev >/dev/null 2>&1; then
                success "libevdev-dev is installed"
            else
                warn "libevdev-dev is NOT installed"
                all_ok=false
            fi
            ;;
        fedora|rhel|centos)
            if rpm -q libevdev-devel >/dev/null 2>&1; then
                success "libevdev-devel is installed"
            else
                warn "libevdev-devel is NOT installed"
                all_ok=false
            fi
            ;;
        arch|manjaro)
            if pacman -Q libevdev >/dev/null 2>&1; then
                success "libevdev is installed"
            else
                warn "libevdev is NOT installed"
                all_ok=false
            fi
            ;;
        *)
            info "Unknown distro, skipping dependency check"
            ;;
    esac

    echo ""
    if $all_ok; then
        success "All checks passed! Ready for keyrx daemon development."
    else
        warn "Some checks failed. Run './scripts/setup_linux.sh' to fix."
        echo ""
        echo "After setup, you may need to:"
        echo "  1. Log out and log back in (for group changes)"
        echo "  2. Or run: newgrp input && newgrp uinput"
    fi

    return 0
}

# Install system dependencies
install_deps() {
    info "Installing system dependencies..."

    local distro
    distro=$(detect_distro)

    case "$distro" in
        ubuntu|debian|pop)
            info "Detected Debian/Ubuntu-based system"
            sudo apt-get update
            sudo apt-get install -y libevdev-dev
            success "Installed libevdev-dev"
            ;;
        fedora|rhel|centos)
            info "Detected Fedora/RHEL-based system"
            sudo dnf install -y libevdev-devel
            success "Installed libevdev-devel"
            ;;
        arch|manjaro)
            info "Detected Arch-based system"
            sudo pacman -S --noconfirm libevdev
            success "Installed libevdev"
            ;;
        *)
            warn "Unknown distribution: $distro"
            warn "Please manually install libevdev development headers"
            return 1
            ;;
    esac
}

# Configure user groups
setup_groups() {
    info "Configuring user groups..."

    # Add user to input group
    if user_in_group "input"; then
        success "User '$USER' already in 'input' group"
    else
        info "Adding user '$USER' to 'input' group..."
        sudo usermod -aG input "$USER"
        success "Added '$USER' to 'input' group"
    fi

    # Create and add user to uinput group
    if ! group_exists "uinput"; then
        info "Creating 'uinput' group..."
        sudo groupadd -f uinput
        success "Created 'uinput' group"
    fi

    if user_in_group "uinput"; then
        success "User '$USER' already in 'uinput' group"
    else
        info "Adding user '$USER' to 'uinput' group..."
        sudo usermod -aG uinput "$USER"
        success "Added '$USER' to 'uinput' group"
    fi

    warn "You must log out and log back in for group changes to take effect"
    warn "Or run: newgrp input && newgrp uinput"
}

# Install udev rules
setup_udev() {
    info "Installing udev rules..."

    local rules_file="/etc/udev/rules.d/99-keyrx.rules"
    local project_udev_file="$PROJECT_ROOT/keyrx_daemon/udev/99-keyrx.rules"

    # Check if the source file exists in the project
    if [ ! -f "$project_udev_file" ]; then
        error "udev rules file not found: $project_udev_file"
        error "Please ensure the keyrx_daemon/udev/99-keyrx.rules file exists in the project"
        return 1
    fi

    # Install rules file to system
    sudo cp "$project_udev_file" "$rules_file"
    success "Installed $rules_file"

    # Reload udev rules
    info "Reloading udev rules..."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    success "udev rules reloaded"
}

# Load uinput module
setup_uinput_module() {
    info "Setting up uinput kernel module..."

    # Load module now
    if ! lsmod | grep -q "^uinput"; then
        info "Loading uinput module..."
        sudo modprobe uinput
        success "Loaded uinput module"
    else
        success "uinput module already loaded"
    fi

    # Configure to load at boot
    local modules_file="/etc/modules-load.d/keyrx.conf"
    if [ ! -f "$modules_file" ]; then
        echo "uinput" | sudo tee "$modules_file" > /dev/null
        success "Configured uinput to load at boot ($modules_file)"
    else
        if grep -q "^uinput$" "$modules_file"; then
            success "uinput already configured to load at boot"
        else
            echo "uinput" | sudo tee -a "$modules_file" > /dev/null
            success "Added uinput to $modules_file"
        fi
    fi
}

# Create systemd service file
create_systemd_service() {
    info "Creating systemd service file..."

    local service_dir="$PROJECT_ROOT/keyrx_daemon/systemd"
    mkdir -p "$service_dir"

    cat > "$service_dir/keyrx.service" << 'EOF'
[Unit]
Description=keyrx Keyboard Remapping Daemon
Documentation=https://github.com/yourusername/keyrx
After=local-fs.target
Wants=local-fs.target

[Service]
Type=simple
# Set config path via environment or modify ExecStart
Environment=KEYRX_CONFIG=/etc/keyrx/config.krx
ExecStart=/usr/local/bin/keyrx_daemon run --config ${KEYRX_CONFIG}
ExecReload=/bin/kill -HUP $MAINPID

# Restart on failure
Restart=on-failure
RestartSec=1
# Limit restart attempts (max 5 in 30 seconds)
StartLimitIntervalSec=30
StartLimitBurst=5

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
# Allow access to input devices
SupplementaryGroups=input uinput
# ReadWritePaths for logging if needed
# ReadWritePaths=/var/log/keyrx

[Install]
WantedBy=multi-user.target
EOF

    success "Created $service_dir/keyrx.service"

    # Also create user service variant
    cat > "$service_dir/keyrx-user.service" << 'EOF'
[Unit]
Description=keyrx Keyboard Remapping Daemon (User Service)
Documentation=https://github.com/yourusername/keyrx

[Service]
Type=simple
ExecStart=%h/.local/bin/keyrx_daemon run --config %h/.config/keyrx/config.krx
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=1

[Install]
WantedBy=default.target
EOF

    success "Created $service_dir/keyrx-user.service"

    echo ""
    info "To install system service:"
    echo "  sudo cp $service_dir/keyrx.service /etc/systemd/system/"
    echo "  sudo systemctl daemon-reload"
    echo "  sudo systemctl enable keyrx"
    echo "  sudo systemctl start keyrx"
    echo ""
    info "To install user service:"
    echo "  mkdir -p ~/.config/systemd/user/"
    echo "  cp $service_dir/keyrx-user.service ~/.config/systemd/user/"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable keyrx-user"
    echo "  systemctl --user start keyrx-user"
}

# Full setup
full_setup() {
    info "Running full Linux setup for keyrx daemon..."
    echo ""

    install_deps
    echo ""

    setup_groups
    echo ""

    setup_uinput_module
    echo ""

    setup_udev
    echo ""

    create_systemd_service
    echo ""

    success "Setup complete!"
    echo ""
    warn "IMPORTANT: You must log out and log back in for group changes to take effect."
    warn "Or run: newgrp input && newgrp uinput"
    echo ""
    info "Run './scripts/setup_linux.sh --check' to verify setup."
}

# Main
main() {
    check_linux

    case "${1:-}" in
        --check|-c)
            check_status
            ;;
        --groups|-g)
            setup_groups
            ;;
        --udev|-u)
            setup_udev
            ;;
        --deps|-d)
            install_deps
            ;;
        --module|-m)
            setup_uinput_module
            ;;
        --service|-s)
            create_systemd_service
            ;;
        --help|-h)
            echo "Usage: $0 [OPTION]"
            echo ""
            echo "Set up Linux environment for keyrx daemon development and testing."
            echo ""
            echo "Options:"
            echo "  (none)      Run full setup (requires sudo)"
            echo "  --check     Check current setup status"
            echo "  --groups    Only configure user groups"
            echo "  --udev      Only install udev rules"
            echo "  --deps      Only install system dependencies"
            echo "  --module    Only load uinput kernel module"
            echo "  --service   Only create systemd service files"
            echo "  --help      Show this help message"
            ;;
        "")
            full_setup
            ;;
        *)
            error "Unknown option: $1"
            echo "Run '$0 --help' for usage."
            exit 1
            ;;
    esac
}

main "$@"
