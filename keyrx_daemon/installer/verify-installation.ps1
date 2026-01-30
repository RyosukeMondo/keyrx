# Verify installation success post-install
# Check binary exists, version matches, daemon starts, API responds
# Exit 0 always (don't fail installation), but show MessageBox with status
# Used by WiX CustomAction "VerifyInstallation"

param(
    [Parameter(Mandatory = $true)]
    [string]$InstallDir,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion,

    [Parameter(Mandatory = $false)]
    [string]$ApiUrl = "http://localhost:9867/api/health",

    [Parameter(Mandatory = $false)]
    [int]$StartTimeoutSeconds = 5,

    [Parameter(Mandatory = $false)]
    [switch]$ShowMessageBox = $true
)

$ErrorActionPreference = "Continue"

$ProcessName = "keyrx_daemon"
$BinaryPath = Join-Path $InstallDir "bin\keyrx_daemon.exe"

function Write-InstallerLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
}

function Test-ProcessRunning {
    param([string]$Name)
    $processes = Get-Process -Name $Name -ErrorAction SilentlyContinue
    return ($null -ne $processes -and $processes.Count -gt 0)
}

function Show-InstallationResult {
    param(
        [bool]$Success,
        [string[]]$Messages
    )

    if (-not $ShowMessageBox) {
        return
    }

    $title = if ($Success) { "KeyRx Installation Successful" } else { "KeyRx Installation Warning" }
    $icon = if ($Success) { 64 } else { 48 }  # 64=Information, 48=Warning

    $message = $Messages -join "`n"

    Add-Type -AssemblyName PresentationFramework
    [System.Windows.MessageBox]::Show($message, $title, 'OK', $icon) | Out-Null
}

try {
    Write-InstallerLog "Starting post-install verification"
    Write-InstallerLog "Install directory: $InstallDir"
    Write-InstallerLog "Expected version: $ExpectedVersion"

    $results = @()
    $allChecksPass = $true

    # Check 1: Binary exists
    Write-InstallerLog "Check 1: Verifying binary exists at $BinaryPath"
    if (-not (Test-Path $BinaryPath)) {
        Write-InstallerLog "FAIL: Binary not found at $BinaryPath" "ERROR"
        $results += "❌ Binary file not found at installation path"
        $allChecksPass = $false
    }
    else {
        $fileInfo = Get-Item $BinaryPath
        Write-InstallerLog "PASS: Binary found ($($fileInfo.Length) bytes)"
        $results += "✓ Binary installed successfully ($([math]::Round($fileInfo.Length / 1MB, 2)) MB)"
    }

    # Check 2: Binary version matches MSI version
    if (Test-Path $BinaryPath) {
        Write-InstallerLog "Check 2: Verifying binary version"
        try {
            $versionOutput = & $BinaryPath --version 2>&1

            if ($versionOutput -match "(\d+\.\d+\.\d+)") {
                $actualVersion = $matches[1]
                $expected = $ExpectedVersion -replace "(\d+\.\d+\.\d+).*", '$1'

                if ($expected -eq $actualVersion) {
                    Write-InstallerLog "PASS: Version matches ($actualVersion)"
                    $results += "✓ Version verified: $actualVersion"
                }
                else {
                    Write-InstallerLog "FAIL: Version mismatch (expected: $expected, actual: $actualVersion)" "ERROR"
                    $results += "❌ Version mismatch (expected: $expected, got: $actualVersion)"
                    $allChecksPass = $false
                }
            }
            else {
                Write-InstallerLog "WARN: Could not parse version from output: $versionOutput" "WARN"
                $results += "⚠ Could not verify version"
                $allChecksPass = $false
            }
        }
        catch {
            Write-InstallerLog "FAIL: Version check failed: $($_.Exception.Message)" "ERROR"
            $results += "❌ Failed to check version"
            $allChecksPass = $false
        }
    }

    # Check 3: Attempt to start daemon (if not already running)
    Write-InstallerLog "Check 3: Verifying daemon can start"
    $daemonWasRunning = Test-ProcessRunning -Name $ProcessName
    $daemonStartedByUs = $false

    if ($daemonWasRunning) {
        Write-InstallerLog "Daemon is already running"
        $results += "✓ Daemon is already running"
    }
    else {
        try {
            Write-InstallerLog "Starting daemon..."
            Start-Process -FilePath $BinaryPath -ArgumentList "run" -WindowStyle Hidden
            $daemonStartedByUs = $true

            # Wait for daemon to start (with timeout)
            $waited = 0
            while (-not (Test-ProcessRunning -Name $ProcessName) -and $waited -lt $StartTimeoutSeconds) {
                Start-Sleep -Milliseconds 500
                $waited += 0.5
            }

            if (Test-ProcessRunning -Name $ProcessName) {
                Write-InstallerLog "PASS: Daemon started successfully"
                $results += "✓ Daemon started successfully"
            }
            else {
                Write-InstallerLog "FAIL: Daemon did not start within $StartTimeoutSeconds seconds" "WARN"
                $results += "⚠ Daemon start timeout (check logs)"
                $allChecksPass = $false
            }
        }
        catch {
            Write-InstallerLog "FAIL: Failed to start daemon: $($_.Exception.Message)" "ERROR"
            $results += "❌ Failed to start daemon: $($_.Exception.Message)"
            $allChecksPass = $false
        }
    }

    # Check 4: API health check
    if (Test-ProcessRunning -Name $ProcessName) {
        Write-InstallerLog "Check 4: Verifying API responds at $ApiUrl"

        # Wait a moment for API to be ready
        Start-Sleep -Seconds 1

        try {
            $response = Invoke-RestMethod -Uri $ApiUrl -Method Get -TimeoutSec 3 -ErrorAction Stop
            Write-InstallerLog "PASS: API responded successfully"
            Write-InstallerLog "API response: $($response | ConvertTo-Json -Compress)"
            $results += "✓ API health check passed"
        }
        catch {
            Write-InstallerLog "FAIL: API health check failed: $($_.Exception.Message)" "WARN"
            $results += "⚠ API not responding (may need manual start)"
            $allChecksPass = $false
        }
    }
    else {
        Write-InstallerLog "SKIP: API health check (daemon not running)"
        $results += "⚠ API check skipped (daemon not running)"
    }

    # Show results
    if ($allChecksPass) {
        Write-InstallerLog "SUCCESS: All post-install checks passed"
        $finalMessage = "Installation verified successfully!`n`n" + ($results -join "`n") + "`n`nKeyRx is ready to use."
        Show-InstallationResult -Success $true -Messages $finalMessage
    }
    else {
        Write-InstallerLog "WARNING: Some post-install checks failed" "WARN"
        $finalMessage = "Installation completed with warnings:`n`n" + ($results -join "`n") + "`n`nYou may need to start the daemon manually."
        Show-InstallationResult -Success $false -Messages $finalMessage
    }

    # Always exit 0 - don't fail the installation
    exit 0
}
catch {
    Write-InstallerLog "EXCEPTION: $($_.Exception.Message)" "ERROR"
    $errorMessage = "Post-install verification encountered an error:`n`n$($_.Exception.Message)`n`nInstallation may still be successful. Please verify manually."
    Show-InstallationResult -Success $false -Messages $errorMessage
    exit 0
}
