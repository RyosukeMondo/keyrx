# Diagnose Installation - Comprehensive Troubleshooting
#
# This script diagnoses installation issues by checking all version sources,
# binary timestamps, daemon state, admin rights, file locks, and suggests fixes.
#
# Usage:
#   .\scripts\diagnose-installation.ps1                     # Full diagnosis
#   .\scripts\diagnose-installation.ps1 -Json               # JSON output
#   .\scripts\diagnose-installation.ps1 -AutoFix            # Attempt auto-fix

param(
    [switch]$Json,
    [switch]$AutoFix
)

$ErrorActionPreference = "Continue"

# Diagnostic results
$diagnosis = @{
    timestamp = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    systemInfo = @{}
    versions = @{}
    files = @{}
    processes = @{}
    network = @{}
    issues = @()
    suggestions = @()
}

function Add-Issue {
    param(
        [string]$Severity,  # "critical", "error", "warning", "info"
        [string]$Message,
        [string]$Fix = ""
    )

    $issue = @{
        severity = $Severity
        message = $Message
        fix = $Fix
    }

    $diagnosis.issues += $issue

    if (-not $Json) {
        $color = switch ($Severity) {
            "critical" { "Red" }
            "error" { "Red" }
            "warning" { "Yellow" }
            "info" { "Cyan" }
        }
        $icon = switch ($Severity) {
            "critical" { "⛔" }
            "error" { "✗" }
            "warning" { "⚠" }
            "info" { "ℹ" }
        }
        Write-Host "  $icon $Message" -ForegroundColor $color
        if ($Fix) {
            Write-Host "     Fix: $Fix" -ForegroundColor Gray
        }
    }
}

function Test-AdminRights {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-FileVersion {
    param([string]$Path)

    try {
        if (-not (Test-Path $Path)) {
            return $null
        }

        $versionOutput = & $Path --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $versionOutput -match "KeyRx v([\d.]+)") {
            return $matches[1]
        }
        return $null
    } catch {
        return $null
    }
}

function Get-FileLocks {
    param([string]$Path)

    try {
        $processes = Get-Process | Where-Object {
            $_.Modules | Where-Object { $_.FileName -eq $Path }
        }
        return $processes
    } catch {
        return @()
    }
}

# Header
if (-not $Json) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " KeyRx Installation Diagnostics" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
}

# 1. SYSTEM INFORMATION
if (-not $Json) {
    Write-Host "[System Information]" -ForegroundColor Cyan
}

$diagnosis.systemInfo.os = [System.Environment]::OSVersion.VersionString
$diagnosis.systemInfo.username = $env:USERNAME
$diagnosis.systemInfo.computername = $env:COMPUTERNAME
$diagnosis.systemInfo.isAdmin = Test-AdminRights
$diagnosis.systemInfo.powershellVersion = $PSVersionTable.PSVersion.ToString()

if (-not $Json) {
    Write-Host "  OS: $($diagnosis.systemInfo.os)" -ForegroundColor Gray
    Write-Host "  User: $($diagnosis.systemInfo.username)" -ForegroundColor Gray
    Write-Host "  Admin: $($diagnosis.systemInfo.isAdmin)" -ForegroundColor Gray
    Write-Host ""
}

if (-not $diagnosis.systemInfo.isAdmin) {
    Add-Issue -Severity "critical" -Message "NOT running as Administrator" -Fix "Right-click PowerShell and select 'Run as Administrator'"
}

# 2. VERSION CHECK
if (-not $Json) {
    Write-Host "[Version Analysis]" -ForegroundColor Cyan
}

# Cargo.toml version
$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([\d.]+)"') {
    $diagnosis.versions.cargoToml = $matches[1]
    if (-not $Json) {
        Write-Host "  Cargo.toml: $($diagnosis.versions.cargoToml)" -ForegroundColor Gray
    }
}

# WiX installer version
$wixFile = Get-Content "keyrx_daemon\keyrx_installer.wxs" -Raw
if ($wixFile -match 'Version="([\d.]+)"') {
    $diagnosis.versions.wixInstaller = $matches[1]
    if (-not $Json) {
        Write-Host "  WiX Installer: $($diagnosis.versions.wixInstaller)" -ForegroundColor Gray
    }
}

# Source binary version
$sourceBinary = "target\release\keyrx_daemon.exe"
$diagnosis.versions.sourceBinary = Get-FileVersion -Path $sourceBinary
if (-not $Json) {
    Write-Host "  Source Binary: $($diagnosis.versions.sourceBinary)" -ForegroundColor Gray
}

