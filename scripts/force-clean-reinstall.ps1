# Force Clean Reinstall - Complete KeyRx Reinstallation
#
# This script performs a complete clean reinstall of KeyRx:
# - Stops daemon forcefully
# - Uninstalls MSI cleanly
# - Removes all state files
# - Cleans build artifacts
# - Rebuilds UI and daemon
# - Builds fresh installer
# - Installs new MSI
# - Verifies installation
#
# Usage:
#   .\scripts\force-clean-reinstall.ps1                 # Interactive mode
#   .\scripts\force-clean-reinstall.ps1 -Confirm        # Auto-confirm
#   .\scripts\force-clean-reinstall.ps1 -SkipBackup     # Don't backup state
#   .\scripts\force-clean-reinstall.ps1 -NoVerify       # Skip final verification

param(
    [switch]$Confirm,
    [switch]$SkipBackup,
    [switch]$NoVerify,
    [switch]$KeepState
)

$ErrorActionPreference = "Stop"

# =========================
# PREREQUISITE CHECKS
# =========================

function Test-AdminRights {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "===================================================" -ForegroundColor Cyan
    Write-Host " $Message" -ForegroundColor Cyan
    Write-Host "===================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Progress-Status {
    param(
        [string]$Activity,
        [string]$Status,
        [int]$PercentComplete
    )
    Write-Progress -Activity $Activity -Status $Status -PercentComplete $PercentComplete
}

function Write-Success {
    param([string]$Message)
    Write-Host "  ✓ $Message" -ForegroundColor Green
}

function Write-Warning-Message {
    param([string]$Message)
    Write-Host "  ⚠ $Message" -ForegroundColor Yellow
}

function Write-Error-Message {
    param([string]$Message)
    Write-Host "  ✗ $Message" -ForegroundColor Red
}

function Invoke-Rollback {
    param([string]$Reason)
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host " ROLLBACK INITIATED" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Error-Message "Rollback reason: $Reason"
    Write-Host ""
    Write-Host "Rollback actions:" -ForegroundColor Yellow
    Write-Host "  1. Restore state files from backup (if available)" -ForegroundColor White
    Write-Host "  2. Reinstall previous MSI (if available)" -ForegroundColor White
    Write-Host ""
    Write-Host "To restore manually:" -ForegroundColor Cyan
    if (Test-Path "$env:TEMP\keyrx_backup") {
        Write-Host "  Copy-Item -Recurse $env:TEMP\keyrx_backup\* $env:USERPROFILE\.keyrx\" -ForegroundColor White
    }
    Write-Host ""
    exit 1
}

# Header
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " KeyRx Force Clean Reinstall" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check admin rights
if (-not (Test-AdminRights)) {
    Write-Error-Message "NOT running as Administrator"
    Write-Host ""
    Write-Host "This script requires administrator privileges." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To fix:" -ForegroundColor Cyan
    Write-Host "  1. Right-click PowerShell" -ForegroundColor White
    Write-Host "  2. Select 'Run as Administrator'" -ForegroundColor White
    Write-Host "  3. Run this script again" -ForegroundColor White
    Write-Host ""
    exit 1
}

Write-Success "Running with administrator privileges"

# Check required tools
$requiredTools = @(
    @{ Name = "cargo"; Command = "cargo --version" },
    @{ Name = "npm"; Command = "npm --version" }
)

foreach ($tool in $requiredTools) {
    try {
        $null = Invoke-Expression $tool.Command 2>&1
        Write-Success "$($tool.Name) is available"
    } catch {
        Write-Error-Message "$($tool.Name) not found"
        Write-Host ""
        Write-Host "Please install $($tool.Name) and try again." -ForegroundColor Yellow
        exit 1
    }
}

