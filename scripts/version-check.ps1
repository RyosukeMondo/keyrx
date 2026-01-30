# version-check.ps1 - Verify version consistency across all sources
#
# Usage:
#   .\scripts\version-check.ps1       # Check all versions
#   .\scripts\version-check.ps1 -Json # Output JSON format

param([switch]$Json = $false)

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$CargoToml = Join-Path $ProjectRoot "Cargo.toml"
$PackageJson = Join-Path $ProjectRoot "keyrx_ui\package.json"
$InstallerWxs = Join-Path $ProjectRoot "keyrx_daemon\keyrx_installer.wxs"
$InstallerPs1 = Join-Path $ProjectRoot "scripts\build_windows_installer.ps1"
$InstalledBinary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
$DaemonApiUrl = "http://localhost:9867/api/health"

function Get-CargoVersion {
    if (-not (Test-Path $CargoToml)) {
        throw "Cargo.toml not found"
    }
    $content = Get-Content $CargoToml -Raw
    if ($content -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
        return $Matches[1]
    }
    throw "Failed to extract Cargo.toml version"
}

function Get-PackageJsonVersion {
    if (-not (Test-Path $PackageJson)) {
        throw "package.json not found"
    }
    $json = Get-Content $PackageJson -Raw | ConvertFrom-Json
    return $json.version
}

function Get-InstallerWxsVersion {
    if (-not (Test-Path $InstallerWxs)) {
        throw "keyrx_installer.wxs not found"
    }
    $content = Get-Content $InstallerWxs -Raw
    if ($content -match 'Version="([0-9]+\.[0-9]+\.[0-9]+)\.0"') {
        return $Matches[1]
    }
    throw "Failed to extract WXS version"
}

function Get-InstallerPs1Version {
    if (-not (Test-Path $InstallerPs1)) {
        throw "build_windows_installer.ps1 not found"
    }
    $content = Get-Content $InstallerPs1 -Raw
    if ($content -match '\$Version\s*=\s*"([^"]+)"') {
        return $Matches[1]
    }
    throw "Failed to extract PS1 version"
}

function Get-InstalledBinaryVersion {
    if (-not (Test-Path $InstalledBinary)) {
        return $null
    }
    try {
        $output = & $InstalledBinary --version 2>&1
        if ($output -match '([0-9]+\.[0-9]+\.[0-9]+)') {
            return $Matches[1]
        }
        return "ERROR: Could not parse"
    }
    catch {
        return "ERROR: Failed to execute"
    }
}

function Get-RunningDaemonVersion {
    try {
        $response = Invoke-RestMethod -Uri $DaemonApiUrl -TimeoutSec 2 -ErrorAction Stop
        if ($response.version) {
            return $response.version
        }
        return "ERROR: No version field"
    }
    catch {
        return $null
    }
}

