# Quick Fix Guide - Get KeyRx Working Now!

## TL;DR

Your daemon is on port **9868**, but UI expects port **9867**.

**30-second fix**:
```powershell
# Right-click → Run as Administrator
.\FIX_PORT.ps1
```

## What We Found

### ✅ Good News
- Daemon works perfectly
- Default profile (24KB, 276 commands) activates in **2.1 seconds**
- Keyboard interception works
- No blocking I/O issues (already fixed in v0.1.3)

### ❌ The Problem
**Port mismatch**:
```
Daemon:  http://localhost:9868  (settings.json)
Web UI:  http://localhost:9867  (hardcoded)
```

## Step-by-Step Fix

### Step 1: Run FIX_PORT.ps1 (30 seconds)

Right-click `FIX_PORT.ps1` → **Run as Administrator**

Or from PowerShell:
```powershell
cd C:\Users\ryosu\repos\keyrx
.\FIX_PORT.ps1
```

This will:
1. ✅ Change daemon port to 9867
2. ✅ Restart daemon
3. ✅ Verify connection

### Step 2: Open Web UI

Open your browser to:
```
http://localhost:9867
```

Should now show "Connected" ✅

### Step 3: Test Profile Activation

1. Go to **Profiles** page
2. Click **Activate** on `default` profile
3. Should activate in ~2 seconds
4. Go to **Metrics** page
5. Type some keys
6. Should see events appearing

### Step 4: Verify Remapping Works

1. Go to **Config** page
2. Should show your active profile
3. Press keys that should be remapped
4. Should output remapped keys

## If Still Not Working

### Check Daemon is Running
```powershell
Get-Process -Name keyrx_daemon
```

Should show:
```
ProcessName  Id  CPU(s)
-----------  --  ------
keyrx_daemon XXXX X.XX
```

### Check Port is Listening
```powershell
Test-NetConnection -ComputerName localhost -Port 9867
```

Should show: `TcpTestSucceeded : True`

### Test API Directly
```powershell
curl http://localhost:9867/api/health
```

Should return:
```json
{"status":"ok","version":"0.1.5"}
```

### Check Daemon Logs
```powershell
.\DEBUG_ACTIVATION.ps1
```

Captures full logs to `DEBUG_yyyyMMdd_HHmmss.txt`

## Files You Now Have

### Quick Fixes
- **FIX_PORT.ps1** - Fix port mismatch (30 seconds)
- **TEST_ACTIVATION_9868.ps1** - Test current setup

### Diagnostics
- **DEBUG_ACTIVATION.ps1** - Full diagnostics
- **GATHER_LOGS.ps1** - Collect all logs
- **analyze_profiles.ps1** - Check profile complexity

### Documentation
- **BUGS_FOUND.md** - Complete bug report
- **ACTIVATION_BUG_REPORT.md** - Technical analysis
- **QUICK_FIX_GUIDE.md** - This file

### Tests
- **profile_activation_test.rs** - Comprehensive test suite

## Common Issues

### Issue: "Access denied" when running script
**Fix**: Right-click → Run as Administrator

### Issue: Daemon not starting
**Check**:
```powershell
Get-EventLog -LogName Application -Source keyrx* -Newest 5
```

### Issue: Port already in use
**Fix**:
```powershell
# Kill all daemon processes
taskkill /F /IM keyrx_daemon.exe

# Restart
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"
```

### Issue: Web UI shows "Disconnected" after fix
**Check browser console** (F12):
- Should show API requests to `http://localhost:9867`
- If still going to `http://localhost:9868`, clear browser cache

## Performance Notes

Your `default.rhai` profile:
- **Size**: 24,411 bytes
- **Lines**: 450
- **Commands**: 276 (tap_hold, map, when_start)
- **Activation time**: 2.1 seconds ✅

This is **perfectly fine**! No performance issues.

## What's Next?

After fixing the port:
1. ✅ Web UI will connect
2. ✅ Profile activation will work
3. ✅ Keyboard remapping will work
4. ✅ Metrics will show events

Then you can:
- Create new profiles
- Modify existing profiles
- Test different configurations
- Use all features normally

## Support

If you hit any issues:
1. Run `.\DEBUG_ACTIVATION.ps1`
2. Check `DEBUG_*.txt` log file
3. Look for errors in `daemon_stderr_*.log`
4. Share the error messages for help

---

**Status**: Ready to fix!
**Time**: 30 seconds
**Risk**: None (just changing port number)

**Run**: `.\FIX_PORT.ps1` as Administrator