# Installed binary version
$installedBinary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$diagnosis.versions.installedBinary = Get-FileVersion -Path $installedBinary
if (-not $Json) {
    Write-Host "  Installed Binary: $($diagnosis.versions.installedBinary)" -ForegroundColor Gray
}

# MSI version
$msiPath = "target\installer\KeyRx-0.1.5-x64.msi"
if (Test-Path $msiPath) {
    try {
        $windowsInstaller = New-Object -ComObject WindowsInstaller.Installer
        $database = $windowsInstaller.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $windowsInstaller, @($msiPath, 0))
        $query = "SELECT Value FROM Property WHERE Property='ProductVersion'"
        $view = $database.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $database, ($query))
        $view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null)
        $record = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)
        $diagnosis.versions.msi = $record.GetType().InvokeMember("StringData", "GetProperty", $null, $record, 1)
        if (-not $Json) {
            Write-Host "  MSI Package: $($diagnosis.versions.msi)" -ForegroundColor Gray
        }
    } catch {
        if (-not $Json) {
            Write-Host "  MSI Package: ERROR - Could not read" -ForegroundColor Red
        }
    }
}

if (-not $Json) { Write-Host "" }

# Version consistency check
$versions = @(
    $diagnosis.versions.cargoToml,
    $diagnosis.versions.wixInstaller,
    $diagnosis.versions.sourceBinary,
    $diagnosis.versions.installedBinary,
    $diagnosis.versions.msi
) | Where-Object { $_ } | ForEach-Object { $_.Split('.')[0..2] -join '.' }

$uniqueVersions = $versions | Select-Object -Unique
if ($uniqueVersions.Count -gt 1) {
    Add-Issue -Severity "error" -Message "VERSION MISMATCH DETECTED: Multiple versions found: $($uniqueVersions -join ', ')" -Fix "Run scripts\force-clean-reinstall.ps1 to rebuild and reinstall"
}

# 3. FILE ANALYSIS
if (-not $Json) {
    Write-Host "[File Analysis]" -ForegroundColor Cyan
}

# Source binary
if (Test-Path $sourceBinary) {
    $sourceItem = Get-Item $sourceBinary
    $diagnosis.files.sourceBinary = @{
        exists = $true
        path = $sourceBinary
        size = $sourceItem.Length
        timestamp = $sourceItem.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        ageInHours = [math]::Round(((Get-Date) - $sourceItem.LastWriteTime).TotalHours, 1)
    }
    if (-not $Json) {
        Write-Host "  Source Binary: EXISTS" -ForegroundColor Green
        Write-Host "    Size: $([math]::Round($sourceItem.Length / 1MB, 2)) MB" -ForegroundColor Gray
        Write-Host "    Modified: $($diagnosis.files.sourceBinary.timestamp) ($($diagnosis.files.sourceBinary.ageInHours) hours ago)" -ForegroundColor Gray
    }
} else {
    $diagnosis.files.sourceBinary = @{ exists = $false }
    Add-Issue -Severity "error" -Message "Source binary not found: $sourceBinary" -Fix "Run: cargo build --release"
    if (-not $Json) {
        Write-Host "  Source Binary: NOT FOUND" -ForegroundColor Red
    }
}

# Installed binary
if (Test-Path $installedBinary) {
    $installedItem = Get-Item $installedBinary
    $diagnosis.files.installedBinary = @{
        exists = $true
        path = $installedBinary
        size = $installedItem.Length
        timestamp = $installedItem.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        ageInHours = [math]::Round(((Get-Date) - $installedItem.LastWriteTime).TotalHours, 1)
    }
    if (-not $Json) {
        Write-Host "  Installed Binary: EXISTS" -ForegroundColor Green
        Write-Host "    Size: $([math]::Round($installedItem.Length / 1MB, 2)) MB" -ForegroundColor Gray
        Write-Host "    Modified: $($diagnosis.files.installedBinary.timestamp) ($($diagnosis.files.installedBinary.ageInHours) hours ago)" -ForegroundColor Gray
    }

    # Check if installed binary is older than source
    if ($diagnosis.files.sourceBinary.exists -and $installedItem.LastWriteTime -lt $sourceItem.LastWriteTime) {
        $ageDiff = [math]::Round(($sourceItem.LastWriteTime - $installedItem.LastWriteTime).TotalHours, 1)
        Add-Issue -Severity "warning" -Message "Installed binary is $ageDiff hours older than source binary" -Fix "Reinstall: msiexec /i target\installer\KeyRx-0.1.5-x64.msi"
    }

    # Check for file locks
    $locks = Get-FileLocks -Path $installedBinary
    if ($locks.Count -gt 0) {
        $diagnosis.files.installedBinaryLocks = $locks | ForEach-Object { @{ pid = $_.Id; name = $_.Name } }
        Add-Issue -Severity "info" -Message "Installed binary is in use by process(es): $($locks.Name -join ', ')" -Fix "Stop daemon before reinstalling"
    }
} else {
    $diagnosis.files.installedBinary = @{ exists = $false }
    Add-Issue -Severity "error" -Message "Installed binary not found: $installedBinary" -Fix "Install: msiexec /i target\installer\KeyRx-0.1.5-x64.msi"
    if (-not $Json) {
        Write-Host "  Installed Binary: NOT FOUND" -ForegroundColor Red
    }
}

