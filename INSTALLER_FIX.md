# Installer Fix - No Admin Required

## 🔧 Issue Fixed

**Original Problem:**
- Installer required administrator privileges
- Failed with "Access Denied" error (code 5) when run without admin
- Error: "RegCreateKeyEx failed" when trying to modify system PATH

**Solution:**
- Changed installer to work **without admin privileges**
- Supports both admin and non-admin installation
- PATH is added to system-wide if admin, or user-level if not admin

## ✅ What Changed

### Before (Required Admin)
```
PrivilegesRequired=admin
Root: HKLM (HKEY_LOCAL_MACHINE)
```

### After (Works Without Admin)
```
PrivilegesRequired=lowest
Root: HKA (auto - HKLM if admin, HKCU if not)
Root: HKCU fallback for PATH (user-level)
```

## 🚀 How to Install

### Option 1: With Admin (Recommended)

**Right-click installer → "Run as administrator"**

Benefits:
- ✅ Installs to `C:\Program Files\KeyRx`
- ✅ Adds to system-wide PATH (all users)
- ✅ Creates system-wide registry entries

### Option 2: Without Admin (Works Now!)

**Just double-click the installer**

Benefits:
- ✅ Installs to `C:\Users\<username>\AppData\Local\Programs\KeyRx`
- ✅ Adds to user PATH (current user only)
- ✅ Creates user registry entries
- ✅ No UAC prompt required

## 📦 New Installer

The fixed installer has been rebuilt:

```
target\windows-installer\keyrx_0.1.0.0_x64_setup.exe
```

**Size:** 8.9 MB
**Build date:** Just now

## 🎯 Test the Fix

### Test 1: Install Without Admin

```powershell
# Just run it (no admin)
.\target\windows-installer\keyrx_0.1.0.0_x64_setup.exe
```

Should install successfully to:
- `%LOCALAPPDATA%\Programs\KeyRx`

### Test 2: Install With Admin (Recommended)

```powershell
# Right-click → Run as administrator
# Or via PowerShell:
Start-Process .\target\windows-installer\keyrx_0.1.0.0_x64_setup.exe -Verb RunAs
```

Should install to:
- `C:\Program Files\KeyRx`

### Test 3: Verify Installation

```powershell
# Check if in PATH
where.exe keyrx_daemon

# Run daemon
keyrx_daemon --version
```

## 📊 Comparison

| Feature | Admin Install | Non-Admin Install |
|---------|--------------|-------------------|
| **Location** | `C:\Program Files\KeyRx` | `%LOCALAPPDATA%\Programs\KeyRx` |
| **PATH** | System-wide (all users) | Current user only |
| **Registry** | HKLM (system) | HKCU (user) |
| **UAC Prompt** | Yes | No |
| **Recommended** | ✅ Yes | For testing/portable |

## 🔍 Technical Details

### Registry Changes

**With Admin:**
```
HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment
  Path = %PATH%;C:\Program Files\KeyRx

HKLM\Software\KeyRx
  InstallPath = C:\Program Files\KeyRx
  Version = 0.1.0.0
```

**Without Admin:**
```
HKCU\Environment
  Path = %PATH%;%LOCALAPPDATA%\Programs\KeyRx

HKCU\Software\KeyRx
  InstallPath = %LOCALAPPDATA%\Programs\KeyRx
  Version = 0.1.0.0
```

### Code Changes in keyrx-installer.iss

1. **Privilege Level:**
   ```pascal
   PrivilegesRequired=lowest  // Was: admin
   ```

2. **Registry Root (Auto):**
   ```pascal
   Root: HKA  // Was: HKLM
   // HKA = HKLM if admin, HKCU if not
   ```

3. **PATH Fallback:**
   ```pascal
   ; System PATH (if admin)
   Root: HKA; Subkey: "SYSTEM\...\Environment"; ValueName: "Path"

   ; User PATH (if not admin)
   Root: HKCU; Subkey: "Environment"; ValueName: "Path"
   Check: not IsAdmin
   ```

4. **Smart PATH Check:**
   ```pascal
   function NeedsAddPath(Param: string): boolean;
   begin
     // Check HKLM if admin, HKCU if not
     if IsAdmin then
       // Check system PATH
     else
       // Check user PATH
   end;
   ```

## ✨ Benefits of This Approach

1. **No More Errors**
   - ✅ No "Access Denied" errors
   - ✅ Works on restricted machines
   - ✅ Works in corporate environments

2. **Flexible Installation**
   - ✅ User choice: admin or non-admin
   - ✅ Portable installation option
   - ✅ No forced UAC prompt

3. **Best of Both Worlds**
   - ✅ System-wide if admin privileges available
   - ✅ User-level fallback if not
   - ✅ Always succeeds

## 🎯 Recommendations

### For End Users
**Always run as administrator for best experience:**
- Right-click → "Run as administrator"
- Or accept UAC prompt if it appears

### For Developers/Testing
**Non-admin install works great for:**
- Quick testing
- Development machines
- CI/CD environments
- Portable installations

### For Deployment
**Include in documentation:**
```markdown
## Installation

### Windows

Download `keyrx_0.1.0.0_x64_setup.exe` from Releases.

**Recommended:** Right-click → "Run as administrator"

*Note: Can also install without admin rights (user-level PATH only)*
```

## 🐛 If You Still Have Issues

### Issue: "Access Denied" on uninstall

**Solution:** Uninstall with same privileges as install
```powershell
# If installed with admin, uninstall with admin
Start-Process "C:\Program Files\KeyRx\unins000.exe" -Verb RunAs
```

### Issue: Not in PATH after install

**Solution:** Restart terminal
```powershell
# Reload environment
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### Issue: Want to change from user to system install

**Solution:**
1. Uninstall current version
2. Re-install with admin privileges

## 📝 Summary

| Item | Status |
|------|--------|
| **Admin Required** | ❌ No (but recommended) |
| **Works Without Admin** | ✅ Yes |
| **System-Wide PATH** | ✅ With admin |
| **User-Level PATH** | ✅ Without admin |
| **UAC Prompt** | Optional |
| **Flexible Installation** | ✅ Yes |

**The installer is now much more user-friendly and works in all scenarios!**

---

## 🚀 Ready to Use

The fixed installer is here:
```
target\windows-installer\keyrx_0.1.0.0_x64_setup.exe
```

Try it now:
```powershell
# Test without admin
.\target\windows-installer\keyrx_0.1.0.0_x64_setup.exe
```

✅ **No more "Access Denied" errors!**
