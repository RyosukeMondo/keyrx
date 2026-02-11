# Common utilities for KeyRx Windows scripts
# SSOT: All shared functions in one place

#Requires -Version 5.1

# Strict mode for better error detection
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Project paths (SSOT)
$script:ProjectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$script:TargetDir = Join-Path $ProjectRoot "target"
$script:ReleaseDir = Join-Path $TargetDir "release"
$script:DaemonExe = Join-Path $ReleaseDir "keyrx_daemon.exe"
$script:UiDir = Join-Path $ProjectRoot "keyrx_ui"

# Colors for output
function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Cyan
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Step {
    param([string]$Message)
    Write-Host "`n▶️  $Message" -ForegroundColor Blue
}

# Check if daemon is running
function Test-DaemonRunning {
    $process = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
    return $null -ne $process
}

# Get daemon PID
function Get-DaemonPid {
    $process = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
    if ($process) {
        return $process.Id
    }
    return $null
}

# Kill daemon process
function Stop-Daemon {
    param([switch]$Force)

    if (Test-DaemonRunning) {
        $daemonPid = Get-DaemonPid
        Write-Info "Stopping daemon (PID $daemonPid)..."

        if ($Force) {
            Stop-Process -Name "keyrx_daemon" -Force -ErrorAction SilentlyContinue
        } else {
            Stop-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue
        }

        Start-Sleep -Seconds 2

        if (-not (Test-DaemonRunning)) {
            Write-Success "Daemon stopped"
            return $true
        } else {
            Write-Warning-Custom "Daemon still running, trying force kill..."
            Stop-Process -Name "keyrx_daemon" -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 1
            return -not (Test-DaemonRunning)
        }
    } else {
        Write-Info "Daemon not running"
        return $true
    }
}

# Wait for daemon to be ready
function Wait-DaemonReady {
    param(
        [int]$TimeoutSeconds = 10,
        [int]$Port = 9867
    )

    $url = "http://localhost:$Port/api/health"
    $startTime = Get-Date

    Write-Info "Waiting for daemon to be ready..."

    while ((Get-Date) -lt ($startTime.AddSeconds($TimeoutSeconds))) {
        try {
            $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 1 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-Success "Daemon is ready!"
                return $true
            }
        } catch {
            # Daemon not ready yet
        }
        Start-Sleep -Milliseconds 500
    }

    Write-Error-Custom "Daemon did not become ready within $TimeoutSeconds seconds"
    return $false
}

# Check if port is in use
function Test-PortInUse {
    param([int]$Port = 9867)

    $connection = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
    return $null -ne $connection
}

# Test API endpoint
function Test-ApiEndpoint {
    param(
        [string]$Endpoint,
        [int]$Port = 9867
    )

    $url = "http://localhost:$Port$Endpoint"
    try {
        $response = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 5
        return $response
    } catch {
        Write-Error-Custom "Failed to query $Endpoint : $_"
        return $null
    }
}

# Ensure we're in project root
function Set-ProjectRoot {
    Set-Location $script:ProjectRoot
    Write-Info "Working directory: $script:ProjectRoot"
}

# Check cargo is installed
function Test-CargoInstalled {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        Write-Error-Custom "Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
    return $true
}

# Check npm is installed
function Test-NpmInstalled {
    $npm = Get-Command npm -ErrorAction SilentlyContinue
    if (-not $npm) {
        Write-Error-Custom "npm not found. Please install Node.js from https://nodejs.org/"
        exit 1
    }
    return $true
}

# Functions and variables are automatically available when dot-sourced
# No need for Export-ModuleMember with dot-sourcing
