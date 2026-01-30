# Test SSOT Port Configuration
# Verifies all port configs are in sync

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " SSOT Port Configuration Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$failed = $false

# Extract ports from different sources
Write-Host "[1/5] Checking Rust DEFAULT_PORT..." -ForegroundColor Yellow
$rustFile = "keyrx_daemon\src\services\settings_service.rs"
if (Test-Path $rustFile) {
    $rustContent = Get-Content $rustFile -Raw
    if ($rustContent -match "pub const DEFAULT_PORT:\s*u16\s*=\s*(\d+)") {
        $rustPort = $matches[1]
        Write-Host "  Rust DEFAULT_PORT: $rustPort" -ForegroundColor Green
    } else {
        Write-Host "  ERROR: Could not find DEFAULT_PORT in Rust source" -ForegroundColor Red
        $failed = $true
    }
} else {
    Write-Host "  ERROR: Rust source not found: $rustFile" -ForegroundColor Red
    $failed = $true
}

Write-Host ""
Write-Host "[2/5] Checking .env.development..." -ForegroundColor Yellow
$envDevFile = "keyrx_ui\.env.development"
if (Test-Path $envDevFile) {
    $envContent = Get-Content $envDevFile -Raw
    if ($envContent -match "VITE_API_URL=http://localhost:(\d+)") {
        $envPort = $matches[1]
        Write-Host "  .env.development port: $envPort" -ForegroundColor Green
    } else {
        Write-Host "  ERROR: Could not find VITE_API_URL in .env.development" -ForegroundColor Red
        $failed = $true
    }
} else {
    Write-Host "  ERROR: .env.development not found" -ForegroundColor Red
    $failed = $true
}

Write-Host ""
Write-Host "[3/5] Checking vite.config.ts proxy..." -ForegroundColor Yellow
$viteFile = "keyrx_ui\vite.config.ts"
if (Test-Path $viteFile) {
    $viteContent = Get-Content $viteFile -Raw
    if ($viteContent -match "target:\s*['""]http://localhost:(\d+)['""]") {
        $vitePort = $matches[1]
        Write-Host "  vite.config.ts port: $vitePort" -ForegroundColor Green
    } else {
        Write-Host "  ERROR: Could not find proxy target in vite.config.ts" -ForegroundColor Red
        $failed = $true
    }
} else {
    Write-Host "  ERROR: vite.config.ts not found" -ForegroundColor Red
    $failed = $true
}

Write-Host ""
Write-Host "[4/5] Checking .env.production..." -ForegroundColor Yellow
$envProdFile = "keyrx_ui\.env.production"
if (Test-Path $envProdFile) {
    $envProdContent = Get-Content $envProdFile -Raw
    if ($envProdContent -match "VITE_API_URL=\s*$") {
        Write-Host "  .env.production: Empty (uses window.location.origin) ✓" -ForegroundColor Green
    } else {
        Write-Host "  WARNING: .env.production has a value (should be empty for dynamic origin)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ERROR: .env.production not found" -ForegroundColor Red
    $failed = $true
}

Write-Host ""
Write-Host "[5/5] Checking runtime settings.json..." -ForegroundColor Yellow
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    $settingsContent = Get-Content $settingsFile -Raw | ConvertFrom-Json
    $runtimePort = $settingsContent.port
    Write-Host "  Runtime port (settings.json): $runtimePort" -ForegroundColor Cyan

    if ($runtimePort -eq [int]$rustPort) {
        Write-Host "    ✓ Matches DEFAULT_PORT" -ForegroundColor Green
    } else {
        Write-Host "    ⚠ Different from DEFAULT_PORT ($rustPort)" -ForegroundColor Yellow
        Write-Host "      This overrides the default at runtime" -ForegroundColor Gray
    }
} else {
    Write-Host "  No settings.json (using DEFAULT_PORT: $rustPort)" -ForegroundColor Gray
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " SSOT Verification" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not $failed) {
    # Check if all build-time configs match
    if ($rustPort -eq $envPort -and $envPort -eq $vitePort) {
        Write-Host "✓ SSOT Verified!" -ForegroundColor Green
        Write-Host "  All build-time configs use port: $rustPort" -ForegroundColor Green
        Write-Host ""

        # Check if runtime matches
        if (Test-Path $settingsFile) {
            if ($runtimePort -eq [int]$rustPort) {
                Write-Host "✓ Runtime matches build-time: $runtimePort" -ForegroundColor Green
            } else {
                Write-Host "⚠ Runtime override detected!" -ForegroundColor Yellow
                Write-Host "  Build-time: $rustPort" -ForegroundColor White
                Write-Host "  Runtime:    $runtimePort" -ForegroundColor White
                Write-Host ""
                Write-Host "This is OK if intentional. To remove override:" -ForegroundColor Gray
                Write-Host "  Remove-Item '$settingsFile'" -ForegroundColor Gray
            }
        } else {
            Write-Host "✓ No runtime override (using DEFAULT_PORT: $rustPort)" -ForegroundColor Green
        }

        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Yellow
        Write-Host "1. If you change DEFAULT_PORT in Rust, run:" -ForegroundColor White
        Write-Host "   cd keyrx_ui; npm run sync-port" -ForegroundColor Gray
        Write-Host "2. Then rebuild:" -ForegroundColor White
        Write-Host "   .\REBUILD_SSOT.bat" -ForegroundColor Gray
    } else {
        Write-Host "✗ SSOT Violation Detected!" -ForegroundColor Red
        Write-Host ""
        Write-Host "Port mismatch:" -ForegroundColor Yellow
        Write-Host "  Rust DEFAULT_PORT: $rustPort" -ForegroundColor White
        Write-Host "  .env.development:  $envPort" -ForegroundColor White
        Write-Host "  vite.config.ts:    $vitePort" -ForegroundColor White
        Write-Host ""
        Write-Host "Fix:" -ForegroundColor Yellow
        Write-Host "  cd keyrx_ui" -ForegroundColor White
        Write-Host "  npm run sync-port" -ForegroundColor White
        $failed = $true
    }
} else {
    Write-Host "✗ SSOT Test Failed!" -ForegroundColor Red
    Write-Host "  Some configuration files are missing or invalid" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan

if ($failed) {
    exit 1
} else {
    exit 0
}
