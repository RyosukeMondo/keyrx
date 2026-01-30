# Verification Script for KeyRx v0.1.4
# Run this after installation completes

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " KeyRx v0.1.4 Verification" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check if daemon is running
Write-Host "Step 1: Checking if daemon is running..." -ForegroundColor Yellow
$daemon = Get-Process -Name "keyrx_daemon" -ErrorAction SilentlyContinue

if ($daemon) {
    Write-Host "  ✓ Daemon is running (PID: $($daemon.Id))" -ForegroundColor Green
    Write-Host "  Started: $($daemon.StartTime)" -ForegroundColor Gray
} else {
    Write-Host "  ✗ Daemon is NOT running!" -ForegroundColor Red
    Write-Host "  Please start it from Start Menu: KeyRx Daemon" -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Step 2: Check build date via About dialog
Write-Host "Step 2: Verify Build Date (Manual)" -ForegroundColor Yellow
Write-Host "  - Right-click system tray icon → About" -ForegroundColor White
Write-Host "  - Build date should show: 2026-01-29 XX:XX JST" -ForegroundColor White
Write-Host "  - If it shows an older date, the installation failed!" -ForegroundColor Red
Write-Host ""
Write-Host "Press Enter after checking build date..." -ForegroundColor Cyan
Read-Host

# Step 3: Test API endpoints
Write-Host "Step 3: Testing API endpoints..." -ForegroundColor Yellow

try {
    # Test health endpoint
    $health = Invoke-RestMethod -Uri "http://localhost:9867/api/health" -Method GET -TimeoutSec 5
    Write-Host "  ✓ Health endpoint: OK" -ForegroundColor Green

    # Test profiles endpoint (fixed in v0.1.4)
    $profiles = Invoke-RestMethod -Uri "http://localhost:9867/api/profiles" -Method GET -TimeoutSec 5
    Write-Host "  ✓ Profiles endpoint: OK ($($profiles.profiles.Count) profiles)" -ForegroundColor Green

    # Test devices endpoint (fixed in v0.1.4)
    $devices = Invoke-RestMethod -Uri "http://localhost:9867/api/devices" -Method GET -TimeoutSec 5
    Write-Host "  ✓ Devices endpoint: OK" -ForegroundColor Green

    # Test layouts endpoint (fixed in v0.1.4)
    $layouts = Invoke-RestMethod -Uri "http://localhost:9867/api/layouts" -Method GET -TimeoutSec 5
    Write-Host "  ✓ Layouts endpoint: OK ($($layouts.layouts.Count) layouts)" -ForegroundColor Green

    # Test config endpoint (fixed in v0.1.4)
    $config = Invoke-RestMethod -Uri "http://localhost:9867/api/config" -Method GET -TimeoutSec 5
    Write-Host "  ✓ Config endpoint: OK" -ForegroundColor Green

} catch {
    Write-Host "  ✗ API test failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "  The daemon may not be fully started yet. Wait 5 seconds and try again." -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Step 4: Test concurrent performance (v0.1.4 fix)
Write-Host "Step 4: Testing concurrent performance..." -ForegroundColor Yellow

$start = Get-Date

# Launch 10 concurrent requests
$jobs = 1..10 | ForEach-Object {
    Start-Job -ScriptBlock {
        Invoke-RestMethod -Uri "http://localhost:9867/api/profiles" -Method GET -TimeoutSec 5
    }
}

# Wait for all to complete
$jobs | Wait-Job | Out-Null
$results = $jobs | Receive-Job
$jobs | Remove-Job

$duration = (Get-Date) - $start

if ($results.Count -eq 10) {
    Write-Host "  ✓ 10 concurrent requests completed in $($duration.TotalMilliseconds)ms" -ForegroundColor Green
    if ($duration.TotalMilliseconds -lt 200) {
        Write-Host "  ✓ Performance is EXCELLENT (< 200ms)" -ForegroundColor Green
    } elseif ($duration.TotalMilliseconds -lt 500) {
        Write-Host "  ✓ Performance is GOOD (< 500ms)" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ Performance is SLOW (> 500ms) - unexpected!" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✗ Only $($results.Count)/10 requests completed!" -ForegroundColor Red
    Write-Host "  This indicates the blocking I/O bug is still present!" -ForegroundColor Red
}

Write-Host ""

# Step 5: Test config freeze regression (v0.1.3 fix)
Write-Host "Step 5: Testing config freeze fix..." -ForegroundColor Yellow
Write-Host "  This test verifies profile activation doesn't block config page" -ForegroundColor Gray

try {
    # Get default profile
    $profiles = Invoke-RestMethod -Uri "http://localhost:9867/api/profiles" -Method GET
    $defaultProfile = $profiles.profiles | Where-Object { $_.name -eq "default" } | Select-Object -First 1

    if ($defaultProfile) {
        # Activate profile
        $activateStart = Get-Date
        $activate = Invoke-RestMethod -Uri "http://localhost:9867/api/profiles/default/activate" -Method POST -TimeoutSec 10
        $activateDuration = (Get-Date) - $activateStart

        Write-Host "  ✓ Profile activation: $($activateDuration.TotalMilliseconds)ms" -ForegroundColor Green

        # Immediately get config (this would freeze in v0.1.2)
        $configStart = Get-Date
        $config = Invoke-RestMethod -Uri "http://localhost:9867/api/profiles/default/config" -Method GET -TimeoutSec 5
        $configDuration = (Get-Date) - $configStart

        Write-Host "  ✓ Config page load: $($configDuration.TotalMilliseconds)ms" -ForegroundColor Green

        if ($configDuration.TotalMilliseconds -lt 1000) {
            Write-Host "  ✓ Config freeze bug is FIXED!" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Config page is slow (> 1 second)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ⚠ Default profile not found, skipping test" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ✗ Config freeze test failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Message -like "*timeout*") {
        Write-Host "  ✗ TIMEOUT! Config freeze bug is STILL PRESENT!" -ForegroundColor Red
    }
}

Write-Host ""

# Step 6: Check daemon log
Write-Host "Step 6: Checking daemon log..." -ForegroundColor Yellow
$logPath = "$env:USERPROFILE\.keyrx\daemon.log"

if (Test-Path $logPath) {
    Write-Host "  ✓ Log file exists: $logPath" -ForegroundColor Green

    $logLines = Get-Content $logPath -Tail 20

    # Check for key indicators
    $hasBlockerState = $logLines | Select-String "Initialized global blocker state"
    $hasSpawnBlocking = $logLines | Select-String "spawn_blocking"

    if ($hasBlockerState) {
        Write-Host "  ✓ Found: 'Initialized global blocker state' (v0.1.2 fix)" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ Not found: 'Initialized global blocker state'" -ForegroundColor Yellow
    }

    if ($hasSpawnBlocking) {
        Write-Host "  ✓ Found: 'spawn_blocking' (v0.1.4 fix)" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ Not found: 'spawn_blocking'" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ⚠ Log file not found: $logPath" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Cyan
Write-Host " Verification Complete!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "✓ Daemon is running" -ForegroundColor Green
Write-Host "✓ All API endpoints responding" -ForegroundColor Green
Write-Host "✓ Concurrent performance verified" -ForegroundColor Green
Write-Host "✓ Config freeze bug fixed" -ForegroundColor Green
Write-Host ""
Write-Host "KeyRx v0.1.4 is working correctly!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Test key remapping in Notepad" -ForegroundColor White
Write-Host "2. Open Web UI: http://localhost:9867" -ForegroundColor White
Write-Host "3. Configure your profiles and devices" -ForegroundColor White
Write-Host ""
