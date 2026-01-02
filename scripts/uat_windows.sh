#!/usr/bin/env bash
# uat_windows.sh - User Acceptance Test setup for Windows VM
#
# Usage: ./scripts/uat_windows.sh
#
# This script:
# 1. Ensures Windows VM is running
# 2. Builds release binaries (compiler + daemon)
# 3. Installs to C:\Program Files\keyrx
# 4. Creates launch.bat for compile -> krx -> daemon workflow
# 5. Copies sample user_layout.rhai

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VAGRANT_DIR="$PROJECT_ROOT/vagrant/windows"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Helper to run vagrant commands and filter fog warnings
vagrant_quiet() {
    # Use PIPESTATUS to preserve vagrant's exit code
    set +e
    local output
    output=$(vagrant "$@" 2>&1)
    local exit_code=$?
    set -e
    echo "$output" | grep -v "\[fog\]\[WARNING\] Unrecognized arguments" || true
    return $exit_code
}

# Check if VM is running
check_vm_running() {
    cd "$VAGRANT_DIR"
    local status=$(vagrant status --machine-readable 2>/dev/null | grep "state," | cut -d',' -f4)
    [[ "$status" == "running" ]]
}

# Start VM if not running
ensure_vm_running() {
    log_step "Checking Windows VM status..."

    if check_vm_running; then
        log_info "VM is already running"
    else
        log_warn "VM is not running, starting..."
        cd "$VAGRANT_DIR"
        vagrant_quiet up
        log_info "VM started successfully"
    fi
}

# Sync files to VM
sync_files() {
    log_step "Syncing project files to VM..."
    cd "$VAGRANT_DIR"
    vagrant_quiet rsync
    log_info "Files synced"
}

# Build UI on Linux (faster than building on Windows)
build_ui_linux() {
    log_step "Checking UI build..."
    cd "$PROJECT_ROOT"

    # Check if UI is already built
    if [ -d "keyrx_ui/dist" ] && [ -f "keyrx_ui/dist/index.html" ]; then
        log_info "UI already built, using existing dist"
        return 0
    fi

    log_warn "UI not built yet. Building now (this may take a few minutes)..."

    # Build using the complete build script
    ./scripts/build.sh --release --quiet || {
        log_error "Build failed. Please run './scripts/build.sh --release' manually to see errors."
        exit 1
    }

    log_info "Build completed successfully"
}

# Build release binaries on Windows
build_release() {
    log_step "Building release binaries on Windows..."
    cd "$VAGRANT_DIR"

    log_info "Building keyrx_compiler..."
    # Cargo outputs to stderr, so we can't rely on exit codes. Check if binary exists instead.
    vagrant_quiet winrm -c 'cd C:\vagrant_project; cargo build -p keyrx_compiler --release 2>&1 | Out-Null; if (Test-Path "target\release\keyrx_compiler.exe") { exit 0 } else { exit 1 }' || {
        log_error "Compiler build failed - binary not found"
        exit 1
    }

    log_info "Building keyrx_daemon (with embedded UI)..."
    vagrant_quiet winrm -c 'cd C:\vagrant_project; cargo build -p keyrx_daemon --features windows --release 2>&1 | Out-Null; if (Test-Path "target\release\keyrx_daemon.exe") { exit 0 } else { exit 1 }' || {
        log_error "Daemon build failed - binary not found"
        exit 1
    }

    log_info "Build completed successfully"
}

# Create installation directory structure
create_install_dirs() {
    log_step "Creating installation directories..."
    cd "$VAGRANT_DIR"

    vagrant_quiet winrm -c 'New-Item -ItemType Directory -Path "C:\Program Files\keyrx" -Force | Out-Null; New-Item -ItemType Directory -Path "C:\Program Files\keyrx\bin" -Force | Out-Null; New-Item -ItemType Directory -Path "C:\Program Files\keyrx\config" -Force | Out-Null; Write-Host "Directories created"' || {
        log_error "Failed to create installation directories"
        exit 1
    }

    log_info "Installation directories created"
}

# Install binaries
install_binaries() {
    log_step "Installing binaries..."
    cd "$VAGRANT_DIR"

    vagrant_quiet winrm -c 'Copy-Item -Path "C:\vagrant_project\target\release\keyrx_compiler.exe" -Destination "C:\Program Files\keyrx\bin\keyrx_compiler.exe" -Force; Copy-Item -Path "C:\vagrant_project\target\release\keyrx_daemon.exe" -Destination "C:\Program Files\keyrx\bin\keyrx_daemon.exe" -Force; Write-Host "Binaries installed"' || {
        log_error "Failed to install binaries"
        exit 1
    }

    log_info "Binaries installed to C:\Program Files\keyrx\bin"
}

