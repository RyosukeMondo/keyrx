#Requires -RunAsAdministrator
# Restart Daemon with Full Logging

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Restarting Daemon with Logging" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Stop daemon
Write-Host "[1] Stopping daemon..." -ForegroundColor Yellow
taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
Start-Sleep -Seconds 3

$remaining = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($remaining) {
    Write-Host "  WARNING: Still running" -ForegroundColor Yellow
} else {
    Write-Host "  Stopped" -ForegroundColor Green
}

# 2. Check settings
Write-Host ""
Write-Host "[2] Checking settings..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    $settings = Get-Content $settingsFile | ConvertFrom-Json
    Write-Host "  Port: $($settings.port)"
    if ($settings.port -ne 9867) {
        Write-Host "  WARNING: Port is not 9867!" -ForegroundColor Yellow
        Write-Host "  Removing settings to use default..."
        Remove-Item $settingsFile
    }
} else {
    Write-Host "  No settings (will use default 9867)" -ForegroundColor Green
}

# 3. Start daemon with logging
Write-Host ""
Write-Host "[3] Starting daemon with logging..." -ForegroundColor Yellow

$daemonPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$stdoutLog = "daemon_stdout_$timestamp.log"
$stderrLog = "daemon_stderr_$timestamp.log"

if (Test-Path $daemonPath) {
    Start-Process -FilePath $daemonPath -ArgumentList "run" `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -WindowStyle Hidden

    Write-Host "  Started (logs: $stdoutLog, $stderrLog)" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Binary not found!" -ForegroundColor Red
    exit 1
}

# 4. Wait and check
Write-Host ""
Write-Host "[4] Waiting 8 seconds for startup..." -ForegroundColor Yellow
Start-Sleep -Seconds 8

# 5. Check process
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

# 6. Check ports
Write-Host ""
Write-Host "[6] Checking ports..." -ForegroundColor Yellow
$port9867 = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
if ($port9867) {
    Write-Host "  Port 9867: Listening (PID: $($port9867.OwningProcess))" -ForegroundColor Green
} else {
    Write-Host "  Port 9867: NOT listening!" -ForegroundColor Red
}

# 7. Test API
Write-Host ""
Write-Host "[7] Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  API: OK" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "  API: FAILED" -ForegroundColor Red
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Yellow

    Write-Host ""
    Write-Host "Stderr log (last 50 lines):"
    Get-Content $stderrLog -Tail 50 -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "  $_"
    }
}

# 8. Show logs
Write-Host ""
Write-Host "[8] Recent log entries:" -ForegroundColor Yellow
Get-Content $stderrLog -Tail 20 -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  $_"
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Logs saved to:" -ForegroundColor Green
Write-Host "  $stdoutLog"
Write-Host "  $stderrLog"
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

pause
