#Requires -RunAsAdministrator
# Immediate Diagnostics - Find Why Activation Fails

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logFile = "DIAGNOSTIC_$timestamp.txt"

function Log {
    param($message, $color = "White")
    Write-Host $message -ForegroundColor $color
    Add-Content -Path $logFile -Value $message
}

Log "========================================" "Cyan"
Log " IMMEDIATE DIAGNOSTICS - $timestamp" "Cyan"
Log "========================================" "Cyan"
Log ""

# 1. Check daemon
Log "[1] Daemon Process:" "Yellow"
$daemon = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($daemon) {
    Log "  PID: $($daemon.Id)"
    Log "  Started: $($daemon.StartTime)"
    Log "  Path: $($daemon.Path)"
} else {
    Log "  ERROR: Daemon not running!" "Red"
}

# 2. Check port
Log ""
Log "[2] Port 9867:" "Yellow"
$port = Get-NetTCPConnection -LocalPort 9867 -State Listen -ErrorAction SilentlyContinue
if ($port) {
    Log "  Listening: Yes (PID: $($port.OwningProcess))"
} else {
    Log "  ERROR: Port 9867 not listening!" "Red"
}

# 3. API Health
Log ""
Log "[3] API Health:" "Yellow"
try {
    $health = Invoke-RestMethod -Uri http://localhost:9867/api/health -TimeoutSec 5
    Log "  Status: OK" "Green"
    Log "  Version: $($health.version)" "Green"
} catch {
    Log "  ERROR: $($_.Exception.Message)" "Red"
}

# 4. List profiles
Log ""
Log "[4] List Profiles:" "Yellow"
try {
    $profiles = Invoke-RestMethod -Uri http://localhost:9867/api/profiles -TimeoutSec 5
    foreach ($p in $profiles.profiles) {
        $active = if ($p.isActive) { "ACTIVE" } else { "" }
        Log "  - $($p.name) $active"
    }
} catch {
    Log "  ERROR: $($_.Exception.Message)" "Red"
}

# 5. Test activation with timing
Log ""
Log "[5] Test profile-a Activation:" "Yellow"
$start = Get-Date
try {
    $response = Invoke-WebRequest -Uri http://localhost:9867/api/profiles/profile-a/activate -Method POST -TimeoutSec 30
    $elapsed = ((Get-Date) - $start).TotalSeconds

    Log "  HTTP Status: $($response.StatusCode)" "Green"
    Log "  Time: $elapsed seconds" "Green"
    Log "  Response: $($response.Content)"
} catch {
    $elapsed = ((Get-Date) - $start).TotalSeconds
    Log "  FAILED after $elapsed seconds" "Red"
    Log "  Status: $($_.Exception.Response.StatusCode)" "Red"
    Log "  Error: $($_.Exception.Message)" "Red"

    if ($_.Exception.Response) {
        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $body = $reader.ReadToEnd()
        Log "  Body: $body" "Yellow"
    }
}

# 6. Check daemon logs
Log ""
Log "[6] Recent Daemon Logs:" "Yellow"
$daemonLogs = Get-ChildItem -Path . -Filter "daemon_stderr_*.log" -ErrorAction SilentlyContinue |
              Sort-Object LastWriteTime -Descending |
              Select-Object -First 1

if ($daemonLogs) {
    Log "  Log: $($daemonLogs.FullName)"
    Log "  Last 30 lines:"
    Get-Content $daemonLogs.FullName -Tail 30 | ForEach-Object {
        Log "    $_"
    }
} else {
    Log "  No daemon stderr logs found" "Yellow"
}

# 7. Check active profile file
Log ""
Log "[7] Active Profile File:" "Yellow"
$activeFile = "$env:APPDATA\keyrx\.active"
if (Test-Path $activeFile) {
    $active = Get-Content $activeFile -Raw
    Log "  Content: $active"
} else {
    Log "  No .active file" "Yellow"
}

# 8. Check settings
Log ""
Log "[8] Daemon Settings:" "Yellow"
$settingsFile = "$env:APPDATA\keyrx\settings.json"
if (Test-Path $settingsFile) {
    $settings = Get-Content $settingsFile -Raw
    Log "  Settings: $settings"
} else {
    Log "  No settings.json (using defaults)" "Yellow"
}

# 9. Check .krx files
Log ""
Log "[9] Compiled Profiles:" "Yellow"
$profilesDir = "$env:APPDATA\keyrx\profiles"
if (Test-Path $profilesDir) {
    Get-ChildItem -Path $profilesDir -Filter "*.krx" | ForEach-Object {
        Log "  $($_.Name): $($_.Length) bytes, $($_.LastWriteTime)"
    }
} else {
    Log "  Profiles directory not found" "Red"
}

# 10. Test metrics endpoint
Log ""
Log "[10] Metrics Endpoint:" "Yellow"
try {
    $metrics = Invoke-RestMethod -Uri http://localhost:9867/api/metrics -TimeoutSec 5
    Log "  Metrics returned: $(($metrics | ConvertTo-Json).Length) chars"
} catch {
    Log "  ERROR: $($_.Exception.Message)" "Red"
}

Log ""
Log "========================================" "Cyan"
Log "DIAGNOSTIC COMPLETE" "Cyan"
Log "========================================" "Cyan"
Log ""
Log "Log saved to: $logFile" "Green"
Log ""

pause
