# Test Installation - Verify Binary Version and Functionality
# This script catches deployment issues before user discovers them

param(
    [string]$ExpectedTimestamp = ""  # Auto-detect from source binary if not specified
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Testing KeyRx Installation" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Auto-detect expected timestamp from source binary if not specified
if ([string]::IsNullOrEmpty($ExpectedTimestamp)) {
    $sourceBinary = "target\release\keyrx_daemon.exe"
    if (Test-Path $sourceBinary) {
        $sourceItem = Get-Item $sourceBinary
        $ExpectedTimestamp = $sourceItem.LastWriteTime.ToString("yyyy/MM/dd HH:mm:ss")
        Write-Host "Auto-detected expected timestamp: $ExpectedTimestamp" -ForegroundColor Gray
    } else {
        Write-Host "Warning: Could not auto-detect timestamp (source binary not found)" -ForegroundColor Yellow
        $ExpectedTimestamp = "unknown"
    }
}
Write-Host ""

$failed = $false

# Test 1: Binary exists
Write-Host "[1/7] Checking binary exists..." -ForegroundColor Yellow
$binaryPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
if (Test-Path $binaryPath) {
    Write-Host "  ✓ Binary found" -ForegroundColor Green
} else {
    Write-Host "  ✗ Binary NOT found at: $binaryPath" -ForegroundColor Red
    $failed = $true
}

# Test 2: Binary timestamp
Write-Host "[2/7] Checking binary timestamp..." -ForegroundColor Yellow
if (Test-Path $binaryPath) {
    $binary = Get-Item $binaryPath
    $timestamp = $binary.LastWriteTime.ToString("yyyy/MM/dd HH:mm:ss")

    if ($timestamp -eq $ExpectedTimestamp) {
        Write-Host "  ✓ Timestamp matches: $timestamp" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Timestamp mismatch!" -ForegroundColor Red
        Write-Host "    Expected: $ExpectedTimestamp" -ForegroundColor Gray
        Write-Host "    Got:      $timestamp" -ForegroundColor Gray
        $failed = $true
    }
}

# Test 3: Daemon process
Write-Host "[3/7] Checking daemon process..." -ForegroundColor Yellow
$process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($process) {
    Write-Host "  ✓ Daemon is running (PID: $($process.Id))" -ForegroundColor Green
} else {
    Write-Host "  ✗ Daemon is NOT running" -ForegroundColor Red
    Write-Host "    Starting daemon..." -ForegroundColor Yellow
    Start-Process $binaryPath -ArgumentList "run"
    Start-Sleep -Seconds 8
    $process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($process) {
        Write-Host "  ✓ Daemon started successfully (PID: $($process.Id))" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Failed to start daemon" -ForegroundColor Red
        $failed = $true
    }
}

# Test 4: API health
Write-Host "[4/7] Checking API health..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 10
    if ($health.status -eq "ok") {
        Write-Host "  ✓ API is responding (version: $($health.version))" -ForegroundColor Green
    } else {
        Write-Host "  ✗ API health check failed: $($health.status)" -ForegroundColor Red
        $failed = $true
    }
} catch {
    Write-Host "  ✗ API not responding: $($_.Exception.Message)" -ForegroundColor Red
    $failed = $true
}

# Test 5: Profiles endpoint
Write-Host "[5/7] Checking profiles endpoint..." -ForegroundColor Yellow
try {
    $profiles = Invoke-RestMethod http://localhost:9867/api/profiles -TimeoutSec 5
    $count = $profiles.profiles.Count
    Write-Host "  ✓ Profiles endpoint works ($count profiles found)" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Profiles endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
    $failed = $true
}

# Test 6: Configuration loaded (indicates remapping capability)
Write-Host "[6/7] Checking configuration..." -ForegroundColor Yellow
try {
    # Check if a profile is active
    $profiles = Invoke-RestMethod http://localhost:9867/api/profiles -TimeoutSec 5
    $activeProfile = $profiles.profiles | Where-Object { $_.active -eq $true }

    if ($activeProfile) {
        Write-Host "  ✓ Active profile found: $($activeProfile.name)" -ForegroundColor Green
        Write-Host "    Keyboard remapping should work" -ForegroundColor Gray
    } else {
        Write-Host "  ⚠ No active profile" -ForegroundColor Yellow
        Write-Host "    Go to http://localhost:9867/profiles and activate a profile" -ForegroundColor Gray
        Write-Host "    Keyboard remapping requires an active profile" -ForegroundColor Gray
        # Not a failure - just needs configuration
    }

    # Check if config endpoint works
    $config = Invoke-RestMethod http://localhost:9867/api/config -TimeoutSec 5
    if ($config) {
        Write-Host "  ✓ Configuration service working" -ForegroundColor Green
    }
} catch {
    Write-Host "  ✗ Configuration test failed: $($_.Exception.Message)" -ForegroundColor Red
    $failed = $true
}

# Test 7: Port listening
Write-Host "[7/7] Checking port 9867..." -ForegroundColor Yellow
$portOpen = Test-NetConnection -ComputerName localhost -Port 9867 -InformationLevel Quiet -WarningAction SilentlyContinue
if ($portOpen) {
    Write-Host "  ✓ Port 9867 is open" -ForegroundColor Green
} else {
    Write-Host "  ✗ Port 9867 is NOT open" -ForegroundColor Red
    $failed = $true
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
if ($failed) {
    Write-Host " Installation Test FAILED" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Action Required:" -ForegroundColor Yellow
    Write-Host "1. Right-click UPDATE_BINARY.ps1 → Run as Administrator" -ForegroundColor White
    Write-Host "2. Re-run this test script" -ForegroundColor White
    exit 1
} else {
    Write-Host " Installation Test PASSED" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "✓ All checks passed!" -ForegroundColor Green
    Write-Host "✓ Keyboard remapping should work correctly" -ForegroundColor Green
    exit 0
}
