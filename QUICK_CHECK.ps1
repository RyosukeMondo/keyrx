# Quick Check (No Admin Required)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Quick Diagnostic Check" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Daemon running?
Write-Host "[1] Daemon:" -ForegroundColor Yellow
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  Running (PID: $($daemon.Id))" -ForegroundColor Green
} else {
    Write-Host "  NOT RUNNING" -ForegroundColor Red
    exit
}

# 2. API responding?
Write-Host ""
Write-Host "[2] API Health:" -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 5
    Write-Host "  OK - Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "  FAILED: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# 3. Test activation
Write-Host ""
Write-Host "[3] Testing profile-a activation:" -ForegroundColor Yellow
$start = Get-Date
try {
    $response = Invoke-RestMethod -Uri http://localhost:9867/api/profiles/profile-a/activate -Method POST -TimeoutSec 30
    $elapsed = ((Get-Date) - $start).TotalSeconds
    Write-Host "  SUCCESS in $elapsed seconds!" -ForegroundColor Green
    $response | ConvertTo-Json
} catch {
    $elapsed = ((Get-Date) - $start).TotalSeconds
    Write-Host "  FAILED after $elapsed seconds" -ForegroundColor Red
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red

    if ($_.Exception.Response) {
        Write-Host "  Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Yellow
    }
}

Write-Host ""
