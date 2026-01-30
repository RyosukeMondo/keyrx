# Stop KeyRx daemon with retry logic and timeout
# Exit 0 on success (daemon stopped or already stopped)
# Exit 1 on failure (daemon still running after timeout)
# Used by WiX CustomAction "StopDaemonBeforeUpgrade"

$ErrorActionPreference = "Continue"

$ProcessName = "keyrx_daemon"
$MaxRetries = 3
$RetryDelaySeconds = 2
$TotalTimeoutSeconds = 10

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

function Stop-ProcessGracefully {
    param([string]$Name)
    try {
        $processes = Get-Process -Name $Name -ErrorAction SilentlyContinue
        if ($null -eq $processes) {
            return $true
        }

        foreach ($proc in $processes) {
            Write-InstallerLog "Attempting graceful shutdown of PID $($proc.Id)"
            $proc.CloseMainWindow() | Out-Null
        }

        Start-Sleep -Seconds 1
        return -not (Test-ProcessRunning -Name $Name)
    }
    catch {
        Write-InstallerLog "Graceful shutdown failed: $($_.Exception.Message)" "WARN"
        return $false
    }
}

function Stop-ProcessForcefully {
    param([string]$Name)
    try {
        $result = taskkill /IM "$Name.exe" /F /T 2>&1
        Write-InstallerLog "Force kill result: $result"
        Start-Sleep -Milliseconds 500
        return -not (Test-ProcessRunning -Name $Name)
    }
    catch {
        Write-InstallerLog "Force kill failed: $($_.Exception.Message)" "WARN"
        return $false
    }
}

try {
    Write-InstallerLog "Starting daemon stop procedure"

    # Check if daemon is running
    if (-not (Test-ProcessRunning -Name $ProcessName)) {
        Write-InstallerLog "Daemon is not running - nothing to stop"
        exit 0
    }

    $startTime = Get-Date
    $attempt = 0

    # Retry loop with timeout
    while ((Test-ProcessRunning -Name $ProcessName) -and $attempt -lt $MaxRetries) {
        $attempt++
        $elapsed = ((Get-Date) - $startTime).TotalSeconds

        # Check total timeout
        if ($elapsed -ge $TotalTimeoutSeconds) {
            Write-InstallerLog "Timeout exceeded ($TotalTimeoutSeconds seconds) - daemon still running" "ERROR"
            exit 1
        }

        Write-InstallerLog "Stop attempt $attempt of $MaxRetries"

        # Attempt 1-2: Graceful shutdown
        if ($attempt -le 2) {
            Write-InstallerLog "Attempting graceful shutdown..."
            $stopped = Stop-ProcessGracefully -Name $ProcessName

            if ($stopped) {
                Write-InstallerLog "SUCCESS: Daemon stopped gracefully on attempt $attempt"
                exit 0
            }

            Write-InstallerLog "Graceful shutdown failed, waiting $RetryDelaySeconds seconds before retry"
            Start-Sleep -Seconds $RetryDelaySeconds
        }
        # Final attempt: Force kill
        else {
            Write-InstallerLog "Attempting force kill..."
            $stopped = Stop-ProcessForcefully -Name $ProcessName

            if ($stopped) {
                Write-InstallerLog "SUCCESS: Daemon stopped forcefully on attempt $attempt"
                exit 0
            }

            Write-InstallerLog "Force kill failed" "ERROR"
        }
    }

    # Final check
    if (Test-ProcessRunning -Name $ProcessName) {
        Write-InstallerLog "FAILURE: Daemon still running after $attempt attempts" "ERROR"
        Write-InstallerLog "Elapsed time: $([math]::Round($elapsed, 1)) seconds" "ERROR"

        # List remaining processes for diagnostics
        $processes = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
        foreach ($proc in $processes) {
            Write-InstallerLog "  Still running: PID $($proc.Id), Started $($proc.StartTime)" "ERROR"
        }

        exit 1
    }

    Write-InstallerLog "SUCCESS: Daemon stopped successfully"
    exit 0
}
catch {
    Write-InstallerLog "EXCEPTION: $($_.Exception.Message)" "ERROR"
    # Don't fail the installation for unexpected errors - daemon might already be stopped
    exit 0
}
