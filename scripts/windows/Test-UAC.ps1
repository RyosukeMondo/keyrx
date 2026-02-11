# Simple UAC elevation test

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# If not admin, re-launch with elevation
if (-not (Test-Administrator)) {
    Write-Host "Not admin - requesting elevation..." -ForegroundColor Yellow

    $scriptPath = $MyInvocation.MyCommand.Path
    $arguments = "-NoExit -NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""

    Start-Process powershell.exe -Verb RunAs -ArgumentList $arguments
    exit
}

# Now running as admin
Write-Host ""
Write-Host "SUCCESS! Running as administrator!" -ForegroundColor Green
Write-Host ""
Write-Host "Current user: $($env:USERNAME)" -ForegroundColor Cyan
Write-Host "Computer: $($env:COMPUTERNAME)" -ForegroundColor Cyan
Write-Host ""

# Kill all keyrx_daemon processes
Write-Host "Looking for keyrx_daemon processes..." -ForegroundColor Yellow
$processes = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue

if ($processes) {
    Write-Host "Found $($processes.Count) process(es):" -ForegroundColor Cyan
    foreach ($proc in $processes) {
        Write-Host "  PID $($proc.Id) - Started: $($proc.StartTime)" -ForegroundColor White
        try {
            Stop-Process -Id $proc.Id -Force -ErrorAction Stop
            Write-Host "  ✓ Killed PID $($proc.Id)" -ForegroundColor Green
        } catch {
            Write-Host "  ✗ Failed to kill PID $($proc.Id): $_" -ForegroundColor Red
        }
    }
} else {
    Write-Host "No keyrx_daemon processes found" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Press any key to close..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