# MSI file
if (Test-Path $msiPath) {
    $msiItem = Get-Item $msiPath
    $diagnosis.files.msi = @{
        exists = $true
        path = $msiPath
        size = $msiItem.Length
        timestamp = $msiItem.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
    }
    if (-not $Json) {
        Write-Host "  MSI Package: EXISTS" -ForegroundColor Green
        Write-Host "    Size: $([math]::Round($msiItem.Length / 1MB, 2)) MB" -ForegroundColor Gray
        Write-Host "    Modified: $($diagnosis.files.msi.timestamp)" -ForegroundColor Gray
    }
} else {
    $diagnosis.files.msi = @{ exists = $false }
    Add-Issue -Severity "error" -Message "MSI package not found: $msiPath" -Fix "Build MSI: .\scripts\build_windows_installer.ps1"
    if (-not $Json) {
        Write-Host "  MSI Package: NOT FOUND" -ForegroundColor Red
    }
}

if (-not $Json) { Write-Host "" }

# 4. PROCESS ANALYSIS
if (-not $Json) {
    Write-Host "[Process Analysis]" -ForegroundColor Cyan
}

$process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($process) {
    $diagnosis.processes.daemon = @{
        running = $true
        pid = $process.Id
        startTime = $process.StartTime.ToString("yyyy-MM-dd HH:mm:ss")
        uptimeHours = [math]::Round(((Get-Date) - $process.StartTime).TotalHours, 1)
        workingSet = $process.WorkingSet64
        path = $process.Path
    }
    if (-not $Json) {
        Write-Host "  Daemon: RUNNING" -ForegroundColor Green
        Write-Host "    PID: $($process.Id)" -ForegroundColor Gray
        Write-Host "    Started: $($diagnosis.processes.daemon.startTime) ($($diagnosis.processes.daemon.uptimeHours) hours ago)" -ForegroundColor Gray
        Write-Host "    Memory: $([math]::Round($process.WorkingSet64 / 1MB, 2)) MB" -ForegroundColor Gray
        Write-Host "    Path: $($process.Path)" -ForegroundColor Gray
    }

    # Check if daemon binary matches installed binary
    if ($process.Path -ne $installedBinary) {
        Add-Issue -Severity "warning" -Message "Daemon running from unexpected location: $($process.Path)" -Fix "Stop daemon and restart from: $installedBinary"
    }
} else {
    $diagnosis.processes.daemon = @{ running = $false }
    Add-Issue -Severity "info" -Message "Daemon is not running" -Fix "Start daemon: & '$installedBinary' run"
    if (-not $Json) {
        Write-Host "  Daemon: NOT RUNNING" -ForegroundColor Yellow
    }
}

if (-not $Json) { Write-Host "" }

# 5. NETWORK ANALYSIS
if (-not $Json) {
    Write-Host "[Network Analysis]" -ForegroundColor Cyan
}

