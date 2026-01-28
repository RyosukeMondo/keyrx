# ✅ Installation Successful! How to Run KeyRx

## 🎉 Good News

Installation succeeded! KeyRx is now installed on your system.

## 🔑 Important: Running the Daemon Requires Admin

The daemon needs administrator privileges to access keyboard input (like needing root on Linux).

## 🚀 How to Run KeyRx Daemon

### Method 1: Start Menu (Easiest)

1. Press **Windows key**
2. Search for "**KeyRx Daemon**"
3. **Right-click** → **Run as administrator**
4. Accept the UAC prompt ✅

### Method 2: PowerShell

```powershell
# Find where it's installed
where.exe keyrx_daemon

# Run with admin elevation
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
```

### Method 3: Create "Always Run as Admin" Shortcut

1. Find **Start Menu → KeyRx → KeyRx Daemon**
2. **Right-click** → **Properties**
3. Click **Advanced** button
4. Check ☑ **Run as administrator**
5. Click **OK**, **OK**

Now you can just double-click and it will automatically ask for admin!

## 📊 After Starting

Once the daemon is running:

1. **Open Web UI**: http://localhost:9867
   - Or use Start Menu → **KeyRx Web UI**

2. **Check Status**:
   ```powershell
   Get-Process keyrx_daemon
   ```

3. **Stop Daemon**:
   ```powershell
   Stop-Process -Name keyrx_daemon
   ```

## ⚠️ About the Installation Error You Saw

**Error at end of install:** "CreateProcess failed; code 740"

**What happened:**
- ✅ Installation succeeded
- ❌ Auto-launch failed (needs admin)

**Why:** The installer can install without admin (to your user folder), but the daemon needs admin to run.

**Solution:** Just run the daemon with admin rights using one of the methods above.

## 🎯 Quick Start Workflow

```powershell
# 1. Run daemon as admin
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs

# 2. Open Web UI in browser
Start-Process http://localhost:9867

# 3. When done, stop daemon
Stop-Process -Name keyrx_daemon
```

## 📍 Where Is KeyRx Installed?

Check where it was installed:

```powershell
where.exe keyrx_daemon
```

Likely locations:
- **With admin install:** `C:\Program Files\KeyRx\keyrx_daemon.exe`
- **Without admin install:** `%LOCALAPPDATA%\Programs\KeyRx\keyrx_daemon.exe`

## 🔧 Optional: Auto-Start on Login

If you want KeyRx to start automatically when you log in:

```powershell
# Create scheduled task (run as admin)
$action = New-ScheduledTaskAction -Execute "keyrx_daemon.exe" -Argument "run"
$trigger = New-ScheduledTaskTrigger -AtLogOn
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Highest

Register-ScheduledTask -TaskName "KeyRx Daemon" -Action $action -Trigger $trigger -Principal $principal -Description "Start KeyRx on login"
```

To remove auto-start:
```powershell
Unregister-ScheduledTask -TaskName "KeyRx Daemon" -Confirm:$false
```

## 🐛 Troubleshooting

### "keyrx_daemon not found"

**Restart your terminal** to reload PATH:
```powershell
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### "Web UI not loading"

1. Make sure daemon is running:
   ```powershell
   Get-Process keyrx_daemon
   ```

2. If not running, start it as admin:
   ```powershell
   Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
   ```

3. Try opening UI:
   ```powershell
   Start-Process http://localhost:9867
   ```

### "Still getting code 740 error"

You're trying to run without admin. Always use one of these:
- Right-click → Run as administrator
- Use `Start-Process` with `-Verb RunAs`
- Set shortcut to "Always run as admin"

## 📚 More Info

- `WINDOWS_ADMIN_REQUIRED.md` - Detailed admin requirements
- `INSTALLER_FIX.md` - Why installation doesn't need admin
- `BUILD_COMPLETE.md` - Full installation guide

---

## 🎯 Summary

| Action | Admin Needed? | How |
|--------|--------------|-----|
| **Install** | ❌ No | Just run installer |
| **Run Daemon** | ✅ **YES** | Right-click → Run as admin |
| **Use Web UI** | ❌ No | Just open browser |
| **Compile configs** | ❌ No | `keyrx_compiler.exe` |

**Next step:** Run the daemon as admin, then open http://localhost:9867 🚀
