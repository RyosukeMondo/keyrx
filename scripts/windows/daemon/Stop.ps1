# Stop KeyRx daemon
# MECE: Only stops daemon

#Requires -Version 5.1

param(
    [switch]$Force         # Force kill without graceful shutdown
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Write-Step "Stopping KeyRx Daemon"

if (-not (Test-DaemonRunning)) {
    Write-Info "Daemon is not running"
    exit 0
}

$daemonPid = Get-DaemonPid
Write-Info "Found daemon process (PID $daemonPid)"

if (Stop-Daemon -Force:$Force) {
    Write-Success "✨ Daemon stopped successfully!"
} else {
    Write-Error-Custom "Failed to stop daemon"
    Write-Info "Try using Task Manager to end the process"
    exit 1
}