# Confirmation prompt
if (-not $Confirm) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host " WARNING: DESTRUCTIVE OPERATION" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "This script will:" -ForegroundColor Yellow
    Write-Host "  1. Force stop the KeyRx daemon" -ForegroundColor White
    Write-Host "  2. Uninstall KeyRx MSI" -ForegroundColor White
    if (-not $KeepState) {
        Write-Host "  3. DELETE all state files (~/.keyrx/)" -ForegroundColor Red
    } else {
        Write-Host "  3. Keep state files (--KeepState specified)" -ForegroundColor Green
    }
    Write-Host "  4. Clean build artifacts" -ForegroundColor White
    Write-Host "  5. Rebuild UI and daemon" -ForegroundColor White
    Write-Host "  6. Build fresh installer" -ForegroundColor White
    Write-Host "  7. Install new MSI" -ForegroundColor White
    Write-Host ""

    $response = Read-Host "Continue? (yes/no)"
    if ($response -ne "yes") {
        Write-Host ""
        Write-Host "Cancelled by user." -ForegroundColor Yellow
        exit 0
    }
}

# =========================
# STEP 1: STOP DAEMON
# =========================

Write-Step "STEP 1/8: Stopping Daemon"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Stopping daemon..." -PercentComplete 12

$daemonProcess = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue

if ($daemonProcess) {
    Write-Host "  Found daemon process (PID: $($daemonProcess.Id))" -ForegroundColor Gray

    # Try graceful stop first
    Write-Host "  Attempting graceful stop..." -ForegroundColor Gray
    try {
        $daemonProcess | Stop-Process -ErrorAction Stop
        Start-Sleep -Seconds 2
    } catch {
        Write-Warning-Message "Graceful stop failed, using force"
    }

    # Force kill if still running
    $daemonProcess = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($daemonProcess) {
        Write-Host "  Force stopping..." -ForegroundColor Gray
        try {
            $daemonProcess | Stop-Process -Force -ErrorAction Stop
            Start-Sleep -Seconds 1
        } catch {
            Write-Warning-Message "Force stop failed: $($_.Exception.Message)"
        }
    }

    # Verify stopped
    $daemonProcess = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($daemonProcess) {
        Invoke-Rollback "Failed to stop daemon process (PID: $($daemonProcess.Id))"
    }

    Write-Success "Daemon stopped successfully"
} else {
    Write-Success "Daemon not running"
}

# =========================
# STEP 2: BACKUP STATE
# =========================

if (-not $SkipBackup -and -not $KeepState) {
    Write-Step "STEP 2/8: Backing Up State"
    Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Backing up state..." -PercentComplete 25

    $stateDir = Join-Path $env:USERPROFILE ".keyrx"
    $backupDir = Join-Path $env:TEMP "keyrx_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"

    if (Test-Path $stateDir) {
        try {
            Write-Host "  Backing up $stateDir to $backupDir" -ForegroundColor Gray
            Copy-Item -Recurse -Force $stateDir $backupDir
            Write-Success "State backed up to: $backupDir"
        } catch {
            Write-Warning-Message "Backup failed: $($_.Exception.Message)"
            Write-Host "  Continuing without backup..." -ForegroundColor Yellow
        }
    } else {
        Write-Success "No state directory found (fresh install)"
    }
} else {
    Write-Step "STEP 2/8: Backup Skipped"
    Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Skipping backup..." -PercentComplete 25
    if ($KeepState) {
        Write-Success "State will be preserved (--KeepState)"
    } else {
        Write-Success "Backup skipped (--SkipBackup)"
    }
}

# =========================
# STEP 3: UNINSTALL MSI
# =========================

Write-Step "STEP 3/8: Uninstalling MSI"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Uninstalling..." -PercentComplete 37

# Find installed KeyRx product
$installedProduct = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "KeyRx*" }

if ($installedProduct) {
    Write-Host "  Found installed product: $($installedProduct.Name) v$($installedProduct.Version)" -ForegroundColor Gray

    try {
        Write-Host "  Uninstalling..." -ForegroundColor Gray
        $installedProduct.Uninstall() | Out-Null
        Start-Sleep -Seconds 3
        Write-Success "MSI uninstalled successfully"
    } catch {
        Write-Warning-Message "MSI uninstall failed: $($_.Exception.Message)"
        Write-Host "  Attempting direct msiexec uninstall..." -ForegroundColor Yellow

        $productCode = $installedProduct.IdentifyingNumber
        $uninstallResult = Start-Process msiexec.exe -ArgumentList "/x $productCode /qn" -Wait -PassThru

        if ($uninstallResult.ExitCode -eq 0) {
            Write-Success "MSI uninstalled via msiexec"
        } else {
            Write-Warning-Message "msiexec uninstall returned code: $($uninstallResult.ExitCode)"
            Write-Host "  Continuing anyway..." -ForegroundColor Yellow
        }
    }
} else {
    Write-Success "No MSI installation found"
}

