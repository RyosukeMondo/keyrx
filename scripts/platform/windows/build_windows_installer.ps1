# Build Windows MSI installer using WiX Toolset
#
# Prerequisites:
# - WiX Toolset 3.x or 4.x installed
# - Release binaries already built (cargo build --release)
#
# Usage: .\scripts\build_windows_installer.ps1

param(
    [string]$Version = "0.1.0",
    [string]$OutputDir = "target\installer"
)

$ErrorActionPreference = "Stop"

Write-Host "================================" -ForegroundColor Cyan
Write-Host " KeyRx Windows Installer Build" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check if WiX is installed
$wixBinPath = "C:\Program Files (x86)\WiX Toolset v3.14\bin"
$candlePath = Join-Path $wixBinPath "candle.exe"
$lightPath = Join-Path $wixBinPath "light.exe"

if (-not (Test-Path $candlePath)) {
    Write-Host "ERROR: WiX Toolset not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install WiX Toolset:" -ForegroundColor Yellow
    Write-Host "  choco install wixtoolset" -ForegroundColor Yellow
    Write-Host "  OR download from: https://wixtoolset.org/releases/" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found WiX at: $wixBinPath" -ForegroundColor Green

# Check if release binaries exist
$daemonExe = "target\release\keyrx_daemon.exe"
$compilerExe = "target\release\keyrx_compiler.exe"

if (-not (Test-Path $daemonExe)) {
    Write-Host "ERROR: keyrx_daemon.exe not found!" -ForegroundColor Red
    Write-Host "Run: cargo build --release" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $compilerExe)) {
    Write-Host "ERROR: keyrx_compiler.exe not found!" -ForegroundColor Red
    Write-Host "Run: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found release binaries" -ForegroundColor Green

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Compile WiX source
Write-Host ""
Write-Host "Compiling WiX source..." -ForegroundColor Cyan
$wixObj = Join-Path $OutputDir "keyrx_installer.wixobj"

& $candlePath `
    -nologo `
    -ext WixUIExtension `
    -ext WixUtilExtension `
    -dVersion="$Version" `
    -out "$wixObj" `
    "keyrx_daemon\keyrx_installer.wxs"

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: WiX compilation failed!" -ForegroundColor Red
    exit 1
}

Write-Host "WiX compilation successful" -ForegroundColor Green

# Link WiX object to create MSI
Write-Host ""
Write-Host "Linking MSI..." -ForegroundColor Cyan
$msiPath = Join-Path $OutputDir "KeyRx-$Version-x64.msi"

& $lightPath `
    -nologo `
    -ext WixUIExtension `
    -ext WixUtilExtension `
    -spdb `
    -out "$msiPath" `
    "$wixObj"

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: MSI linking failed!" -ForegroundColor Red
    exit 1
}

Write-Host "MSI created successfully" -ForegroundColor Green

# Display results
Write-Host ""
Write-Host "================================" -ForegroundColor Cyan
Write-Host " Build Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installer: $msiPath" -ForegroundColor Yellow
Write-Host "Size: $((Get-Item $msiPath).Length / 1MB) MB" -ForegroundColor Yellow
Write-Host ""
Write-Host "To install:" -ForegroundColor Cyan
Write-Host "  msiexec /i `"$msiPath`"" -ForegroundColor White
Write-Host ""
Write-Host "To uninstall:" -ForegroundColor Cyan
Write-Host "  msiexec /x `"$msiPath`"" -ForegroundColor White
Write-Host ""
