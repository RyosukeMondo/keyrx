# KeyRx Windows Installer - Complete Guide

This guide covers **three different approaches** to create a self-contained Windows installer for KeyRx, using **only built-in Windows tools**.

## 🎯 Quick Start (Recommended)

### Option 1: IExpress Installer (Best for Distribution)

```powershell
# Build everything and create installer
.\scripts\installer\build-installer.ps1

# Output: keyrx-installer-v0.1.5.exe (self-extracting)
```

**Result:** Professional Windows installer that extracts and runs automatically.

### Option 2: PowerShell Script Installer (Simplest)

```powershell
# Create self-contained PowerShell script
.\scripts\installer\create-simple-installer.ps1

# Output: keyrx-installer-v0.1.5.ps1 (all files embedded)
```

**Result:** Single PowerShell script with everything embedded as Base64.

## 📦 Three Installer Approaches

### Approach 1: IExpress (Recommended)

**What it is:** Built-in Windows tool that creates self-extracting executables.

**Pros:**
- Professional Windows installer experience
- Creates .exe file (familiar to users)
- Built into every Windows installation
- Can display license and custom prompts
- Supports silent installation

**Cons:**
- Requires building on Windows
- Slightly more complex setup

**How to build:**

```powershell
# Full build (binaries + UI + installer)
.\scripts\installer\build-installer.ps1

# Skip builds (use existing binaries)
.\scripts\installer\build-installer.ps1 -SkipBuild -SkipUI
```

**Output:**
- `scripts/installer/keyrx-installer-v0.1.5.exe`

**How users install:**

```powershell
# Interactive installation
.\keyrx-installer-v0.1.5.exe

# Silent installation
.\keyrx-installer-v0.1.5.exe /Q
```

**What it does:**
1. Extracts all files to temp directory
2. Runs `install.ps1` PowerShell script
3. Copies files to `C:\Program Files\KeyRx`
4. Adds to system PATH
5. Creates shortcuts
6. Registers in Windows Programs list

### Approach 2: Self-Contained PowerShell Script

**What it is:** Single .ps1 file with all binaries embedded as Base64.

**Pros:**
- Simplest approach
- Single file contains everything
- Easy to modify and customize
- Can be easily inspected by users

**Cons:**
- Larger file size (Base64 encoding increases size by ~33%)
- Users must run PowerShell
- Less professional appearance

**How to build:**

```powershell
# Create self-contained script
.\scripts\installer\create-simple-installer.ps1

# Skip build (use existing binaries)
.\scripts\installer\create-simple-installer.ps1 -SkipBuild
```

**Output:**
- `scripts/installer/keyrx-installer-v0.1.5.ps1`

**How users install:**

```powershell
# Run as administrator
powershell.exe -ExecutionPolicy Bypass -File keyrx-installer-v0.1.5.ps1

# Or right-click → "Run with PowerShell" (as Administrator)
```

**What it does:**
1. Decodes Base64 embedded binaries
2. Writes binaries to installation directory
3. Adds to system PATH
4. Creates shortcuts
5. Creates uninstaller

### Approach 3: Manual IExpress (For Customization)

**What it is:** Using IExpress GUI to create custom installer.

**Pros:**
- Full control over installer behavior
- Visual interface for configuration
- No scripting required

**Cons:**
- Manual process (not scriptable)
- Tedious for repeated builds

**How to build:**

1. Run `iexpress.exe` from Start Menu
2. Choose "Create new Self Extraction Directive file"
3. Select "Install program"
4. Add files: `install.ps1`, binaries, UI files
5. Set installation command: `powershell.exe -ExecutionPolicy Bypass -File install.ps1`
6. Configure prompts and messages
7. Save SED file for reuse
8. Build the package

## 🔧 Installation Script Features

All approaches use the same `install.ps1` script with these features:

### Standard Installation

- Checks for administrator privileges
- Creates installation directory (`C:\Program Files\KeyRx`)
- Copies all executables and files
- Adds installation directory to system PATH
- Creates desktop shortcut
- Creates Start Menu shortcut
- Registers in Windows Programs list
- Creates uninstaller script

### Uninstallation

Users can uninstall via:

- **Windows Settings:** Settings → Apps → KeyRx → Uninstall
- **Control Panel:** Programs and Features → KeyRx → Uninstall
- **Manual:** Run `uninstall.ps1` from installation directory

Uninstaller:
- Stops running daemon
- Removes from system PATH
- Deletes shortcuts
- Removes registry entries
- Deletes installation directory

## 📋 Prerequisites

### For Building Installers

- Windows 10 or 11
- PowerShell 5.1+ (included with Windows)
- Rust toolchain (for building binaries)
- Node.js 18+ (for building UI)
- IExpress (included with Windows)

### For Running Installers

- Windows 10 or 11
- Administrator privileges
- **No other requirements!**

## 🎨 Customization

### Change Installation Path

Edit `install.ps1` before building:

