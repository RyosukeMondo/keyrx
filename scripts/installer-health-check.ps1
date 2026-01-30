# Installer Health Check - Comprehensive MSI and Installation Verification
#
# This script performs comprehensive pre-flight and post-install validation
# to catch issues before users discover them.
#
# Usage:
#   .\scripts\installer-health-check.ps1                    # Full health check
#   .\scripts\installer-health-check.ps1 -PreInstall        # Pre-install validation only
#   .\scripts\installer-health-check.ps1 -PostInstall       # Post-install validation only
#   .\scripts\installer-health-check.ps1 -Json              # JSON output for CI

param(
    [switch]$PreInstall,
    [switch]$PostInstall,
    [switch]$Json,
    [string]$MsiPath = "target\installer\KeyRx-0.1.5-x64.msi"
)

$ErrorActionPreference = "Continue"

# Results tracking
$results = @{
    timestamp = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    checks = @()
    passed = 0
    failed = 0
    warnings = 0
}

function Add-CheckResult {
    param(
        [string]$Name,
        [string]$Status,  # "pass", "fail", "warn"
        [string]$Message,
        [hashtable]$Details = @{}
    )

    $result = @{
        name = $Name
        status = $Status
        message = $Message
        details = $Details
    }

    $results.checks += $result

    switch ($Status) {
        "pass" {
            $results.passed++
            if (-not $Json) {
                Write-Host "  ✓ $Name" -ForegroundColor Green
                Write-Host "    $Message" -ForegroundColor Gray
            }
        }
        "fail" {
            $results.failed++
            if (-not $Json) {
                Write-Host "  ✗ $Name" -ForegroundColor Red
                Write-Host "    $Message" -ForegroundColor Gray
            }
        }
        "warn" {
            $results.warnings++
            if (-not $Json) {
                Write-Host "  ⚠ $Name" -ForegroundColor Yellow
                Write-Host "    $Message" -ForegroundColor Gray
            }
        }
    }
}

function Test-AdminRights {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-BinaryVersion {
    param([string]$BinaryPath)

    try {
        if (-not (Test-Path $BinaryPath)) {
            return $null
        }

        $versionOutput = & $BinaryPath --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $versionOutput -match "KeyRx v([\d.]+)") {
            return $matches[1]
        }
        return $null
    } catch {
        return $null
    }
}

function Get-MsiVersion {
    param([string]$MsiPath)

    try {
        $windowsInstaller = New-Object -ComObject WindowsInstaller.Installer
        $database = $windowsInstaller.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $windowsInstaller, @($MsiPath, 0))
        $query = "SELECT Value FROM Property WHERE Property='ProductVersion'"
        $view = $database.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $database, ($query))
        $view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null)
        $record = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)
        $version = $record.GetType().InvokeMember("StringData", "GetProperty", $null, $record, 1)
        return $version
    } catch {
        return $null
    }
}

# Header
if (-not $Json) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " KeyRx Installer Health Check" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
}

# Determine which checks to run
$runPreInstall = $PreInstall -or (-not $PostInstall)
$runPostInstall = $PostInstall -or (-not $PreInstall)

