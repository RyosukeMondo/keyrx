#!/usr/bin/env pwsh
# Build KeyRx Windows Installer
# This script builds the complete installer package

param(
    [string]$InstallerType = "inno",  # inno, wix, or nsis
    [switch]$SkipBuild = $false,      # Skip cargo/npm build
    [switch]$SkipChecksum = $false,   # Skip checksum generation
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

# Script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

# Colors
function Write-Info { Write-Host "INFO: $args" -ForegroundColor Cyan }
function Write-Success { Write-Host "SUCCESS: $args" -ForegroundColor Green }
function Write-Error { Write-Host "ERROR: $args" -ForegroundColor Red }
function Write-Warning { Write-Host "WARNING: $args" -ForegroundColor Yellow }

Write-Info "KeyRx Windows Installer Build Script"
Write-Info "Installer Type: $InstallerType"
Write-Info "Root Directory: $RootDir"

# Check prerequisites
function Test-InstallerTool {
    param([string]$Tool, [string]$TestCommand)

    if ($Verbose) { Write-Info "Checking for $Tool..." }

    try {
        $null = & $TestCommand 2>&1
        return $true
    } catch {
        return $false
    }
}

# Determine which installer to use
$UseInno = $false
$UseWix = $false
$UseNsis = $false

if ($InstallerType -eq "inno") {
    if (Test-InstallerTool "Inno Setup" "iscc" "/?" ) {
        $UseInno = $true
        Write-Success "Inno Setup found"
    } else {
        Write-Error "Inno Setup not found. Download from https://jrsoftware.org/isdl.php"
        Write-Info "Attempting to find Inno Setup in common locations..."

        $CommonPaths = @(
            "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
            "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
            "${env:ProgramFiles(x86)}\Inno Setup 5\ISCC.exe"
        )

        foreach ($Path in $CommonPaths) {
            if (Test-Path $Path) {
                Write-Success "Found Inno Setup at: $Path"
                $env:PATH += ";$(Split-Path -Parent $Path)"
                $UseInno = $true
                break
            }
        }

        if (-not $UseInno) {
            Write-Error "Could not find Inno Setup. Please install or try -InstallerType wix or nsis"
            exit 1
        }
    }
} elseif ($InstallerType -eq "wix") {
    if (Test-InstallerTool "WiX" "candle" "-?" ) {
        $UseWix = $true
        Write-Success "WiX Toolset found"
    } else {
        Write-Error "WiX Toolset not found. Download from https://wixtoolset.org/"
        exit 1
    }
} elseif ($InstallerType -eq "nsis") {
    if (Test-InstallerTool "NSIS" "makensis" "/VERSION") {
        $UseNsis = $true
        Write-Success "NSIS found"
    } else {
        Write-Error "NSIS not found. Download from https://nsis.sourceforge.io/"
        exit 1
    }
} else {
    Write-Error "Unknown installer type: $InstallerType (use: inno, wix, or nsis)"
    exit 1
}

# Step 1: Build release binaries
if (-not $SkipBuild) {
    Write-Info "Step 1: Building release binaries..."

    Push-Location $RootDir

    Write-Info "Running cargo build --release --workspace..."
    cargo build --release --workspace
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed"
        Pop-Location
        exit 1
    }

    # Verify binaries exist
    $DaemonPath = Join-Path $RootDir "target\release\keyrx_daemon.exe"
    $CompilerPath = Join-Path $RootDir "target\release\keyrx_compiler.exe"

    if (-not (Test-Path $DaemonPath)) {
        Write-Error "Daemon binary not found: $DaemonPath"
        Pop-Location
        exit 1
    }

    if (-not (Test-Path $CompilerPath)) {
        Write-Error "Compiler binary not found: $CompilerPath"
        Pop-Location
        exit 1
    }

    Write-Success "Binaries built successfully"
    Write-Info "  Daemon: $DaemonPath ($(((Get-Item $DaemonPath).Length / 1MB).ToString('F2')) MB)"
    Write-Info "  Compiler: $CompilerPath ($(((Get-Item $CompilerPath).Length / 1MB).ToString('F2')) MB)"

    Pop-Location
} else {
    Write-Warning "Skipping cargo build (--SkipBuild)"
}

# Step 2: Build Web UI
if (-not $SkipBuild) {
    Write-Info "Step 2: Building Web UI..."

    $UiDir = Join-Path $RootDir "keyrx_ui"
    Push-Location $UiDir

    Write-Info "Running npm install..."
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error "npm install failed"
        Pop-Location
        exit 1
    }

    Write-Info "Running npm run build..."
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "npm build failed"
        Pop-Location
        exit 1
    }

    # Verify dist exists
    $DistPath = Join-Path $UiDir "dist"
    if (-not (Test-Path $DistPath)) {
        Write-Error "UI dist not found: $DistPath"
        Pop-Location
        exit 1
    }

    Write-Success "Web UI built successfully"
    Pop-Location
} else {
    Write-Warning "Skipping UI build (--SkipBuild)"
}