```powershell
param(
    [string]$InstallPath = "C:\CustomPath\KeyRx",  # Change default here
    [switch]$Silent
)
```

### Add Custom Files

Add files to the installer by placing them in the temporary directory before running IExpress:

```powershell
# In build-installer.ps1, after copying files:
Copy-Item "path\to\custom\file.txt" $TempDir
```

### Customize Messages

Edit the SED file or the IExpress configuration:

```ini
[Strings]
InstallPrompt=Custom installation prompt here
FinishMessage=Custom completion message here
```

### Add License Display

Place `LICENSE.txt` in the temp directory and configure IExpress to display it.

## 🚀 Distribution

### What to Distribute

**Option 1 (IExpress):**
- Distribute `keyrx-installer-v0.1.5.exe`
- Single file, ~5-10 MB
- Users just run the .exe

**Option 2 (PowerShell):**
- Distribute `keyrx-installer-v0.1.5.ps1`
- Single file, ~5-10 MB
- Users run with PowerShell (as Administrator)

### Where to Distribute

- GitHub Releases
- Company website
- Direct download links
- USB drives
- Network shares

### Security Considerations

**Code Signing (Optional but Recommended):**

For professional distribution, sign the installer:

```powershell
# Get a code signing certificate from a trusted CA
# Then sign the executable:
signtool sign /f mycert.pfx /p password /t http://timestamp.digicert.com keyrx-installer-v0.1.5.exe
```

Benefits:
- Windows SmartScreen won't block it
- Users see verified publisher
- Professional appearance

**Without Code Signing:**
- Windows SmartScreen may show warning
- Users need to click "More info" → "Run anyway"
- Still works, just less professional

## 🧪 Testing

### Test Installation

```powershell
# Run installer
.\keyrx-installer-v0.1.5.exe

# Verify installation
keyrx_daemon --version
keyrx_compiler --help

# Check shortcuts
ls "$env:USERPROFILE\Desktop\KeyRx.lnk"
ls "$env:PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\KeyRx\KeyRx.lnk"

# Check registry
Get-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\KeyRx"
```

### Test Uninstallation

```powershell
# Run uninstaller
& "$env:ProgramFiles\KeyRx\uninstall.ps1"

# Verify removal
Test-Path "$env:ProgramFiles\KeyRx"  # Should be False
keyrx_daemon --version  # Should fail (not in PATH)
```

### Test Silent Installation

```powershell
# Silent install
.\keyrx-installer-v0.1.5.exe /Q

# Silent uninstall
powershell.exe -ExecutionPolicy Bypass -File "$env:ProgramFiles\KeyRx\uninstall.ps1" -Silent
```

## 🐛 Troubleshooting

### "IExpress not found"

- Check `C:\Windows\System32\iexpress.exe`
- If missing, Windows installation may be incomplete
- Try running `where iexpress` to locate it

### "Access Denied"

- Run installer as Administrator
- Right-click → "Run as administrator"

### "Execution Policy" Error

```powershell
# Temporarily bypass execution policy
powershell.exe -ExecutionPolicy Bypass -File installer.ps1
```

### Installer Created But Fails to Run

- Check IExpress output for errors
- Verify all source files exist
- Check SED file syntax
- Try running in IExpress GUI for better error messages

### Files Not Extracted

- Check temp directory permissions
- Verify Base64 encoding is correct
- Check for antivirus interference

## 📊 File Sizes

Approximate sizes for v0.1.5:

| Component | Size |
|-----------|------|
| `keyrx_daemon.exe` | ~2 MB |
| `keyrx_compiler.exe` | ~1.5 MB |
| `keyrx_ui/dist` | ~1 MB |
| **Total (IExpress)** | **~5 MB** |
| **Total (PowerShell)** | **~7 MB** (Base64 overhead) |

## 🔐 Security Best Practices

### For Developers

1. **Code Signing:** Sign the installer with a trusted certificate
2. **Virus Scanning:** Scan the installer before distribution
3. **Checksums:** Provide SHA256 checksums for verification
4. **HTTPS:** Distribute via HTTPS only
5. **Verify Sources:** Build from verified source code only

### For Users

1. **Download from Official Sources:** Only download from official GitHub or website
2. **Verify Checksums:** Compare SHA256 hash with published value
3. **Check Code Signature:** Verify publisher is "KeyRx Contributors"
4. **Scan with Antivirus:** Scan installer before running
5. **Run as Administrator:** Required for system-level installation

## 📚 Additional Resources

- [IExpress Documentation](https://docs.microsoft.com/en-us/windows-hardware/drivers/devtest/iexpress)
- [PowerShell Installation Best Practices](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell)
- [Windows Installer Guidelines](https://docs.microsoft.com/en-us/windows/win32/msi/windows-installer-portal)

## 🤝 Contributing

Improvements to the installer are welcome:

- Better error handling
- More customization options
- Additional features
- Better documentation

## 📝 License

AGPL-3.0-or-later (same as KeyRx project)

---

**No external tools required - everything uses Windows built-in features!**
