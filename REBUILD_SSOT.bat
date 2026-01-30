@echo off
REM Rebuild with SSOT - Single Source of Truth
REM This ensures all version/build info is regenerated fresh
REM Right-click and "Run as Administrator"

echo ========================================
echo  SSOT Rebuild - KeyRx v0.1.5
echo ========================================
echo.
echo This will:
echo 1. Clean all cached build artifacts
echo 2. Regenerate version.ts with current timestamp
echo 3. Rebuild UI from scratch
echo 4. Rebuild daemon with fresh BUILD_DATE
echo 5. Install to Program Files
echo 6. Verify SSOT compliance
echo.

REM Check admin
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Requires administrator privileges!
    echo Right-click and select "Run as Administrator"
    pause
    exit /b 1
)

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo [1/7] Stopping daemon...
taskkill /F /IM keyrx_daemon.exe >nul 2>&1
timeout /t 3 /nobreak >nul

echo [2/7] Cleaning build artifacts...
REM Force clean to remove cached build.rs outputs
if exist "target\release\keyrx_daemon.exe" del /F /Q "target\release\keyrx_daemon.exe" >nul 2>&1
if exist "target\release\keyrx_daemon.pdb" del /F /Q "target\release\keyrx_daemon.pdb" >nul 2>&1
if exist "target\release\.fingerprint\keyrx_daemon-*" rd /S /Q "target\release\.fingerprint\keyrx_daemon-*" >nul 2>&1
if exist "target\release\build\keyrx_daemon-*" rd /S /Q "target\release\build\keyrx_daemon-*" >nul 2>&1
if exist "target\release\deps\keyrx_daemon*" del /F /Q "target\release\deps\keyrx_daemon*" >nul 2>&1
echo   Cleaned daemon artifacts

echo [3/7] Regenerating UI version.ts...
cd keyrx_ui
call node ..\scripts\generate-version.js
if %errorLevel% neq 0 (
    echo   ERROR: Failed to generate version.ts
    cd ..
    pause
    exit /b 1
)
cd ..

echo [4/7] Syncing port configuration (SSOT)...
cd keyrx_ui
call npm run sync-port
if %errorLevel% neq 0 (
    echo   ERROR: Port sync failed!
    cd ..
    pause
    exit /b 1
)

echo [5/7] Rebuilding UI (PRODUCTION MODE)...
REM Skip WASM build on Windows (already built), just compile TypeScript and Vite
call npx tsc -b >nul 2>&1
if %errorLevel% neq 0 (
    echo   ERROR: TypeScript compilation failed!
    cd ..
    pause
    exit /b 1
)
call npx vite build --mode production >nul 2>&1
if %errorLevel% neq 0 (
    echo   ERROR: Vite build failed!
    cd ..
    pause
    exit /b 1
)
echo   UI built successfully
cd ..

echo [6/7] Rebuilding daemon (clean build)...
REM Clean build forces build.rs to regenerate BUILD_DATE
cargo build --release -p keyrx_daemon 2>&1 | findstr /C:"Compiling keyrx_daemon" /C:"Finished" /C:"error"
if %errorLevel% neq 0 (
    echo   Build completed (check for errors above)
)

echo [7/7] Installing fresh binary...
copy /Y "target\release\keyrx_daemon.exe" "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" >nul 2>&1
if %errorLevel% neq 0 (
    echo   ERROR: Failed to copy binary!
    pause
    exit /b 1
)

echo [8/8] Verifying timestamps...
powershell -Command "Get-Item 'target\release\keyrx_daemon.exe' | Select-Object @{N='Source';E={$_.LastWriteTime.ToString('yyyy/MM/dd HH:mm:ss')}}"
powershell -Command "Get-Item 'C:\Program Files\KeyRx\bin\keyrx_daemon.exe' | Select-Object @{N='Installed';E={$_.LastWriteTime.ToString('yyyy/MM/dd HH:mm:ss')}}"

echo.
echo ========================================
echo  SSOT Rebuild Complete!
echo ========================================
echo.
echo Starting daemon...
start "" "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" run

timeout /t 10 /nobreak >nul

echo Testing...
powershell -Command "try { $h = Invoke-RestMethod http://localhost:9867/api/health; Write-Host 'API OK - Version:' $h.version } catch { Write-Host 'API not ready yet' -ForegroundColor Yellow }"

echo.
echo Next steps:
echo 1. Check system tray icon - Right-click - About
echo    - Should show current build time (not 14:21)
echo 2. Open http://localhost:9867
echo    - Footer should show v0.1.2 with current date
echo 3. Activate a profile and test remapping
echo.
pause
