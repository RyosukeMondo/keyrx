#!/usr/bin/env pwsh
#
# Windows E2E Remapping Tests
#
# This script runs end-to-end tests for key remapping on Windows:
# - Key event simulation
# - Remapping verification (A → B)
# - Metrics endpoint validation
#
# Usage:
#   .\Run-E2E-Remap-Tests.ps1
#   .\Run-E2E-Remap-Tests.ps1 -Verbose
#   .\Run-E2E-Remap-Tests.ps1 -TestName "test_windows_key_remap_e2e"

param(
    [string]$TestName = "",
    [switch]$Verbose,
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

# Colors
$ColorReset = "`e[0m"
$ColorGreen = "`e[32m"
$ColorYellow = "`e[33m"
$ColorRed = "`e[31m"
$ColorBlue = "`e[34m"

function Write-Step {
    param([string]$Message)
    Write-Host "${ColorBlue}▶ $Message${ColorReset}"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${ColorGreen}✓ $Message${ColorReset}"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${ColorYellow}⚠ $Message${ColorReset}"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${ColorRed}✗ $Message${ColorReset}"
}

Write-Host @"
${ColorBlue}
╔══════════════════════════════════════════════════════════════════╗
║         Windows E2E Key Remapping Tests - keyrx                  ║
╚══════════════════════════════════════════════════════════════════╝
${ColorReset}
"@

# Get project root
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Write-Step "Project root: $ProjectRoot"
Set-Location $ProjectRoot

# Step 1: Environment check
Write-Step "Checking environment..."

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo not found. Please install Rust: https://rustup.rs"
    exit 1
}

$RustVersion = cargo --version
Write-Success "Rust: $RustVersion"

# Step 2: Build (optional)
if (-not $NoBuild) {
    Write-Step "Building keyrx_daemon with Windows features..."
    $BuildArgs = @(
        "build",
        "--package", "keyrx_daemon",
        "--features", "windows",
        "--release"
    )

    & cargo @BuildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
    Write-Success "Build completed"
} else {
    Write-Warning "Skipping build (--NoBuild flag)"
}

# Step 3: Run tests
Write-Step "Running E2E remapping tests..."

$TestArgs = @(
    "test",
    "--package", "keyrx_daemon",
    "--test", "windows_e2e_remap",
    "--features", "windows"
)

if ($Verbose) {
    $TestArgs += "--", "--nocapture", "--test-threads=1"
}

if ($TestName) {
    Write-Step "Running specific test: $TestName"
    $TestArgs += $TestName
}

& cargo @TestArgs

if ($LASTEXITCODE -ne 0) {
    Write-Error "Tests failed with exit code $LASTEXITCODE"
    exit 1
}

Write-Success "All tests passed!"

# Step 4: Summary
Write-Host @"

${ColorGreen}
╔══════════════════════════════════════════════════════════════════╗
║                    ✅ ALL TESTS PASSED                            ║
╚══════════════════════════════════════════════════════════════════╝

Test Summary:
  ✓ Key event simulation working
  ✓ Remapping applied correctly (A → B)
  ✓ Metrics endpoints detecting events
  ✓ Latency tracking functional

${ColorReset}
"@

# Step 5: Next steps
Write-Host @"
${ColorYellow}Next Steps:${ColorReset}
1. Test in Vagrant VM: ${ColorBlue}vagrant up && vagrant ssh${ColorReset}
2. Run specific test: ${ColorBlue}.\Run-E2E-Remap-Tests.ps1 -TestName "test_windows_key_remap_e2e"${ColorReset}
3. Run with verbose output: ${ColorBlue}.\Run-E2E-Remap-Tests.ps1 -Verbose${ColorReset}

${ColorYellow}Manual Testing:${ColorReset}
1. Start daemon: ${ColorBlue}.\target\release\keyrx_daemon.exe${ColorReset}
2. Open browser: ${ColorBlue}http://localhost:3030/metrics${ColorReset}
3. Simulate keys via UI simulator page

"@

exit 0