# =========================
# STEP 4: REMOVE STATE
# =========================

if (-not $KeepState) {
    Write-Step "STEP 4/8: Removing State Files"
    Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Removing state..." -PercentComplete 50

    $stateDir = Join-Path $env:USERPROFILE ".keyrx"

    if (Test-Path $stateDir) {
        try {
            Write-Host "  Removing $stateDir" -ForegroundColor Gray
            Remove-Item -Recurse -Force $stateDir
            Write-Success "State files removed"
        } catch {
            Write-Warning-Message "Failed to remove state files: $($_.Exception.Message)"
            Write-Host "  You may need to remove manually: $stateDir" -ForegroundColor Yellow
        }
    } else {
        Write-Success "No state files found"
    }
} else {
    Write-Step "STEP 4/8: Preserving State Files"
    Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Preserving state..." -PercentComplete 50
    Write-Success "State files preserved"
}

# =========================
# STEP 5: CLEAN BUILD
# =========================

Write-Step "STEP 5/8: Cleaning Build Artifacts"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Cleaning build..." -PercentComplete 62

try {
    Write-Host "  Running: cargo clean" -ForegroundColor Gray
    $cleanResult = cargo clean 2>&1
    Write-Success "Build artifacts cleaned"
} catch {
    Write-Warning-Message "cargo clean failed: $($_.Exception.Message)"
    Write-Host "  Continuing anyway..." -ForegroundColor Yellow
}

# Clean UI build artifacts
$uiBuildDir = "keyrx_ui\dist"
if (Test-Path $uiBuildDir) {
    try {
        Write-Host "  Removing $uiBuildDir" -ForegroundColor Gray
        Remove-Item -Recurse -Force $uiBuildDir
        Write-Success "UI build artifacts cleaned"
    } catch {
        Write-Warning-Message "Failed to clean UI build: $($_.Exception.Message)"
    }
}

# Clean installer artifacts
$installerDir = "target\installer"
if (Test-Path $installerDir) {
    try {
        Write-Host "  Removing $installerDir" -ForegroundColor Gray
        Remove-Item -Recurse -Force $installerDir
        Write-Success "Installer artifacts cleaned"
    } catch {
        Write-Warning-Message "Failed to clean installer: $($_.Exception.Message)"
    }
}

# =========================
# STEP 6: REBUILD UI
# =========================

Write-Step "STEP 6/8: Rebuilding UI"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Building UI..." -PercentComplete 75

try {
    Write-Host "  Changing to keyrx_ui directory..." -ForegroundColor Gray
    Push-Location keyrx_ui

    Write-Host "  Running: npm run build" -ForegroundColor Gray
    $npmBuildResult = npm run build 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Success "UI built successfully"
    } else {
        throw "npm build failed with exit code $LASTEXITCODE"
    }
} catch {
    Pop-Location
    Invoke-Rollback "UI build failed: $($_.Exception.Message)"
} finally {
    Pop-Location
}

# =========================
# STEP 7: REBUILD DAEMON & INSTALLER
# =========================

Write-Step "STEP 7/8: Rebuilding Daemon and Installer"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Building daemon..." -PercentComplete 87

try {
    Write-Host "  Running: cargo build --release -p keyrx_daemon" -ForegroundColor Gray
    $cargoBuildResult = cargo build --release -p keyrx_daemon 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Success "Daemon built successfully"
    } else {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
} catch {
    Invoke-Rollback "Daemon build failed: $($_.Exception.Message)"
}

# Verify binary exists
$daemonBinary = "target\release\keyrx_daemon.exe"
if (-not (Test-Path $daemonBinary)) {
    Invoke-Rollback "Daemon binary not found at: $daemonBinary"
}

