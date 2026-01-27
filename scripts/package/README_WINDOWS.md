# Windows Installer Build Guide

This directory contains scripts to build Windows installers for KeyRx.

## Overview

We provide two installer options:

1. **Inno Setup** (Recommended) - Simple, popular, easy to use
2. **WiX Toolset** - More complex, produces MSI files

## Quick Start (Inno Setup)

### Prerequisites

1. **Inno Setup 6** - Download from https://jrsoftware.org/isdl.php
2. **Rust toolchain** with `x86_64-pc-windows-msvc` target
3. **Node.js 18+**

### Build

```powershell
# Full build with tests
.\scripts\package\build-windows-innosetup.ps1

# Skip tests (faster)
.\scripts\package\build-windows-innosetup.ps1 -SkipTests

# Clean build
.\scripts\package\build-windows-innosetup.ps1 -Clean
```

### Output

```
target\windows-installer\keyrx_<version>_x64_setup.exe
```

### Install

Double-click the installer or run silently:

```powershell
.\target\windows-installer\keyrx_0.2.5_x64_setup.exe /SILENT
```

## Alternative: WiX Toolset

### Prerequisites

1. **WiX Toolset v3** - Download from https://wixtoolset.org/releases/
2. Add WiX bin directory to PATH: `C:\Program Files (x86)\WiX Toolset v3.x\bin`

### Build

```powershell
# Full build
.\scripts\package\build-windows-installer.ps1

# Skip tests
.\scripts\package\build-windows-installer.ps1 -SkipTests
```

### Output

```
target\windows-installer\keyrx_<version>_x64.msi
```

### Install

```powershell
# GUI install
msiexec /i target\windows-installer\keyrx_0.2.5_x64.msi

# Silent install
msiexec /i target\windows-installer\keyrx_0.2.5_x64.msi /quiet
```

## Comparison

| Feature | Inno Setup | WiX |
|---------|------------|-----|
| **Ease of Use** | ⭐⭐⭐⭐⭐ Simple | ⭐⭐⭐ Complex XML |
| **Output** | `.exe` installer | `.msi` package |
| **Customization** | Pascal script | XML + C# |
| **File Size** | Smaller (better compression) | Larger |
| **Enterprise** | Good | Excellent (Group Policy) |
| **Update Support** | Manual | Built-in upgrade logic |
| **Learning Curve** | Low | High |

**Recommendation:** Use Inno Setup for most cases. Use WiX only if you need MSI for enterprise deployment or Group Policy.

## Build Process

Both scripts follow the same build sequence:

1. **Check Dependencies** - Verify all required tools are installed
2. **Build WASM** - Compile keyrx_core to WebAssembly
3. **Build UI** - Build React frontend (production bundle)
4. **Build Daemon** - Compile keyrx_daemon with embedded UI
5. **Run Tests** - Execute test suite (unless `-SkipTests`)
6. **Package** - Create installer with Inno Setup or WiX

## Installer Features

Both installers include:

- ✅ **Binaries** - `keyrx_daemon.exe`, `keyrx_compiler.exe`
- ✅ **Example Config** - `user_layout.krx`
- ✅ **Documentation** - README, LICENSE
- ✅ **PATH Integration** - Optional add to system PATH
- ✅ **Start Menu Shortcuts** - KeyRx Daemon, Web UI, Compiler
- ✅ **Desktop Shortcut** - Optional
- ✅ **Uninstaller** - Complete removal support
- ✅ **Automatic Updates** - Version detection

## Testing on Windows

### Option 1: Direct Testing (Current Machine)

Since you're already on Windows, test directly:

```powershell
# Run UAT script (builds and starts daemon)
.\scripts\windows\UAT.ps1

# Or manually
cargo build --release --features windows
.\target\release\keyrx_daemon.exe run
```

### Option 2: Vagrant VM (Linux → Windows)

If developing on Linux, use Vagrant:

```bash
# From Linux host
./scripts/windows_test_vm.sh

# Or manually
cd vagrant/windows
vagrant up
vagrant ssh
cd C:\vagrant_project
cargo test -p keyrx_daemon --features windows
```

## Troubleshooting

### "Inno Setup not found"

- Download from https://jrsoftware.org/isdl.php
- Install to default location: `C:\Program Files (x86)\Inno Setup 6`
- Or set `INNO_SETUP` environment variable to `ISCC.exe` path

### "WiX not found"

- Download from https://wixtoolset.org/releases/
- Install to default location
- Add to PATH: `C:\Program Files (x86)\WiX Toolset v3.x\bin`
- Or set `WIX` environment variable

### "WASM build failed"

```powershell
# Install wasm-pack
cargo install wasm-pack

# Add wasm target
rustup target add wasm32-unknown-unknown

# Clean and rebuild
Remove-Item -Recurse -Force keyrx_ui\src\wasm\pkg
cd keyrx_core
wasm-pack build --target web --out-dir ..\keyrx_ui\src\wasm\pkg --release -- --features wasm
```

### "UI build failed"

```powershell
cd keyrx_ui
npm ci
npm run build
```

### "Tests failed"

```powershell
# Run tests with output
cargo test --workspace --features windows -- --nocapture

# Run specific test
cargo test -p keyrx_daemon --features windows <test_name>
```

## Manual Build (Without Scripts)

If scripts don't work, build manually:

```powershell
# 1. Build WASM
cd keyrx_core
wasm-pack build --target web --out-dir ..\keyrx_ui\src\wasm\pkg --release -- --features wasm

# 2. Build UI
cd ..\keyrx_ui
npm ci
npm run build

# 3. Build daemon
cd ..
cargo build --release --bin keyrx_daemon --bin keyrx_compiler --features windows

# 4. Create installer (Inno Setup)
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" scripts\package\keyrx-installer.iss

# Or (WiX)
candle -nologo -out target\windows-installer\keyrx.wixobj -arch x64 keyrx_daemon\keyrx_installer.wxs
light -nologo -out target\windows-installer\keyrx.msi -ext WixUIExtension keyrx.wixobj
```

## Release Automation

For automated releases, see `.github/workflows/release.yml`. The CI pipeline:

1. Builds on Windows runners
2. Creates installers
3. Uploads to GitHub Releases
4. Generates release notes

## References

- **Inno Setup Documentation**: https://jrsoftware.org/ishelp/
- **WiX Toolset Documentation**: https://wixtoolset.org/documentation/
- **Project README**: `../../README.md`
- **Linux Installers**: `build-deb.sh`, `build-tarball.sh`
