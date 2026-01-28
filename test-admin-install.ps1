# Test Script for Admin Auto-Start Installer
# This script verifies the new installer properly sets up auto-start with admin rights

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyRx Admin Install Test Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if installer exists
$installerPath = "target\windows-installer\keyrx_0.1.1.0_x64_setup.exe"
if (-not (Test-Path $installerPath)) {
    Write-Host "[ERROR] Installer not found at: $installerPath" -ForegroundColor Red
    Write-Host "[INFO] Run: cd scripts/package && iscc keyrx-installer.iss" -ForegroundColor Yellow
    exit 1
}

$installerSize = (Get-Item $installerPath).Length / 1MB
Write-Host "[OK] Installer found: $installerPath (${installerSize}MB)" -ForegroundColor Green
Write-Host ""

# Instructions
Write-Host "=== Installation Test Instructions ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "1. Run the installer:" -ForegroundColor White
Write-Host "   .\target\windows-installer\keyrx_0.1.1.0_x64_setup.exe" -ForegroundColor Cyan
Write-Host ""
Write-Host "2. During installation, CHECK these options:" -ForegroundColor White
Write-Host "   [x] Auto-start daemon on Windows login" -ForegroundColor Green
Write-Host "   [x] Launch KeyRx Daemon now" -ForegroundColor Green
Write-Host ""
Write-Host "3. Click Install → UAC will prompt for admin → Approve" -ForegroundColor White
Write-Host ""
Write-Host "4. After install completes, verify:" -ForegroundColor White
Write-Host ""

Write-Host "=== Verification Steps ===" -ForegroundColor Yellow
Write-Host ""

Write-Host "[1] Check Task Scheduler entry:" -ForegroundColor Cyan
Write-Host '    schtasks /Query /TN "KeyRx Daemon" /V /FO LIST' -ForegroundColor Gray
Write-Host ""
Write-Host "    Expected output:" -ForegroundColor White
Write-Host "    - TaskName: \KeyRx Daemon" -ForegroundColor Gray
Write-Host "    - Run With Highest Privileges: Yes" -ForegroundColor Gray
Write-Host "    - Triggers: At log on of any user" -ForegroundColor Gray
Write-Host ""

Write-Host "[2] Check daemon is running:" -ForegroundColor Cyan
Write-Host "    Get-Process keyrx_daemon" -ForegroundColor Gray
Write-Host "    curl http://localhost:9867/api/health" -ForegroundColor Gray
Write-Host ""
Write-Host "    Expected: {`"status`":`"ok`",`"version`":`"0.1.1`"}" -ForegroundColor Gray
Write-Host ""

Write-Host "[3] Test keyboard remapping:" -ForegroundColor Cyan
Write-Host "    a. Open web UI: http://localhost:9867" -ForegroundColor Gray
Write-Host "    b. Go to Profiles page" -ForegroundColor Gray
Write-Host "    c. Activate 'default' profile" -ForegroundColor Gray
Write-Host "    d. Type in any text editor" -ForegroundColor Gray
Write-Host "    e. Verify remapping is working" -ForegroundColor Gray
Write-Host ""

Write-Host "[4] Test auto-start:" -ForegroundColor Cyan
Write-Host "    a. Restart Windows" -ForegroundColor Gray
Write-Host "    b. After login (5 seconds), check:" -ForegroundColor Gray
Write-Host "       Get-Process keyrx_daemon" -ForegroundColor Gray
Write-Host "       curl http://localhost:9867/api/health" -ForegroundColor Gray
Write-Host "    c. Daemon should be running automatically" -ForegroundColor Gray
Write-Host ""

Write-Host "=== Manual Test (Optional) ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "To test without auto-start:" -ForegroundColor White
Write-Host "1. Run installer again" -ForegroundColor Gray
Write-Host "2. UNCHECK 'Auto-start daemon on Windows login'" -ForegroundColor Gray
Write-Host "3. UNCHECK 'Launch KeyRx Daemon now'" -ForegroundColor Gray
Write-Host "4. After install, manually launch: Start Menu → KeyRx Daemon" -ForegroundColor Gray
Write-Host "5. UAC should prompt for admin" -ForegroundColor Gray
Write-Host "6. Verify daemon starts with admin rights" -ForegroundColor Gray
Write-Host ""

Write-Host "=== Troubleshooting ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "If daemon doesn't start:" -ForegroundColor White
Write-Host "1. Check Task Scheduler: schtasks /Query /TN `"KeyRx Daemon`"" -ForegroundColor Gray
Write-Host "2. Check logs: target\release\daemon.log" -ForegroundColor Gray
Write-Host "3. Run manually: keyrx_daemon run" -ForegroundColor Gray
Write-Host "4. Check firewall isn't blocking port 9867" -ForegroundColor Gray
Write-Host ""

Write-Host "If remapping doesn't work:" -ForegroundColor White
Write-Host "1. Verify daemon is running with admin rights" -ForegroundColor Gray
Write-Host "2. Activate a profile in web UI" -ForegroundColor Gray
Write-Host "3. Check /api/status shows active_profile" -ForegroundColor Gray
Write-Host "4. Type slowly and check /api/metrics/events" -ForegroundColor Gray
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Press Enter to open installer location..." -ForegroundColor Yellow
Read-Host

Start-Process "explorer.exe" -ArgumentList "/select,`"$(Resolve-Path $installerPath)`""
