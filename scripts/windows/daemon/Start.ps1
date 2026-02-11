# Start KeyRx daemon
# MECE: Only starts daemon, doesn't build or test

#Requires -Version 5.1

param(
    [switch]$Release,      # Use release build (default: debug)
    [switch]$Background,   # Run in background
    [switch]$Wait,         # Wait for daemon to be ready
    [int]$Port = 9867,     # Web server port
    [string]$Profile       # Profile to activate (optional)
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Set-ProjectRoot

Write-Step "Starting KeyRx Daemon"

# Check if already running
if (Test-DaemonRunning) {
    $daemonPid = Get-DaemonPid
    Write-Warning-Custom "Daemon already running (PID $daemonPid)"

    $response = Read-Host "Stop and restart? (y/N)"
    if ($response -eq 'y') {
        Stop-Daemon -Force | Out-Null
    } else {
        Write-Info "Keeping existing daemon"
        exit 0
    }
}

# Determine binary path
$binaryPath = if ($Release) { $script:DaemonExe } else { Join-Path $script:TargetDir "debug\keyrx_daemon.exe" }

if (-not (Test-Path $binaryPath)) {
    Write-Error-Custom "Daemon binary not found: $binaryPath"
    Write-Info "Run build script first: .\scripts\windows\build\Build.ps1"
    exit 1
}

# Build command arguments
$daemonArgs = @("run")
if ($Profile) {
    $profilePath = Join-Path $env:APPDATA "keyrx\profiles\$Profile.krx"
    if (Test-Path $profilePath) {
        $daemonArgs += "--config"
        $daemonArgs += $profilePath
        Write-Info "Using profile: $Profile"
    } else {
        Write-Warning-Custom "Profile not found: $Profile (using default)"
    }
}

# Start daemon
Write-Info "Starting daemon: $binaryPath"
Write-Info "Port: $Port"

if ($Background) {
    # Start in background
    $process = Start-Process -FilePath $binaryPath -ArgumentList $daemonArgs -WindowStyle Hidden -PassThru
    Write-Success "Daemon started in background (PID $($process.Id))"
} else {
    # Start in foreground
    Write-Info "Starting in foreground mode (Ctrl+C to stop)..."
    Write-Info ""
    & $binaryPath @daemonArgs
    exit $LASTEXITCODE
}

# Wait for ready if requested
if ($Wait) {
    if (Wait-DaemonReady -Port $Port) {
        Write-Success "Daemon is ready at http://localhost:$Port"

        # Verify version and build
        $health = Test-ApiEndpoint -Endpoint "/api/health" -Port $Port
        if ($health) {
            Write-Success "Version: $($health.version) | Build: $($health.build_time) | Git: $($health.git_hash)"
        }

        # Show status
        $status = Test-ApiEndpoint -Endpoint "/api/status" -Port $Port
        if ($status) {
            $runningStatus = if ($status.daemon_running) { "Running ✓" } else { "Not Running ✗" }
            Write-Info "Daemon: $runningStatus"
            if ($status.active_profile) {
                Write-Info "Profile: $($status.active_profile) | Devices: $($status.device_count)"
            } else {
                Write-Warning-Custom "No active profile loaded"
            }
        }
    } else {
        Write-Error-Custom "Daemon failed to become ready"
        exit 1
    }
}

Write-Success "✨ Daemon started successfully!"
Write-Info "Web UI: http://localhost:$Port"
