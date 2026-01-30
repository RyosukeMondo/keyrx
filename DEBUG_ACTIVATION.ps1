# Debug Profile Activation - Run as Administrator
# This script diagnoses why default profile fails while profile-a succeeds

param(
    [switch]$SkipRestart
)

$ErrorActionPreference = "Continue"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " KeyRx Activation Debug" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check admin
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: Requires administrator privileges!" -ForegroundColor Red
    Write-Host "Right-click and select 'Run as Administrator'" -ForegroundColor Yellow
    pause
    exit 1
}

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logFile = "DEBUG_$timestamp.txt"

function Log {
    param($message)
    $line = "[$(Get-Date -Format 'HH:mm:ss')] $message"
    Write-Host $line
    Add-Content -Path $logFile -Value $line
}

# Step 1: Current state
Log "=== STEP 1: Current State ==="
Log ""

Log "Binary version:"
$binary = Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ErrorAction SilentlyContinue
if ($binary) {
    Log "  Path: $($binary.FullName)"
    Log "  Size: $($binary.Length) bytes"
    Log "  Modified: $($binary.LastWriteTime.ToString('yyyy/MM/dd HH:mm:ss'))"
} else {
    Log "  ERROR: Binary not found!"
}

Log ""
Log "Running processes:"
Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue | ForEach-Object {
    Log "  PID: $($_.Id), Start: $($_.StartTime), CPU: $($_.CPU)s"
}

Log ""
Log "Port 9867 status:"
Get-NetTCPConnection -LocalPort 9867 -ErrorAction SilentlyContinue | ForEach-Object {
    Log "  State: $($_.State), PID: $($_.OwningProcess)"
}

# Step 2: Stop daemon
if (-not $SkipRestart) {
    Log ""
    Log "=== STEP 2: Stopping Daemon ==="
    taskkill /F /IM keyrx_daemon.exe 2>&1 | Out-Null
    Start-Sleep -Seconds 3

    $remaining = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($remaining) {
        Log "  WARNING: Daemon still running (PID: $($remaining.Id))"
    } else {
        Log "  OK: Daemon stopped"
    }
}

# Step 3: Start with logging
if (-not $SkipRestart) {
    Log ""
    Log "=== STEP 3: Starting Daemon with Logging ==="

    $daemonPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    $stdoutLog = "daemon_stdout_$timestamp.log"
    $stderrLog = "daemon_stderr_$timestamp.log"

    Start-Process -FilePath $daemonPath -ArgumentList "run" `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -WindowStyle Hidden

    Log "  Started daemon (logs: $stdoutLog, $stderrLog)"
    Log "  Waiting 8 seconds for startup..."
    Start-Sleep -Seconds 8
}

# Step 4: Test API
Log ""
Log "=== STEP 4: API Health Check ==="

try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 10
    Log "  OK: API responding"
    Log "  Version: $($health.version)"
    Log "  Status: $($health.status)"
} catch {
    Log "  ERROR: API not responding"
    Log "  Error: $($_.Exception.Message)"

    # Check stderr log
    if (Test-Path $stderrLog) {
        Log ""
        Log "Daemon stderr:"
        Get-Content $stderrLog -ErrorAction SilentlyContinue | ForEach-Object {
            Log "  $_"
        }
    }

    Write-Host ""
    Write-Host "CRITICAL: Daemon failed to start API server!" -ForegroundColor Red
    Write-Host "Check logs: $stderrLog" -ForegroundColor Yellow
    pause
    exit 1
}

# Step 5: Profile comparison
Log ""
Log "=== STEP 5: Profile File Comparison ==="

$configDir = "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\config"

Log ""
Log "default.rhai:"
$defaultRhai = Join-Path $configDir "default.rhai"
if (Test-Path $defaultRhai) {
    $size = (Get-Item $defaultRhai).Length
    Log "  Size: $size bytes"
    if ($size -gt 10000) {
        Log "  WARNING: Very large file ($size bytes) - may cause timeout"
    }

    # Show first 500 chars
    $content = Get-Content $defaultRhai -Raw
    Log "  First 500 chars:"
    $preview = $content.Substring(0, [Math]::Min(500, $content.Length))
    $preview -split "`n" | ForEach-Object {
        Log "    $_"
    }
}

Log ""
Log "profile-a.rhai:"
$profileARhai = Join-Path $configDir "profile-a.rhai"
if (Test-Path $profileARhai) {
    $size = (Get-Item $profileARhai).Length
    Log "  Size: $size bytes"
    $content = Get-Content $profileARhai -Raw
    Log "  Content:"
    $content -split "`n" | ForEach-Object {
        Log "    $_"
    }
}

# Step 6: Test activations
Log ""
Log "=== STEP 6: Test Profile Activations ==="

Log ""
Log "Testing profile-a activation (known working)..."
try {
    $response = Invoke-RestMethod -Uri http://localhost:9867/api/profiles/profile-a/activate `
        -Method POST -TimeoutSec 30
    Log "  OK: profile-a activated successfully"
    Start-Sleep -Seconds 2
} catch {
    Log "  ERROR: profile-a activation failed"
    Log "  Error: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $body = $reader.ReadToEnd()
        Log "  Response: $body"
    }
}

Log ""
Log "Testing default profile activation (failing)..."
try {
    $response = Invoke-RestMethod -Uri http://localhost:9867/api/profiles/default/activate `
        -Method POST -TimeoutSec 60 -Verbose
    Log "  OK: default activated successfully"
} catch {
    Log "  ERROR: default activation failed"
    Log "  Error: $($_.Exception.Message)"

    if ($_.Exception.Response) {
        $statusCode = $_.Exception.Response.StatusCode.value__
        Log "  HTTP Status: $statusCode"

        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $body = $reader.ReadToEnd()
        Log "  Response Body: $body"
    }

    # Check if daemon logs show anything
    if (Test-Path $stderrLog) {
        Log ""
        Log "Daemon stderr after failed activation:"
        Get-Content $stderrLog -Tail 50 -ErrorAction SilentlyContinue | ForEach-Object {
            Log "  $_"
        }
    }
}

# Step 7: Check metrics
Log ""
Log "=== STEP 7: Metrics Check ==="

try {
    $metrics = Invoke-RestMethod -Uri http://localhost:9867/api/metrics -TimeoutSec 5
    Log "  Metrics endpoint responding"
    Log "  Sample metrics:"
    $metrics | ConvertTo-Json -Depth 3 | ForEach-Object {
        $_ -split "`n" | Select-Object -First 20 | ForEach-Object {
            Log "    $_"
        }
    }
} catch {
    Log "  ERROR: Metrics endpoint failed"
    Log "  Error: $($_.Exception.Message)"
}

# Summary
Log ""
Log "========================================"
Log "=== DIAGNOSTIC COMPLETE ==="
Log "========================================"
Log ""
Log "Log file: $logFile"

if (Test-Path $stderrLog) {
    $stderrContent = Get-Content $stderrLog -Raw
    if ($stderrContent.Length -gt 0) {
        Log ""
        Log "Full daemon stderr log:"
        $stderrContent -split "`n" | ForEach-Object {
            Log "  $_"
        }
    }
}

Write-Host ""
Write-Host "Diagnostic complete! Check $logFile for full details." -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Check if default.rhai is too complex (>10KB)" -ForegroundColor White
Write-Host "2. Try simplifying default.rhai" -ForegroundColor White
Write-Host "3. Check daemon stderr logs for compilation errors" -ForegroundColor White
Write-Host ""
pause