# Check port 9867
try {
    $portOpen = Test-NetConnection -ComputerName localhost -Port 9867 -InformationLevel Quiet -WarningAction SilentlyContinue
    $diagnosis.network.port9867 = @{ open = $portOpen }

    if ($portOpen) {
        if (-not $Json) {
            Write-Host "  Port 9867: OPEN" -ForegroundColor Green
        }

        # Check API health
        try {
            $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 5 -ErrorAction Stop
            $diagnosis.network.apiHealth = @{
                responding = $true
                status = $health.status
                version = $health.version
            }
            if (-not $Json) {
                Write-Host "  API Health: OK (version: $($health.version))" -ForegroundColor Green
            }

            # Check version consistency
            if ($diagnosis.versions.installedBinary -and $health.version -ne $diagnosis.versions.installedBinary) {
                Add-Issue -Severity "warning" -Message "API version ($($health.version)) differs from binary version ($($diagnosis.versions.installedBinary))" -Fix "Restart daemon"
            }
        } catch {
            $diagnosis.network.apiHealth = @{
                responding = $false
                error = $_.Exception.Message
            }
            Add-Issue -Severity "error" -Message "API not responding: $($_.Exception.Message)" -Fix "Restart daemon"
            if (-not $Json) {
                Write-Host "  API Health: ERROR" -ForegroundColor Red
                Write-Host "    $($_.Exception.Message)" -ForegroundColor Gray
            }
        }
    } else {
        Add-Issue -Severity "error" -Message "Port 9867 is not open" -Fix "Start daemon"
        if (-not $Json) {
            Write-Host "  Port 9867: CLOSED" -ForegroundColor Red
        }
    }
} catch {
    if (-not $Json) {
        Write-Host "  Port 9867: ERROR checking port" -ForegroundColor Red
    }
}

if (-not $Json) { Write-Host "" }

# 6. SUGGESTED ACTIONS
if (-not $Json) {
    Write-Host "[Suggested Actions]" -ForegroundColor Cyan
}

$criticalIssues = $diagnosis.issues | Where-Object { $_.severity -eq "critical" }
$errorIssues = $diagnosis.issues | Where-Object { $_.severity -eq "error" }

if ($criticalIssues.Count -gt 0) {
    $diagnosis.suggestions += "CRITICAL: Run as Administrator"
    if (-not $Json) {
        Write-Host "  1. Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Red
    }
}

if ($errorIssues.Count -gt 0) {
    $hasVersionMismatch = $errorIssues | Where-Object { $_.message -like "*VERSION MISMATCH*" }
    if ($hasVersionMismatch) {
        $diagnosis.suggestions += "Run force-clean-reinstall.ps1 to fix version mismatch"
        if (-not $Json) {
            Write-Host "  2. Run: .\scripts\force-clean-reinstall.ps1" -ForegroundColor Yellow
        }
    } else {
        $diagnosis.suggestions += "Run installer-health-check.ps1 for detailed analysis"
        if (-not $Json) {
            Write-Host "  2. Run: .\scripts\installer-health-check.ps1" -ForegroundColor Yellow
        }
    }
}

if ($diagnosis.suggestions.Count -eq 0) {
    $diagnosis.suggestions += "No critical issues found"
    if (-not $Json) {
        Write-Host "  No critical issues found. Installation appears healthy." -ForegroundColor Green
    }
}

if (-not $Json) { Write-Host "" }

# Summary
if (-not $Json) {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " Diagnosis Summary" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""

    $criticalCount = ($diagnosis.issues | Where-Object { $_.severity -eq "critical" }).Count
    $errorCount = ($diagnosis.issues | Where-Object { $_.severity -eq "error" }).Count
    $warningCount = ($diagnosis.issues | Where-Object { $_.severity -eq "warning" }).Count

    Write-Host "  Critical: $criticalCount" -ForegroundColor $(if ($criticalCount -gt 0) { "Red" } else { "Gray" })
    Write-Host "  Errors:   $errorCount" -ForegroundColor $(if ($errorCount -gt 0) { "Red" } else { "Gray" })
    Write-Host "  Warnings: $warningCount" -ForegroundColor $(if ($warningCount -gt 0) { "Yellow" } else { "Gray" })
    Write-Host ""

    if ($criticalCount -gt 0 -or $errorCount -gt 0) {
        Write-Host "========================================" -ForegroundColor Red
        Write-Host " ACTION REQUIRED" -ForegroundColor Red
        Write-Host "========================================" -ForegroundColor Red
        Write-Host ""
        Write-Host "Issues found. Follow the suggested actions above." -ForegroundColor Yellow
        exit 1
    } else {
        Write-Host "========================================" -ForegroundColor Green
        Write-Host " DIAGNOSIS COMPLETE" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        if ($warningCount -gt 0) {
            Write-Host "Installation is functional but has warnings." -ForegroundColor Yellow
        } else {
            Write-Host "Installation is healthy." -ForegroundColor Green
        }
        exit 0
    }
} else {
    # JSON output
    $diagnosis | ConvertTo-Json -Depth 10
    exit $(if (($diagnosis.issues | Where-Object { $_.severity -in @("critical", "error") }).Count -gt 0) { 1 } else { 0 })
}
