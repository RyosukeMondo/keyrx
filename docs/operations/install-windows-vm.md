# Install and Run keyrx on Windows VM

This guide shows how to build, install, and run keyrx on the Vagrant Windows VM so you can test it via RDP from your Windows PC.

## Workflow Overview

```
Your Windows PC → RDP → Vagrant Windows VM (on Linux host) → keyrx running
```

You'll connect from your Windows PC to the VM via RDP and interact with keyrx.

## Quick Start

### 1. Start the Vagrant Windows VM

```bash
# On Linux host
cd vagrant/windows
vagrant up

# Wait for provisioning to complete (~20 minutes first time)
```

### 2. Build keyrx for Windows

```bash
# Sync files to VM
vagrant rsync

# Build release binary
vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'
```

### 3. Connect via RDP

**From your Windows PC:**
```cmd
mstsc /v:LINUX_HOST_IP:13389
```

Or use the GUI:
- Press `Win+R`, type `mstsc`, press Enter
- Computer: `LINUX_HOST_IP:13389`
- Username: `vagrant`
- Password: `vagrant`

**From Linux host (testing):**
```bash
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard
```

### 4. Run keyrx in the VM

**In the RDP session (PowerShell):**
```powershell
cd C:\vagrant_project

# Run the daemon
.\target\release\keyrx_daemon.exe
```

## Detailed Setup

### Build Options

#### Option 1: Build in VM (Recommended)

```bash
# On Linux host
cd vagrant/windows
vagrant rsync
vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'
```

**Advantages:**
- Native Windows build
- Guaranteed compatibility
- Uses MSVC toolchain

#### Option 2: Cross-compile on Linux

```bash
# On Linux host
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu --features windows

# Copy to VM
cd vagrant/windows
mkdir -p /tmp/keyrx_windows
cp ../../target/x86_64-pc-windows-gnu/release/keyrx_daemon.exe /tmp/keyrx_windows/
vagrant upload /tmp/keyrx_windows/keyrx_daemon.exe C:/Users/vagrant/Desktop/keyrx_daemon.exe
```

### Installation Options

#### Option 1: Run from Build Directory (Quick Testing)

```powershell
# In RDP session
cd C:\vagrant_project\target\release
.\keyrx_daemon.exe
```

#### Option 2: Install to Windows Program Files

```powershell
# In RDP session (PowerShell as Administrator)
cd C:\vagrant_project

# Create installation directory
New-Item -ItemType Directory -Path "C:\Program Files\keyrx" -Force

# Copy binaries
Copy-Item target\release\keyrx_daemon.exe "C:\Program Files\keyrx\"
Copy-Item target\release\keyrx_compiler.exe "C:\Program Files\keyrx\" -ErrorAction SilentlyContinue

# Add to PATH
$env:Path += ";C:\Program Files\keyrx"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::Machine)
```

#### Option 3: Install as Windows Service

```powershell
# In RDP session (PowerShell as Administrator)
cd C:\vagrant_project

# Install NSSM (Non-Sucking Service Manager)
choco install nssm -y

# Create service
nssm install keyrx "C:\Program Files\keyrx\keyrx_daemon.exe"
nssm set keyrx Description "KeyRx Keyboard Remapping Daemon"
nssm set keyrx Start SERVICE_AUTO_START

# Start service
nssm start keyrx

# Check status
nssm status keyrx
```

### Running keyrx

#### Run Manually (Testing)

```powershell
# In RDP session
cd C:\vagrant_project\target\release
.\keyrx_daemon.exe
```

**With custom config:**
```powershell
.\keyrx_daemon.exe --config C:\Users\vagrant\.config\keyrx\config.krx
```

**With debug logging:**
```powershell
.\keyrx_daemon.exe --log-level debug
```

#### Run on Startup (Autostart)

**Option A: Task Scheduler**

```powershell
# In RDP session (PowerShell)
$action = New-ScheduledTaskAction -Execute "C:\Program Files\keyrx\keyrx_daemon.exe"
$trigger = New-ScheduledTaskTrigger -AtLogOn
$principal = New-ScheduledTaskPrincipal -UserId "vagrant" -LogonType Interactive
Register-ScheduledTask -TaskName "keyrx" -Action $action -Trigger $trigger -Principal $principal
```

**Option B: Startup Folder**

```powershell
# Create shortcut in startup folder
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\keyrx.lnk")
$Shortcut.TargetPath = "C:\Program Files\keyrx\keyrx_daemon.exe"
$Shortcut.Save()
```

### Configuration

#### Create Config File

```powershell
# In RDP session
mkdir C:\Users\vagrant\.config\keyrx -Force

# Create example config
@"
// KeyRx Configuration for Windows

// Example: Swap Caps Lock and Escape
map(KEY_CAPSLOCK, KEY_ESC)
map(KEY_ESC, KEY_CAPSLOCK)

// Example: Make Right Alt a Ctrl
map(KEY_RIGHTALT, KEY_LEFTCTRL)

info("KeyRx Windows configuration loaded!");
"@ | Out-File -FilePath C:\Users\vagrant\.config\keyrx\config.rhai -Encoding UTF8
```

#### Compile Config

```powershell
# Compile the Rhai config to .krx binary
cd C:\vagrant_project\target\release
.\keyrx_compiler.exe C:\Users\vagrant\.config\keyrx\config.rhai -o C:\Users\vagrant\.config\keyrx\config.krx
```

### Testing keyrx

#### Test 1: Verify Daemon Starts

```powershell
# In RDP session
cd C:\vagrant_project\target\release
.\keyrx_daemon.exe

# Expected output:
# [INFO] keyrx_daemon starting...
# [INFO] Listening for keyboard events...
```

