# Clean test profiles before running tests
$profileDir = Join-Path $env:LOCALAPPDATA "keyrx\profiles"
$deviceRegistry = Join-Path $env:LOCALAPPDATA "keyrx\devices.json"

Write-Host "Cleaning test data..."

if (Test-Path $profileDir) {
    Write-Host "Removing profiles directory: $profileDir"
    Remove-Item -Recurse -Force $profileDir
}

if (Test-Path $deviceRegistry) {
    Write-Host "Removing device registry: $deviceRegistry"
    Remove-Item -Force $deviceRegistry
}

Write-Host "Test data cleaned successfully"
