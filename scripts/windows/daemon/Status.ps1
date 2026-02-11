# Check KeyRx daemon status
# MECE: Only displays status, doesn't modify anything

#Requires -Version 5.1

param(
    [switch]$Json,         # Output as JSON
    [int]$Port = 9867      # Web server port
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Write-Step "KeyRx Daemon Status"

# Check process
$processRunning = Test-DaemonRunning
$daemonPid = Get-DaemonPid

if ($processRunning) {
    Write-Success "Process: Running (PID $daemonPid)"
} else {
    Write-Info "Process: Not running"
}

# Check port
$portInUse = Test-PortInUse -Port $Port
if ($portInUse) {
    Write-Success "Port $Port : In use"
} else {
    Write-Info "Port $Port : Available"
}

# Check API if process is running
if ($processRunning) {
    Write-Step "API Status"

    # Health check
    $health = Test-ApiEndpoint -Endpoint "/api/health" -Port $Port
    if ($health) {
        Write-Success "Health: OK (version $($health.version))"
    } else {
        Write-Warning-Custom "Health: API not responding"
    }

    # Daemon status
    $status = Test-ApiEndpoint -Endpoint "/api/status" -Port $Port
    if ($status) {
        $daemonStatus = if ($status.daemon_running) { "Running" } else { "Not Running" }
        $statusColor = if ($status.daemon_running) { "Green" } else { "Yellow" }
        Write-Host "Daemon Status: " -NoNewline
        Write-Host $daemonStatus -ForegroundColor $statusColor

        if ($status.active_profile) {
            Write-Info "Active Profile: $($status.active_profile)"
        } else {
            Write-Info "Active Profile: None"
        }

        if ($status.uptime_secs) {
            $uptime = [TimeSpan]::FromSeconds($status.uptime_secs)
            Write-Info "Uptime: $($uptime.ToString('hh\:mm\:ss'))"
        }

        if ($status.device_count) {
            Write-Info "Devices: $($status.device_count)"
        }

        if ($Json) {
            Write-Host "`nFull Status (JSON):" -ForegroundColor Cyan
            $status | ConvertTo-Json -Depth 10 | Write-Host
        }
    }

    # Profiles
    $profiles = Test-ApiEndpoint -Endpoint "/api/profiles" -Port $Port
    if ($profiles -and $profiles.profiles) {
        Write-Step "Profiles ($($profiles.profiles.Count) total)"
        foreach ($profile in $profiles.profiles | Select-Object -First 5) {
            $active = if ($profile.is_active) { " [ACTIVE]" } else { "" }
            Write-Host "  • $($profile.name)$active" -ForegroundColor $(if ($profile.is_active) { "Green" } else { "White" })
        }
        if ($profiles.profiles.Count -gt 5) {
            Write-Host "  ... and $($profiles.profiles.Count - 5) more" -ForegroundColor Gray
        }
    }
}

Write-Host ""
Write-Info "Web UI: http://localhost:$Port"
