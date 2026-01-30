@echo off
REM Helper to run UPDATE_BINARY.ps1 as Administrator
REM Double-click this file to update the binary

echo Starting UPDATE_BINARY.ps1 as Administrator...
echo.

powershell -Command "Start-Process powershell -ArgumentList '-ExecutionPolicy Bypass -File \"%~dp0UPDATE_BINARY.ps1\"' -Verb RunAs"
