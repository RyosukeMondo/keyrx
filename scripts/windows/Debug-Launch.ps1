# Debug Launch Script for KeyRx Daemon
# Launches daemon with debug logging and admin privileges

param(
    [string]$ConfigPath = "",
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Show-Usage {
    Write-Host @"
Launch KeyRx daemon with debug logging

USAGE:
    .\scripts\windows\Debug-Launch.ps1 [OPTIONS]

OPTIONS:
    -ConfigPath PATH    Path to .rhai config file (optional, uses active profile if not specified)
    -Help               Show this help

EXAMPLES:
    # Use active profile with debug logging
    .\scripts\windows\Debug-Launch.ps1

    # Use specific config with debug logging
    .\scripts\windows\Debug-Launch.ps1 -ConfigPath examples\user_layout.rhai

OUTPUT:
    Debug logs will be shown in console and saved to:
    %TEMP%\keyrx-debug.log
"@
    exit 0
}

if ($Help) {
    Show-Usage
}

# Find daemon
$DaemonPath = $null
$Locations = @(
    "C:\Program Files\KeyRx\keyrx_daemon.exe",
    "$env:LOCALAPPDATA\Programs\KeyRx\keyrx_daemon.exe",
    ".\target\release\keyrx_daemon.exe",
    ".\target\debug\keyrx_daemon.exe"
)

foreach ($loc in $Locations) {
    if (Test-Path $loc) {
        $DaemonPath = (Get-Item $loc).FullName
        break
    }
}

if (-not $DaemonPath) {
    Write-Host "[ERROR] keyrx_daemon.exe not found" -ForegroundColor Red
    Write-Host "Searched in:" -ForegroundColor Yellow
    foreach ($loc in $Locations) {
        Write-Host "  $loc"
    }
    exit 1
}

Write-Host "[INFO] Found daemon: $DaemonPath" -ForegroundColor Green

# Stop existing daemon
$existing = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "[INFO] Stopping existing daemon..." -ForegroundColor Yellow
    Stop-Process -Name keyrx_daemon -Force
    Start-Sleep -Seconds 2
}

# Prepare arguments
$Args = @("run", "--debug")

if ($ConfigPath) {
    if (Test-Path $ConfigPath) {
        $ConfigPath = (Get-Item $ConfigPath).FullName
        $Args += @("--config", $ConfigPath)
        Write-Host "[INFO] Using config: $ConfigPath" -ForegroundColor Cyan
    } else {
        Write-Host "[ERROR] Config file not found: $ConfigPath" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "[INFO] Using active profile from %APPDATA%\keyrx" -ForegroundColor Cyan
}

# Setup logging
$LogFile = "$env:TEMP\keyrx-debug.log"
Write-Host "[INFO] Debug log: $LogFile" -ForegroundColor Cyan

# Launch daemon with admin
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Launching KeyRx Daemon (Debug Mode)" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "The daemon will:" -ForegroundColor Yellow
Write-Host "  1. Request administrator privileges (UAC prompt)" -ForegroundColor Yellow
Write-Host "  2. Show debug logs in console" -ForegroundColor Yellow
Write-Host "  3. Save logs to: $LogFile" -ForegroundColor Yellow
Write-Host ""
Write-Host "Press Ctrl+C to stop the daemon" -ForegroundColor Yellow
Write-Host ""

# Launch with admin and redirect output to log file
$ArgString = $Args -join " "

# Create a script block that will run as admin
$ScriptBlock = @"
Set-Location "$PWD"
`$env:RUST_LOG = "debug"
& "$DaemonPath" $ArgString 2>&1 | Tee-Object -FilePath "$LogFile"
"@

# Write script to temp file
$TempScript = "$env:TEMP\keyrx-debug-launch.ps1"
$ScriptBlock | Out-File -FilePath $TempScript -Encoding UTF8

# Launch as admin
Start-Process powershell.exe -ArgumentList "-NoExit", "-ExecutionPolicy", "Bypass", "-File", "`"$TempScript`"" -Verb RunAs

Write-Host "[INFO] Daemon launched in new window with debug logging" -ForegroundColor Green
Write-Host "[INFO] Check the new PowerShell window for debug output" -ForegroundColor Green
Write-Host "[INFO] Log file: $LogFile" -ForegroundColor Cyan
Write-Host ""
Write-Host "To view logs:" -ForegroundColor Yellow
Write-Host "  Get-Content `"$LogFile`" -Tail 50 -Wait" -ForegroundColor White
Write-Host ""
