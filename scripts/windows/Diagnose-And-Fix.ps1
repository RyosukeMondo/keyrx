# Diagnose and Fix KeyRx Issues
# Checks current state, rebuilds if needed, and relaunches with debug logging

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  KeyRx Diagnostics and Fix" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Check daemon status
Write-Host "[1/6] Checking daemon status..." -ForegroundColor Yellow
$daemon = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Write-Host "  ✓ Daemon is running (PID: $($daemon.Id), Started: $($daemon.StartTime))" -ForegroundColor Green

    # Check if debug log exists
    $logPath = "$env:TEMP\keyrx-debug.log"
    if (Test-Path $logPath) {
        $logSize = (Get-Item $logPath).Length
        Write-Host "  ✓ Debug log found ($([math]::Round($logSize/1KB, 2)) KB)" -ForegroundColor Green
    } else {
        Write-Host "  ✗ No debug log found (daemon not started with --debug)" -ForegroundColor Red
    }
} else {
    Write-Host "  ✗ Daemon is NOT running" -ForegroundColor Red
}

# 2. Check build timestamps
Write-Host ""
Write-Host "[2/6] Checking build timestamps..." -ForegroundColor Yellow

$wasmFile = "keyrx_ui\src\wasm\pkg\keyrx_core_bg.wasm"
$uiDist = "keyrx_ui\dist\index.html"
$daemonExe = "target\release\keyrx_daemon.exe"

if (Test-Path $wasmFile) {
    $wasmTime = (Get-Item $wasmFile).LastWriteTime
    Write-Host "  ✓ WASM built: $wasmTime" -ForegroundColor Green
} else {
    Write-Host "  ✗ WASM not built" -ForegroundColor Red
}

if (Test-Path $uiDist) {
    $uiTime = (Get-Item $uiDist).LastWriteTime
    Write-Host "  ✓ UI built: $uiTime" -ForegroundColor Green
} else {
    Write-Host "  ✗ UI not built" -ForegroundColor Red
}

if (Test-Path $daemonExe) {
    $daemonTime = (Get-Item $daemonExe).LastWriteTime
    Write-Host "  ✓ Daemon built: $daemonTime" -ForegroundColor Green
} else {
    Write-Host "  ✗ Daemon not built" -ForegroundColor Red
}

# 3. Check profile status
Write-Host ""
Write-Host "[3/6] Checking profile status..." -ForegroundColor Yellow

