# SSH Tunnel Setup for Windows

Quick guide for accessing Linux development servers from Windows via SSH port forwarding.

## Prerequisites

- SSH client installed on Windows (built-in on Windows 10+)
- PowerShell 5.1+ or PowerShell Core 7+
- SSH access to Linux server

## Quick Start

### 1. Configure the Script

Edit `ssh-tunnel.ps1` and update:

```powershell
# Ports to forward
$tunnel_ports = @{
    4200 = 4200  # Web UI HTTP
    8080 = 8080  # Backend API
}

# Your Linux server details
$ssh_user = "rmondo"
$ssh_host = "192.168.11.53"  # Change to your Linux IP
```

### 2. Run the Script

**From Windows PowerShell:**

```powershell
cd \\wsl$\Ubuntu\home\rmondo\repos\keyrx\scripts
.\ssh-tunnel.ps1
```

Or if accessing via network share/copied to Windows:

```powershell
.\ssh-tunnel.ps1
```

### 3. Access Services

Once connected, open your Windows browser:
- Web UI: http://localhost:4200
- Backend API: http://localhost:8080

## Configuration Options

### Add/Remove Ports

```powershell
$tunnel_ports = @{
    4200 = 4200  # Local port 4200 -> Remote port 4200
    4443 = 4443  # HTTPS dev server
    8080 = 8080  # Backend API
    8888 = 8888  # Test server
    3000 = 3000  # Add any other port
}
```

### Background Mode

Run tunnels in background (no interactive shell):

```powershell
$run_in_background = $true
```

To stop background tunnel:
```powershell
# Find the SSH process
Get-Process ssh

# Kill it
Stop-Process -Name ssh
```

### Interactive Mode (Default)

Keeps an interactive SSH session open. Press `Ctrl+C` to close tunnels.

```powershell
$run_in_background = $false
```

## Troubleshooting

### Port Already in Use

If you get "bind: Address already in use":

**Option 1: Find and kill the process**
```powershell
# Windows PowerShell
netstat -ano | findstr :4200
taskkill /PID <PID> /F
```

**Option 2: Change local port**
```powershell
$tunnel_ports = @{
    4201 = 4200  # Use 4201 locally, connect to 4200 on remote
}
```

### SSH Connection Refused

Check:
1. Linux server IP is correct
2. SSH server is running on Linux: `sudo systemctl status sshd`
3. Firewall allows SSH (port 22)
4. Can ping Linux server from Windows

### Permission Denied

Run PowerShell as Administrator or check SSH key permissions:

```powershell
# Generate SSH key (if needed)
ssh-keygen -t ed25519

# Copy to Linux server
ssh-copy-id rmondo@192.168.11.53
```

## Advanced Usage

### Custom SSH Port

If SSH runs on non-standard port:

```powershell
$ssh_args += "-p"
$ssh_args += "2222"  # Your SSH port
$ssh_args += "${ssh_user}@${ssh_host}"
```

### SSH Config File

Create `~/.ssh/config` on Windows:

```
Host keyrx-dev
    HostName 192.168.11.53
    User rmondo
    Port 22
    IdentityFile ~/.ssh/id_ed25519
```

Then simplify script:
```powershell
$ssh_host = "keyrx-dev"
$ssh_user = ""  # Not needed with config
```

## Example Workflow

1. **On Linux**: Start dev servers
   ```bash
   cd ~/repos/keyrx/ui
   ./scripts/start-http-dev-server.sh  # Port 4200
   ```

2. **On Windows**: Run tunnel script
   ```powershell
   .\ssh-tunnel.ps1
   ```

3. **On Windows**: Open browser
   - Navigate to http://localhost:4200

4. **When Done**: Press `Ctrl+C` in PowerShell to close tunnels
