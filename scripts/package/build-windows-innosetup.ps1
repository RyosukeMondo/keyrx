# Build Windows installer for keyrx using Inno Setup
# Usage: .\scripts\package\build-windows-innosetup.ps1 [-Clean] [-SkipTests]
#
# Inno Setup is simpler and more popular than WiX.
# Download from: https://jrsoftware.org/isdl.php

param(
    [switch]$Clean,
    [switch]$SkipTests,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Get project root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName

# Parse version from Cargo.toml
$CargoToml = Get-Content (Join-Path $ProjectRoot "Cargo.toml")
$VersionLine = $CargoToml | Select-String -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
$Version = $VersionLine.Matches.Groups[1].Value

# Configuration
$BuildDir = Join-Path $ProjectRoot "target\windows-installer"
$IssFile = Join-Path $ScriptDir "keyrx-installer.iss"

function Show-Usage {
    Write-Host @"
Build Windows installer for keyrx using Inno Setup

USAGE:
    .\scripts\package\build-windows-innosetup.ps1 [OPTIONS]

OPTIONS:
    -Clean          Clean build artifacts before building
    -SkipTests      Skip running tests
    -Help           Show this help message

DEPENDENCIES:
    - Inno Setup 6 (https://jrsoftware.org/isdl.php)
    - Rust toolchain (x86_64-pc-windows-msvc)
    - Node.js 18+

ENVIRONMENT VARIABLES:
    INNO_SETUP      Path to Inno Setup installation (auto-detected)

EXAMPLES:
    # Basic build
    .\scripts\package\build-windows-innosetup.ps1

    # Clean build
    .\scripts\package\build-windows-innosetup.ps1 -Clean

    # Skip tests (faster)
    .\scripts\package\build-windows-innosetup.ps1 -SkipTests

OUTPUT:
    target\windows-installer\keyrx_${Version}_x64_setup.exe
"@
    exit 0
}

if ($Help) {
    Show-Usage
}

function Write-Step($Message) {
    Write-Host "[BUILD] $Message" -ForegroundColor Cyan
}

function Write-Success($Message) {
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Error($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Find Inno Setup compiler
function Find-InnoSetup {
    # Check common locations
    $CommonPaths = @(
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe",
        "C:\Program Files (x86)\Inno Setup 5\ISCC.exe",
        "C:\Program Files\Inno Setup 5\ISCC.exe"
    )

    foreach ($path in $CommonPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    # Check in PATH
    $iscc = Get-Command iscc -ErrorAction SilentlyContinue
    if ($iscc) {
        return $iscc.Source
    }

    # Check environment variable
    if ($env:INNO_SETUP -and (Test-Path $env:INNO_SETUP)) {
        return $env:INNO_SETUP
    }

    return $null
}

# Check dependencies
function Test-Dependencies {
    Write-Step "Checking dependencies..."

    # Check Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Rust toolchain not found. Install from https://rustup.rs/"
        exit 1
    }

    # Check MSVC target
    $targets = & rustup target list --installed
    if ($targets -notcontains "x86_64-pc-windows-msvc") {
        Write-Step "Installing x86_64-pc-windows-msvc target..."
        rustup target add x86_64-pc-windows-msvc
    }

    # Check Node.js
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Error "Node.js not found. Install from https://nodejs.org/"
        exit 1
    }

    # Check Inno Setup
    $script:IsccPath = Find-InnoSetup
    if (-not $script:IsccPath) {
        Write-Error @"
Inno Setup not found.

Download and install Inno Setup 6 from:
  https://jrsoftware.org/isdl.php

Alternatively, set INNO_SETUP environment variable to ISCC.exe path.
"@
        exit 1
    }

    # Try to get version (may fail, that's ok)
    try {
        $isccOutput = & $script:IsccPath 2>&1 | Out-String
        $isccVersion = $isccOutput | Select-String -Pattern "Inno Setup (\d+\.\d+)" | ForEach-Object { $_.Matches.Groups[1].Value }
        if ($isccVersion) {
            Write-Success "Using Inno Setup v$isccVersion at: $script:IsccPath"
        } else {
            Write-Success "Using Inno Setup at: $script:IsccPath"
        }
    } catch {
        Write-Success "Using Inno Setup at: $script:IsccPath"
    }
}

# Clean build artifacts
function Invoke-Clean {
    Write-Step "Cleaning build artifacts..."

    if (Test-Path $BuildDir) {
        Remove-Item -Recurse -Force $BuildDir
    }

    # Clean Rust builds
    cargo clean --release

    # Clean UI build
    $UiDir = Join-Path $ProjectRoot "keyrx_ui"
    $UiDistDir = Join-Path $UiDir "dist"
    if (Test-Path $UiDistDir) {
        Remove-Item -Recurse -Force $UiDistDir
    }

    Write-Success "Clean completed"
}

# Build WASM module
function Build-Wasm {
    Write-Step "Building WASM module..."

    $CoreDir = Join-Path $ProjectRoot "keyrx_core"
    $OutputDir = Join-Path $ProjectRoot "keyrx_ui\src\wasm\pkg"

    Set-Location $CoreDir
    wasm-pack build --target web --out-dir $OutputDir --release -- --features wasm

    if ($LASTEXITCODE -ne 0) {
        Write-Error "WASM build failed"
        exit 1
    }

    Set-Location $ProjectRoot
    Write-Success "WASM build completed"
}

# Build UI
function Build-Ui {
    Write-Step "Building Web UI..."

    $UiDir = Join-Path $ProjectRoot "keyrx_ui"
    Set-Location $UiDir

    # Install dependencies
    if (-not (Test-Path "node_modules")) {
        Write-Step "Installing npm dependencies..."
        npm ci
    }

    # Build production bundle
    npm run build

    if ($LASTEXITCODE -ne 0) {
        Write-Error "UI build failed"
        exit 1
    }

    Set-Location $ProjectRoot
    Write-Success "UI build completed"
}

# Build daemon
function Build-Daemon {
    Write-Step "Building keyrx daemon (release)..."

    cargo build --release --bin keyrx_daemon --bin keyrx_compiler --features windows

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Daemon build failed"
        exit 1
    }

    Write-Success "Daemon build completed"
}

# Run tests
function Invoke-Tests {
    Write-Step "Running tests..."

    cargo test --workspace --features windows

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed"
        exit 1
    }

    Write-Success "All tests passed"
}

# Build installer
function Build-Installer {
    Write-Step "Building installer with Inno Setup..."

    if (-not (Test-Path $IssFile)) {
        Write-Error "Inno Setup script not found: $IssFile"
        exit 1
    }

    # Create build directory
    New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null

    # Run Inno Setup compiler
    & $script:IsccPath /O"$BuildDir" $IssFile

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Inno Setup compilation failed"
        exit 1
    }

    # Find the output file
    $OutputFile = Get-ChildItem $BuildDir -Filter "keyrx_*_setup.exe" | Select-Object -First 1

    if (-not $OutputFile) {
        Write-Error "Installer file not created"
        exit 1
    }

    Write-Success "Installer created: $($OutputFile.FullName)"
    return $OutputFile.FullName
}

