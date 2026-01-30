# Gather All Logs and System Info
# Run this script to collect diagnostic information

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$outputFile = "LOGS_$timestamp.txt"

Write-Host "Gathering logs and system info..." -ForegroundColor Cyan
Write-Host "Output: $outputFile" -ForegroundColor Gray
Write-Host ""

# Start output file
"KeyRx Diagnostic Log" | Out-File $outputFile
"Generated: $(Get-Date)" | Out-File $outputFile -Append
"=" * 80 | Out-File $outputFile -Append
"" | Out-File $outputFile -Append

# 1. Daemon Process Info
"========== DAEMON PROCESS INFO ==========" | Out-File $outputFile -Append
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    "Status: RUNNING" | Out-File $outputFile -Append
    "PID: $($daemon.Id)" | Out-File $outputFile -Append
    "Start Time: $($daemon.StartTime)" | Out-File $outputFile -Append
    "Memory (MB): $([math]::Round($daemon.WorkingSet64 / 1MB, 2))" | Out-File $outputFile -Append
} else {
    "Status: NOT RUNNING" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 2. Binary Info
"========== BINARY INFO ==========" | Out-File $outputFile -Append
$binary = "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
if (Test-Path $binary) {
    $file = Get-Item $binary
    "Path: $binary" | Out-File $outputFile -Append
    "Size: $($file.Length) bytes" | Out-File $outputFile -Append
    "Last Modified: $($file.LastWriteTime)" | Out-File $outputFile -Append

    # Get file version (if available)
    $version = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($binary)
    "Product Version: $($version.ProductVersion)" | Out-File $outputFile -Append
} else {
    "Binary not found at: $binary" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 3. API Health Check
"========== API HEALTH CHECK ==========" | Out-File $outputFile -Append
try {
    $health = Invoke-RestMethod http://localhost:9867/api/health -TimeoutSec 5
    "Status: OK" | Out-File $outputFile -Append
    $health | ConvertTo-Json | Out-File $outputFile -Append
} catch {
    "Status: FAILED" | Out-File $outputFile -Append
    "Error: $($_.Exception.Message)" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 4. Active Profile
"========== ACTIVE PROFILE ==========" | Out-File $outputFile -Append
try {
    $profiles = Invoke-RestMethod http://localhost:9867/api/profiles -TimeoutSec 5
    $active = $profiles.profiles | Where-Object { $_.active -eq $true }
    if ($active) {
        "Active Profile: $($active.name)" | Out-File $outputFile -Append
        "Activated At: $($active.activatedAt)" | Out-File $outputFile -Append
    } else {
        "No active profile" | Out-File $outputFile -Append
    }
} catch {
    "Failed to get profiles: $($_.Exception.Message)" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 5. Daemon Log (if exists)
"========== DAEMON LOG (Last 100 lines) ==========" | Out-File $outputFile -Append
$logPath = "$env:USERPROFILE\AppData\Roaming\keyrx\daemon.log"
if (Test-Path $logPath) {
    "Log found at: $logPath" | Out-File $outputFile -Append
    Get-Content $logPath -Tail 100 -ErrorAction SilentlyContinue | Out-File $outputFile -Append
} else {
    $logPath2 = "$env:USERPROFILE\.keyrx\daemon.log"
    if (Test-Path $logPath2) {
        "Log found at: $logPath2" | Out-File $outputFile -Append
        Get-Content $logPath2 -Tail 100 -ErrorAction SilentlyContinue | Out-File $outputFile -Append
    } else {
        "No daemon log found at:" | Out-File $outputFile -Append
        "  - $logPath" | Out-File $outputFile -Append
        "  - $logPath2" | Out-File $outputFile -Append
    }
}
"" | Out-File $outputFile -Append

# 6. Config Files
"========== CONFIG FILES ==========" | Out-File $outputFile -Append
$configDir = "$env:USERPROFILE\AppData\Roaming\keyrx"
if (Test-Path $configDir) {
    "Config directory: $configDir" | Out-File $outputFile -Append
    Get-ChildItem $configDir -File -Recurse -ErrorAction SilentlyContinue |
        Select-Object FullName,Length,LastWriteTime |
        Format-Table -AutoSize |
        Out-String -Width 200 |
        Out-File $outputFile -Append
} else {
    "Config directory not found: $configDir" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 7. Network Ports
"========== NETWORK PORTS ==========" | Out-File $outputFile -Append
"Checking port 9867..." | Out-File $outputFile -Append
$portCheck = Test-NetConnection -ComputerName localhost -Port 9867 -InformationLevel Quiet -WarningAction SilentlyContinue
if ($portCheck) {
    "Port 9867: OPEN (daemon is listening)" | Out-File $outputFile -Append
} else {
    "Port 9867: CLOSED (daemon NOT listening)" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

# 8. Windows Event Log (application errors)
"========== WINDOWS EVENT LOG (Last 10 application errors) ==========" | Out-File $outputFile -Append
try {
    Get-EventLog -LogName Application -EntryType Error -Newest 10 -ErrorAction SilentlyContinue |
        Where-Object { $_.Source -like "*keyrx*" -or $_.Message -like "*keyrx*" } |
        Select-Object TimeGenerated,Source,Message |
        Format-Table -AutoSize |
        Out-String -Width 200 |
        Out-File $outputFile -Append
} catch {
    "No relevant errors found" | Out-File $outputFile -Append
}
"" | Out-File $outputFile -Append

Write-Host "========================================" -ForegroundColor Green
Write-Host " Log Collection Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Output file: $outputFile" -ForegroundColor Cyan
Write-Host ""
Write-Host "You can now:" -ForegroundColor Yellow
Write-Host "1. Review the log file" -ForegroundColor White
Write-Host "2. Share it for troubleshooting" -ForegroundColor White
Write-Host "3. Check for error messages" -ForegroundColor White
Write-Host ""
