# Windows Build & Test Guide

Complete guide for testing and building KeyRx on Windows.

## 🚀 Quick Start

### 1. Test on Windows (Current Machine)

Since you're already on Windows, test directly:

```powershell
# Quick test (build only)
.\scripts\package\test-windows.ps1 -Quick

# Full test (with tests)
.\scripts\package\test-windows.ps1

# Or use existing UAT script
.\scripts\windows\UAT.ps1
```

### 2. Build Installer

**Option A: Inno Setup (Recommended - Simpler)**

```powershell
# Install Inno Setup first: https://jrsoftware.org/isdl.php
.\scripts\package\build-windows-innosetup.ps1

# Output: target\windows-installer\keyrx_<version>_x64_setup.exe
```

**Option B: WiX Toolset (MSI format)**

```powershell
# Install WiX first: https://wixtoolset.org/releases/
.\scripts\package\build-windows-installer.ps1

# Output: target\windows-installer\keyrx_<version>_x64.msi
```

## 📋 What Was Created

### New Files

1. **`scripts/package/build-windows-innosetup.ps1`**
   - Main build script using Inno Setup (recommended)
   - Simple, popular installer format
   - Output: `.exe` installer

2. **`scripts/package/build-windows-installer.ps1`**
   - Alternative build script using WiX
   - MSI format for enterprise deployment
   - Output: `.msi` package

3. **`scripts/package/keyrx-installer.iss`**
   - Inno Setup configuration
   - Defines installer behavior, shortcuts, PATH integration

4. **`scripts/package/test-windows.ps1`**
   - Quick test script for Windows builds
   - Validates build and tests pass

5. **`scripts/package/README_WINDOWS.md`**
   - Comprehensive documentation
   - Troubleshooting guide
   - Comparison of installers

6. **`WINDOWS_BUILD_GUIDE.md`** (this file)
   - Quick reference guide

### Updated Files

1. **`keyrx_daemon/keyrx_installer.wxs`**
   - Added ARPURLINFOABOUT property
   - Added comments for clarity

## 🔧 Prerequisites

### Required

- **Rust** - Already installed (with `x86_64-pc-windows-msvc` target)
- **Node.js 18+** - For UI building
- **Git** - For version control

### For Installers

Choose one:

- **Inno Setup 6** (Recommended)
  - Download: https://jrsoftware.org/isdl.php
  - Simple GUI installer
  - ~4 MB download

- **WiX Toolset v3**
  - Download: https://wixtoolset.org/releases/
  - MSI format
  - More complex setup

## 📊 Build Process Comparison

| Step | Inno Setup | WiX |
|------|------------|-----|
| **Install Tool** | Simple GUI installer | Complex MSI installer |
| **Script Language** | Pascal (in `.iss` file) | XML (in `.wxs` file) |
| **Build Command** | `ISCC.exe keyrx-installer.iss` | `candle` + `light` |
| **Output Format** | `.exe` (self-extracting) | `.msi` (Windows Installer) |
| **File Size** | Smaller (better compression) | Larger |
| **Enterprise Support** | Good | Excellent |
| **Ease of Use** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

## 🎯 Testing Strategy

### Level 1: Quick Smoke Test (2 minutes)

```powershell
.\scripts\package\test-windows.ps1 -Quick
```

Validates:
- ✅ Code compiles
- ✅ Binaries are created
- ✅ Version info works

### Level 2: Full Test (5 minutes)

```powershell
.\scripts\package\test-windows.ps1
```

Validates:
- ✅ All Level 1 checks
- ✅ Full test suite passes
- ✅ Windows-specific features work

### Level 3: UAT (User Acceptance Test)

```powershell
.\scripts\windows\UAT.ps1
```

Validates:
- ✅ All Level 2 checks
- ✅ WASM builds correctly
- ✅ UI builds and embeds
- ✅ Daemon starts and serves UI
- ✅ End-to-end functionality

### Level 4: Installer Test

```powershell
# Build installer
.\scripts\package\build-windows-innosetup.ps1

# Test silent install
$installer = Get-ChildItem target\windows-installer\*.exe | Select-Object -First 1
& $installer /VERYSILENT /SUPPRESSMSGBOXES /LOG="install.log"

# Verify installation
& "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" --version

# Test uninstall
$uninstaller = "C:\Program Files\KeyRx\unins000.exe"
& $uninstaller /VERYSILENT
```

## 🏗️ Build Workflow

### For Development

```powershell
# 1. Make changes
# 2. Quick test
.\scripts\package\test-windows.ps1 -Quick

# 3. Full test before commit
.\scripts\package\test-windows.ps1

# 4. UAT before release
.\scripts\windows\UAT.ps1
```

