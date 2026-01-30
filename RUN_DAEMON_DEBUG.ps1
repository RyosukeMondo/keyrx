# Run daemon in visible console to see errors

Write-Host "Stopping any existing daemon..." -ForegroundColor Yellow
Stop-Process -Name keyrx_daemon -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

Write-Host "Starting daemon in new window..." -ForegroundColor Yellow
Write-Host "The daemon window will appear. Look for any errors!" -ForegroundColor Cyan
Write-Host ""

# Start in new window so we can see the output
Start-Process powershell -ArgumentList "-NoExit","-Command","cd 'C:\Program Files\KeyRx\bin'; .\keyrx_daemon.exe run"

Write-Host "Daemon should be starting in a new window..." -ForegroundColor Green
Write-Host "Wait 5 seconds, then check http://localhost:9867" -ForegroundColor Cyan