Press `Ctrl+C` to stop.

#### Test 2: Test Key Remapping

```powershell
# Start daemon
.\keyrx_daemon.exe --config C:\Users\vagrant\.config\keyrx\config.krx

# Open Notepad
notepad.exe

# Test remapping:
# - Press Caps Lock → Should produce Escape behavior
# - Press Escape → Should toggle Caps Lock
```

#### Test 3: Check System Tray (If GUI Enabled)

If keyrx has a system tray icon:
- Look in Windows system tray (bottom-right)
- Right-click icon for options
- Should show: Status, Configuration, Exit

### RDP Connection Tips

#### Connect from Windows PC

**Using mstsc (Remote Desktop Connection):**
```cmd
mstsc /v:LINUX_HOST_IP:13389 /f
```

Flags:
- `/f` - Fullscreen mode
- `/w:1920 /h:1080` - Specific resolution
- `/admin` - Administrative session

**Using Remote Desktop Connection GUI:**
1. Press `Win+R`, type `mstsc`, Enter
2. Click "Show Options"
3. Computer: `LINUX_HOST_IP:13389`
4. Username: `vagrant`
5. Click "Connect"
6. Enter password: `vagrant`

#### Connect from Linux Host

```bash
# FreeRDP
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard

# Remmina (GUI)
remmina -c rdp://vagrant:vagrant@localhost:13389
```

#### Performance Settings

For better RDP performance:

```bash
# Lower color depth
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /bpp:16

# Disable visual effects
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant -wallpaper -aero -window-drag
```

### Troubleshooting

#### Build Fails: "cargo not found"

```powershell
# In RDP session, check Rust installation
rustc --version
cargo --version

# If missing, re-provision VM
exit  # Close RDP
cd vagrant/windows
vagrant provision --provision-with configure-rust
```

#### Daemon Fails: "Failed to initialize RawInput"

**Cause:** RawInput requires GUI session.

**Solution:** Ensure you're running in RDP (GUI) session, not WinRM (CLI).

```powershell
# Run in RDP session, NOT via vagrant winrm
```

#### Key Remapping Not Working

**Possible causes:**

1. **RDP input limitations** - See `docs/testing-windows-rdp.md`
2. **Config not loaded** - Check config path
3. **Permissions issue** - Run as Administrator

**Test with elevated permissions:**
```powershell
# In RDP session, run as Administrator
Start-Process powershell -Verb RunAs
cd C:\vagrant_project\target\release
.\keyrx_daemon.exe
```

#### Port 13389 Not Accessible

```bash
# On Linux host, check port forwarding
cd vagrant/windows
vagrant port

# Expected output:
# 3389 (guest) => 13389 (host)

# Check firewall
sudo ufw status
sudo ufw allow 13389/tcp
```

#### RDP Connection Refused

```bash
# Check VM is running
vagrant status

# Restart RDP service in VM
vagrant winrm -c 'Restart-Service TermService -Force'

# Check if RDP is listening
vagrant winrm -c 'Get-NetTCPConnection -LocalPort 3389'
```

### Automated Setup Script

Create a setup script for convenience:

```bash
# On Linux host
cat > vagrant/windows/setup_keyrx.sh << 'EOF'
#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Starting Vagrant VM..."
vagrant up

echo "Syncing files..."
vagrant rsync

echo "Building keyrx..."
vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'

echo "Installing keyrx..."
vagrant winrm -c 'New-Item -ItemType Directory -Path "C:\Program Files\keyrx" -Force; Copy-Item C:\vagrant_project\target\release\*.exe "C:\Program Files\keyrx\"'

echo "Creating config directory..."
vagrant winrm -c 'New-Item -ItemType Directory -Path "C:\Users\vagrant\.config\keyrx" -Force'

echo ""
echo "========================================="
echo "  keyrx Setup Complete!"
echo "========================================="
echo ""
echo "Connect via RDP:"
echo "  mstsc /v:LINUX_HOST_IP:13389"
echo "  Username: vagrant"
echo "  Password: vagrant"
echo ""
echo "Run keyrx in RDP session:"
echo "  C:\Program Files\keyrx\keyrx_daemon.exe"
echo ""
EOF

chmod +x vagrant/windows/setup_keyrx.sh
```

**Run it:**
```bash
./vagrant/windows/setup_keyrx.sh
```

### Quick Commands Reference

```bash
# === On Linux Host ===

# Start VM
cd vagrant/windows && vagrant up

# Sync files
vagrant rsync

# Build
vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'

# Check status
vagrant status

# Connect via RDP (Linux)
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080

# Stop VM
vagrant halt
```

```powershell
# === In Windows VM (RDP Session) ===

# Navigate to project
cd C:\vagrant_project

# Build
cargo build --release --features windows

# Run daemon
.\target\release\keyrx_daemon.exe

# Run with config
.\target\release\keyrx_daemon.exe --config C:\Users\vagrant\.config\keyrx\config.krx

# Check if running
Get-Process keyrx_daemon

# Stop daemon
Stop-Process -Name keyrx_daemon
```

## Next Steps

After installation:

1. **Create your config** - Edit `C:\Users\vagrant\.config\keyrx\config.rhai`
2. **Compile config** - Run `keyrx_compiler.exe config.rhai -o config.krx`
3. **Test remapping** - Run daemon and test in Notepad
4. **Set up autostart** - Use Task Scheduler or startup folder
5. **Monitor logs** - Check Event Viewer for issues

## See Also

- **RDP Testing Guide**: `docs/testing-windows-rdp.md`
- **Windows VM Setup**: `vagrant/windows/README.md`
- **Deployment Guide**: `docs/DEPLOYMENT.md`
