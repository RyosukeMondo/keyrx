#!/bin/bash
set -e

# Script to run tests in the Windows Vagrant VM

# Check libvirt permissions
if ! groups | grep -q "libvirt" && ! sudo -n true 2>/dev/null; then
    # If not in group and no passwordless sudo (though we prefer not to use sudo for vagrant)
    # Actually just check if we can access the socket if it exists
    if [ -e /var/run/libvirt/libvirt-sock ] && [ ! -w /var/run/libvirt/libvirt-sock ]; then
        echo "Error: No permission to access /var/run/libvirt/libvirt-sock."
        echo "Please ensure you are in the 'libvirt' group:"
        echo "  sudo usermod -aG libvirt \$USER"
        echo "Then log out and log back in."
        exit 1
    fi
fi

VAGRANT_DIR="vagrant/windows"
PROJECT_ROOT=$(pwd)

if [ ! -d "$VAGRANT_DIR" ]; then
    echo "Error: Vagrant directory $VAGRANT_DIR not found."
    echo "Please run this script from the project root."
    exit 1
fi

cd "$VAGRANT_DIR"

# Check status
STATUS=$(vagrant status --machine-readable | grep ",state," | cut -d, -f4)

if [ "$STATUS" != "running" ]; then
    echo "Windows VM is not running (Status: $STATUS)."
    echo "Starting VM... (this may take a while if it's the first time)"
    vagrant up
else
    echo "Windows VM is running."
fi

# Sync files
echo "Syncing files to VM..."
vagrant rsync

# Run tests
echo "Running Windows tests..."
# Using winrm for better reliability on Windows guests. 
# We run cargo test for keyrx_daemon with windows feature.
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows'

# Also run UAT script if requested
if [ "$1" == "--uat" ]; then
    echo "Running UAT script..."
    vagrant winrm -c 'cd C:\vagrant_project; powershell -ExecutionPolicy Bypass -File scripts\windows\UAT.ps1'
fi

echo "Done."
