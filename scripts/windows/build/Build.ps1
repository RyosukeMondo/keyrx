# Full rebuild of KeyRx daemon
# MECE: Handles only building, not testing or running

#Requires -Version 5.1

param(
    [switch]$Release,      # Build in release mode (optimized)
    [switch]$Debug,        # Build in debug mode (default)
    [switch]$Clean,        # Clean before building
    [switch]$Verbose       # Verbose cargo output
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Set-ProjectRoot
Test-CargoInstalled | Out-Null

Write-Step "Building KeyRx Daemon"

# Determine build mode
$buildMode = if ($Release) { "release" } else { "debug" }
$modeFlag = if ($Release) { "--release" } else { "" }

Write-Info "Build mode: $buildMode"

# Clean if requested
if ($Clean) {
    Write-Step "Cleaning build artifacts..."
    & cargo clean
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Clean failed"
        exit 1
    }
    Write-Success "Clean complete"
}

# Build daemon
Write-Step "Compiling daemon..."

$cargoArgs = @("build", "-p", "keyrx_daemon")
if ($Release) {
    $cargoArgs += "--release"
}
if ($Verbose) {
    $cargoArgs += "--verbose"
}

$startTime = Get-Date
& cargo @cargoArgs

if ($LASTEXITCODE -ne 0) {
    Write-Error-Custom "Build failed"
    exit 1
}

$duration = (Get-Date) - $startTime
Write-Success "Build complete in $($duration.TotalSeconds.ToString('F2'))s"

# Show binary location
$binaryPath = if ($Release) { $script:DaemonExe } else { Join-Path $script:TargetDir "debug\keyrx_daemon.exe" }
if (Test-Path $binaryPath) {
    $size = (Get-Item $binaryPath).Length / 1MB
    Write-Info "Binary: $binaryPath ($($size.ToString('F2')) MB)"
} else {
    Write-Warning-Custom "Binary not found at expected location: $binaryPath"
}

Write-Success "✨ Build successful!"
