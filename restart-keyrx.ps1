# Restart KeyRx Daemon with v0.1.1 (Fixed Version)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Restarting KeyRx Daemon v0.1.1" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Stop old daemon
Write-Host "[1/3] Stopping old daemon..." -ForegroundColor Yellow
Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Verify stopped
$oldProc = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($oldProc) {
    Write-Host "[ERROR] Could not stop old daemon. Please close it manually." -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Old daemon stopped" -ForegroundColor Green

# Step 2: Start new daemon
Write-Host "[2/3] Starting new daemon v0.1.1..." -ForegroundColor Yellow
$daemonPath = "target\release\keyrx_daemon.exe"

if (-not (Test-Path $daemonPath)) {
    Write-Host "[ERROR] Daemon not found at: $daemonPath" -ForegroundColor Red
    Write-Host "[INFO] Run: cargo build --release -p keyrx_daemon" -ForegroundColor Yellow
    exit 1
}

Start-Process -FilePath $daemonPath -WindowStyle Hidden
Start-Sleep -Seconds 5

# Step 3: Verify new daemon
Write-Host "[3/3] Verifying new daemon..." -ForegroundColor Yellow

try {
    $response = Invoke-WebRequest -Uri "http://localhost:9867/api/health" -UseBasicParsing
    $health = $response.Content | ConvertFrom-Json

    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  SUCCESS!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Version: $($health.version)" -ForegroundColor Cyan
    Write-Host "Status:  $($health.status)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Web UI:  http://localhost:9867" -ForegroundColor Cyan
    Write-Host ""

    if ($health.version -eq "0.1.1") {
        Write-Host "[OK] Running v0.1.1 with all 67 bug fixes!" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Version is $($health.version), expected 0.1.1" -ForegroundColor Yellow
    }

    # Open browser
    Write-Host ""
    Write-Host "Opening web UI in browser..." -ForegroundColor Cyan
    Start-Process "http://localhost:9867"

} catch {
    Write-Host "[ERROR] Daemon not responding: $_" -ForegroundColor Red
    Write-Host "[INFO] Check if port 9867 is already in use" -ForegroundColor Yellow
    exit 1
}
