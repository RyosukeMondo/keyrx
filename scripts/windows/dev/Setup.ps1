# First-time development environment setup
# MECE: One-time setup tasks only

#Requires -Version 5.1

param(
    [switch]$SkipRust,     # Skip Rust toolchain check
    [switch]$SkipNode,     # Skip Node.js check
    [switch]$SkipBuild     # Skip initial build
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Set-ProjectRoot

Write-Step "KeyRx Development Environment Setup"

# Check prerequisites
if (-not $SkipRust) {
    Write-Step "Checking Rust toolchain..."

    if (Test-CargoInstalled) {
        $rustVersion = & cargo --version
        Write-Success "Rust installed: $rustVersion"

        # Check for Windows target
        $targets = & rustup target list --installed
        if ($targets -contains "x86_64-pc-windows-msvc") {
            Write-Success "Windows MSVC target installed"
        } else {
            Write-Info "Installing Windows MSVC target..."
            & rustup target add x86_64-pc-windows-msvc
        }
    } else {
        Write-Error-Custom "Rust not found. Install from https://rustup.rs/"
        exit 1
    }
}

if (-not $SkipNode) {
    Write-Step "Checking Node.js..."

    if (Test-NpmInstalled) {
        $nodeVersion = & node --version
        $npmVersion = & npm --version
        Write-Success "Node.js installed: $nodeVersion"
        Write-Success "npm installed: $npmVersion"
    } else {
        Write-Error-Custom "Node.js not found. Install from https://nodejs.org/"
        exit 1
    }
}

# Install Rust tools
Write-Step "Installing Rust development tools..."

$tools = @(
    @{Name="cargo-watch"; Package="cargo-watch"},
    @{Name="cargo-tarpaulin"; Package="cargo-tarpaulin"}
)

foreach ($tool in $tools) {
    $installed = & cargo install --list | Select-String -Pattern $tool.Name
    if ($installed) {
        Write-Success "$($tool.Name) already installed"
    } else {
        Write-Info "Installing $($tool.Name)..."
        & cargo install $tool.Package
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$($tool.Name) installed"
        } else {
            Write-Warning-Custom "Failed to install $($tool.Name)"
        }
    }
}

# Install UI dependencies
Write-Step "Installing UI dependencies..."

Push-Location $script:UiDir

if (Test-Path "package.json") {
    if (Test-Path "node_modules") {
        Write-Info "node_modules exists, skipping npm install"
    } else {
        & npm install
        if ($LASTEXITCODE -eq 0) {
            Write-Success "UI dependencies installed"
        } else {
            Write-Error-Custom "Failed to install UI dependencies"
            Pop-Location
            exit 1
        }
    }
} else {
    Write-Warning-Custom "package.json not found in UI directory"
}

Pop-Location

# Initial build
if (-not $SkipBuild) {
    Write-Step "Running initial build..."

    & (Join-Path $PSScriptRoot "..\build\Build.ps1") -Release

    if ($LASTEXITCODE -eq 0) {
        Write-Success "Initial build complete"
    } else {
        Write-Error-Custom "Initial build failed"
        exit 1
    }
}

# Create local config directory
$configDir = Join-Path $env:APPDATA "keyrx"
if (-not (Test-Path $configDir)) {
    Write-Step "Creating config directory..."
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    Write-Success "Config directory created: $configDir"
}

# Summary
Write-Host "`n"
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host " SETUP COMPLETE!" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Info "Next steps:"
Write-Host "  1. Build project:  .\scripts\windows\build\Build.ps1 -Release" -ForegroundColor Cyan
Write-Host "  2. Start daemon:   .\scripts\windows\daemon\Start.ps1 -Release -Background -Wait" -ForegroundColor Cyan
Write-Host "  3. Run UAT:        .\scripts\windows\test\UAT.ps1" -ForegroundColor Cyan
Write-Host "  4. Check status:   .\scripts\windows\daemon\Status.ps1" -ForegroundColor Cyan
Write-Host ""
Write-Success "✨ Development environment ready!"