Write-Host "  Binary size: $([math]::Round((Get-Item $daemonBinary).Length / 1MB, 2)) MB" -ForegroundColor Gray

# Build installer
try {
    Write-Host "  Building MSI installer..." -ForegroundColor Gray
    $installerScript = "scripts\build_windows_installer.ps1"

    if (Test-Path $installerScript) {
        & $installerScript

        if ($LASTEXITCODE -eq 0) {
            Write-Success "Installer built successfully"
        } else {
            throw "Installer build failed with exit code $LASTEXITCODE"
        }
    } else {
        throw "Installer build script not found: $installerScript"
    }
} catch {
    Invoke-Rollback "Installer build failed: $($_.Exception.Message)"
}

# =========================
# STEP 8: INSTALL MSI
# =========================

Write-Step "STEP 8/8: Installing MSI"
Write-Progress-Status -Activity "Force Clean Reinstall" -Status "Installing..." -PercentComplete 100

# Find MSI file
$msiPath = Get-ChildItem "target\installer\KeyRx-*.msi" | Select-Object -First 1

if (-not $msiPath) {
    Invoke-Rollback "MSI file not found in target\installer\"
}

Write-Host "  Installing: $($msiPath.Name)" -ForegroundColor Gray
Write-Host "  Size: $([math]::Round($msiPath.Length / 1MB, 2)) MB" -ForegroundColor Gray

try {
    $installArgs = "/i `"$($msiPath.FullName)`" /qn /l*v `"$env:TEMP\keyrx_install.log`""
    Write-Host "  Running: msiexec $installArgs" -ForegroundColor Gray

    $installProcess = Start-Process msiexec.exe -ArgumentList $installArgs -Wait -PassThru

    if ($installProcess.ExitCode -eq 0) {
        Write-Success "MSI installed successfully"
    } else {
        throw "msiexec returned exit code $($installProcess.ExitCode). Check log: $env:TEMP\keyrx_install.log"
    }
} catch {
    Invoke-Rollback "MSI installation failed: $($_.Exception.Message)"
}

# =========================
# VERIFICATION
# =========================

if (-not $NoVerify) {
    Write-Step "Verification"

    # Check if installer-health-check.ps1 exists
    $healthCheckScript = "scripts\installer-health-check.ps1"

    if (Test-Path $healthCheckScript) {
        Write-Host "  Running post-install health check..." -ForegroundColor Gray
        Write-Host ""

        try {
            & $healthCheckScript -PostInstall

            if ($LASTEXITCODE -eq 0) {
                Write-Host ""
                Write-Success "Installation verified successfully"
            } else {
                Write-Host ""
                Write-Warning-Message "Health check reported issues (see above)"
            }
        } catch {
            Write-Warning-Message "Health check failed: $($_.Exception.Message)"
        }
    } else {
        Write-Warning-Message "Health check script not found: $healthCheckScript"

        # Basic verification
        $installedBinary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
        if (Test-Path $installedBinary) {
            Write-Success "Binary installed at: $installedBinary"
        } else {
            Write-Error-Message "Binary NOT found at: $installedBinary"
        }
    }
} else {
    Write-Step "Verification Skipped"
    Write-Success "Installation complete (--NoVerify specified)"
}

# =========================
# COMPLETION
# =========================

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host " FORCE CLEAN REINSTALL COMPLETE" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Start the daemon:" -ForegroundColor White
Write-Host "     & 'C:\Program Files\KeyRx\bin\keyrx_daemon.exe' run" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. Open the web UI:" -ForegroundColor White
Write-Host "     http://localhost:9867" -ForegroundColor Gray
Write-Host ""

if (-not $SkipBackup -and (Test-Path "$env:TEMP\keyrx_backup*")) {
    Write-Host "  Backup location:" -ForegroundColor Yellow
    $backups = Get-ChildItem "$env:TEMP\keyrx_backup*" -Directory | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    Write-Host "     $($backups.FullName)" -ForegroundColor Gray
    Write-Host ""
}

exit 0
