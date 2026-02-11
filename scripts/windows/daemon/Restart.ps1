# Restart KeyRx daemon
# MECE: Combines Stop + Start in one operation

#Requires -Version 5.1

param(
    [switch]$Release,      # Use release build
    [switch]$Wait,         # Wait for daemon to be ready
    [int]$Port = 9867,     # Web server port
    [string]$Profile       # Profile to activate
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Write-Step "Restarting KeyRx Daemon"

# Stop daemon
& (Join-Path $PSScriptRoot "Stop.ps1") -Force

# Start daemon
$startArgs = @{
    Background = $true
    Wait = $Wait
    Port = $Port
}
if ($Release) { $startArgs.Release = $true }
if ($Profile) { $startArgs.Profile = $Profile }

& (Join-Path $PSScriptRoot "Start.ps1") @startArgs

Write-Success "✨ Daemon restarted!"
