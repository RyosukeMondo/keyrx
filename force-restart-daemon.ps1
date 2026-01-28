# Force Restart KeyRx Daemon v0.1.1
# This script forcefully stops all keyrx_daemon processes and starts the new v0.1.1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Force Restart KeyRx Daemon v0.1.1" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Kill ALL keyrx_daemon processes
Write-Host "[1/4] Stopping all keyrx_daemon processes..." -ForegroundColor Yellow
Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3

# Verify all stopped
$stillRunning = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($stillRunning) {
    Write-Host "[ERROR] Could not stop all daemon processes:" -ForegroundColor Red
    $stillRunning | ForEach-Object { Write-Host "  PID $($_.Id) at $($_.Path)" -ForegroundColor Red }
    Write-Host "[INFO] Manually kill these processes and try again" -ForegroundColor Yellow
    exit 1
}
Write-Host "[OK] All old daemons stopped" -ForegroundColor Green

# Step 2: Verify new daemon exists
Write-Host "[2/4] Verifying new daemon build..." -ForegroundColor Yellow
$daemonPath = "target\release\keyrx_daemon.exe"
if (-not (Test-Path $daemonPath)) {
    Write-Host "[ERROR] Daemon not found at: $daemonPath" -ForegroundColor Red
    Write-Host "[INFO] Run: cargo build --release -p keyrx_daemon" -ForegroundColor Yellow
    exit 1
}

# Check file size (should be ~20MB)
$fileSize = (Get-Item $daemonPath).Length / 1MB
Write-Host "[OK] Daemon found (${fileSize}MB)" -ForegroundColor Green

# Step 3: Start new daemon in background
Write-Host "[3/4] Starting new daemon v0.1.1..." -ForegroundColor Yellow
$process = Start-Process -FilePath $daemonPath -WorkingDirectory "target\release" -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 5

# Verify process started
if (-not $process -or $process.HasExited) {
    Write-Host "[ERROR] Daemon process failed to start or exited immediately" -ForegroundColor Red
    Write-Host "[INFO] Check target\release\daemon.log for errors" -ForegroundColor Yellow
    exit 1
}
Write-Host "[OK] Daemon started (PID: $($process.Id))" -ForegroundColor Green

# Step 4: Verify new daemon responds with correct version
Write-Host "[4/4] Verifying version..." -ForegroundColor Yellow

$maxRetries = 10
$retryCount = 0
$success = $false

while ($retryCount -lt $maxRetries -and -not $success) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:9867/api/health" -UseBasicParsing -TimeoutSec 2
        $health = $response.Content | ConvertFrom-Json

        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  SUCCESS!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Version: $($health.version)" -ForegroundColor Cyan
        Write-Host "Status:  $($health.status)" -ForegroundColor Green
        Write-Host "PID:     $($process.Id)" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Web UI:  http://localhost:9867" -ForegroundColor Cyan
        Write-Host ""

        if ($health.version -eq "0.1.1") {
            Write-Host "[OK] Running v0.1.1 with all 67 bug fixes!" -ForegroundColor Green
        } else {
            Write-Host "[WARN] Version is $($health.version), expected 0.1.1" -ForegroundColor Yellow
            Write-Host "[INFO] The binary may not have been rebuilt correctly" -ForegroundColor Yellow
        }

        $success = $true
    }
    catch {
        $retryCount++
        Write-Host "  Waiting for daemon... ($retryCount/$maxRetries)" -ForegroundColor Gray
        Start-Sleep -Seconds 1
    }
}

if (-not $success) {
    Write-Host ""
    Write-Host "[ERROR] Daemon not responding after $maxRetries seconds" -ForegroundColor Red
    Write-Host "[INFO] Check if port 9867 is blocked or daemon crashed" -ForegroundColor Yellow
    Write-Host "[INFO] Check target\release\daemon.log for errors" -ForegroundColor Yellow
    exit 1
}

# Open browser
Write-Host ""
Write-Host "Opening web UI in browser..." -ForegroundColor Cyan
Start-Process "http://localhost:9867"
