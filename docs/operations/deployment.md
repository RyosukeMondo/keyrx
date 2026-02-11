# Windows Deployment Guide

This guide explains how to deploy keyrx Windows binaries from your Linux development environment to a remote Windows PC for testing.

## Overview

**Your setup:**
- **Linux host**: Where you develop and build
- **Windows client**: Target PC accessible via SSH
- **Vagrant Windows VM**: On Linux host, for building Windows binaries

**Deployment workflow:**
1. Build Windows binaries (either in Vagrant VM or cross-compile on Linux)
2. Transfer binaries to Windows client via SSH (SCP or rsync)
3. Optionally run installer on Windows client

## Quick Start

### 1. Setup SSH Access to Windows Client

On your Windows client, ensure OpenSSH Server is installed and running:

```powershell
# On Windows PC (PowerShell as Administrator)
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
Start-Service sshd
Set-Service -Name sshd -StartupType 'Automatic'

# Configure firewall
New-NetFirewallRule -Name sshd -DisplayName 'OpenSSH Server (sshd)' -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22
```

### 2. Setup SSH Key Authentication (Recommended)

```bash
# On Linux host
ssh-copy-id user@windows-client-ip

# Test connection
ssh user@windows-client-ip "echo Connection successful"
```

### 3. Create Deployment Configuration

```bash
# Copy example configuration
cp .deploy.env.example .deploy.env

# Edit with your settings
vim .deploy.env
```

Example `.deploy.env`:
```bash
WINDOWS_CLIENT_HOST=192.168.1.100
WINDOWS_CLIENT_USER=developer
DEPLOY_DIR=C:/keyrx_deploy
BUILD_IN_VM=true
USE_RSYNC=false
```

### 4. Deploy

```bash
# Source configuration and deploy
source .deploy.env && ./scripts/deploy_windows.sh

# Or use flags directly
./scripts/deploy_windows.sh --host 192.168.1.100 --user developer
```

## Deployment Options

### Option 1: SCP (Simple File Copy) - **Recommended for Getting Started**

**Pros:**
- Simple, works out-of-the-box with SSH
- No additional dependencies on Windows
- Best for one-time or infrequent deployments

**Cons:**
- Always copies all files (slower for repeated deployments)

**Usage:**
```bash
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER
```

### Option 2: Rsync (Incremental Sync) - **Recommended for Development**

**Pros:**
- Only transfers changed files (much faster for repeated deployments)
- Preserves timestamps
- Resume interrupted transfers

**Cons:**
- Requires rsync on Windows (available via Git for Windows, WSL, or Cygwin)

**Setup rsync on Windows:**
- Option A: Install Git for Windows (includes rsync in Git Bash)
- Option B: Install WSL and rsync package
- Option C: Install Cygwin with rsync package

**Usage:**
```bash
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --use-rsync
```

### Option 3: SMB Network Share - **Alternative Approach**

Instead of using the deployment script, you can manually share a folder.

**Setup:**
1. **On Windows PC:** Share a folder (e.g., `C:\keyrx_deploy`)
   - Right-click folder → Properties → Sharing → Advanced Sharing
   - Check "Share this folder"
   - Set permissions (Read/Write for your user)

2. **On Linux host:** Mount the share
   ```bash
   # Install CIFS utilities
   sudo apt install cifs-utils

   # Create mount point
   sudo mkdir -p /mnt/windows_deploy

   # Mount the share
   sudo mount -t cifs //192.168.1.100/keyrx_deploy /mnt/windows_deploy \
       -o username=developer,password=yourpassword

   # Or use credentials file for security
   echo "username=developer" > ~/.smbcredentials
   echo "password=yourpassword" >> ~/.smbcredentials
   chmod 600 ~/.smbcredentials

   sudo mount -t cifs //192.168.1.100/keyrx_deploy /mnt/windows_deploy \
       -o credentials=$HOME/.smbcredentials
   ```

3. **Copy files directly:**
   ```bash
   # After building in Vagrant VM
   cp vagrant/windows/target/release/*.exe /mnt/windows_deploy/
   ```

