# Clean all build artifacts
# MECE: Only cleaning, no building

#Requires -Version 5.1

param(
    [switch]$All,          # Clean everything including node_modules
    [switch]$Cargo,        # Clean cargo artifacts only
    [switch]$Npm,          # Clean npm artifacts only
    [switch]$Force         # No confirmation prompt
)

# Import utilities
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")

Set-ProjectRoot

Write-Step "Cleaning KeyRx Build Artifacts"

# Stop daemon if running
if (Test-DaemonRunning) {
    Write-Warning-Custom "Daemon is running. Stopping it first..."
    Stop-Daemon -Force | Out-Null
}

$cleaned = @()

# Clean cargo artifacts
if ($Cargo -or $All -or (-not $Npm)) {
    Write-Step "Cleaning Cargo artifacts..."

    if (Test-Path $script:TargetDir) {
        if (-not $Force) {
            $confirm = Read-Host "Delete target directory? (y/N)"
            if ($confirm -ne 'y') {
                Write-Info "Skipping cargo clean"
            } else {
                & cargo clean
                $cleaned += "Cargo build artifacts"
            }
        } else {
            & cargo clean
            $cleaned += "Cargo build artifacts"
        }
    } else {
        Write-Info "No cargo artifacts to clean"
    }
}

# Clean npm artifacts
if ($Npm -or $All) {
    Write-Step "Cleaning npm artifacts..."

    Push-Location $script:UiDir

    if (Test-Path "node_modules") {
        if (-not $Force) {
            $confirm = Read-Host "Delete node_modules? (y/N)"
            if ($confirm -eq 'y') {
                Remove-Item -Recurse -Force "node_modules"
                $cleaned += "node_modules"
            }
        } else {
            Remove-Item -Recurse -Force "node_modules"
            $cleaned += "node_modules"
        }
    }

    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        $cleaned += "UI dist"
    }

    Pop-Location
}

# Clean temp files
$tempPatterns = @(
    "*.log",
    "*.tmp",
    ".DS_Store",
    "Thumbs.db"
)

foreach ($pattern in $tempPatterns) {
    $files = Get-ChildItem -Path $script:ProjectRoot -Filter $pattern -Recurse -ErrorAction SilentlyContinue
    if ($files) {
        $files | Remove-Item -Force
        $cleaned += "$pattern files"
    }
}

# Summary
Write-Step "Clean Summary"
if ($cleaned.Count -gt 0) {
    Write-Success "Cleaned:"
    foreach ($item in $cleaned) {
        Write-Host "  • $item" -ForegroundColor Green
    }
} else {
    Write-Info "Nothing to clean"
}

Write-Success "✨ Clean complete!"
