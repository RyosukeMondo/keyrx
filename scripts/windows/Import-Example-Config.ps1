# Import Example Config to Web UI System
# Copies example config to profiles directory so it appears in Web UI

param(
    [string]$ConfigName = "user_layout",
    [string]$SourceFile = "examples\user_layout.rhai"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Import Example Config to Web UI" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check source file exists
if (-not (Test-Path $SourceFile)) {
    Write-Host "[ERROR] Source file not found: $SourceFile" -ForegroundColor Red
    exit 1
}

$SourcePath = (Get-Item $SourceFile).FullName
Write-Host "[INFO] Source: $SourcePath" -ForegroundColor Green

# Create profiles directory
$ProfilesDir = "$env:APPDATA\keyrx\profiles"
if (-not (Test-Path $ProfilesDir)) {
    Write-Host "[INFO] Creating profiles directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force $ProfilesDir | Out-Null
}

# Copy file
$DestPath = "$ProfilesDir\$ConfigName.rhai"
Write-Host "[INFO] Destination: $DestPath" -ForegroundColor Green

if (Test-Path $DestPath) {
    Write-Host "[WARN] File already exists. Overwrite? (Y/N)" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -ne 'Y' -and $response -ne 'y') {
        Write-Host "[INFO] Import cancelled" -ForegroundColor Yellow
        exit 0
    }
}

Copy-Item $SourcePath $DestPath -Force
Write-Host "[SUCCESS] Config imported!" -ForegroundColor Green

# Check if daemon is running
$daemon = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if (-not $daemon) {
    Write-Host ""
    Write-Host "[WARN] Daemon is not running" -ForegroundColor Yellow
    Write-Host "[INFO] Start daemon to see the profile in Web UI:" -ForegroundColor Cyan
    Write-Host "       .\scripts\windows\Debug-Launch.ps1" -ForegroundColor White
    exit 0
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Next Steps" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "1. Open Web UI: http://localhost:9867" -ForegroundColor Cyan
Write-Host "2. Click 'Profiles' in sidebar" -ForegroundColor Cyan
Write-Host "3. Click 'Activate' on '$ConfigName' profile" -ForegroundColor Cyan
Write-Host "4. Go to 'Metrics' page and test keys" -ForegroundColor Cyan
Write-Host ""

# Offer to open browser
Write-Host "Open Web UI now? (Y/N)" -ForegroundColor Yellow
$response = Read-Host
if ($response -eq 'Y' -or $response -eq 'y') {
    Start-Process "http://localhost:9867"
    Write-Host "[INFO] Web UI opened in browser" -ForegroundColor Green
}

Write-Host ""
Write-Host "[SUCCESS] Import complete!" -ForegroundColor Green
Write-Host ""
