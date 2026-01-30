# Install Latest Build - v0.1.4
# This script ensures you get the LATEST compiled daemon

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " Installing Latest Build" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Verify we have latest binaries
$daemonBinary = "target\release\keyrx_daemon.exe"
if (-not (Test-Path $daemonBinary)) {
    Write-Host "ERROR: Release binary not found!" -ForegroundColor Red
    Write-Host "Run: cargo build --release" -ForegroundColor Yellow
    exit 1
}

$buildTime = (Get-Item $daemonBinary).LastWriteTime
Write-Host "Daemon binary built: $buildTime" -ForegroundColor Gray

# Step 2: Stop running daemon
Write-Host "Stopping daemon..." -ForegroundColor Yellow
Stop-Process -Name keyrx_daemon -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# Step 3: Install new version (MajorUpgrade will auto-uninstall old version)
Write-Host "Installing new version..." -ForegroundColor Yellow
$installerPath = "target\installer\KeyRx-0.1.4-x64.msi"
if (-not (Test-Path $installerPath)) {
    Write-Host "ERROR: Installer not found!" -ForegroundColor Red
    Write-Host "Run: .\scripts\build_windows_installer.ps1" -ForegroundColor Yellow
    exit 1
}
Start-Process msiexec -ArgumentList "/i","$installerPath","/qn" -Wait -NoNewWindow
Start-Sleep -Seconds 5

# Step 4: Verify daemon started (installer should auto-start)
Write-Host "Verifying daemon..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
$daemonProc = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if (-not $daemonProc) {
    Write-Host "  Daemon not auto-started, starting manually..." -ForegroundColor Yellow
    Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"
    Start-Sleep -Seconds 5
}

# Step 5: Verify installation
Write-Host ""
Write-Host "Verifying installation..." -ForegroundColor Yellow

$daemonProc = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemonProc) {
    Write-Host "  ✓ Daemon is running (PID: $($daemonProc.Id))" -ForegroundColor Green
} else {
    Write-Host "  ✗ Daemon is NOT running!" -ForegroundColor Red
    exit 1
}

# Test API
try {
    $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  ✓ API is responding" -ForegroundColor Green
} catch {
    Write-Host "  ✗ API not responding: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "  Daemon may still be starting. Wait 10 seconds and check http://localhost:9867" -ForegroundColor Yellow
    exit 1
}

# Get installed version info
$installedBinary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$installedTime = (Get-Item $installedBinary).LastWriteTime
Write-Host "  Installed binary: $installedTime" -ForegroundColor Gray

if ($buildTime -ne $installedTime) {
    Write-Host "  ⚠ WARNING: Build time mismatch!" -ForegroundColor Yellow
    Write-Host "    Source: $buildTime" -ForegroundColor Gray
    Write-Host "    Installed: $installedTime" -ForegroundColor Gray
} else {
    Write-Host "  ✓ Build time matches!" -ForegroundColor Green
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host " Installation Complete!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Open Web UI: http://localhost:9867" -ForegroundColor White
Write-Host "2. Go to Profiles → Activate 'default'" -ForegroundColor White
Write-Host "3. Test keys in Notepad" -ForegroundColor White
Write-Host "4. Check build date: Right-click tray icon → About" -ForegroundColor White
Write-Host ""
