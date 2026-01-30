# Test Profile Activation on Port 9868

Write-Host "Testing default profile activation on port 9868..." -ForegroundColor Cyan
Write-Host ""

$start = Get-Date

try {
    $response = Invoke-RestMethod -Uri http://localhost:9868/api/profiles/default/activate -Method POST -TimeoutSec 60
    $elapsed = (Get-Date) - $start

    Write-Host "SUCCESS!" -ForegroundColor Green
    Write-Host "Time taken: $($elapsed.TotalSeconds) seconds" -ForegroundColor Green
    Write-Host ""
    Write-Host "Response:" -ForegroundColor Yellow
    $response | ConvertTo-Json -Depth 3
    Write-Host ""

    # Check if keys are remapping
    Write-Host "Checking metrics..." -ForegroundColor Cyan
    Start-Sleep -Seconds 2
    $metrics = Invoke-RestMethod -Uri http://localhost:9868/api/metrics -TimeoutSec 5
    Write-Host "Metrics:" -ForegroundColor Yellow
    $metrics | ConvertTo-Json -Depth 2

} catch {
    $elapsed = (Get-Date) - $start
    Write-Host "FAILED!" -ForegroundColor Red
    Write-Host "Time taken: $($elapsed.TotalSeconds) seconds" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red

    if ($_.Exception.Response) {
        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $body = $reader.ReadToEnd()
        Write-Host "Response body: $body" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Note: Web UI needs to use port 9868, not 9867!" -ForegroundColor Yellow
pause
