# Build Installer Only (assumes daemon already built)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Building Installer (v0.1.1)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check daemon exists
$DaemonPath = "..\..\target\release\keyrx_daemon.exe"
if (-not (Test-Path $DaemonPath)) {
    Write-Host "[ERROR] Daemon not found at: $DaemonPath" -ForegroundColor Red
    Write-Host "[INFO] Run: cargo build --release --features windows" -ForegroundColor Yellow
    exit 1
}

$DaemonInfo = Get-Item $DaemonPath
Write-Host "[INFO] Daemon: $($DaemonInfo.Length / 1MB) MB (built: $($DaemonInfo.LastWriteTime))" -ForegroundColor Green

# Find Inno Setup
$IsccPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $IsccPath)) {
    Write-Host "[ERROR] Inno Setup not found at: $IsccPath" -ForegroundColor Red
    exit 1
}

Write-Host "[INFO] Inno Setup found" -ForegroundColor Green
Write-Host "[INFO] Building installer..." -ForegroundColor Cyan
Write-Host ""

# Build installer
& $IsccPath keyrx-installer.iss

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Installer build failed with exit code: $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

$InstallerPath = "..\..\target\windows-installer\keyrx_0.1.1.0_x64_setup.exe"
if (Test-Path $InstallerPath) {
    $InstallerInfo = Get-Item $InstallerPath
    Write-Host "Installer created:" -ForegroundColor Cyan
    Write-Host "  Path: $($InstallerInfo.FullName)" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($InstallerInfo.Length/1MB, 2)) MB" -ForegroundColor White
    Write-Host ""
    Write-Host "To install:" -ForegroundColor Yellow
    Write-Host "  1. Right-click installer → Run as administrator" -ForegroundColor White
    Write-Host "  2. Follow installation wizard" -ForegroundColor White
    Write-Host "  3. Installer will detect and uninstall old version" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "[WARN] Installer not found at expected location" -ForegroundColor Yellow
    Write-Host "[INFO] Checking for any .exe in target/windows-installer..." -ForegroundColor Cyan
    Get-ChildItem "..\..\target\windows-installer\*.exe" | ForEach-Object {
        Write-Host "  Found: $($_.Name) ($([math]::Round($_.Length/1MB, 2)) MB)" -ForegroundColor White
    }
}