### Option 4: HTTP Server (Quick & Dirty)

Good for one-off transfers without SSH setup.

**On Linux host:**
```bash
# Serve build directory
cd target/x86_64-pc-windows-gnu/release
python3 -m http.server 8000
```

**On Windows client:**
```powershell
# Download files
Invoke-WebRequest -Uri http://LINUX_HOST:8000/keyrx_daemon.exe -OutFile C:\keyrx_deploy\keyrx_daemon.exe
```

## Build Options

### Build in Vagrant VM (Recommended)

Uses the existing Vagrant Windows VM to build native Windows binaries.

**Pros:**
- Produces true native Windows binaries
- Same environment as Windows testing
- Guaranteed compatibility

**Cons:**
- Slower (VM overhead)
- Requires Vagrant VM running

**Usage:**
```bash
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER
# (default behavior, BUILD_IN_VM=true)
```

### Cross-Compile on Linux

Build Windows binaries directly on Linux using MinGW cross-compiler.

**Pros:**
- Faster (no VM overhead)
- Uses Linux build cache
- Can run without Vagrant

**Cons:**
- May have subtle compatibility issues
- Requires MinGW toolchain setup

**Setup:**
```bash
# Install MinGW cross-compiler
sudo apt install mingw-w64

# Add Windows target
rustup target add x86_64-pc-windows-gnu
```

**Usage:**
```bash
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --no-vm-build
```

## Script Usage

### Basic Deployment

```bash
# Deploy to Windows client
./scripts/deploy_windows.sh --host 192.168.1.100 --user developer

# Using environment variables
export WINDOWS_CLIENT_HOST=192.168.1.100
export WINDOWS_CLIENT_USER=developer
./scripts/deploy_windows.sh
```

### Advanced Options

```bash
# Skip building, deploy existing binaries
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --skip-build

# Deploy only installer package
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --installer-only

# Deploy and run installer
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --run-installer

# Use rsync for faster repeated deployments
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --use-rsync

# Custom deployment directory
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --deploy-dir D:/projects/keyrx

# Quiet mode (minimal output)
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --quiet

# JSON output (for automation)
./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER --json
```

### Complete Workflow Examples

**Development workflow (repeated deployments):**
```bash
# Initial setup
cp .deploy.env.example .deploy.env
# Edit .deploy.env with your settings

# First deployment
source .deploy.env && ./scripts/deploy_windows.sh

# After code changes, fast incremental deployment
source .deploy.env && ./scripts/deploy_windows.sh --use-rsync
```

**CI/CD workflow:**
```bash
# Build in VM, deploy, run installer
./scripts/deploy_windows.sh \
    --host $CI_WINDOWS_HOST \
    --user $CI_WINDOWS_USER \
    --run-installer \
    --json > deployment_result.json
```

## Troubleshooting

### SSH Connection Failed

**Error:** "Cannot connect to Windows host via SSH"

**Solutions:**
1. Verify SSH server is running on Windows:
   ```powershell
   Get-Service sshd
   ```

2. Check firewall allows port 22:
   ```powershell
   Get-NetFirewallRule -Name sshd
   ```

3. Test connection manually:
   ```bash
   ssh user@windows-host
   ```

4. Setup SSH key authentication:
   ```bash
   ssh-copy-id user@windows-host
   ```

### Permission Denied

**Error:** "Permission denied (publickey)"

**Solutions:**
1. Use password authentication (temporary):
   ```bash
   ssh -o PubkeyAuthentication=no user@windows-host
   ```

2. Copy SSH key properly:
   ```bash
   ssh-copy-id -i ~/.ssh/id_rsa.pub user@windows-host
   ```

3. Check SSH config on Windows:
   ```powershell
   # Edit: C:\ProgramData\ssh\sshd_config
   # Ensure these are set:
   PubkeyAuthentication yes
   PasswordAuthentication yes  # Enable temporarily for setup
   ```

### Build Failed in VM

**Error:** "Build failed in Vagrant VM"

**Solutions:**
1. Check VM status:
   ```bash
   cd vagrant/windows
   vagrant status
   ```

