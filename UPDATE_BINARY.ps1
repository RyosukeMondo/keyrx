# Update Binary - Manually Replace with Latest Build
# RIGHT-CLICK this file and select "Run with PowerShell" or "Run as Administrator"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " Update KeyRx Binary" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as admin
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: This script requires administrator privileges!" -ForegroundColor Red
    Write-Host "Right-click the script and select 'Run as Administrator'" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

# Use absolute paths based on script location
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = $scriptDir
$sourceBinary = Join-Path $projectRoot "target\release\keyrx_daemon.exe"
$destBinary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"

Write-Host "Project root: $projectRoot" -ForegroundColor Gray
Write-Host ""

# Check source exists
if (-not (Test-Path $sourceBinary)) {
    Write-Host "ERROR: Source binary not found!" -ForegroundColor Red
    Write-Host "Run: cargo build --release -p keyrx_daemon" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

$sourceTime = (Get-Item $sourceBinary).LastWriteTime
Write-Host "Source binary: $sourceTime" -ForegroundColor Gray

# Step 1: Stop daemon
Write-Host ""
Write-Host "Stopping daemon..." -ForegroundColor Yellow
$process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($process) {
    Write-Host "  Found daemon (PID: $($process.Id))" -ForegroundColor Gray
    Stop-Process -Name keyrx_daemon -Force
    Start-Sleep -Seconds 3

    # Verify stopped
    $stillRunning = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($stillRunning) {
        Write-Host "  ERROR: Daemon still running!" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
    Write-Host "  ✓ Daemon stopped" -ForegroundColor Green
} else {
    Write-Host "  Daemon not running" -ForegroundColor Gray
}

# Step 2: Replace binary
Write-Host ""
Write-Host "Replacing binary..." -ForegroundColor Yellow
try {
    Copy-Item $sourceBinary $destBinary -Force
    Write-Host "  ✓ Binary replaced" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: Failed to copy binary" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Verify timestamp
$destTime = (Get-Item $destBinary).LastWriteTime
Write-Host "  Installed binary: $destTime" -ForegroundColor Gray

if ($sourceTime -eq $destTime) {
    Write-Host "  ✓ Build time matches!" -ForegroundColor Green
} else {
    Write-Host "  ⚠ WARNING: Build time mismatch!" -ForegroundColor Yellow
    Write-Host "    Source: $sourceTime" -ForegroundColor Gray
    Write-Host "    Installed: $destTime" -ForegroundColor Gray
}

# Step 3: Start daemon
Write-Host ""
Write-Host "Starting daemon..." -ForegroundColor Yellow
Start-Process $destBinary -ArgumentList "run"
Start-Sleep -Seconds 5

# Verify daemon started
$newProcess = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($newProcess) {
    Write-Host "  ✓ Daemon is running (PID: $($newProcess.Id))" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Daemon failed to start!" -ForegroundColor Red
    Write-Host "  Check logs in %APPDATA%\keyrx\daemon.log" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# Step 4: Test API
Write-Host ""
Write-Host "Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
try {
    $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  ✓ API is responding" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: API not responding" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Wait 10 seconds and check http://localhost:9867" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host " Binary Updated Successfully!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Open Web UI: http://localhost:9867" -ForegroundColor White
Write-Host "2. Test keyboard remapping in Notepad" -ForegroundColor White
Write-Host "3. Check About dialog for build date" -ForegroundColor White
Write-Host ""
Read-Host "Press Enter to exit"
