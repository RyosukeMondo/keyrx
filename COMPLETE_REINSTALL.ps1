# Complete Reinstall of KeyRx v0.1.3 with Thread Safety Fix
# This script thoroughly uninstalls and reinstalls KeyRx

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " KeyRx v0.1.3 Complete Reinstall" -ForegroundColor Cyan
Write-Host " (Thread Safety Fix)" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$newInstallerPath = "$PSScriptRoot\target\installer\KeyRx-0.1.3-x64.msi"

if (-not (Test-Path $newInstallerPath)) {
    Write-Host "ERROR: New installer not found at: $newInstallerPath" -ForegroundColor Red
    exit 1
}

Write-Host "Found new installer: KeyRx-0.1.3-x64.msi" -ForegroundColor Green
Write-Host ""

# Step 1: Kill any running KeyRx processes
Write-Host "Step 1: Stopping all KeyRx processes..." -ForegroundColor Yellow
$processes = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
if ($processes) {
    foreach ($proc in $processes) {
        Write-Host "  Killing process PID: $($proc.Id)" -ForegroundColor Gray
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
    Start-Sleep -Seconds 2
    Write-Host "  All processes stopped." -ForegroundColor Green
} else {
    Write-Host "  No running processes found." -ForegroundColor Gray
}

Write-Host ""

# Step 2: Uninstall ALL versions of KeyRx
Write-Host "Step 2: Uninstalling ALL existing versions..." -ForegroundColor Yellow

$uninstallKeys = @(
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
)

$foundAny = $false
foreach ($key in $uninstallKeys) {
    $apps = Get-ItemProperty $key -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -like "*KeyRx*" }

    foreach ($app in $apps) {
        $foundAny = $true
        Write-Host "  Found: $($app.DisplayName) (Version: $($app.DisplayVersion))" -ForegroundColor Gray

        # Try uninstall string first
        if ($app.UninstallString) {
            Write-Host "  Uninstalling via UninstallString..." -ForegroundColor Gray
            $uninstallCmd = $app.UninstallString -replace "msiexec.exe", "" -replace "/I", "/X"
            Start-Process "msiexec.exe" -ArgumentList "$uninstallCmd /qn" -Wait -NoNewWindow
        }
    }
}

# Also try with old installer filenames
$oldInstallers = @(
    "$PSScriptRoot\target\installer\KeyRx-0.1.0-x64.msi",
    "$PSScriptRoot\target\installer\KeyRx-0.1.2-x64.msi"
)

foreach ($oldInstaller in $oldInstallers) {
    if (Test-Path $oldInstaller) {
        Write-Host "  Uninstalling via $($oldInstaller | Split-Path -Leaf)..." -ForegroundColor Gray
        Start-Process "msiexec.exe" -ArgumentList "/x", "`"$oldInstaller`"", "/qn" -Wait -NoNewWindow
    }
}

if (-not $foundAny) {
    Write-Host "  No existing installations found." -ForegroundColor Gray
}

Write-Host "  Uninstall complete." -ForegroundColor Green
Start-Sleep -Seconds 2

Write-Host ""

# Step 3: Clean up any leftover files
Write-Host "Step 3: Cleaning up leftover files..." -ForegroundColor Yellow
$installDir = "C:\Program Files\KeyRx"
if (Test-Path $installDir) {
    Write-Host "  Removing: $installDir" -ForegroundColor Gray
    Remove-Item -Path $installDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  Cleanup complete." -ForegroundColor Green
} else {
    Write-Host "  No leftover files found." -ForegroundColor Gray
}

Write-Host ""

# Step 4: Install new version
Write-Host "Step 4: Installing KeyRx v0.1.3..." -ForegroundColor Yellow
Write-Host "  This version includes the CRITICAL thread safety fix" -ForegroundColor Cyan
Write-Host "  The installer dialog will appear..." -ForegroundColor Gray
Write-Host ""

Start-Process "msiexec.exe" -ArgumentList "/i", "`"$newInstallerPath`"" -Wait

Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host " Installation Complete!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""

# Wait for daemon to start
Write-Host "Waiting for daemon to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

$daemon = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "✓ Daemon is running (PID: $($daemon.Id))" -ForegroundColor Green
} else {
    Write-Host "⚠ Daemon not detected. It may take a moment to start." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "CRITICAL: Please verify the fixes worked!" -ForegroundColor Red
Write-Host ""
Write-Host "Step 1: Check Build Date (MUST be different):" -ForegroundColor Cyan
Write-Host "  - Right-click system tray icon → About" -ForegroundColor White
Write-Host "  - Build date should show: 2026-01-29 XX:XX JST" -ForegroundColor White
Write-Host "  - If it shows yesterday's date, the fix didn't install!" -ForegroundColor Yellow
Write-Host ""
Write-Host "Step 2: Test Keys in Notepad:" -ForegroundColor Cyan
Write-Host "  W → should output: a (no tab!)" -ForegroundColor White
Write-Host "  E → should output: o (no cascade!)" -ForegroundColor White
Write-Host "  O → should output: t (no cascade!)" -ForegroundColor White
Write-Host ""
Write-Host "Step 3: Test Config Page (NEW FIX):" -ForegroundColor Cyan
Write-Host "  1. Open Web UI: http://localhost:9867" -ForegroundColor White
Write-Host "  2. Go to Profiles page" -ForegroundColor White
Write-Host "  3. Click 'Activate' on default profile" -ForegroundColor White
Write-Host "  4. IMMEDIATELY click on profile name to open config page" -ForegroundColor White
Write-Host "  5. Config page should load INSTANTLY (< 1 second)" -ForegroundColor White
Write-Host "  ❌ OLD: Would freeze/timeout after 5+ seconds" -ForegroundColor Gray
Write-Host "  ✅ NEW: Loads instantly, shows Rhai source code" -ForegroundColor Green
Write-Host ""
Write-Host "Step 4: If STILL broken, check log:" -ForegroundColor Cyan
Write-Host "  Location: C:\Users\$env:USERNAME\.keyrx\daemon.log" -ForegroundColor White
Write-Host "  Look for: '✓ Initialized global blocker state'" -ForegroundColor White
Write-Host "           '✋ BLOCKED scan code: 0x0011 (press)'" -ForegroundColor White
Write-Host "           'spawn_blocking: Starting profile activation'" -ForegroundColor White
Write-Host ""
Write-Host "If issues persist, see:" -ForegroundColor Yellow
Write-Host "  - CRITICAL_FIX_v0.1.2.md (thread safety fix)" -ForegroundColor White
Write-Host "  - CRITICAL_FIX_v0.1.3_CONFIG_FREEZE.md (async runtime fix)" -ForegroundColor White
Write-Host ""
