#Requires -RunAsAdministrator
# Start Daemon with Admin Rights

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Starting Daemon (Requires Admin)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Kill any existing
Write-Host "[1] Killing existing processes..." -ForegroundColor Yellow
taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
Start-Sleep -Seconds 3
Write-Host "  Cleaned up" -ForegroundColor Green

# Remove settings
Write-Host ""
Write-Host "[2] Ensuring default port (9867)..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    Remove-Item $settingsFile -Force
}
Write-Host "  Using default 9867" -ForegroundColor Green

# Start daemon
Write-Host ""
Write-Host "[3] Starting daemon..." -ForegroundColor Yellow
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$daemonPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$stdoutLog = "$PSScriptRoot\daemon_stdout_$timestamp.log"
$stderrLog = "$PSScriptRoot\daemon_stderr_$timestamp.log"

Start-Process -FilePath $daemonPath -ArgumentList "run" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -WindowStyle Hidden

Write-Host "  Started" -ForegroundColor Green
Write-Host "  Stdout: $stdoutLog"
Write-Host "  Stderr: $stderrLog"

# Wait
Write-Host ""
Write-Host "[4] Waiting 10 seconds for initialization..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Check
Write-Host ""
Write-Host "[5] Checking status..." -ForegroundColor Yellow
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  Daemon: Running (PID: $($daemon.Id))" -ForegroundColor Green
} else {
    Write-Host "  Daemon: NOT RUNNING!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Stderr:"
    Get-Content $stderrLog -ErrorAction SilentlyContinue
    pause
    exit 1
}

$port = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
if ($port) {
    Write-Host "  Port 9867: Listening (PID: $($port.OwningProcess))" -ForegroundColor Green
} else {
    Write-Host "  Port 9867: NOT listening!" -ForegroundColor Red
}

# Test API
Write-Host ""
Write-Host "[6] Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  API: SUCCESS!" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " DAEMON READY" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
} catch {
    Write-Host "  API: FAILED - $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Recent stderr (last 50 lines):"
    Get-Content $stderrLog -Tail 50 -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "  $_"
    }
}

Write-Host ""
pause