$profilesDir = "$env:APPDATA\keyrx\profiles"
if (Test-Path $profilesDir) {
    $profiles = Get-ChildItem $profilesDir -Filter "*.rhai" -ErrorAction SilentlyContinue
    if ($profiles) {
        Write-Host "  ✓ Found $($profiles.Count) profile(s):" -ForegroundColor Green
        foreach ($p in $profiles) {
            $krx = $p.FullName -replace '\.rhai$', '.krx'
            $hasKrx = Test-Path $krx
            $status = if ($hasKrx) { "✓ compiled" } else { "✗ not compiled" }
            Write-Host "    - $($p.BaseName): $status" -ForegroundColor $(if ($hasKrx) { "Green" } else { "Yellow" })
        }
    } else {
        Write-Host "  ✗ No profiles found" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✗ Profiles directory doesn't exist" -ForegroundColor Yellow
}

# 4. Propose fix
Write-Host ""
Write-Host "[4/6] Diagnosis complete" -ForegroundColor Yellow
Write-Host ""

$needsRebuild = $false
$needsRestart = $false

if (-not (Test-Path $wasmFile) -or -not (Test-Path $uiDist) -or -not (Test-Path $daemonExe)) {
    Write-Host "  Issue: Build files are missing" -ForegroundColor Red
    Write-Host "  Fix: Need to rebuild" -ForegroundColor Yellow
    $needsRebuild = $true
}

if ($daemon -and -not (Test-Path "$env:TEMP\keyrx-debug.log")) {
    Write-Host "  Issue: Daemon running without debug logging" -ForegroundColor Red
    Write-Host "  Fix: Need to restart with --debug flag" -ForegroundColor Yellow
    $needsRestart = $true
}

if (-not $daemon) {
    Write-Host "  Issue: Daemon not running" -ForegroundColor Red
    Write-Host "  Fix: Need to start daemon" -ForegroundColor Yellow
    $needsRestart = $true
}

# 5. Apply fixes
Write-Host ""
Write-Host "[5/6] Applying fixes..." -ForegroundColor Yellow

if ($needsRebuild) {
    Write-Host ""
    Write-Host "Rebuild needed. This will:" -ForegroundColor Cyan
    Write-Host "  1. Build WASM module" -ForegroundColor Cyan
    Write-Host "  2. Build UI (React + Vite)" -ForegroundColor Cyan
    Write-Host "  3. Build daemon with embedded UI" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "This takes about 2-3 minutes. Proceed? (Y/N)" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -eq 'Y' -or $response -eq 'y') {
        Write-Host ""
        Write-Host "Building WASM..." -ForegroundColor Cyan
        cd keyrx_ui
        npm run build:wasm
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] WASM build failed" -ForegroundColor Red
            exit 1
        }

        Write-Host "Building UI..." -ForegroundColor Cyan
        npm run build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] UI build failed" -ForegroundColor Red
            exit 1
        }
        cd ..

        Write-Host "Building daemon..." -ForegroundColor Cyan
        cargo build --release --features windows
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[ERROR] Daemon build failed" -ForegroundColor Red
            exit 1
        }

        Write-Host "✓ Rebuild complete!" -ForegroundColor Green
        $needsRestart = $true
    } else {
        Write-Host "Rebuild cancelled" -ForegroundColor Yellow
        exit 0
    }
}

if ($needsRestart) {
    Write-Host ""
    Write-Host "Restarting daemon with debug logging..." -ForegroundColor Cyan

    # Stop existing daemon
    if ($daemon) {
        Write-Host "Stopping existing daemon..." -ForegroundColor Yellow
        Stop-Process -Id $daemon.Id -Force
        Start-Sleep -Seconds 2
    }

    # Launch with debug
    Write-Host "Launching daemon with debug..." -ForegroundColor Cyan
    & ".\scripts\windows\Debug-Launch.ps1"

    Start-Sleep -Seconds 3

    # Verify it started
    $newDaemon = Get-Process keyrx_daemon -ErrorAction SilentlyContinue
    if ($newDaemon) {
        Write-Host "✓ Daemon started successfully (PID: $($newDaemon.Id))" -ForegroundColor Green
    } else {
        Write-Host "✗ Failed to start daemon" -ForegroundColor Red
        exit 1
    }
}

# 6. Final status
Write-Host ""
Write-Host "[6/6] Final Status" -ForegroundColor Yellow
Write-Host ""
Write-Host "✓ Daemon running with debug logging" -ForegroundColor Green
Write-Host "✓ Web UI available at: http://localhost:9867" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Open Web UI in browser" -ForegroundColor White
Write-Host "2. Import example config: .\scripts\windows\Import-Example-Config.ps1" -ForegroundColor White
Write-Host "3. Go to Profiles page and activate profile" -ForegroundColor White
Write-Host "4. Test on Metrics page" -ForegroundColor White
Write-Host ""
Write-Host "View logs in real-time:" -ForegroundColor Cyan
Write-Host "  Get-Content `"$env:TEMP\keyrx-debug.log`" -Tail 50 -Wait" -ForegroundColor White
Write-Host ""

# Offer to open browser
Write-Host "Open Web UI now? (Y/N)" -ForegroundColor Yellow
$response = Read-Host
if ($response -eq 'Y' -or $response -eq 'y') {
    Start-Process "http://localhost:9867"
    Write-Host "✓ Web UI opened" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Diagnostics Complete" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
