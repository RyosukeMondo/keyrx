# Installing KeyRx v0.1.1

## What's New in v0.1.1

### Auto-Loading Default Profile
- ✅ Daemon automatically creates and loads default profile on first boot
- ✅ Remembers last activated profile between reboots
- ✅ No chicken-and-egg problem - works out of the box
- ✅ Web UI and daemon use identical paths (no configuration mismatch)

### User Benefits
- No `--config` flag needed for normal usage
- Zero-config first boot experience
- Active profile persisted to `%APPDATA%\keyrx\.active`
- Seamless integration between Web UI and daemon

## Installation Steps

### 1. Locate the Installer
```
Path: target\windows-installer\keyrx_0.1.1.0_x64_setup.exe
Size: ~9 MB
```

### 2. Run the Installer

**Important:** Right-click → **Run as administrator**

The installer will:
1. ✅ Detect old version (v0.1.0) if installed
2. ✅ Uninstall it automatically
3. ✅ Install new version (v0.1.1)
4. ✅ Create Start Menu shortcuts
5. ✅ Add to PATH (system-wide if admin, user-level otherwise)

### 3. Start the Daemon

After installation:

**Option A: Start Menu (Easiest)**
1. Press Windows key
2. Search "KeyRx Daemon"
3. Right-click → Run as administrator
4. Accept UAC prompt

**Option B: PowerShell**
```powershell
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
```

**Option C: Debug Mode (Recommended for First Boot)**
```powershell
.\scripts\windows\Debug-Launch.ps1
```

### 4. First Boot Behavior

On first boot with no config:

```
[INFO] No active profile found. Creating default profile...
[INFO] Creating default profile with blank template...
[INFO] Using default profile at: C:\Users\...\keyrx\profiles\default.krx
[INFO] Loaded 0 key mappings from profile 'default'
```

**What happened:**
- ✅ Created `%APPDATA%\keyrx\profiles\default.rhai`
- ✅ Compiled to `default.krx`
- ✅ Activated as default profile
- ✅ Future boots will load this automatically

### 5. Import Your Configuration

**Option A: Via Script**
```powershell
.\scripts\windows\Import-Example-Config.ps1
```

**Option B: Via Web UI**
1. Open http://localhost:9867
2. Go to Profiles page
3. Click on "default" profile path
4. Copy your config content
5. Paste and save

### 6. Activate and Test

1. Go to http://localhost:9867/profiles
2. Click "Activate" on your profile
3. Wait for green notification
4. Go to /metrics page
5. Press keys → Should see remapped output!

## Upgrade from v0.1.0

### What Happens During Upgrade

1. **Installer detects v0.1.0**
   - Shows message: "KeyRx daemon is currently running. Setup will now close it. Continue?"
   - Stops daemon automatically
   - Uninstalls v0.1.0
   - Installs v0.1.1

2. **Your Configuration**
   - ✅ Preserved in `%APPDATA%\keyrx\profiles\`
   - ✅ Active profile setting preserved
   - ✅ Settings.json preserved

3. **After Upgrade**
   - Start daemon (with debug): `.\scripts\windows\Debug-Launch.ps1`
   - Should load your previously active profile automatically
   - No need to reconfigure anything!

### Verification After Upgrade

```powershell
# 1. Check version
keyrx_daemon.exe --version
# Should show: keyrx_daemon 0.1.1

# 2. Check active profile
Get-Content "$env:APPDATA\keyrx\.active"
# Shows your active profile name

# 3. Start daemon and verify
.\scripts\windows\Debug-Launch.ps1

# 4. Check logs for version
Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "version"

# 5. Check Web UI
Start-Process http://localhost:9867
# Should show your profiles, active one has green checkmark
```

## Troubleshooting

### "Installer won't detect old version"

The installer uses these methods to detect old installation:
- PID file check: `%APPDATA%\keyrx\daemon.pid`
- Process check: `Get-Process keyrx_daemon`
- Registry check: `HKLM\Software\KeyRx` or `HKCU\Software\KeyRx`

**Manual cleanup if needed:**
```powershell
# 1. Stop daemon
Stop-Process -Name keyrx_daemon -Force

# 2. Remove PID file
Remove-Item "$env:APPDATA\keyrx\daemon.pid" -ErrorAction SilentlyContinue

# 3. Run installer
# Right-click → Run as administrator
```

### "Daemon doesn't load my profile after upgrade"

```powershell
# Check active profile
Get-Content "$env:APPDATA\keyrx\.active"

# If empty or wrong, activate via Web UI:
Start-Process http://localhost:9867/profiles
# Click "Activate" on your profile
```

### "No profiles found after upgrade"

Your profiles should be preserved:
```powershell
# List profiles
Get-ChildItem "$env:APPDATA\keyrx\profiles\"

# If empty, import example:
.\scripts\windows\Import-Example-Config.ps1
```

## Rollback to v0.1.0 (if needed)

If you need to rollback:

1. **Uninstall v0.1.1**
   - Start Menu → KeyRx → Uninstall
   - Or: Control Panel → Programs → KeyRx → Uninstall

2. **Reinstall v0.1.0**
   - Locate old installer: `target\windows-installer\keyrx_0.1.0.0_x64_setup.exe`
   - Right-click → Run as administrator

3. **Note:** v0.1.0 doesn't auto-load profiles, you'll need to use `--config` flag

## Key Differences: v0.1.0 vs v0.1.1

| Feature | v0.1.0 | v0.1.1 |
|---------|--------|--------|
| **Daemon startup** | Requires `--config` | Auto-loads active profile |
| **First boot** | "File not found" error | Auto-creates default profile |
| **Profile memory** | Not saved | Persists to `.active` file |
| **Web UI integration** | Works | Works (identical paths) |
| **Config path** | Manual | Automatic |

## Files and Locations

| File | Location |
|------|----------|
| **Installer** | `target\windows-installer\keyrx_0.1.1.0_x64_setup.exe` |
| **Daemon binary** | `C:\Program Files\KeyRx\keyrx_daemon.exe` (if admin install) |
| **Daemon binary** | `%LOCALAPPDATA%\Programs\KeyRx\keyrx_daemon.exe` (non-admin) |
| **Profiles** | `%APPDATA%\keyrx\profiles\` |
| **Active marker** | `%APPDATA%\keyrx\.active` |
| **Settings** | `%APPDATA%\keyrx\settings.json` |
| **Debug logs** | `%TEMP%\keyrx-debug.log` |

## Post-Installation Checklist

- [ ] Installer completed without errors
- [ ] `keyrx_daemon.exe --version` shows 0.1.1
- [ ] Daemon starts (run as admin)
- [ ] Default profile auto-created on first boot
- [ ] Web UI accessible at http://localhost:9867
- [ ] Profiles page shows default profile
- [ ] Import example config (optional)
- [ ] Activate profile works
- [ ] Metrics page shows key events
- [ ] Daemon remembers active profile on restart

## Support

- **Documentation**: `DAEMON_DEFAULT_BEHAVIOR.md`
- **Troubleshooting**: `TROUBLESHOOTING.md`
- **Admin Requirements**: `WINDOWS_ADMIN_REQUIRED.md`
- **How to Run**: `HOW_TO_RUN.md`

---

**Summary:**
- v0.1.1 installer: `keyrx_0.1.1.0_x64_setup.exe`
- Right-click → Run as administrator
- Installer detects and uninstalls v0.1.0 automatically
- Your profiles and settings are preserved
- Daemon now auto-loads default profile on first boot
