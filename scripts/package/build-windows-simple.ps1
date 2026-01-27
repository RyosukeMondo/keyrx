# Simple Windows build script (no Inno Setup required for testing)
# Just builds binaries in release mode
# Usage: .\scripts\package\build-windows-simple.ps1

param(
    [switch]$SkipTests,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName

function Show-Usage {
    Write-Host @"
Simple Windows build (creates binaries only, no installer)

USAGE:
    .\scripts\package\build-windows-simple.ps1 [OPTIONS]

OPTIONS:
    -SkipTests      Skip running tests
    -Help           Show this help

OUTPUT:
    target\release\keyrx_daemon.exe
    target\release\keyrx_compiler.exe
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
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-ErrorMsg($Message) {
    Write-Host "[FAIL] $Message" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyRx Windows Build (Simple)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Set-Location $ProjectRoot
$StartTime = Get-Date

# Step 1: Build WASM
Write-Step "Step 1/4: Building WASM module..."
$CoreDir = Join-Path $ProjectRoot "keyrx_core"
$OutputDir = Join-Path $ProjectRoot "keyrx_ui\src\wasm\pkg"

Set-Location $CoreDir
wasm-pack build --target web --out-dir $OutputDir --release -- --features wasm

if ($LASTEXITCODE -ne 0) {
    Write-ErrorMsg "WASM build failed"
    exit 1
}
Write-Success "WASM build completed"

# Step 2: Build UI (TypeScript only, skip npm build to avoid bash script)
Write-Step "Step 2/4: Building UI..."
$UiDir = Join-Path $ProjectRoot "keyrx_ui"
Set-Location $UiDir

# Install dependencies if needed
if (-not (Test-Path "node_modules")) {
    Write-Step "Installing npm dependencies..."
    npm ci
}

# Generate version file
Write-Step "Generating version..."
node ..\scripts\generate-version.js

# Build TypeScript
Write-Step "Building TypeScript..."
npx tsc -b

# Build with Vite (this embeds the WASM)
Write-Step "Building with Vite..."
npx vite build

if ($LASTEXITCODE -ne 0) {
    Write-ErrorMsg "UI build failed"
    exit 1
}
Write-Success "UI build completed"

# Step 3: Build daemon
Set-Location $ProjectRoot
Write-Step "Step 3/4: Building daemon (release)..."

# Touch static_files.rs to force re-embedding UI
$StaticFiles = Join-Path $ProjectRoot "keyrx_daemon\src\web\static_files.rs"
if (Test-Path $StaticFiles) {
    (Get-Item $StaticFiles).LastWriteTime = Get-Date
    Write-Step "  → Re-embedding UI into daemon"
}

cargo build --release --bin keyrx_daemon --bin keyrx_compiler --features windows

if ($LASTEXITCODE -ne 0) {
    Write-ErrorMsg "Daemon build failed"
    exit 1
}
Write-Success "Daemon build completed"

# Step 4: Tests
if (-not $SkipTests) {
    Write-Step "Step 4/4: Running tests..."
    cargo test --workspace --features windows --release

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "Tests failed"
        exit 1
    }
    Write-Success "Tests passed"
} else {
    Write-Step "Step 4/4: Skipping tests"
}

$Duration = (Get-Date) - $StartTime
$DurationSec = [math]::Round($Duration.TotalSeconds, 1)

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Binaries:" -ForegroundColor Cyan
Write-Host "  target\release\keyrx_daemon.exe"
Write-Host "  target\release\keyrx_compiler.exe"
Write-Host ""

$DaemonExe = Join-Path $ProjectRoot "target\release\keyrx_daemon.exe"
$DaemonSize = (Get-Item $DaemonExe).Length
$DaemonSizeMB = [math]::Round($DaemonSize / 1024 / 1024, 2)
Write-Host "Daemon size: $DaemonSizeMB MB"
Write-Host "Build time: $DurationSec seconds"
Write-Host ""
Write-Host "Test daemon:" -ForegroundColor Yellow
Write-Host "  .\target\release\keyrx_daemon.exe --version"
Write-Host "  .\target\release\keyrx_daemon.exe run"
Write-Host ""
