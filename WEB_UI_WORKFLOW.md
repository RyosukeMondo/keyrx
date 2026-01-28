# KeyRx Web UI Workflow Guide

## ✅ Yes, Web UI Integration Is Designed

The daemon and Web UI communicate through REST API:
- Profile creation, activation, editing, deletion
- Automatic compilation (.rhai → .krx)
- Daemon restarts automatically when profile is activated

## 📋 How to Import Your Example Config

You have `examples\user_layout.rhai` and want to use it through the Web UI.

### Option 1: Copy File Manually (Recommended)

```powershell
# 1. Create profiles directory if it doesn't exist
New-Item -ItemType Directory -Force "$env:APPDATA\keyrx\profiles"

# 2. Copy your example config
Copy-Item examples\user_layout.rhai "$env:APPDATA\keyrx\profiles\user_layout.rhai"

# 3. Open Web UI
Start-Process http://localhost:9867

# 4. In Web UI:
#    - Click "Profiles" in sidebar
#    - You should see "user_layout" profile
#    - Click "Activate" button
#    - Daemon will compile and load it automatically!
```

### Option 2: Use Web UI to Create, Then Copy Content

```powershell
# 1. Open Web UI: http://localhost:9867

# 2. Click "Create Profile"
#    - Name: user_layout
#    - Template: Blank
#    - Click "Create"

# 3. Copy your config content
Get-Content examples\user_layout.rhai | Set-Clipboard

# 4. In Web UI:
#    - Click "Edit" on user_layout profile
#    - Paste your config
#    - Save

# 5. Click "Activate"
```

## 🎯 Web UI Profile Management

### Create New Profile
1. Click **"Profiles"** in sidebar
2. Click **"Create Profile"** button
3. Enter name and select template
4. Click **"Create"**

### Activate Profile
1. Click **"Activate"** on any inactive profile
2. Daemon automatically:
   - Compiles .rhai → .krx
   - Loads the config
   - Restarts to apply changes
3. Green checkmark shows active profile

### Edit Profile
1. Click **"Edit"** button on profile card
2. Modify configuration in editor
3. Save changes
4. Click **"Activate"** to apply

### Delete Profile
1. Click **"Delete"** button
2. Confirm deletion
3. Cannot delete active profile

## 🔍 Verify It's Working

After activating your profile:

### 1. Check Metrics Page
```
http://localhost:9867/metrics
```
Press keys - you should see remapped output.

### 2. Check Debug Logs
```powershell
Get-Content "$env:TEMP\keyrx-debug.log" -Tail 20
```
Should show:
```
[INFO] Loaded X key mappings from profile 'user_layout'
[DEBUG] KeyEvent { ... }
```

## 🐛 If No Remapping Happens

### Check 1: Profile is Active
Web UI should show green checkmark on your profile.

### Check 2: Config Loaded
```powershell
Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "Loaded.*mappings"
```
Should show: `Loaded 256 key mappings` (not 0).

### Check 3: File Exists
```powershell
Test-Path "$env:APPDATA\keyrx\profiles\user_layout.krx"
```
Should return `True`.

### Check 4: Daemon Restarted
After activation, daemon should restart. Check process start time:
```powershell
Get-Process keyrx_daemon | Select-Object StartTime
```

## 📁 File Locations

| File | Location |
|------|----------|
| Source (.rhai) | `%APPDATA%\keyrx\profiles\{name}.rhai` |
| Compiled (.krx) | `%APPDATA%\keyrx\profiles\{name}.krx` |
| Settings | `%APPDATA%\keyrx\settings.json` |
| Debug Logs | `%TEMP%\keyrx-debug.log` |

Typical path: `C:\Users\{username}\AppData\Roaming\keyrx\profiles\`

## 🚀 Quick Start Workflow

```powershell
# 1. Make sure daemon is running with debug
.\scripts\windows\Debug-Launch.ps1

# 2. Import your example config
Copy-Item examples\user_layout.rhai "$env:APPDATA\keyrx\profiles\user_layout.rhai"

# 3. Open Web UI
Start-Process http://localhost:9867

# 4. Activate profile in Web UI
# - Go to Profiles page
# - Click "Activate" on user_layout
# - Wait for green notification "Profile 'user_layout' applied!"

# 5. Test remapping
# - Go to Metrics page
# - Press keys
# - Should see remapped output
```

## 💡 Pro Tips

1. **Always check logs first** - `Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait`
2. **Use Web UI for activation** - It handles compilation and restart automatically
3. **Don't use CLI for normal usage** - Web UI is the primary interface
4. **Active profile = Running profile** - Green checkmark means daemon is using it
5. **Metrics page is your friend** - Real-time verification of remapping

## 🔧 Troubleshooting

### "Profile not found"
- Check file exists: `Test-Path "$env:APPDATA\keyrx\profiles\{name}.rhai"`
- Refresh Web UI (F5)

### "Compilation failed"
- Check debug logs for syntax errors
- Use Web UI editor with syntax highlighting

### "Activation succeeded but no remapping"
- Verify daemon restarted: `Get-Process keyrx_daemon | Select StartTime`
- Check metrics page for key events
- Look for compilation errors in logs

---

**Summary:**
- ✅ Web UI integration IS designed
- ✅ Copy .rhai to `%APPDATA%\keyrx\profiles\`
- ✅ Activate through Web UI
- ✅ No command line needed for normal usage
