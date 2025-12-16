#!/usr/bin/env bash
# Setup script for KeyRx Linux input permissions.
# - Adds the current user to the input group (for evdev/uinput access)
# - Loads the uinput kernel module
# - Installs recommended udev rules to set permissions on input/uinput devices

set -euo pipefail

RULES_PATH="/etc/udev/rules.d/99-keyrx.rules"

echo "KeyRx Linux input setup"
echo "This script will use sudo to adjust permissions and load uinput."

# 1) Ensure user is in the input group
if id -nG "$USER" | grep -qw input; then
    echo "User '$USER' is already in the input group."
else
    echo "Adding '$USER' to the input group..."
    sudo usermod -aG input "$USER"
    echo "User added. You must log out and back in (or run 'newgrp input') for this to take effect."
fi

# 2) Load uinput module
if lsmod | grep -q '^uinput'; then
    echo "uinput module already loaded."
else
    echo "Loading uinput module..."
    sudo modprobe uinput
fi

# 3) Install udev rules
if [ -f "$RULES_PATH" ]; then
    echo "udev rules already present at $RULES_PATH"
else
    echo "Installing udev rules to $RULES_PATH ..."
    cat <<'EOF' | sudo tee "$RULES_PATH" >/dev/null
# KeyRx input/uinput permissions
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input"
EOF
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    echo "udev rules installed and reloaded."
fi

echo "Done. If you were just added to the input group, log out/in or run 'newgrp input' before using KeyRx."
