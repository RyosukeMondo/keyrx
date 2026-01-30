# Validate binary version matches MSI version
# Exit 0 on success, exit 1 on failure
# Used by WiX CustomAction "ValidateBinaryVersion"

param(
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion,

    [Parameter(Mandatory = $false)]
    [int]$MaxAgeHours = 24
)

$ErrorActionPreference = "Stop"

function Write-InstallerLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
}

try {
    Write-InstallerLog "Starting binary version validation"
    Write-InstallerLog "Binary path: $BinaryPath"
    Write-InstallerLog "Expected version: $ExpectedVersion"

    # Check binary exists
    if (-not (Test-Path $BinaryPath)) {
        Write-InstallerLog "ERROR: Binary not found at $BinaryPath" "ERROR"
        exit 1
    }

    # Get binary file info
    $fileInfo = Get-Item $BinaryPath
    Write-InstallerLog "Binary size: $($fileInfo.Length) bytes"
    Write-InstallerLog "Binary timestamp: $($fileInfo.LastWriteTime)"

    # Check binary timestamp (within last 24 hours)
    $ageHours = (Get-Date) - $fileInfo.LastWriteTime
    if ($ageHours.TotalHours -gt $MaxAgeHours) {
        Write-InstallerLog "WARNING: Binary is $([math]::Round($ageHours.TotalHours, 1)) hours old (expected < $MaxAgeHours hours)" "ERROR"
        Write-InstallerLog "This suggests the binary was not rebuilt before packaging the installer." "ERROR"
        exit 1
    }
    Write-InstallerLog "Binary age: $([math]::Round($ageHours.TotalHours, 1)) hours (within acceptable range)"

    # Execute binary with --version flag
    $versionOutput = & $BinaryPath --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-InstallerLog "ERROR: Failed to execute binary --version (exit code: $LASTEXITCODE)" "ERROR"
        Write-InstallerLog "Output: $versionOutput" "ERROR"
        exit 1
    }

    Write-InstallerLog "Version output: $versionOutput"

    # Parse version from output (format: "keyrx_daemon 0.1.5" or similar)
    if ($versionOutput -match "(\d+\.\d+\.\d+)") {
        $actualVersion = $matches[1]
        Write-InstallerLog "Parsed binary version: $actualVersion"

        # Compare versions (normalize to X.Y.Z format)
        $expected = $ExpectedVersion -replace "(\d+\.\d+\.\d+).*", '$1'
        $actual = $actualVersion -replace "(\d+\.\d+\.\d+).*", '$1'

        if ($expected -ne $actual) {
            Write-InstallerLog "ERROR: Version mismatch!" "ERROR"
            Write-InstallerLog "  Expected: $expected (from MSI)" "ERROR"
            Write-InstallerLog "  Actual:   $actual (from binary)" "ERROR"
            Write-InstallerLog "This indicates the installer was built with an outdated binary." "ERROR"
            Write-InstallerLog "Please rebuild the project and regenerate the installer." "ERROR"
            exit 1
        }

        Write-InstallerLog "SUCCESS: Version validation passed ($actual)"
        exit 0
    }
    else {
        Write-InstallerLog "ERROR: Could not parse version from binary output" "ERROR"
        Write-InstallerLog "Output was: $versionOutput" "ERROR"
        exit 1
    }
}
catch {
    Write-InstallerLog "EXCEPTION: $($_.Exception.Message)" "ERROR"
    Write-InstallerLog "Stack trace: $($_.ScriptStackTrace)" "ERROR"
    exit 1
}
