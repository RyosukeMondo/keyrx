# Quick test script for Windows build
# Usage: .\scripts\package\test-windows.ps1

param(
    [switch]$Quick,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName

function Show-Usage {
    Write-Host @"
Test keyrx on Windows

USAGE:
    .\scripts\package\test-windows.ps1 [OPTIONS]

OPTIONS:
    -Quick      Quick test (build only, skip tests)
    -Help       Show this help

EXAMPLES:
    # Full test with tests
    .\scripts\package\test-windows.ps1

    # Quick build check
    .\scripts\package\test-windows.ps1 -Quick
"@
    exit 0
}

if ($Help) {
    Show-Usage
}

function Write-Step($Message) {
    Write-Host "[TEST] $Message" -ForegroundColor Cyan
}

function Write-Success($Message) {
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Error($Message) {
    Write-Host "[FAIL] $Message" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyRx Windows Build Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Set-Location $ProjectRoot

# Test 1: Build check
Write-Step "Building binaries..."
cargo build --bin keyrx_daemon --bin keyrx_compiler --features windows

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}
Write-Success "Build successful"

# Test 2: Run tests (unless Quick mode)
if (-not $Quick) {
    Write-Step "Running tests..."
    cargo test --workspace --features windows

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed"
        exit 1
    }
    Write-Success "Tests passed"
}

# Test 3: Check binaries exist
Write-Step "Checking binaries..."
$DaemonExe = "target\debug\keyrx_daemon.exe"
$CompilerExe = "target\debug\keyrx_compiler.exe"

if (-not (Test-Path $DaemonExe)) {
    Write-Error "Daemon binary not found: $DaemonExe"
    exit 1
}
if (-not (Test-Path $CompilerExe)) {
    Write-Error "Compiler binary not found: $CompilerExe"
    exit 1
}
Write-Success "Binaries exist"

# Test 4: Check binary versions
Write-Step "Checking versions..."
$DaemonVersion = & $DaemonExe --version 2>&1
Write-Host "  Daemon: $DaemonVersion"

$CompilerVersion = & $CompilerExe --version 2>&1
Write-Host "  Compiler: $CompilerVersion"

Write-Success "Version check complete"

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  All Tests Passed!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test daemon: .\target\debug\keyrx_daemon.exe run"
Write-Host "  2. Build installer: .\scripts\package\build-windows-innosetup.ps1"
Write-Host ""
