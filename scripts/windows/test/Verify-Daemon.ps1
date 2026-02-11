# Verify daemon version and functionality
# MECE: Autonomous daemon verification without browser

#Requires -Version 5.1

param(
    [int]$Port = 9867,          # Daemon port
    [string]$ExpectedVersion    # Expected version (optional)
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Write-Step "KeyRx Daemon Verification"

# Helper function to call API using curl (avoids PowerShell header issues)
function Invoke-CurlApi {
    param([string]$Endpoint, [int]$Port, [string]$Method = "GET")

    $url = "http://localhost:$Port$Endpoint"
    try {
        if ($Method -eq "POST") {
            $json = & curl.exe -s -X POST $url 2>&1
        } else {
            $json = & curl.exe -s $url 2>&1
        }

        if ($LASTEXITCODE -ne 0) {
            return $null
        }

        return $json | ConvertFrom-Json
    } catch {
        Write-Warning-Custom "API call failed: $_"
        return $null
    }
}

# Test 1: Health Check with Version
Write-Host "`n▶️  Test 1: Health & Version Check" -ForegroundColor Cyan
try {
    $health = Invoke-CurlApi -Endpoint "/api/health" -Port $Port
    if (-not $health) {
        throw "No response from health endpoint"
    }

    Write-Success "  Version: $($health.version)"
    Write-Info "  Build Time: $($health.build_time)"
    Write-Info "  Git Hash: $($health.git_hash)"
    Write-Info "  Platform: $($health.platform)"
    Write-Info "  Admin Rights: $($health.admin_rights)"
    Write-Info "  Hook Installed: $($health.hook_installed)"

    # Verify expected version if provided
    if ($ExpectedVersion -and $health.version -ne $ExpectedVersion) {
        Write-Error-Custom "  Version mismatch! Expected: $ExpectedVersion, Got: $($health.version)"
        exit 1
    }
} catch {
    Write-Error-Custom "  Health check failed: $_"
    exit 1
}

# Test 2: Daemon Status
Write-Host "`n▶️  Test 2: Daemon Status" -ForegroundColor Cyan
try {
    $status = Invoke-CurlApi -Endpoint "/api/status" -Port $Port
    if (-not $status) {
        throw "No response from status endpoint"
    }

    $daemonStatus = if ($status.daemon_running) { "Running [OK]" } else { "Not Running [FAIL]" }
    $statusColor = if ($status.daemon_running) { "Green" } else { "Red" }

    Write-Host "  Daemon: " -NoNewline
    Write-Host $daemonStatus -ForegroundColor $statusColor
    Write-Info "  Uptime: $($status.uptime_secs) seconds"
    Write-Info "  Active Profile: $($status.active_profile)"
    Write-Info "  Devices: $($status.device_count)"

    if (-not $status.daemon_running) {
        Write-Warning-Custom "  Daemon event loop is not running!"
    }
} catch {
    Write-Error-Custom "  Status check failed: $_"
    exit 1
}

# Test 3: Profile Activation
Write-Host "`n▶️  Test 3: Profile Activation Test" -ForegroundColor Cyan
try {
    # List profiles
    $profilesData = Invoke-CurlApi -Endpoint "/api/profiles" -Port $Port
    if (-not $profilesData) {
        throw "No response from profiles endpoint"
    }
    $profiles = $profilesData.profiles

    Write-Info "  Found $($profiles.Count) profiles"

    if ($profiles.Count -gt 0) {
        # Get first profile
        $testProfile = $profiles[0].name
        Write-Info "  Testing with profile: $testProfile"

        # Activate profile
        $activate = Invoke-CurlApi -Endpoint "/api/profiles/$testProfile/activate" -Port $Port -Method POST

        if ($activate.success) {
            Write-Success "  Profile activated successfully!"
            Write-Info "  Compile time: $($activate.compile_time_ms)ms"
            Write-Info "  Reload time: $($activate.reload_time_ms)ms"
        } else {
            Write-Warning-Custom "  Profile activation failed: $($activate.error)"
        }
    } else {
        Write-Warning-Custom "  No profiles available for testing"
    }
} catch {
    Write-Error-Custom "  Profile test failed: $_"
}

# Summary
Write-Host "`n" -NoNewline
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host " VERIFICATION COMPLETE" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Success "✨ All critical checks passed!"
Write-Info "Daemon URL: http://localhost:$Port"