# PRE-INSTALL CHECKS
if ($runPreInstall) {
    if (-not $Json) {
        Write-Host "[Pre-Install Validation]" -ForegroundColor Cyan
        Write-Host ""
    }

    # 1. Admin rights
    if (-not $Json) { Write-Host "[1/6] Checking admin rights..." -ForegroundColor Yellow }
    if (Test-AdminRights) {
        Add-CheckResult -Name "Admin Rights" -Status "pass" -Message "Running with administrator privileges"
    } else {
        Add-CheckResult -Name "Admin Rights" -Status "fail" -Message "NOT running as administrator. Installation will fail."
    }

    # 2. MSI file exists
    if (-not $Json) { Write-Host "[2/6] Checking MSI file..." -ForegroundColor Yellow }
    if (Test-Path $MsiPath) {
        $msiItem = Get-Item $MsiPath
        $sizeInMB = [math]::Round($msiItem.Length / 1MB, 2)
        Add-CheckResult -Name "MSI File" -Status "pass" -Message "Found MSI ($sizeInMB MB)" -Details @{
            path = $MsiPath
            size = $msiItem.Length
            timestamp = $msiItem.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        }
    } else {
        Add-CheckResult -Name "MSI File" -Status "fail" -Message "MSI not found at: $MsiPath"
    }

    # 3. MSI integrity
    if (-not $Json) { Write-Host "[3/6] Checking MSI integrity..." -ForegroundColor Yellow }
    $msiVersion = Get-MsiVersion -MsiPath $MsiPath
    if ($msiVersion) {
        Add-CheckResult -Name "MSI Integrity" -Status "pass" -Message "MSI version: $msiVersion" -Details @{
            version = $msiVersion
        }
    } else {
        Add-CheckResult -Name "MSI Integrity" -Status "fail" -Message "Could not read MSI database (corrupted or invalid)"
    }

    # 4. Source binary version
    if (-not $Json) { Write-Host "[4/6] Checking source binary version..." -ForegroundColor Yellow }
    $sourceBinary = "target\release\keyrx_daemon.exe"
    $sourceBinaryVersion = Get-BinaryVersion -BinaryPath $sourceBinary
    if ($sourceBinaryVersion) {
        Add-CheckResult -Name "Source Binary" -Status "pass" -Message "Version: $sourceBinaryVersion" -Details @{
            path = $sourceBinary
            version = $sourceBinaryVersion
        }
    } else {
        Add-CheckResult -Name "Source Binary" -Status "warn" -Message "Could not determine version (binary may not exist or is invalid)"
    }

    # 5. Version consistency
    if (-not $Json) { Write-Host "[5/6] Checking version consistency..." -ForegroundColor Yellow }
    if ($msiVersion -and $sourceBinaryVersion) {
        # MSI version is "0.1.5.0", binary version is "0.1.5"
        $msiVersionShort = $msiVersion.Split('.')[0..2] -join '.'
        if ($msiVersionShort -eq $sourceBinaryVersion) {
            Add-CheckResult -Name "Version Match" -Status "pass" -Message "MSI and binary versions match ($sourceBinaryVersion)"
        } else {
            Add-CheckResult -Name "Version Match" -Status "fail" -Message "VERSION MISMATCH! MSI: $msiVersion, Binary: $sourceBinaryVersion"
        }
    } else {
        Add-CheckResult -Name "Version Match" -Status "warn" -Message "Could not verify version consistency (missing MSI or binary)"
    }

    # 6. Existing installation
    if (-not $Json) { Write-Host "[6/6] Checking existing installation..." -ForegroundColor Yellow }
    $installedPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    if (Test-Path $installedPath) {
        $installedVersion = Get-BinaryVersion -BinaryPath $installedPath
        if ($installedVersion) {
            Add-CheckResult -Name "Existing Installation" -Status "warn" -Message "KeyRx v$installedVersion already installed (will be upgraded)" -Details @{
                path = $installedPath
                version = $installedVersion
            }
        } else {
            Add-CheckResult -Name "Existing Installation" -Status "warn" -Message "KeyRx is installed but version unknown"
        }
    } else {
        Add-CheckResult -Name "Existing Installation" -Status "pass" -Message "No existing installation found (fresh install)"
    }

    if (-not $Json) { Write-Host "" }
}

