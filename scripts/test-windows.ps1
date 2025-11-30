# Check if cargo is installed
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed or not in PATH." -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/"
    exit 1
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$CoreDir = Join-Path $ProjectRoot "core"

Write-Host "=== KeyRx Windows Test Runner ===" -ForegroundColor Cyan

# 1. Run Unit Tests
Write-Host "`n[1/3] Running Cargo Tests..." -ForegroundColor Yellow
Push-Location $CoreDir
try {
    cargo test --all-features
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Tests failed!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

# 2. Build Release
Write-Host "`n[2/3] Building Release Binary..." -ForegroundColor Yellow
Push-Location $CoreDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

# 3. Run Doctor
Write-Host "`n[3/3] Running KeyRx Doctor..." -ForegroundColor Yellow
$BinaryPath = Join-Path $CoreDir "target\release\keyrx.exe"
if (Test-Path $BinaryPath) {
    & $BinaryPath doctor
} else {
    Write-Host "Binary not found at $BinaryPath" -ForegroundColor Red
}

Write-Host "`nAll tests passed successfully!" -ForegroundColor Green
