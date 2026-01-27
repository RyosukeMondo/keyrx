# Build Windows MSI installer for keyrx using WiX Toolset
# Usage: .\scripts\package\build-windows-installer.ps1 [-Clean] [-SkipTests]

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
$VersionMsi = "$Version.0"  # MSI requires 4-part version

# Configuration
$BuildDir = Join-Path $ProjectRoot "target\windows-installer"
$WixFile = Join-Path $ProjectRoot "keyrx_daemon\keyrx_installer.wxs"
$OutputMsi = Join-Path $BuildDir "keyrx_$Version`_x64.msi"

function Show-Usage {
    Write-Host @"
Build Windows MSI installer for keyrx

USAGE:
    .\scripts\package\build-windows-installer.ps1 [OPTIONS]

OPTIONS:
    -Clean          Clean build artifacts before building
    -SkipTests      Skip running tests
    -Help           Show this help message

DEPENDENCIES:
    - WiX Toolset v3 (https://wixtoolset.org/releases/)
    - Rust toolchain (x86_64-pc-windows-msvc)
    - Node.js 18+

ENVIRONMENT VARIABLES:
    WIX             Path to WiX installation (auto-detected if in PATH)

EXAMPLES:
    # Basic build
    .\scripts\package\build-windows-installer.ps1

    # Clean build
    .\scripts\package\build-windows-installer.ps1 -Clean

    # Skip tests (faster)
    .\scripts\package\build-windows-installer.ps1 -SkipTests

OUTPUT:
    target\windows-installer\keyrx_${Version}_x64.msi
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

    # Check WiX
    $candle = Get-Command candle -ErrorAction SilentlyContinue
    $light = Get-Command light -ErrorAction SilentlyContinue

    if (-not $candle -or -not $light) {
        Write-Error @"
WiX Toolset not found in PATH.

Install WiX Toolset v3 from: https://wixtoolset.org/releases/

After installation, add WiX bin directory to PATH:
  C:\Program Files (x86)\WiX Toolset v3.x\bin
"@
        exit 1
    }

    $wixVersion = & candle -? 2>&1 | Select-String -Pattern "version (\d+\.\d+)" | ForEach-Object { $_.Matches.Groups[1].Value }
    Write-Success "Using WiX Toolset v$wixVersion"
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

# Update WiX file with version
function Update-WixVersion {
    Write-Step "Updating WiX version to $VersionMsi..."

    $wixContent = Get-Content $WixFile -Raw
    $wixContent = $wixContent -replace 'Version="[\d.]+"', "Version=`"$VersionMsi`""

    Set-Content $WixFile $wixContent -NoNewline

    Write-Success "WiX version updated"
}

# Build MSI installer
function Build-Msi {
    Write-Step "Building MSI installer with WiX..."

    # Create build directory
    New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null

    $WixObj = Join-Path $BuildDir "keyrx_installer.wixobj"

    # Compile WiX
    Write-Step "Running candle (compile)..."
    candle -nologo -out $WixObj -arch x64 $WixFile

    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX compilation failed"
        exit 1
    }

    # Link to MSI
    Write-Step "Running light (link)..."
    light -nologo -out $OutputMsi -ext WixUIExtension -cultures:en-us $WixObj

    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX linking failed"
        exit 1
    }

    if (-not (Test-Path $OutputMsi)) {
        Write-Error "MSI file not created"
        exit 1
    }

    Write-Success "MSI installer created: $OutputMsi"
}

# Show results
function Show-Results {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  Windows Installer Build Complete!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installer: " -NoNewline
    Write-Host $OutputMsi -ForegroundColor Cyan
    Write-Host ""

    $MsiSize = (Get-Item $OutputMsi).Length
    $MsiSizeMB = [math]::Round($MsiSize / 1024 / 1024, 2)
    Write-Host "Size: $MsiSizeMB MB"
    Write-Host ""
    Write-Host "To install:"
    Write-Host "  msiexec /i `"$OutputMsi`"" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To install silently:"
    Write-Host "  msiexec /i `"$OutputMsi`" /quiet" -ForegroundColor Yellow
    Write-Host ""
}

# Main execution
function Main {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  KeyRx Windows Installer Builder" -ForegroundColor Cyan
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

    Update-WixVersion
    Build-Msi
    Show-Results

    $Duration = (Get-Date) - $StartTime
    Write-Host "Build completed in $([math]::Round($Duration.TotalSeconds, 1))s" -ForegroundColor Green
    Write-Host ""
}

try {
    Main
}
catch {
    Write-Error "Build failed: $_"
    exit 1
}