# POST-INSTALL CHECKS
if ($runPostInstall) {
    if (-not $Json) {
        Write-Host "[Post-Install Validation]" -ForegroundColor Cyan
        Write-Host ""
    }

    # 7. Binary exists
    if (-not $Json) { Write-Host "[7/11] Checking installed binary..." -ForegroundColor Yellow }
    $installedPath = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    if (Test-Path $installedPath) {
        $installedItem = Get-Item $installedPath
        $sizeInMB = [math]::Round($installedItem.Length / 1MB, 2)
        Add-CheckResult -Name "Binary Installed" -Status "pass" -Message "Binary found ($sizeInMB MB)" -Details @{
            path = $installedPath
            size = $installedItem.Length
            timestamp = $installedItem.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        }
    } else {
        Add-CheckResult -Name "Binary Installed" -Status "fail" -Message "Binary NOT found at: $installedPath"
    }

    # 8. Binary version
    if (-not $Json) { Write-Host "[8/11] Checking binary version..." -ForegroundColor Yellow }
    $installedVersion = Get-BinaryVersion -BinaryPath $installedPath
    if ($installedVersion) {
        Add-CheckResult -Name "Binary Version" -Status "pass" -Message "Version: $installedVersion" -Details @{
            version = $installedVersion
        }
    } else {
        Add-CheckResult -Name "Binary Version" -Status "fail" -Message "Could not determine version (binary invalid or corrupted)"
    }

    # 9. PATH environment variable
    if (-not $Json) { Write-Host "[9/11] Checking PATH..." -ForegroundColor Yellow }
    $path = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    if ($path -like "*C:\Program Files\KeyRx\bin*") {
        Add-CheckResult -Name "PATH Variable" -Status "pass" -Message "KeyRx bin directory in PATH"
    } else {
        Add-CheckResult -Name "PATH Variable" -Status "warn" -Message "KeyRx bin directory NOT in PATH (may require restart)"
    }

    # 10. Daemon process
    if (-not $Json) { Write-Host "[10/11] Checking daemon process..." -ForegroundColor Yellow }
    $process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
    if ($process) {
        Add-CheckResult -Name "Daemon Running" -Status "pass" -Message "Daemon is running (PID: $($process.Id))" -Details @{
            pid = $process.Id
            startTime = $process.StartTime.ToString("yyyy-MM-dd HH:mm:ss")
        }
    } else {
        Add-CheckResult -Name "Daemon Running" -Status "warn" -Message "Daemon is NOT running (normal for fresh install, start manually)"
    }

    # 11. API connectivity
    if (-not $Json) { Write-Host "[11/11] Checking API connectivity..." -ForegroundColor Yellow }
    try {
        $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 5 -ErrorAction Stop
        if ($health.status -eq "ok") {
            Add-CheckResult -Name "API Health" -Status "pass" -Message "API responding (version: $($health.version))" -Details @{
                apiVersion = $health.version
            }
        } else {
            Add-CheckResult -Name "API Health" -Status "warn" -Message "API health check returned non-OK status: $($health.status)"
        }
    } catch {
        if ($process) {
            Add-CheckResult -Name "API Health" -Status "fail" -Message "API not responding: $($_.Exception.Message)"
        } else {
            Add-CheckResult -Name "API Health" -Status "warn" -Message "API not responding (daemon not running)"
        }
    }

    if (-not $Json) { Write-Host "" }
}

# Summary
if (-not $Json) {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " Summary" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Passed:   $($results.passed)" -ForegroundColor Green
    Write-Host "  Failed:   $($results.failed)" -ForegroundColor $(if ($results.failed -gt 0) { "Red" } else { "Gray" })
    Write-Host "  Warnings: $($results.warnings)" -ForegroundColor $(if ($results.warnings -gt 0) { "Yellow" } else { "Gray" })
    Write-Host ""

    if ($results.failed -gt 0) {
        Write-Host "========================================" -ForegroundColor Red
        Write-Host " HEALTH CHECK FAILED" -ForegroundColor Red
        Write-Host "========================================" -ForegroundColor Red
        Write-Host ""
        Write-Host "Critical issues found. Please review the failures above." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Common fixes:" -ForegroundColor Cyan
        Write-Host "  1. Run as Administrator" -ForegroundColor White
        Write-Host "  2. Rebuild binaries: cargo build --release" -ForegroundColor White
        Write-Host "  3. Rebuild MSI: .\scripts\build_windows_installer.ps1" -ForegroundColor White
        Write-Host "  4. Check version consistency in Cargo.toml and keyrx_installer.wxs" -ForegroundColor White
        Write-Host ""
        exit 1
    } elseif ($results.warnings -gt 0) {
        Write-Host "========================================" -ForegroundColor Yellow
        Write-Host " HEALTH CHECK PASSED WITH WARNINGS" -ForegroundColor Yellow
        Write-Host "========================================" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "All critical checks passed, but some warnings were found." -ForegroundColor Yellow
        Write-Host "Review the warnings above and address if necessary." -ForegroundColor Yellow
        Write-Host ""
        exit 0
    } else {
        Write-Host "========================================" -ForegroundColor Green
        Write-Host " HEALTH CHECK PASSED" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "All checks passed! Installation is healthy." -ForegroundColor Green
        Write-Host ""
        exit 0
    }
} else {
    # JSON output
    $results | ConvertTo-Json -Depth 10
    exit $(if ($results.failed -gt 0) { 1 } else { 0 })
}