try {
    $cargoVersion = Get-CargoVersion
    $packageJsonVersion = Get-PackageJsonVersion
    $wxsVersion = Get-InstallerWxsVersion
    $ps1Version = Get-InstallerPs1Version
    $installedVersion = Get-InstalledBinaryVersion
    $runningVersion = Get-RunningDaemonVersion

    $mismatch = $false
    $mismatches = @()

    if ($packageJsonVersion -ne $cargoVersion) {
        $mismatch = $true
        $mismatches += "package.json ($packageJsonVersion) != Cargo.toml ($cargoVersion)"
    }

    if ($wxsVersion -ne $cargoVersion) {
        $mismatch = $true
        $mismatches += "keyrx_installer.wxs ($wxsVersion) != Cargo.toml ($cargoVersion)"
    }

    if ($ps1Version -ne $cargoVersion) {
        $mismatch = $true
        $mismatches += "build_windows_installer.ps1 ($ps1Version) != Cargo.toml ($cargoVersion)"
    }

    if ($installedVersion -and $installedVersion -ne $cargoVersion -and -not $installedVersion.StartsWith("ERROR")) {
        $mismatch = $true
        $mismatches += "Installed binary ($installedVersion) != Cargo.toml ($cargoVersion)"
    }

    if ($runningVersion -and $runningVersion -ne $cargoVersion -and -not $runningVersion.StartsWith("ERROR")) {
        $mismatch = $true
        $mismatches += "Running daemon ($runningVersion) != Cargo.toml ($cargoVersion)"
    }

    if ($Json) {
        @{
            CargoToml = $cargoVersion
            PackageJson = $packageJsonVersion
            InstallerWxs = $wxsVersion
            BuildScript = $ps1Version
            InstalledBinary = if ($installedVersion) { $installedVersion } else { "N/A" }
            RunningDaemon = if ($runningVersion) { $runningVersion } else { "N/A" }
            HasMismatch = $mismatch
            Mismatches = $mismatches
        } | ConvertTo-Json -Depth 10
        exit $(if ($mismatch) { 1 } else { 0 })
    }

    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "         Version Consistency Check                         " -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""

    $tableData = @(
        @{Source = "Cargo.toml (SSOT)"; Version = $cargoVersion; Status = "SSOT" },
        @{Source = "package.json"; Version = $packageJsonVersion; Status = if ($packageJsonVersion -eq $cargoVersion) { "OK" } else { "MISMATCH" } },
        @{Source = "keyrx_installer.wxs"; Version = $wxsVersion; Status = if ($wxsVersion -eq $cargoVersion) { "OK" } else { "MISMATCH" } },
        @{Source = "build_windows_installer.ps1"; Version = $ps1Version; Status = if ($ps1Version -eq $cargoVersion) { "OK" } else { "MISMATCH" } },
        @{Source = "Installed binary"; Version = if ($installedVersion) { $installedVersion } else { "N/A" }; Status = if (-not $installedVersion) { "N/A" } elseif ($installedVersion.StartsWith("ERROR")) { "ERROR" } elseif ($installedVersion -eq $cargoVersion) { "OK" } else { "MISMATCH" } },
        @{Source = "Running daemon"; Version = if ($runningVersion) { $runningVersion } else { "N/A" }; Status = if (-not $runningVersion) { "N/A" } elseif ($runningVersion.StartsWith("ERROR")) { "ERROR" } elseif ($runningVersion -eq $cargoVersion) { "OK" } else { "MISMATCH" } }
    )

    $maxSourceLen = ($tableData | ForEach-Object { $_.Source.Length } | Measure-Object -Maximum).Maximum
    $maxVersionLen = ($tableData | ForEach-Object { $_.Version.Length } | Measure-Object -Maximum).Maximum

    Write-Host ("{0,-$maxSourceLen}  {1,-$maxVersionLen}  {2}" -f "Source", "Version", "Status")
    Write-Host ("{0,-$maxSourceLen}  {1,-$maxVersionLen}  {2}" -f ("-" * $maxSourceLen), ("-" * $maxVersionLen), "--------")

    foreach ($row in $tableData) {
        $color = switch ($row.Status) {
            "SSOT" { "Cyan" }
            "OK" { "Green" }
            "MISMATCH" { "Red" }
            "ERROR" { "Red" }
            "N/A" { "Gray" }
            default { "White" }
        }

        Write-Host ("{0,-$maxSourceLen}  {1,-$maxVersionLen}  " -f $row.Source, $row.Version) -NoNewline
        Write-Host $row.Status -ForegroundColor $color
    }

    Write-Host ""

    if ($mismatch) {
        Write-Host "============================================================" -ForegroundColor Red
        Write-Host " VERSION MISMATCH DETECTED                                 " -ForegroundColor Red
        Write-Host "============================================================" -ForegroundColor Red
        Write-Host ""

        foreach ($msg in $mismatches) {
            Write-Host "  * $msg" -ForegroundColor Yellow
        }

        Write-Host ""
        Write-Host "To fix source file mismatches, run:" -ForegroundColor Yellow
        Write-Host "  .\scripts\sync-version.sh" -ForegroundColor White
        Write-Host ""

        if ($installedVersion -and $installedVersion -ne $cargoVersion) {
            Write-Host "To update installed binary:" -ForegroundColor Yellow
            Write-Host "  cargo build --release" -ForegroundColor White
            Write-Host "  .\scripts\build_windows_installer.ps1" -ForegroundColor White
            Write-Host "  msiexec /i target\installer\KeyRx-$cargoVersion-x64.msi" -ForegroundColor White
            Write-Host ""
        }
    }
    else {
        Write-Host "============================================================" -ForegroundColor Green
        Write-Host " All versions synchronized: $cargoVersion" -ForegroundColor Green
        Write-Host "============================================================" -ForegroundColor Green
        Write-Host ""
    }

    exit $(if ($mismatch) { 1 } else { 0 })
}
catch {
    if ($Json) {
        @{
            Error = $_.Exception.Message
            HasMismatch = $true
        } | ConvertTo-Json
    }
    else {
        Write-Host ""
        Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host ""
    }
    exit 1
}
