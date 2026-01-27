# Debug Metrics E2E Test Script
# This script builds the daemon with debug logging and helps diagnose why metrics aren't showing events

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "======================================== " -ForegroundColor Cyan
Write-Host "  Metrics E2E Debug Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build daemon with debug logging
Write-Host "[1/6] Building daemon with debug logging..." -ForegroundColor Yellow
cd $PSScriptRoot\..\..\
cargo build --release --features windows

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Build failed" -ForegroundColor Red
    exit 1
}

Write-Host "[SUCCESS] Daemon built" -ForegroundColor Green
Write-Host ""

# Step 2: Stop existing daemon
Write-Host "[2/6] Stopping existing daemon..." -ForegroundColor Yellow
$DaemonProcess = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($DaemonProcess) {
    Stop-Process -Name keyrx_daemon -Force
    Start-Sleep -Seconds 2
    Write-Host "[SUCCESS] Stopped existing daemon" -ForegroundColor Green
} else {
    Write-Host "[INFO] No daemon running" -ForegroundColor Gray
}
Write-Host ""

# Step 3: Set up logging
Write-Host "[3/6] Setting up debug logging..." -ForegroundColor Yellow
$LogFile = "$env:TEMP\keyrx-metrics-debug.log"
$env:RUST_LOG = "debug"
Write-Host "[INFO] Log file: $LogFile" -ForegroundColor Gray
Write-Host "[INFO] RUST_LOG=$env:RUST_LOG" -ForegroundColor Gray
Write-Host ""

# Step 4: Start daemon in background
Write-Host "[4/6] Starting daemon with debug logging..." -ForegroundColor Yellow

$DaemonPath = ".\target\release\keyrx_daemon.exe"
$ScriptContent = @"
Set-Location -Path '$PWD'
`$env:RUST_LOG = 'debug'
& '$DaemonPath' run 2>&1 | Tee-Object -FilePath '$LogFile'
"@

$TempScript = "$env:TEMP\keyrx-metrics-debug-launch.ps1"
Set-Content -Path $TempScript -Value $ScriptContent

# Start elevated PowerShell in a new window
Start-Process powershell.exe -ArgumentList "-NoExit", "-File", $TempScript -Verb RunAs

Write-Host "[SUCCESS] Daemon starting in elevated PowerShell window" -ForegroundColor Green
Write-Host ""

# Wait for daemon to start
Write-Host "[5/6] Waiting for daemon to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Check if daemon is running
$DaemonRunning = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($DaemonRunning) {
    Write-Host "[SUCCESS] Daemon is running (PID: $($DaemonRunning.Id))" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Daemon failed to start. Check elevated PowerShell window for errors." -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 6: Open browser and tail log
Write-Host "[6/6] Opening browser and monitoring logs..." -ForegroundColor Yellow
Start-Process "http://localhost:9867/metrics"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Debugging Instructions" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. The daemon is running in an elevated PowerShell window" -ForegroundColor White
Write-Host "2. Press some keys on your keyboard" -ForegroundColor White
Write-Host "3. Check the daemon window for debug output" -ForegroundColor White
Write-Host "4. Check the browser /metrics page" -ForegroundColor White
Write-Host ""
Write-Host "Expected debug output:" -ForegroundColor Yellow
Write-Host "  - 'Windows Raw Input: Captured keyboard event'" -ForegroundColor Gray
Write-Host "  - 'Event loop: About to broadcast ... event'" -ForegroundColor Gray
Write-Host "  - 'Broadcasting key event (subscribers: X)'" -ForegroundColor Gray
Write-Host "  - 'Successfully broadcast key event to X receivers'" -ForegroundColor Gray
Write-Host "  - 'WebSocket client connected'" -ForegroundColor Gray
Write-Host "  - 'WebSocket received daemon event: KeyEvent(...)'" -ForegroundColor Gray
Write-Host ""
Write-Host "Log file: $LogFile" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Enter to tail the log file..." -ForegroundColor Yellow
Read-Host

# Tail the log file
Get-Content $LogFile -Tail 50 -Wait
