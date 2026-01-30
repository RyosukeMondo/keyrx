# Restart Daemon on Correct Port (9867)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Restarting Daemon (Port 9867)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Stop daemon
Write-Host "[1] Stopping daemon..." -ForegroundColor Yellow
taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
Start-Sleep -Seconds 3
Write-Host "  Stopped" -ForegroundColor Green

# Verify settings removed
Write-Host ""
Write-Host "[2] Verifying settings..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    Write-Host "  WARNING: settings.json still exists!" -ForegroundColor Red
    Remove-Item $settingsFile -Force
    Write-Host "  Removed" -ForegroundColor Green
} else {
    Write-Host "  No settings.json (will use default 9867)" -ForegroundColor Green
}

# Start daemon
Write-Host ""
Write-Host "[3] Starting daemon..." -ForegroundColor Yellow
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$daemonPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$stdoutLog = "daemon_stdout_$timestamp.log"
$stderrLog = "daemon_stderr_$timestamp.log"

Start-Process -FilePath $daemonPath -ArgumentList "run" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -WindowStyle Hidden

Write-Host "  Started (logs: $stderrLog)" -ForegroundColor Green

# Wait
Write-Host ""
Write-Host "[4] Waiting 8 seconds..." -ForegroundColor Yellow
Start-Sleep -Seconds 8

# Check process
Write-Host ""
Write-Host "[5] Checking daemon..." -ForegroundColor Yellow
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  Running (PID: $($daemon.Id))" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Not running!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Stderr log:"
    Get-Content $stderrLog -ErrorAction SilentlyContinue
    pause
    exit 1
}

# Check port
Write-Host ""
Write-Host "[6] Checking port 9867..." -ForegroundColor Yellow
$port = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
if ($port) {
    Write-Host "  Listening (PID: $($port.OwningProcess))" -ForegroundColor Green
} else {
    Write-Host "  WARNING: Port not listening yet" -ForegroundColor Yellow
}

# Test API
Write-Host ""
Write-Host "[7] Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  SUCCESS!" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "  FAILED: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Stderr log (last 30 lines):"
    Get-Content $stderrLog -Tail 30 -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Logs: $stderrLog" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
pause
