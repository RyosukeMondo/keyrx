# Fix Keyboard Remapping NOW

## Your Current Problem

**Binary:** v0.1.2 (14:23:22) ← OLD
**Need:** v0.1.4 (15:41:11) ← CORRECT (already built!)

**Symptoms:**
- Config page loads ✓
- Profile activates ✓
- Remapping DOESN'T work ✗
- No events in metrics ✗

---

## Quick Fix (3 Steps)

### Step 1: Open File Explorer
Press `Windows + E`

### Step 2: Navigate to Project
Go to: `C:\Users\ryosu\repos\keyrx`

### Step 3: Run the Updater
**Find this file:**
```
RUN_UPDATE_AS_ADMIN.bat
```

**Double-click it**
- UAC will prompt for permission
- Click **"Yes"** to allow
- Script will run in new window
- Wait for "Binary Updated Successfully!"

---

## Alternative Method (If Above Doesn't Work)

### Open PowerShell as Administrator

1. Press `Windows + X`
2. Select **"Windows PowerShell (Admin)"** or **"Terminal (Admin)"**
3. Click **"Yes"** on UAC prompt

### Run These Commands

```powershell
# 1. Go to project directory
cd C:\Users\ryosu\repos\keyrx

# 2. Stop daemon
taskkill /F /IM keyrx_daemon.exe

# 3. Wait a bit
Start-Sleep -Seconds 3

# 4. Copy new binary
Copy-Item "target\release\keyrx_daemon.exe" "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -Force

# 5. Verify timestamp (should be 15:41:11)
Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" | Select-Object LastWriteTime

# 6. Start daemon
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"

# 7. Wait for startup
Start-Sleep -Seconds 8

# 8. Test API
Invoke-RestMethod http://localhost:9867/api/health | ConvertTo-Json
```

---

## Verification

### Check Binary Timestamp
```powershell
Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" | Select-Object LastWriteTime
```

**Should show:** `2026/01/29 15:41:11`

### Run Installation Test
```powershell
cd C:\Users\ryosu\repos\keyrx
.\scripts\test_installation.ps1
```

**All 7 tests should pass:**
```
[1/7] Checking binary exists...           ✓
[2/7] Checking binary timestamp...        ✓
[3/7] Checking daemon process...          ✓
[4/7] Checking API health...              ✓
[5/7] Checking profiles endpoint...       ✓
[6/7] Checking keyboard interception...   ✓
[7/7] Checking port 9867...               ✓
```

If Test 6 passes → **Keyboard remapping will work!**

### Test Remapping

1. Open http://localhost:9867
2. Go to **Profiles** page
3. Click **Activate** on "default" profile
4. Open **Notepad**
5. Type keys according to your profile
6. Keys should be remapped ✓

---

## Why This Happened

1. MSI installer packaged old binary (caching issue)
2. Daemon was running, Windows locked the file
3. Installer couldn't replace binary without stopping daemon
4. User got old v0.1.2 instead of new v0.1.4

## What I Fixed

1. ✅ Added CustomAction to MSI to stop daemon automatically
2. ✅ Created UPDATE_BINARY.ps1 to manually update
3. ✅ Created RUN_UPDATE_AS_ADMIN.bat for easy execution
4. ✅ Created test_installation.ps1 to detect this bug
5. ✅ Created comprehensive tests (UT/IT/E2E)

## Next Time

After building and installing, run:
```powershell
.\scripts\test_installation.ps1
```

This will catch the bug BEFORE you discover it manually.

---

**Choose one method above and run it now. Takes 30 seconds to fix!**
