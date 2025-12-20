#!/usr/bin/env pwsh
# SSH Port Forwarding Script for KeyRx Development
# Tunnels localhost ports from Linux server to Windows client

#region Configuration
# Ports to forward (local:remote)
# Add or remove ports as needed
$tunnel_ports = @{
    4200 = 4200  # Web UI HTTP Dev Server
    4443 = 4443  # Web UI HTTPS Dev Server
    8080 = 8080  # Backend API Server
    8888 = 8888  # Backend Test Server
}

# SSH connection details
$ssh_user = "rmondo"
$ssh_host = "192.168.11.53"  # Update with your Linux IP
# $ssh_host = "your-linux-hostname"  # Or use hostname

# SSH options
$run_in_background = $false  # Set to $true to run as background process
#endregion

#region Main Script
Write-Host "🌐 KeyRx SSH Port Forwarding" -ForegroundColor Cyan
Write-Host "=" * 50 -ForegroundColor Cyan
Write-Host ""

# Build SSH command with port forwarding arguments
$port_args = @()
foreach ($local_port in $tunnel_ports.Keys) {
    $remote_port = $tunnel_ports[$local_port]
    $port_args += "-L"
    $port_args += "${local_port}:localhost:${remote_port}"
    Write-Host "📌 Forwarding: localhost:$local_port -> ${ssh_host}:${remote_port}" -ForegroundColor Green
}

Write-Host ""
Write-Host "🔗 Connecting to: ${ssh_user}@${ssh_host}" -ForegroundColor Yellow
Write-Host ""

# Build full SSH command
$ssh_args = @()
$ssh_args += $port_args

if ($run_in_background) {
    # Background mode: no shell, just tunneling
    $ssh_args += "-N"
    $ssh_args += "-f"
    Write-Host "ℹ️  Running in background mode (no interactive shell)" -ForegroundColor Cyan
} else {
    # Interactive mode: keep connection alive, disable pseudo-terminal issues
    $ssh_args += "-N"  # No remote command, just port forwarding
    Write-Host "ℹ️  Running in foreground (port forwarding only)" -ForegroundColor Cyan
    Write-Host "ℹ️  Press Ctrl+C to stop and close tunnels" -ForegroundColor Cyan
}

# Add verbose mode for debugging
$ssh_args += "-v"

$ssh_args += "${ssh_user}@${ssh_host}"

Write-Host ""
Write-Host "🚀 Executing: ssh $($ssh_args -join ' ')" -ForegroundColor DarkGray
Write-Host ""
Write-Host "📡 Establishing connection..." -ForegroundColor Yellow
Write-Host ""

# Execute SSH command
$exitCode = 0
try {
    # Use Start-Process for better control and error visibility
    if ($run_in_background) {
        $process = Start-Process -FilePath "ssh" -ArgumentList $ssh_args -NoNewWindow -PassThru
        Write-Host "✅ Background tunnel started (PID: $($process.Id))" -ForegroundColor Green
        Write-Host "ℹ️  To stop: Stop-Process -Id $($process.Id)" -ForegroundColor Cyan
    } else {
        # Foreground mode - wait for completion
        & ssh $ssh_args
        $exitCode = $LASTEXITCODE
    }
} catch {
    Write-Host ""
    Write-Host "❌ SSH connection failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 Troubleshooting:" -ForegroundColor Yellow
    Write-Host "   1. Verify SSH host: $ssh_host" -ForegroundColor Gray
    Write-Host "   2. Test connection: ssh ${ssh_user}@${ssh_host}" -ForegroundColor Gray
    Write-Host "   3. Check SSH key/password authentication" -ForegroundColor Gray
    exit 1
}

if (-not $run_in_background) {
    Write-Host ""
    if ($exitCode -eq 0) {
        Write-Host "✅ Tunnels closed" -ForegroundColor Green
    } else {
        Write-Host "⚠️  SSH exited with code: $exitCode" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "💡 Common issues:" -ForegroundColor Yellow
        Write-Host "   - Exit code 255: Connection refused/authentication failed" -ForegroundColor Gray
        Write-Host "   - Check if host is reachable: ping $ssh_host" -ForegroundColor Gray
        Write-Host "   - Verify SSH service running on remote host" -ForegroundColor Gray
    }
}
#endregion
