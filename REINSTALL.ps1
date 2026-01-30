# Reinstall KeyRx v0.1.2 with Thread Safety Fix
# This script uninstalls the old version and installs the new one

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " KeyRx v0.1.2 Reinstallation Script" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$installerPath = "$PSScriptRoot\target\installer\KeyRx-0.1.0-x64.msi"

if (-not (Test-Path $installerPath)) {
    Write-Host "ERROR: Installer not found at: $installerPath" -ForegroundColor Red
    Write-Host "Please build the installer first with: scripts\build_windows_installer.ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host "Step 1: Checking for existing installation..." -ForegroundColor Yellow

# Check if KeyRx is installed
$installed = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" |
    Where-Object { $_.DisplayName -like "*KeyRx*" }

if ($installed) {
    Write-Host "Found existing KeyRx installation: $($installed.DisplayName)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Step 2: Stopping KeyRx daemon..." -ForegroundColor Yellow

    # Try to stop via tray icon or kill process
    $keyrxProcess = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
    if ($keyrxProcess) {
        Write-Host "Stopping keyrx_daemon process (PID: $($keyrxProcess.Id))..." -ForegroundColor Yellow
        Stop-Process -Name "keyrx_daemon" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }

    Write-Host ""
    Write-Host "Step 3: Uninstalling old version..." -ForegroundColor Yellow
    Write-Host "This may take a minute..." -ForegroundColor Gray

    # Uninstall silently
    Start-Process "msiexec.exe" -ArgumentList "/x", $installerPath, "/qn" -Wait -NoNewWindow

    Write-Host "Old version uninstalled." -ForegroundColor Green
    Start-Sleep -Seconds 2
} else {
    Write-Host "No existing installation found." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Step 4: Installing KeyRx v0.1.2 (Thread Safety Fix)..." -ForegroundColor Yellow
Write-Host "The installer dialog will appear. Please follow the prompts." -ForegroundColor Gray
Write-Host ""

# Install with UI
Start-Process "msiexec.exe" -ArgumentList "/i", $installerPath -Wait

Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host " Installation Complete!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Cyan
Write-Host "1. Check system tray for KeyRx icon" -ForegroundColor White
Write-Host "2. Right-click tray icon -> About -> Verify build time shows TODAY (JST)" -ForegroundColor White
Write-Host "3. Open Web UI -> Activate profile" -ForegroundColor White
Write-Host "4. Test keys (W, E, O) in Notepad" -ForegroundColor White
Write-Host ""
Write-Host "Expected Results:" -ForegroundColor Cyan
Write-Host "- W -> a (single character, no tab)" -ForegroundColor White
Write-Host "- E -> o (single character, no cascade)" -ForegroundColor White
Write-Host "- O -> t (single character, no cascade)" -ForegroundColor White
Write-Host ""
Write-Host "If still broken, check log at:" -ForegroundColor Yellow
Write-Host "C:\Users\$env:USERNAME\.keyrx\daemon.log" -ForegroundColor White
Write-Host ""
