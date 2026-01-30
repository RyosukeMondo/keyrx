@echo off
REM Install freshly compiled daemon
REM Right-click this file and select "Run as Administrator"

echo ========================================
echo  Installing Fresh Build
echo ========================================
echo.

REM Check if running as admin
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: This script requires administrator privileges!
    echo Right-click this file and select "Run as Administrator"
    pause
    exit /b 1
)

REM Get script directory
set SCRIPT_DIR=%~dp0
echo Script directory: %SCRIPT_DIR%

echo Stopping daemon...
taskkill /F /IM keyrx_daemon.exe >nul 2>&1
timeout /t 3 /nobreak >nul

echo Copying new binary...
set SOURCE_BINARY=%SCRIPT_DIR%target\release\keyrx_daemon.exe
set DEST_BINARY=C:\Program Files\KeyRx\bin\keyrx_daemon.exe

echo Source: %SOURCE_BINARY%
echo Dest: %DEST_BINARY%
echo.

if not exist "%SOURCE_BINARY%" (
    echo ERROR: Source binary not found!
    echo Expected at: %SOURCE_BINARY%
    echo.
    echo Run: cargo build --release -p keyrx_daemon
    pause
    exit /b 1
)

copy /Y "%SOURCE_BINARY%" "%DEST_BINARY%"
if %errorLevel% neq 0 (
    echo ERROR: Failed to copy binary!
    pause
    exit /b 1
)

echo Verifying...
powershell -Command "Get-Item 'C:\Program Files\KeyRx\bin\keyrx_daemon.exe' | Select-Object LastWriteTime"

echo.
echo Starting daemon...
start "" "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" run

timeout /t 8 /nobreak >nul

echo.
echo Testing API...
powershell -Command "try { Invoke-RestMethod http://localhost:9867/api/health | ConvertTo-Json } catch { Write-Host 'API not ready yet' -ForegroundColor Yellow }"

echo.
echo ========================================
echo  Installation Complete!
echo ========================================
echo.
echo Next steps:
echo 1. Check system tray icon - right-click -^> About
echo 2. Build time should now be current (18:36)
echo 3. Open http://localhost:9867
echo 4. Activate a profile and test remapping
echo.
pause
