# Quick Reinstall - v0.1.4 with Automatic Daemon Stopping

Write-Host "Installing..." -ForegroundColor Yellow
Write-Host "(Installer will automatically stop existing daemon)" -ForegroundColor Gray
Start-Process msiexec -ArgumentList "/i","$PSScriptRoot\target\installer\KeyRx-0.1.4-x64.msi","/qn" -Wait

Write-Host "Starting daemon..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"

Write-Host "Waiting for web server..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

Write-Host "Testing..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 5
    Write-Host "SUCCESS! Daemon is running on port 9867" -ForegroundColor Green
    Write-Host "Opening web UI..." -ForegroundColor Cyan
    Start-Process "http://localhost:9867"
} catch {
    Write-Host "FAILED! Daemon not responding on port 9867" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}
