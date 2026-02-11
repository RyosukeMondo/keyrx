# Complete daemon restart with clean rebuild and UAC elevation
# One-click: kills old daemon, rebuilds, starts new, verifies

#Requires -Version 5.1

param(
    [switch]$SkipBuild,     # Skip clean rebuild
    [switch]$SkipVerify,    # Skip verification step
    [switch]$NoBrowser      # Don't open browser
)

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# If not admin, re-launch with elevation
if (-not (Test-Administrator)) {
    Write-Host "🔐 Requesting administrator privileges..." -ForegroundColor Yellow
    Write-Host "Please click 'Yes' on the UAC prompt`n" -ForegroundColor Cyan

    $scriptPath = $MyInvocation.MyCommand.Path
    $arguments = "-NoExit -NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""

    if ($SkipBuild) { $arguments += " -SkipBuild" }
    if ($SkipVerify) { $arguments += " -SkipVerify" }
    if ($NoBrowser) { $arguments += " -NoBrowser" }

    try {
        Start-Process powershell.exe -Verb RunAs -ArgumentList $arguments
    } catch {
        Write-Host "Failed to elevate: $_" -ForegroundColor Red
        Write-Host "`nPress any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    }
    exit
}

# Now running as administrator
Write-Host ""
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host " KeyRx Fresh Restart [Admin Mode]" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Get project root (assume script is in scripts/windows/)
$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path (Split-Path -Parent $scriptRoot) -Parent
Set-Location $projectRoot

Write-Host "Project: $projectRoot" -ForegroundColor Gray
Write-Host ""

# Step 1: Force kill all keyrx_daemon processes
Write-Host "▶ Step 1: Stopping old daemons" -ForegroundColor Cyan
$processes = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($processes) {
    foreach ($proc in $processes) {
        Write-Host "  Killing PID $($proc.Id)..." -ForegroundColor Yellow
        try {
            Stop-Process -Id $proc.Id -Force -ErrorAction Stop
            Write-Host "  ✓ Stopped PID $($proc.Id)" -ForegroundColor Green
        } catch {
            Write-Host "  ✗ Failed: $_" -ForegroundColor Red
        }
    }
    Start-Sleep -Seconds 2
} else {
    Write-Host "  No running daemons found" -ForegroundColor Gray
}

# Verify all stopped
$remaining = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($remaining) {
    Write-Host "  ✗ Failed to stop all daemons" -ForegroundColor Red
    Write-Host "`nPress any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}
Write-Host "  ✓ All daemons stopped`n" -ForegroundColor Green

# Step 2: Clean rebuild (unless skipped)
if (-not $SkipBuild) {
    Write-Host "▶ Step 2: Clean rebuild" -ForegroundColor Cyan

    # Clean
    Write-Host "  Cleaning build artifacts..." -ForegroundColor Yellow
    & cargo clean 2>&1 | Out-Null

    # Record build start time
    $buildStartTime = Get-Date
    Write-Host "  Build started: $($buildStartTime.ToString('HH:mm:ss'))" -ForegroundColor Gray

    # Build
    Write-Host "  Building release binary (this takes ~5 minutes)..." -ForegroundColor Yellow
    $buildOutput = & cargo build --release -p keyrx_daemon 2>&1

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ✗ Build failed!" -ForegroundColor Red
        Write-Host "`nBuild output:" -ForegroundColor Yellow
        $buildOutput | Select-Object -Last 20 | ForEach-Object { Write-Host "    $_" }
        Write-Host "`nPress any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }

    $buildEndTime = Get-Date
    $buildDuration = ($buildEndTime - $buildStartTime).TotalSeconds
    Write-Host "  ✓ Build complete in $([math]::Round($buildDuration, 1))s" -ForegroundColor Green

    # Verify binary timestamp is fresh
    $binaryPath = Join-Path $projectRoot "target\release\keyrx_daemon.exe"
    if (Test-Path $binaryPath) {
        $binaryTime = (Get-Item $binaryPath).LastWriteTime
        $age = (Get-Date) - $binaryTime

        if ($age.TotalMinutes -lt 1) {
            Write-Host "  ✓ Binary is fresh (modified $([math]::Round($age.TotalSeconds))s ago)" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Binary seems old (modified $([math]::Round($age.TotalMinutes, 1))m ago)" -ForegroundColor Yellow
        }
    }
    Write-Host ""
} else {
    Write-Host "▶ Step 2: Using existing binary (skip build)`n" -ForegroundColor Cyan
}

# Step 3: Start new daemon
Write-Host "▶ Step 3: Starting fresh daemon" -ForegroundColor Cyan

$daemonPath = Join-Path $projectRoot "target\release\keyrx_daemon.exe"
if (-not (Test-Path $daemonPath)) {
    Write-Host "  ✗ Binary not found: $daemonPath" -ForegroundColor Red
    Write-Host "`nPress any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host "  Binary: $daemonPath" -ForegroundColor Gray
Write-Host "  Port: 9867" -ForegroundColor Gray

# Start daemon in background
try {
    $process = Start-Process -FilePath $daemonPath -ArgumentList "run" -WindowStyle Hidden -PassThru
    Write-Host "  ✓ Daemon started (PID $($process.Id))" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Failed to start: $_" -ForegroundColor Red
    Write-Host "`nPress any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}
Write-Host ""

# Step 4: Wait for ready
Write-Host "▶ Step 4: Waiting for daemon to be ready" -ForegroundColor Cyan

$maxWait = 20
$waited = 0
$ready = $false

while ($waited -lt $maxWait) {
    Start-Sleep -Seconds 1
    $waited++

    try {
        $response = Invoke-WebRequest -Uri "http://localhost:9867/api/health" -Method Get -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            $ready = $true
            break
        }
    } catch {
        # Still waiting
    }

    if ($waited % 5 -eq 0) {
        Write-Host "  Waiting... ($waited/$maxWait seconds)" -ForegroundColor Gray
    }
}

if (-not $ready) {
    Write-Host "  ✗ Daemon did not become ready within $maxWait seconds" -ForegroundColor Red
    Write-Host "`nPress any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}
Write-Host "  ✓ Daemon is ready!`n" -ForegroundColor Green

# Step 5: Verify build (unless skipped)
if (-not $SkipVerify) {
    Write-Host "▶ Step 5: Verifying build" -ForegroundColor Cyan

    try {
        $health = Invoke-RestMethod -Uri "http://localhost:9867/api/health" -Method Get -ErrorAction Stop

        Write-Host "  Version: $($health.version)" -ForegroundColor White
        Write-Host "  Build Time: $($health.build_time)" -ForegroundColor White
        Write-Host "  Git Hash: $($health.git_hash)" -ForegroundColor White

        # Parse build time and check if it's recent
        if ($health.build_time -match '(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})') {
            $buildDateTime = Get-Date -Year $Matches[1] -Month $Matches[2] -Day $Matches[3] -Hour $Matches[4] -Minute $Matches[5] -Second 0
            $age = (Get-Date) - $buildDateTime

            if ($age.TotalMinutes -lt 10) {
                Write-Host "  ✓ Build is fresh ($([math]::Round($age.TotalMinutes, 1))m old)" -ForegroundColor Green
            } else {
                Write-Host "  ⚠ Build seems old ($([math]::Round($age.TotalHours, 1))h old)" -ForegroundColor Yellow
            }
        }

        # Check daemon status
        $status = Invoke-RestMethod -Uri "http://localhost:9867/api/status" -Method Get -ErrorAction Stop
        if ($status.daemon_running) {
            Write-Host "  ✓ Daemon event loop: Running" -ForegroundColor Green
            Write-Host "  ✓ Active profile: $($status.active_profile)" -ForegroundColor Green
            Write-Host "  ✓ Devices: $($status.device_count)" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Daemon event loop not running" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "  ⚠ Could not verify: $_" -ForegroundColor Yellow
    }
    Write-Host ""
}

# Step 6: Open browser (unless skipped)
if (-not $NoBrowser) {
    Write-Host "▶ Step 6: Opening Web UI" -ForegroundColor Cyan
    try {
        Start-Process "http://localhost:9867"
        Write-Host "  ✓ Browser opened`n" -ForegroundColor Green
    } catch {
        Write-Host "  ⚠ Could not open browser: $_`n" -ForegroundColor Yellow
    }
}

# Summary
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host " ✨ RESTART COMPLETE!" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "✓ Daemon running at http://localhost:9867" -ForegroundColor Green
Write-Host ""
Write-Host "Commands:" -ForegroundColor Cyan
Write-Host "  Status:  .\scripts\windows\daemon\Status.ps1" -ForegroundColor Gray
Write-Host "  Stop:    .\scripts\windows\daemon\Stop.ps1" -ForegroundColor Gray
Write-Host "  Verify:  .\scripts\windows\test\Verify-Daemon.ps1" -ForegroundColor Gray
Write-Host ""
Write-Host "Press any key to close..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