2. Manually test build:
   ```bash
   cd vagrant/windows
   vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'
   ```

3. Restore VM snapshot:
   ```bash
   vagrant snapshot restore provisioned
   ```

### Rsync Not Found on Windows

**Error:** "rsync: command not found" on Windows

**Solutions:**
1. Install Git for Windows (includes rsync)
2. Or install WSL and rsync package
3. Or fall back to SCP:
   ```bash
   ./scripts/deploy_windows.sh --host WIN_HOST --user WIN_USER
   # (without --use-rsync flag)
   ```

### Files Not Syncing to Windows Client

**Error:** Files appear to copy but don't show up on Windows

**Solutions:**
1. Check deployment directory exists:
   ```bash
   ssh user@windows-host "powershell -Command 'Test-Path C:\keyrx_deploy'"
   ```

2. Verify correct path format:
   ```bash
   # Use forward slashes in --deploy-dir
   --deploy-dir C:/keyrx_deploy
   ```

3. Check Windows permissions:
   ```bash
   ssh user@windows-host "powershell -Command 'Get-Acl C:\keyrx_deploy'"
   ```

## File Transfer Performance Comparison

| Method | First Transfer | Repeated Transfer | Setup Complexity |
|--------|---------------|-------------------|------------------|
| SCP    | ~30s (50MB)   | ~30s (50MB)       | Low (SSH only)   |
| Rsync  | ~30s (50MB)   | ~5s (changed only)| Medium (rsync on Windows) |
| SMB Share | ~10s (direct) | ~10s (direct)  | Medium (share setup) |
| HTTP   | ~15s (download) | ~15s            | Low (temporary)  |

**Recommendation:**
- **First-time setup or one-off:** Use SCP
- **Active development:** Use rsync (setup once, save time repeatedly)
- **Persistent shared workspace:** Use SMB share

## Security Considerations

### SSH Key Authentication

Always use SSH keys instead of passwords for automated deployments:

```bash
# Generate SSH key if you don't have one
ssh-keygen -t ed25519 -C "deployment-key"

# Copy to Windows client
ssh-copy-id user@windows-host

# Verify passwordless login
ssh user@windows-host "echo Success"
```

### Protecting Credentials

Never commit `.deploy.env` file (it's in `.gitignore`):

```bash
# Check it's ignored
git status .deploy.env
# Should show: nothing to commit
```

Use environment variables in CI/CD instead of files:

```bash
# In CI/CD environment
export WINDOWS_CLIENT_HOST=$CI_SECRET_WIN_HOST
export WINDOWS_CLIENT_USER=$CI_SECRET_WIN_USER
./scripts/deploy_windows.sh
```

### Firewall Configuration

Restrict SSH access to known IPs only:

```powershell
# On Windows, restrict SSH to specific IP
New-NetFirewallRule -Name sshd-restricted -DisplayName 'OpenSSH Server (Restricted)' `
    -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22 `
    -RemoteAddress 192.168.1.0/24
```

## Integration with Development Workflow

### Automated Testing After Deployment

Create a wrapper script for test automation:

```bash
#!/bin/bash
# deploy_and_test.sh

set -e

# Deploy
source .deploy.env
./scripts/deploy_windows.sh --use-rsync

# Run tests on Windows client
ssh $WINDOWS_CLIENT_USER@$WINDOWS_CLIENT_HOST \
    "cd $DEPLOY_DIR && keyrx_daemon.exe --test"

echo "Deployment and testing completed successfully"
```

### Git Hooks Integration

Run deployment after successful commits:

```bash
# .git/hooks/post-commit
#!/bin/bash

# Only deploy if on deployment branch
if [ "$(git branch --show-current)" = "deploy-dev" ]; then
    echo "Deploying to Windows client..."
    source .deploy.env
    ./scripts/deploy_windows.sh --use-rsync --quiet
fi
```

## References

- **Script documentation:** `scripts/CLAUDE.md`
- **Windows VM setup:** `vagrant/windows/README.md`
- **Development guide:** `.claude/CLAUDE.md`
- **OpenSSH for Windows:** https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_install_firstuse
