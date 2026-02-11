# Quick Connect to Windows VM

## TL;DR

```bash
# RDP (for GUI work, building, debugging)
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard

# Console (for REAL key remapping testing)
virt-manager
# → Double-click "keyrx2-win-test" VM

# WinRM (for automation, running tests)
cd vagrant/windows
vagrant winrm -c 'cd C:\vagrant_project; cargo test --features windows'
```

## Connection Methods at a Glance

| Method | Command | Input Testing | Use For |
|--------|---------|---------------|---------|
| **RDP** | `xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant` | ⚠️ Partial | Building, debugging, UI work |
| **Console** | `virt-manager` (GUI) | ✅ Full | **Key remapping testing** |
| **WinRM** | `vagrant winrm -c "command"` | ❌ CLI only | Automated tests, scripting |

## RDP Quick Commands

### Basic Connection
```bash
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080
```

### With Clipboard & Fullscreen
```bash
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /f +clipboard
```

### With Shared Folder
```bash
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant \
    /drive:share,/home/$(whoami)/shared_with_vm \
    +clipboard
```

### Using Remmina (GUI)
```bash
# Install
sudo apt install remmina remmina-plugin-rdp

# Connect
remmina -c rdp://vagrant:vagrant@localhost:13389
```

## Console Access (virt-manager)

### Install
```bash
sudo apt install virt-manager
```

### Launch
```bash
# GUI method
virt-manager

# Or connect directly via virsh
virsh console keyrx2-win-test
```

### In virt-manager GUI:
1. Double-click "keyrx2-win-test" VM
2. Console window opens → Physical keyboard input path ✅
3. Login: `vagrant` / `vagrant`

## Testing Checklist

Before claiming "key remapping works", test in **console mode**:

```bash
# 1. Open console
virt-manager
# Double-click VM

# 2. Login (vagrant/vagrant)

# 3. Build and run
cd C:\vagrant_project
cargo build --features windows
.\target\debug\keyrx_daemon.exe

# 4. Test remapping in console window
# Type keys directly in the virt-manager console
```

## Why Console for Testing?

**RDP keyboard input path:**
```
Your keyboard → FreeRDP client → Network → RDP server → Terminal Services
                                                         ↓
                                               ⚠️ May bypass RawInput/hooks!
```

**Console keyboard input path:**
```
Your keyboard → libvirt → QEMU → Windows VM → RawInput API
                                              ↓
                                         ✅ Full capture!
```

## Common Tasks

### Build in RDP (Fast & Comfortable)
```bash
# Connect via RDP
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080

# In RDP session: Open PowerShell
cd C:\vagrant_project
cargo build --features windows
```

### Test in Console (Accurate)
```bash
# Open console
virt-manager

# In console: Run tests
cargo test -p keyrx_daemon --features windows
```

### Automate via WinRM
```bash
cd vagrant/windows
vagrant winrm -c 'cd C:\vagrant_project; cargo build --features windows'
```

## Troubleshooting

### RDP Connection Refused
```bash
# Check VM is running
cd vagrant/windows
vagrant status

# Check port forwarding
vagrant port

# Restart RDP service in VM
vagrant winrm -c 'Restart-Service TermService -Force'
```

### virt-manager Can't Connect
```bash
# Check libvirt
systemctl status libvirtd

# List VMs
virsh list --all

# Start VM if stopped
virsh start keyrx2-win-test
```

### Slow Performance
```bash
# Increase VM resources (edit Vagrantfile)
libvirt.cpus = 8
libvirt.memory = 32768  # 32GB

# Reload VM
vagrant reload
```

## Performance Tips

### RDP Performance
```bash
# Lower color depth for faster RDP
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /bpp:16

# Disable desktop composition
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /compression -wallpaper -aero
```

### Console Performance
- Already optimized (SPICE + QXL video)
- Use Ctrl+Alt+F for fullscreen
- Disable desktop effects in Windows

## File Transfer

### Via RDP Share
```bash
# Share local folder with VM
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant \
    /drive:myshare,/home/user/share

# Access in VM: \\tsclient\myshare
```

### Via Vagrant Sync
```bash
# Sync changes from host → VM
cd vagrant/windows
vagrant rsync

# Files appear in: C:\vagrant_project
```

### Via SCP
```bash
# Copy file to VM (if SSH enabled)
scp file.exe vagrant@localhost:13389:C:/keyrx_deploy/
```

## Summary

**For productivity:** Use RDP
```bash
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard
```

**For accurate testing:** Use console
```bash
virt-manager  # → Double-click VM
```

**For automation:** Use WinRM
```bash
vagrant winrm -c "cargo test --features windows"
```