# Show results
function Show-Results {
    param([string]$InstallerPath)

    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  Windows Installer Build Complete!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installer: " -NoNewline
    Write-Host $InstallerPath -ForegroundColor Cyan
    Write-Host ""

    $InstallerSize = (Get-Item $InstallerPath).Length
    $InstallerSizeMB = [math]::Round($InstallerSize / 1024 / 1024, 2)
    Write-Host "Size: $InstallerSizeMB MB"
    Write-Host ""
    Write-Host "To install:"
    Write-Host "  Double-click the installer" -ForegroundColor Yellow
    Write-Host "  Or run: " -NoNewline
    Write-Host "$InstallerPath /SILENT" -ForegroundColor Yellow
    Write-Host ""
}

# Main execution
function Main {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  KeyRx Windows Installer Builder" -ForegroundColor Cyan
    Write-Host "  (Inno Setup)" -ForegroundColor Cyan
    Write-Host "  Version: $Version" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""

    $StartTime = Get-Date

    Test-Dependencies

    if ($Clean) {
        Invoke-Clean
    }

    # Build sequence
    Build-Wasm
    Build-Ui
    Build-Daemon

    if (-not $SkipTests) {
        Invoke-Tests
    }

    $InstallerPath = Build-Installer
    Show-Results $InstallerPath

    $Duration = (Get-Date) - $StartTime
    Write-Host "Build completed in $([math]::Round($Duration.TotalSeconds, 1))s" -ForegroundColor Green
    Write-Host ""
}

try {
    Set-Location $ProjectRoot
    Main
}
catch {
    Write-Error "Build failed: $_"
    exit 1
}
