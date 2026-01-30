# Fix Port Configuration and Start Daemon
# This script will fix the SSOT violation and start the daemon correctly

Write-Host "================================================" -ForegroundColor Cyan
Write-Host " KeyRx Daemon Restart (Port 9867 SSOT Fix)" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Remove settings.json (SSOT violation)
Write-Host "[1/6] Removing settings.json to use SSOT default (9867)..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    Remove-Item $settingsFile -Force
    Write-Host "  Removed settings.json" -ForegroundColor Green
} else {
    Write-Host "  No settings.json found (good)" -ForegroundColor Green
}

# Step 2: Stop any existing daemon
Write-Host ""
Write-Host "[2/6] Stopping existing daemon..." -ForegroundColor Yellow
$existing = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($existing) {
    Stop-Process -Name keyrx_daemon -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 3
    Write-Host "  Stopped (PID: $($existing.Id))" -ForegroundColor Green
} else {
    Write-Host "  No daemon running" -ForegroundColor Green
}

# Step 3: Check Task Scheduler method
Write-Host ""
Write-Host "[3/6] Checking Task Scheduler entry..." -ForegroundColor Yellow
try {
    $task = schtasks /Query /TN "KeyRx Daemon" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  Task found, starting via scheduler..." -ForegroundColor Green
        schtasks /Run /TN "KeyRx Daemon" 2>&1 | Out-Null
        $startMethod = "Task Scheduler"
    } else {
        throw "No task"
    }
} catch {
    Write-Host "  No Task Scheduler entry, starting directly..." -ForegroundColor Yellow
    $daemonPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    if (Test-Path $daemonPath) {
        # Try direct start (requires admin)
        try {
            $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
            Start-Process -FilePath $daemonPath -ArgumentList "run" `
                -RedirectStandardOutput "daemon_stdout_$timestamp.log" `
                -RedirectStandardError "daemon_stderr_$timestamp.log" `
                -WindowStyle Hidden
            $startMethod = "Direct (admin required)"
        } catch {
            Write-Host "  ERROR: Need administrator rights!" -ForegroundColor Red
            Write-Host ""
            Write-Host "Please run this script as Administrator:" -ForegroundColor Yellow
            Write-Host "  Right-click FIX_AND_START.ps1 -> Run as Administrator"
            pause
            exit 1
        }
    } else {
        Write-Host "  ERROR: Daemon not found at $daemonPath!" -ForegroundColor Red
        pause
        exit 1
    }
}

# Step 4: Wait for startup
Write-Host ""
Write-Host "[4/6] Waiting 10 seconds for daemon initialization..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Step 5: Verify daemon is running
Write-Host ""
Write-Host "[5/6] Checking daemon status..." -ForegroundColor Yellow
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  Daemon: Running (PID: $($daemon.Id))" -ForegroundColor Green

    $port = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
    if ($port) {
        $portOwner = $port.OwningProcess
        if ($portOwner -eq $daemon.Id) {
            Write-Host "  Port 9867: Listening (owned by daemon)" -ForegroundColor Green
        } else {
            Write-Host "  Port 9867: Listening but owned by PID $portOwner (NOT daemon!)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  Port 9867: NOT listening!" -ForegroundColor Red

        # Check what port daemon might be on
        Write-Host ""
        Write-Host "  Checking recent daemon logs..." -ForegroundColor Yellow
        $latestLog = Get-ChildItem -Filter "daemon_stderr_*.log" -ErrorAction SilentlyContinue |
                     Sort-Object LastWriteTime -Descending |
                     Select-Object -First 1
        if ($latestLog) {
            $logContent = Get-Content $latestLog.FullName -Raw
            if ($logContent -match 'Starting web server on http://.*:(\d+)') {
                $actualPort = $matches[1]
                Write-Host "  Daemon is on port $actualPort (should be 9867!)" -ForegroundColor Red
            }
        }
    }
} else {
    Write-Host "  ERROR: Daemon NOT running!" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Check latest logs:" -ForegroundColor Yellow
    $latestStderr = Get-ChildItem -Filter "daemon_stderr_*.log" -ErrorAction SilentlyContinue |
                    Sort-Object LastWriteTime -Descending |
                    Select-Object -First 1
    if ($latestStderr) {
        Write-Host "  Log: $($latestStderr.FullName)"
        Get-Content $latestStderr.FullName -Tail 20 | ForEach-Object {
            Write-Host "    $_"
        }
    }
    pause
    exit 1
}

# Step 6: Test API
Write-Host ""
Write-Host "[6/6] Testing API..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Write-Host "  API: SUCCESS!" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
    Write-Host "  Status: $($health.status)" -ForegroundColor Green

    Write-Host ""
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host " DAEMON READY ON PORT 9867!" -ForegroundColor Green
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Started via: $startMethod" -ForegroundColor Cyan
    Write-Host "Web UI: http://localhost:9867" -ForegroundColor Cyan
    Write-Host ""

} catch {
    Write-Host "  API FAILED: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Daemon is running but API not responding." -ForegroundColor Yellow
    Write-Host "  This usually means:" -ForegroundColor Yellow
    Write-Host "    1. Web server thread crashed during init" -ForegroundColor Yellow
    Write-Host "    2. Wrong port binding" -ForegroundColor Yellow
    Write-Host "    3. Initialization error" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Check latest stderr log:" -ForegroundColor Yellow
    $latestLog = Get-ChildItem -Filter "daemon_stderr_*.log" -ErrorAction SilentlyContinue |
                 Sort-Object LastWriteTime -Descending |
                 Select-Object -First 1
    if ($latestLog) {
        Write-Host "  File: $($latestLog.FullName)" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  Last 50 lines:" -ForegroundColor Cyan
        Get-Content $latestLog.FullName -Tail 50 | ForEach-Object {
            Write-Host "    $_"
        }
    }
    pause
    exit 1
}

pause