# Create launch.bat
create_launch_bat() {
    log_step "Creating launch.bat..."
    cd "$VAGRANT_DIR"

    # Create the batch file content
    cat > /tmp/launch.bat << 'EOFBAT'
@echo off
REM launch.bat - Compile and run keyrx daemon
REM Usage: launch.bat [config_file.rhai]

setlocal EnableDelayedExpansion

set INSTALL_DIR=C:\Program Files\keyrx
set COMPILER=%INSTALL_DIR%\bin\keyrx_compiler.exe
set DAEMON=%INSTALL_DIR%\bin\keyrx_daemon.exe
set CONFIG_DIR=%INSTALL_DIR%\config

REM Default to user_layout.rhai if no argument provided
if "%~1"=="" (
    set RHAI_FILE=%CONFIG_DIR%\user_layout.rhai
) else (
    set RHAI_FILE=%~1
)

set KRX_FILE=%CONFIG_DIR%\user_layout.krx

echo ========================================
echo   KeyRX Launcher
echo ========================================
echo.

REM Check if Rhai file exists
if not exist "%RHAI_FILE%" (
    echo [ERROR] Config file not found: %RHAI_FILE%
    echo.
    echo Please create a config file or specify a valid path:
    echo   launch.bat path\to\config.rhai
    exit /b 1
)

echo [1/3] Compiling configuration...
echo   Input:  %RHAI_FILE%
echo   Output: %KRX_FILE%
echo.

"%COMPILER%" compile "%RHAI_FILE%" -o "%KRX_FILE%"
if errorlevel 1 (
    echo [ERROR] Compilation failed
    pause
    exit /b 1
)

echo [OK] Compilation successful
echo.

echo [2/3] Starting keyrx daemon...
echo   Config: %KRX_FILE%
echo.
echo Press Ctrl+C to stop the daemon
echo.

"%DAEMON%" run --config "%KRX_FILE%"
if errorlevel 1 (
    echo.
    echo [ERROR] Daemon failed to start
    pause
    exit /b 1
)

echo.
echo [3/3] Daemon stopped
pause
EOFBAT

    # Upload to VM
    vagrant_quiet upload /tmp/launch.bat "C:/Program Files/keyrx/launch.bat"

    log_info "launch.bat created at C:\Program Files\keyrx\launch.bat"
}

# Copy sample user_layout.rhai
copy_sample_config() {
    log_step "Copying sample configuration..."
    cd "$VAGRANT_DIR"

    # Upload the sample config
    vagrant_quiet upload "$PROJECT_ROOT/examples/user_layout.rhai" "C:/Program Files/keyrx/config/user_layout.rhai"

    log_info "Sample config copied to C:\Program Files\keyrx\config\user_layout.rhai"
}

# Create desktop shortcut (optional)
create_shortcut() {
    log_step "Creating desktop shortcut..."
    cd "$VAGRANT_DIR"

    vagrant_quiet winrm -c '$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\KeyRX.lnk"); $Shortcut.TargetPath = "C:\Program Files\keyrx\launch.bat"; $Shortcut.WorkingDirectory = "C:\Program Files\keyrx"; $Shortcut.Description = "Start KeyRX Daemon"; $Shortcut.Save(); Write-Host "Desktop shortcut created"' || {
        log_warn "Failed to create desktop shortcut (non-critical)"
    }

    log_info "Desktop shortcut created"
}

# Verify installation
verify_installation() {
    log_step "Verifying installation..."
    cd "$VAGRANT_DIR"

    # Check each file individually
    local files=(
        "C:\Program Files\keyrx\bin\keyrx_compiler.exe"
        "C:\Program Files\keyrx\bin\keyrx_daemon.exe"
        "C:\Program Files\keyrx\launch.bat"
        "C:\Program Files\keyrx\config\user_layout.rhai"
    )

    local all_exist=true
    for file in "${files[@]}"; do
        if vagrant_quiet winrm -c "Test-Path '$file'" 2>/dev/null | grep -q "True"; then
            log_info "[OK] $file"
        else
            log_error "[MISSING] $file"
            all_exist=false
        fi
    done

    if [ "$all_exist" = true ]; then
        log_info "Installation verified successfully"
    else
        log_error "Installation verification failed - some files missing"
        exit 1
    fi
}

# Print usage instructions
print_instructions() {
    echo ""
    echo "=========================================="
    echo "  UAT Setup Complete!"
    echo "=========================================="
    echo ""
    echo "Installation path: C:\Program Files\keyrx"
    echo ""
    echo "Files installed:"
    echo "  - C:\Program Files\keyrx\bin\keyrx_compiler.exe"
    echo "  - C:\Program Files\keyrx\bin\keyrx_daemon.exe"
    echo "  - C:\Program Files\keyrx\launch.bat"
    echo "  - C:\Program Files\keyrx\config\user_layout.rhai"
    echo ""
    echo "To test:"
    echo ""
    echo "  1. Connect to VM via RDP:"
    echo "     xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant"
    echo ""
    echo "  2. Double-click 'KeyRX' shortcut on desktop"
    echo "     OR"
    echo "     Open PowerShell and run:"
    echo "     cd 'C:\Program Files\keyrx'"
    echo "     .\launch.bat"
    echo ""
    echo "  3. Access web UI:"
    echo "     - From Windows VM: http://localhost:9867"
    echo "     - From Linux host: http://localhost:9867"
    echo ""
    echo "  4. Edit config:"
    echo "     notepad 'C:\Program Files\keyrx\config\user_layout.rhai'"
    echo "     Then run launch.bat again"
    echo ""
    echo "To run from command line:"
    echo "  vagrant winrm -c 'cd \"C:\Program Files\keyrx\"; .\launch.bat'"
    echo ""
}

# Main
main() {
    cd "$PROJECT_ROOT"

    echo "=========================================="
    echo "  KeyRX Windows UAT Setup"
    echo "=========================================="
    echo ""

    ensure_vm_running
    build_ui_linux
    sync_files
    build_release
    create_install_dirs
    install_binaries
    create_launch_bat
    copy_sample_config
    create_shortcut
    verify_installation
    print_instructions
}

main "$@"
