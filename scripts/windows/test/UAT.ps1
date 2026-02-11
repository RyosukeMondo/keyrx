# User Acceptance Testing for KeyRx
# MECE: End-to-end UAT scenarios

#Requires -Version 5.1

param(
    [switch]$Quick,        # Run quick tests only
    [switch]$Full,         # Run full comprehensive tests
    [int]$Port = 9867      # Web server port
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Set-ProjectRoot

Write-Step "KeyRx User Acceptance Testing"

# Ensure daemon is running
if (-not (Test-DaemonRunning)) {
    Write-Error-Custom "Daemon is not running. Start it first:"
    Write-Info "  .\scripts\windows\daemon\Start.ps1 -Release -Background -Wait"
    exit 1
}

Write-Success "Daemon is running"

# Test scenarios
$totalTests = 0
$passedTests = 0
$failedTests = @()

function Test-Scenario {
    param(
        [string]$Name,
        [scriptblock]$Test
    )

    $script:totalTests++
    Write-Host "`n▶️  Test $script:totalTests : $Name" -ForegroundColor Cyan

    try {
        $result = & $Test
        if ($result) {
            Write-Success "PASS"
            $script:passedTests++
            return $true
        } else {
            Write-Error-Custom "FAIL"
            $script:failedTests += $Name
            return $false
        }
    } catch {
        Write-Error-Custom "FAIL: $_"
        $script:failedTests += $Name
        return $false
    }
}

# Test 1: Health Check
Test-Scenario "API Health Check" {
    $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/health" -Method Get
    return $response.status -eq "ok"
}

# Test 2: Status API
Test-Scenario "Daemon Status API" {
    $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/status" -Method Get
    Write-Info "  daemon_running: $($response.daemon_running)"
    Write-Info "  active_profile: $($response.active_profile)"
    return $response.status -eq "running"
}

# Test 3: List Profiles
Test-Scenario "List Profiles" {
    $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles" -Method Get
    Write-Info "  Found $($response.profiles.Count) profiles"
    return $response.profiles.Count -gt 0
}

# Test 4: Get Profile Config
Test-Scenario "Get Profile Config" {
    $profiles = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles" -Method Get
    if ($profiles.profiles.Count -gt 0) {
        $profileName = $profiles.profiles[0].name
        $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles/$profileName/config" -Method Get
        Write-Info "  Profile: $profileName"
        Write-Info "  Config length: $($response.config.Length) chars"
        return $response.config.Length -gt 0
    }
    return $false
}

# Test 5: Activate Profile
Test-Scenario "Activate Profile" {
    $profiles = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles" -Method Get
    if ($profiles.profiles.Count -gt 0) {
        $profileName = $profiles.profiles[0].name
        $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles/$profileName/activate" -Method Post
        Write-Info "  Profile: $profileName"
        Write-Info "  Success: $($response.success)"
        if ($response.compile_time_ms) {
            Write-Info "  Compile time: $($response.compile_time_ms)ms"
        }
        if ($response.reload_time_ms) {
            Write-Info "  Reload time: $($response.reload_time_ms)ms"
        }
        return $response.success
    }
    return $false
}

# Test 6: Check Active Profile
Test-Scenario "Verify Active Profile" {
    Start-Sleep -Seconds 1  # Wait for reload
    $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/profiles/active" -Method Get
    Write-Info "  Active: $($response.active_profile)"
    return $null -ne $response.active_profile
}

if ($Full) {
    # Test 7: Diagnostics
    Test-Scenario "Diagnostics Endpoint" {
        $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/diagnostics" -Method Get
        Write-Info "  Version: $($response.version)"
        Write-Info "  Platform: $($response.platform_info.os)/$($response.platform_info.arch)"
        return $response.version -ne $null
    }

    # Test 8: Diagnostics Routes
    Test-Scenario "Diagnostics Routes List" {
        $response = Invoke-RestMethod -Uri "http://localhost:$Port/api/diagnostics/routes" -Method Get
        $apiRouteCount = $response.api_routes.Count
        Write-Info "  API routes: $apiRouteCount"
        return $apiRouteCount -gt 0
    }

    # Test 9: Web UI
    Test-Scenario "Web UI Accessible" {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/" -UseBasicParsing
        return $response.StatusCode -eq 200 -and $response.Content.Contains("keyrx")
    }
}

# Summary
Write-Host "`n"
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host " UAT RESULTS" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Total Tests: $totalTests" -ForegroundColor White
Write-Host "Passed: $passedTests" -ForegroundColor Green
Write-Host "Failed: $($failedTests.Count)" -ForegroundColor $(if ($failedTests.Count -eq 0) { "Green" } else { "Red" })

if ($failedTests.Count -gt 0) {
    Write-Host "`nFailed Tests:" -ForegroundColor Red
    foreach ($test in $failedTests) {
        Write-Host "  ❌ $test" -ForegroundColor Red
    }
}

$passRate = ($passedTests / $totalTests) * 100
Write-Host "`nPass Rate: $($passRate.ToString('F1'))%" -ForegroundColor $(if ($passRate -eq 100) { "Green" } else { "Yellow" })

if ($passRate -eq 100) {
    Write-Success "`n✨ All UAT tests passed!"
    exit 0
} else {
    Write-Warning-Custom "`n⚠️  Some tests failed"
    exit 1
}
