#Requires -RunAsAdministrator
# Deep Keyboard Interception Diagnostics

Write-Host "================================================" -ForegroundColor Cyan
Write-Host " Deep Keyboard Interception Diagnostics" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Check daemon elevation
Write-Host "[1] Checking daemon elevation..." -ForegroundColor Yellow
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  PID: $($daemon.Id)" -ForegroundColor Green

    # Check if elevated
    $isElevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($isElevated) {
        Write-Host "  This script: Running as Administrator" -ForegroundColor Green
    } else {
        Write-Host "  This script: NOT admin (unexpected!)" -ForegroundColor Red
    }

    # Try to check daemon's token
    try {
        $proc = [System.Diagnostics.Process]::GetProcessById($daemon.Id)
        Write-Host "  Daemon path: $($proc.MainModule.FileName)" -ForegroundColor Cyan
    } catch {
        Write-Host "  Cannot access daemon process details" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ERROR: Daemon not running!" -ForegroundColor Red
    exit 1
}

# 2. Check API health
Write-Host ""
Write-Host "[2] API Health..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 5
    Write-Host "  Status: $($health.status)" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 3. Check active profile
Write-Host ""
Write-Host "[3] Active Profile..." -ForegroundColor Yellow
try {
    $profiles = Invoke-RestMethod -Uri http://localhost:9867/api/profiles -TimeoutSec 5
    $active = $profiles.profiles | Where-Object { $_.isActive -eq $true }
    if ($active) {
        Write-Host "  Active: $($active.name)" -ForegroundColor Green
    } else {
        Write-Host "  No active profile!" -ForegroundColor Red
        Write-Host "  Activating default..." -ForegroundColor Yellow
        try {
            $result = Invoke-RestMethod -Uri http://localhost:9867/api/profiles/default/activate -Method POST -TimeoutSec 30
            Write-Host "  Activated default" -ForegroundColor Green
        } catch {
            Write-Host "  Failed to activate: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
}

# 4. Clear and monitor metrics
Write-Host ""
Write-Host "[4] Clearing metrics..." -ForegroundColor Yellow
try {
    Invoke-RestMethod -Uri http://localhost:9867/api/metrics/events -Method DELETE -TimeoutSec 5 | Out-Null
    Write-Host "  Cleared" -ForegroundColor Green
} catch {
    Write-Host "  Warning: $($_.Exception.Message)" -ForegroundColor Yellow
}

# 5. Interactive keyboard test
Write-Host ""
Write-Host "[5] KEYBOARD TEST" -ForegroundColor Yellow
Write-Host "  Press ANY key on your keyboard 5 times..." -ForegroundColor Cyan
Write-Host "  (Press keys in this window or any window)" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Waiting 10 seconds..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# 6. Check captured events
Write-Host ""
Write-Host "[6] Checking captured events..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:9867/api/metrics/events?count=50" -TimeoutSec 5

    if ($response.events -and $response.events.Count -gt 0) {
        Write-Host "  Captured: $($response.events.Count) events" -ForegroundColor Green
        Write-Host ""
        Write-Host "  Recent events:" -ForegroundColor Cyan
        $response.events | Select-Object -First 10 | ForEach-Object {
            $event = $_
            Write-Host "    [$($event.timestamp_us)] $($event.event_type): $($event.key_code)" -ForegroundColor White
        }
    } else {
        Write-Host "  PROBLEM: NO EVENTS CAPTURED!" -ForegroundColor Red
        Write-Host ""
        Write-Host "  This means keyboard interception is NOT working." -ForegroundColor Red
        Write-Host "  Possible causes:" -ForegroundColor Yellow
        Write-Host "    1. Hook not installed" -ForegroundColor Yellow
        Write-Host "    2. Message loop not running" -ForegroundColor Yellow
        Write-Host "    3. Events being filtered out" -ForegroundColor Yellow
        Write-Host "    4. Wrong input device selected" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
}

# 7. Check devices
Write-Host ""
Write-Host "[7] Checking input devices..." -ForegroundColor Yellow
try {
    $devices = Invoke-RestMethod -Uri http://localhost:9867/api/devices -TimeoutSec 5
    if ($devices.devices) {
        Write-Host "  Found $($devices.devices.Count) devices:" -ForegroundColor Green
        $devices.devices | ForEach-Object {
            $dev = $_
            Write-Host "    - $($dev.name) (ID: $($dev.id))" -ForegroundColor Cyan
        }
    } else {
        Write-Host "  No devices found!" -ForegroundColor Red
    }
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
}

# 8. Check latest daemon logs
Write-Host ""
Write-Host "[8] Latest daemon logs..." -ForegroundColor Yellow
$latestLog = Get-ChildItem -Filter "daemon_stderr_*.log" -ErrorAction SilentlyContinue |
             Sort-Object LastWriteTime -Descending |
             Select-Object -First 1

if ($latestLog) {
    Write-Host "  File: $($latestLog.Name)" -ForegroundColor Cyan
    Write-Host "  Last 30 lines:" -ForegroundColor Cyan
    Get-Content $latestLog.FullName -Tail 30 | ForEach-Object {
        if ($_ -match "ERROR|WARN") {
            Write-Host "    $_" -ForegroundColor Red
        } elseif ($_ -match "hook|Hook|interception|keyboard") {
            Write-Host "    $_" -ForegroundColor Yellow
        } else {
            Write-Host "    $_" -ForegroundColor White
        }
    }
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "DIAGNOSTIC COMPLETE" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

pause
