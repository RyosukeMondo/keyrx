#Requires -RunAsAdministrator
# Install Fresh Binary and Start Daemon

Write-Host "================================================" -ForegroundColor Cyan
Write-Host " Install Fresh Binary & Start Daemon" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Stop any existing daemon
Write-Host "[1/7] Stopping existing daemon..." -ForegroundColor Yellow
taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
Start-Sleep -Seconds 3
Write-Host "  Stopped" -ForegroundColor Green

# 2. Remove settings.json (SSOT)
Write-Host ""
Write-Host "[2/7] Ensuring SSOT port (9867)..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    Remove-Item $settingsFile -Force
}
Write-Host "  Using default 9867" -ForegroundColor Green

# 3. Verify fresh binary
Write-Host ""
Write-Host "[3/7] Checking fresh binary..." -ForegroundColor Yellow
$sourceBinary = "target\release\keyrx_daemon.exe"
if (-not (Test-Path $sourceBinary)) {
    Write-Host "  ERROR: $sourceBinary not found!" -ForegroundColor Red
    Write-Host "  Run: cd keyrx_daemon && cargo build --release" -ForegroundColor Yellow
    pause
    exit 1
}
$sourceInfo = Get-Item $sourceBinary
Write-Host "  Source: $($sourceInfo.Length) bytes" -ForegroundColor Cyan
Write-Host "  Built: $($sourceInfo.LastWriteTime)" -ForegroundColor Cyan

# 4. Install binary
Write-Host ""
Write-Host "[4/7] Installing binary..." -ForegroundColor Yellow
$targetPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
try {
    Copy-Item $sourceBinary $targetPath -Force
    Write-Host "  Installed to $targetPath" -ForegroundColor Green

    $targetInfo = Get-Item $targetPath
    Write-Host "  Installed: $($targetInfo.LastWriteTime)" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
    pause
    exit 1
}

# 5. Start daemon
Write-Host ""
Write-Host "[5/7] Starting daemon..." -ForegroundColor Yellow
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$stdoutLog = "$PSScriptRoot\daemon_stdout_$timestamp.log"
$stderrLog = "$PSScriptRoot\daemon_stderr_$timestamp.log"

Start-Process -FilePath $targetPath -ArgumentList "run" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -WindowStyle Hidden

Write-Host "  Started" -ForegroundColor Green
Write-Host "  Logs: $stderrLog" -ForegroundColor Cyan

# 6. Wait and verify
Write-Host ""
Write-Host "[6/7] Waiting 12 seconds for initialization..." -ForegroundColor Yellow
Start-Sleep -Seconds 12

$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  Daemon: Running (PID: $($daemon.Id))" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Daemon not running!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Stderr log:"
    Get-Content $stderrLog -ErrorAction SilentlyContinue
    pause
    exit 1
}

$port = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
if ($port) {
    Write-Host "  Port 9867: Listening (PID: $($port.OwningProcess))" -ForegroundColor Green
} else {
    Write-Host "  WARNING: Port 9867 not listening yet" -ForegroundColor Yellow
}

# 7. Test API
Write-Host ""
Write-Host "[7/7] Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  API: SUCCESS!" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green

    Write-Host ""
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host " DAEMON READY!" -ForegroundColor Green
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Binary timestamp: $($targetInfo.LastWriteTime)" -ForegroundColor Cyan
    Write-Host "Web UI: http://localhost:9867" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Next step: Run DEEP_DIAGNOSTIC.ps1 as Administrator" -ForegroundColor Yellow
    Write-Host "to test keyboard interception." -ForegroundColor Yellow

} catch {
    Write-Host "  API FAILED: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Stderr log (last 50 lines):"
    Get-Content $stderrLog -Tail 50 -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "  $_"
    }
}

Write-Host ""
pause