### For Release

```powershell
# 1. Update version in Cargo.toml
# 2. Run full tests
.\scripts\package\test-windows.ps1

# 3. Build installer
.\scripts\package\build-windows-innosetup.ps1

# 4. Test installer
# ... (see Level 4 above)

# 5. Create GitHub release with installer
```

## 📦 Installer Features

Both installers include:

### Files Installed

- `C:\Program Files\KeyRx\bin\keyrx_daemon.exe` - Main daemon
- `C:\Program Files\KeyRx\bin\keyrx_compiler.exe` - Config compiler
- `C:\Program Files\KeyRx\README.md` - Documentation
- `C:\Program Files\KeyRx\LICENSE` - License file
- `%APPDATA%\keyrx\user_layout.krx` - Example config

### Integration

- ✅ Start Menu shortcuts
  - KeyRx Daemon
  - KeyRx Web UI (opens http://localhost:9867)
  - KeyRx Compiler
  - Uninstall

- ✅ Optional Desktop shortcut

- ✅ Optional PATH integration
  - Allows running `keyrx_daemon` from any terminal

- ✅ App registration
  - Shows in "Add or Remove Programs"
  - Includes uninstaller

## 🐛 Troubleshooting

### "Inno Setup not found"

```powershell
# Download and install
Start-Process "https://jrsoftware.org/isdl.php"

# Or set environment variable
$env:INNO_SETUP = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
```

### "WiX not found"

```powershell
# Download and install WiX
Start-Process "https://wixtoolset.org/releases/"

# Add to PATH
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.14\bin"
```

### "WASM build failed"

```powershell
# Install wasm-pack
cargo install wasm-pack

# Add WASM target
rustup target add wasm32-unknown-unknown

# Clean and retry
Remove-Item -Recurse -Force keyrx_ui\src\wasm\pkg
.\scripts\package\build-windows-innosetup.ps1 -Clean
```

### "UI build failed"

```powershell
cd keyrx_ui
Remove-Item -Recurse -Force node_modules, dist
npm install
npm run build
cd ..
```

### "Tests failed"

```powershell
# Run with verbose output
cargo test --workspace --features windows -- --nocapture

# Run specific test
cargo test -p keyrx_daemon <test_name> -- --nocapture

# Skip tests (if CI/CD will catch issues)
.\scripts\package\build-windows-innosetup.ps1 -SkipTests
```

### "Daemon won't start"

```powershell
# Check if already running
Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Stop-Process -Force

# Try with debug output
.\target\debug\keyrx_daemon.exe run --config examples\user_layout.krx

# Check logs
Get-Content "$env:APPDATA\keyrx\logs\*" | Select-Object -Last 50
```

## 🔄 Comparison with Linux

| Feature | Linux (DEB/TAR) | Windows (EXE/MSI) |
|---------|----------------|-------------------|
| **Build Time** | ~3 min | ~3 min |
| **Dependencies** | System packages | Bundled in installer |
| **Installation** | `dpkg -i` or extract | Double-click or `msiexec` |
| **System Integration** | `.desktop` files | Start Menu, PATH |
| **Service** | systemd | Manual start (service TBD) |
| **Permissions** | udev rules, groups | Admin install |
| **Updates** | `apt upgrade` | New installer |

## 📝 Next Steps

1. **Test Now:**
   ```powershell
   .\scripts\package\test-windows.ps1
   ```

2. **Build Installer:**
   ```powershell
   .\scripts\package\build-windows-innosetup.ps1
   ```

3. **Test Installer:**
   - Install silently
   - Verify PATH
   - Run daemon
   - Test Web UI
   - Uninstall

4. **Update CI/CD:**
   - Add Windows build job to `.github/workflows/release.yml`
   - Upload installers to GitHub Releases

5. **Documentation:**
   - Add Windows installation instructions to main README
   - Update INSTALLER_SETUP.md with Windows section

## 📚 References

- **Inno Setup Docs**: https://jrsoftware.org/ishelp/
- **WiX Docs**: https://wixtoolset.org/documentation/
- **Linux Installers**: `scripts/package/build-deb.sh`, `build-tarball.sh`
- **Package README**: `scripts/package/README.md`
- **Windows Details**: `scripts/package/README_WINDOWS.md`

---

**Summary:**
- ✅ Windows support already exists in codebase
- ✅ New build scripts created (Inno Setup + WiX)
- ✅ Testing scripts created
- ✅ Comprehensive documentation added
- 🎯 **Action:** Test with `.\scripts\package\test-windows.ps1`
- 🎯 **Action:** Build with `.\scripts\package\build-windows-innosetup.ps1`
