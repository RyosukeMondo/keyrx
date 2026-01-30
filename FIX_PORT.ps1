#Requires -RunAsAdministrator
# Fix Port Mismatch - Change daemon to port 9867

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Fix Port Mismatch" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$settingsPath = "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\settings.json"

Write-Host "Current settings:" -ForegroundColor Yellow
if (Test-Path $settingsPath) {
    Get-Content $settingsPath
} else {
    Write-Host "  No settings file found" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Changing port to 9867..." -ForegroundColor Cyan

# Write new settings
$newSettings = @{
    port = 9867
}

$newSettings | ConvertTo-Json | Set-Content $settingsPath

Write-Host "  OK: Settings updated" -ForegroundColor Green
Write-Host ""
Write-Host "New settings:" -ForegroundColor Yellow
Get-Content $settingsPath
Write-Host ""

Write-Host "Restarting daemon..." -ForegroundColor Cyan
taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
Start-Sleep -Seconds 2

Start-Process -FilePath "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run" -WindowStyle Hidden
Write-Host "  OK: Daemon restarted on port 9867" -ForegroundColor Green

Start-Sleep -Seconds 5

Write-Host ""
Write-Host "Testing connection..." -ForegroundColor Cyan

try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 5
    Write-Host "  OK: API responding on port 9867" -ForegroundColor Green
    Write-Host "  Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: API not responding on 9867" -ForegroundColor Red
    Write-Host "  $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Open http://localhost:9867" -ForegroundColor White
Write-Host "2. Try activating profiles" -ForegroundColor White
Write-Host "3. Check metrics page" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
pause
