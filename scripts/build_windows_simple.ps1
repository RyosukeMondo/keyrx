# Simple Windows installer builder (without WiX)
# Creates a portable ZIP distribution
#
# Usage: .\scripts\build_windows_simple.ps1

param(
    [string]$Version = "0.1.0",
    [string]$OutputDir = "target\dist"
)

$ErrorActionPreference = "Stop"

Write-Host "================================" -ForegroundColor Cyan
Write-Host " KeyRx Portable ZIP Builder" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

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

# Create distribution directory
$distName = "KeyRx-$Version-Windows-x64"
$distPath = Join-Path $OutputDir $distName

if (Test-Path $distPath) {
    Remove-Item -Recurse -Force $distPath
}

New-Item -ItemType Directory -Force -Path $distPath | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $distPath "bin") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $distPath "config") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $distPath "logs") | Out-Null

Write-Host "Created distribution directory: $distPath" -ForegroundColor Green

# Copy binaries
Write-Host ""
Write-Host "Copying binaries..." -ForegroundColor Cyan
Copy-Item $daemonExe (Join-Path $distPath "bin\keyrx_daemon.exe")
Copy-Item $compilerExe (Join-Path $distPath "bin\keyrx_compiler.exe")
Write-Host "Binaries copied" -ForegroundColor Green

# Copy documentation
Write-Host ""
Write-Host "Copying documentation..." -ForegroundColor Cyan
if (Test-Path "README.md") {
    Copy-Item "README.md" (Join-Path $distPath "README.md")
}
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" (Join-Path $distPath "LICENSE")
}
if (Test-Path "user_layout.krx") {
    Copy-Item "user_layout.krx" (Join-Path $distPath "config\example_layout.krx")
}
Write-Host "Documentation copied" -ForegroundColor Green

# Create README for portable version
$portableReadme = @"
# KeyRx Portable Distribution

This is a portable distribution of KeyRx keyboard remapper.

## Installation

1. Extract this ZIP file to your preferred location (e.g., C:\Program Files\KeyRx)
2. Add the 'bin' directory to your PATH:
   - Right-click 'This PC' -> Properties -> Advanced system settings
   - Click 'Environment Variables'
   - Under 'System variables', find 'Path' and click 'Edit'
   - Click 'New' and add: C:\path\to\KeyRx\bin
   - Click 'OK' on all dialogs

## Usage

Open PowerShell or Command Prompt:

```powershell
# List available keyboards
keyrx_daemon list-devices

# Run daemon with configuration
keyrx_daemon run --config config\example_layout.krx

# View help
keyrx_daemon --help
```

## Configuration

Configuration files are stored in:
- %LOCALAPPDATA%\keyrx\profiles\
- %LOCALAPPDATA%\keyrx\devices.json

## Web UI

The daemon includes a web UI accessible at:
- http://localhost:9867

## Uninstallation

Simply delete this directory. Configuration files will remain in:
- %LOCALAPPDATA%\keyrx\

## Support

- GitHub: https://github.com/RyosukeMondo/keyrx
- Documentation: See README.md

"@

Set-Content -Path (Join-Path $distPath "PORTABLE_README.txt") -Value $portableReadme
Write-Host "Created portable README" -ForegroundColor Green

# Create ZIP
Write-Host ""
Write-Host "Creating ZIP archive..." -ForegroundColor Cyan
$zipPath = Join-Path $OutputDir "$distName.zip"

if (Test-Path $zipPath) {
    Remove-Item -Force $zipPath
}

Compress-Archive -Path $distPath -DestinationPath $zipPath -CompressionLevel Optimal
Write-Host "ZIP archive created" -ForegroundColor Green

# Display results
$zipSize = (Get-Item $zipPath).Length / 1MB

Write-Host ""
Write-Host "================================" -ForegroundColor Cyan
Write-Host " Build Complete!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Portable ZIP: $zipPath" -ForegroundColor Yellow
Write-Host "Size: $([math]::Round($zipSize, 2)) MB" -ForegroundColor Yellow
Write-Host ""
Write-Host "Distribution directory: $distPath" -ForegroundColor Yellow
Write-Host ""
Write-Host "To test:" -ForegroundColor Cyan
Write-Host "  cd $distPath" -ForegroundColor White
Write-Host "  .\bin\keyrx_daemon.exe --version" -ForegroundColor White
Write-Host ""