# Step 3: Create installer output directory
$OutputDir = Join-Path $RootDir "installer-output"
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
    Write-Info "Created output directory: $OutputDir"
}

# Step 4: Build installer
Write-Info "Step 3: Building installer..."

Push-Location $RootDir

if ($UseInno) {
    Write-Info "Building Inno Setup installer..."

    $IssFile = Join-Path $RootDir "keyrx-installer.iss"
    if (-not (Test-Path $IssFile)) {
        Write-Error "Inno Setup script not found: $IssFile"
        Pop-Location
        exit 1
    }

    iscc $IssFile
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Inno Setup compilation failed"
        Pop-Location
        exit 1
    }

    $InstallerPath = Join-Path $OutputDir "keyrx-setup-v0.1.5-windows-x64.exe"

} elseif ($UseWix) {
    Write-Info "Building WiX MSI installer..."

    $WxsFile = Join-Path $RootDir "scripts\wix\keyrx-installer.wxs"
    if (-not (Test-Path $WxsFile)) {
        Write-Error "WiX script not found: $WxsFile"
        Pop-Location
        exit 1
    }

    # Harvest UI files
    Write-Info "Harvesting UI files with heat.exe..."
    $UiDistPath = Join-Path $RootDir "keyrx_ui\dist"
    $UiWxsPath = Join-Path $RootDir "scripts\wix\ui-files.wxs"

    heat dir $UiDistPath -cg WebUIFiles -dr UiFolder -gg -sfrag -srd -out $UiWxsPath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX heat failed"
        Pop-Location
        exit 1
    }

    # Compile
    Write-Info "Compiling with candle.exe..."
    $WixObjDir = Join-Path $OutputDir "wix-obj"
    if (-not (Test-Path $WixObjDir)) {
        New-Item -ItemType Directory -Path $WixObjDir | Out-Null
    }

    candle $WxsFile $UiWxsPath -o "$WixObjDir\"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX candle failed"
        Pop-Location
        exit 1
    }

    # Link
    Write-Info "Linking with light.exe..."
    light "$WixObjDir\keyrx-installer.wixobj" "$WixObjDir\ui-files.wixobj" -o "$OutputDir\keyrx-setup-v0.1.5-windows-x64.msi" -ext WixUIExtension
    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX light failed"
        Pop-Location
        exit 1
    }

    $InstallerPath = Join-Path $OutputDir "keyrx-setup-v0.1.5-windows-x64.msi"

} elseif ($UseNsis) {
    Write-Info "Building NSIS installer..."

    $NsiFile = Join-Path $RootDir "scripts\nsis\keyrx-installer.nsi"
    if (-not (Test-Path $NsiFile)) {
        Write-Error "NSIS script not found: $NsiFile"
        Pop-Location
        exit 1
    }

    makensis $NsiFile
    if ($LASTEXITCODE -ne 0) {
        Write-Error "NSIS compilation failed"
        Pop-Location
        exit 1
    }

    $InstallerPath = Join-Path $OutputDir "keyrx-setup-v0.1.5-windows-x64.exe"
}

Pop-Location

if (-not (Test-Path $InstallerPath)) {
    Write-Error "Installer not created: $InstallerPath"
    exit 1
}

Write-Success "Installer created: $InstallerPath"
Write-Info "Size: $(((Get-Item $InstallerPath).Length / 1MB).ToString('F2')) MB"

# Step 5: Generate checksum
if (-not $SkipChecksum) {
    Write-Info "Step 4: Generating SHA256 checksum..."

    $Hash = Get-FileHash -Path $InstallerPath -Algorithm SHA256
    $ChecksumFile = "$InstallerPath.sha256"

    $Hash.Hash | Out-File -FilePath $ChecksumFile -Encoding ASCII -NoNewline

    Write-Success "Checksum saved: $ChecksumFile"
    Write-Info "SHA256: $($Hash.Hash)"
} else {
    Write-Warning "Skipping checksum generation (--SkipChecksum)"
}

# Summary
Write-Info ""
Write-Info "========================================="
Write-Success "Build completed successfully!"
Write-Info "========================================="
Write-Info "Installer: $InstallerPath"
if (-not $SkipChecksum) {
    Write-Info "Checksum: $ChecksumFile"
}
Write-Info ""
Write-Info "Next steps:"
Write-Info "  1. Test installer in a clean environment"
Write-Info "  2. Verify all files installed correctly"
Write-Info "  3. Test uninstaller"
Write-Info "  4. Upload to releases"
Write-Info ""
