# KeyRx Self-Contained Windows Installer

This directory contains scripts to build a **completely self-contained** Windows installer using **IExpress** (built into Windows).

## Features

- **No external tools required** to build or run
- **Self-extracting executable** - users just run the .exe
- **Professional installation** with shortcuts, PATH setup, and uninstaller
- **Built with IExpress** - included with every Windows installation
- **No dependencies** - works on any Windows machine

## Quick Start

### Build the Installer

```powershell
# Build everything (binaries + UI + installer)
.\scripts\installer\build-installer.ps1

# Skip builds (use existing binaries)
.\scripts\installer\build-installer.ps1 -SkipBuild -SkipUI
```

### Output

- `keyrx-installer-v0.1.5.exe` - Self-contained installer (ready to distribute)

## How It Works

### 1. Build Process

The `build-installer.ps1` script:

1. Builds release binaries (`keyrx_daemon.exe`, `keyrx_compiler.exe`)
2. Builds the UI (`keyrx_ui/dist`)
3. Copies all files to a temporary directory
4. Generates an IExpress SED configuration file
5. Runs IExpress to create a self-extracting installer
6. Cleans up temporary files

### 2. Installation Process

When users run the installer:

1. IExpress extracts files to a temporary directory
2. Runs `install.ps1` (PowerShell installation script)
3. Installation script:
   - Checks for admin privileges
   - Creates installation directory (`C:\Program Files\KeyRx`)
   - Copies all files
   - Adds to system PATH
   - Creates desktop and start menu shortcuts
   - Registers in Windows Programs list
   - Creates uninstaller script

### 3. Uninstallation

Users can uninstall via:

- Windows Settings → Apps → KeyRx → Uninstall
- Running `uninstall.ps1` from installation directory

## Files

| File | Purpose |
|------|---------|
| `build-installer.ps1` | Main build script |
| `install.ps1` | Installation script (embedded in .exe) |
| `keyrx-installer.sed` | IExpress template (reference) |
| `README.md` | This file |

## Requirements

### Building

- Windows 10/11
- PowerShell 5.1+
- Rust toolchain (for building binaries)
- Node.js 18+ (for building UI)
- IExpress (included with Windows)

### Running the Installer

- Windows 10/11
- Administrator privileges
- **No other requirements!**

## Advanced Usage

### Silent Installation

```powershell
# Install silently to default location
.\keyrx-installer-v0.1.5.exe /Q

# Or run install.ps1 directly
powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Silent
```

### Custom Installation Path

Edit `install.ps1` before building, or modify the installer to accept parameters.

### Skip Specific Builds

```powershell
# Only build binaries (skip UI)
.\build-installer.ps1 -SkipUI

# Only build UI (skip binaries)
.\build-installer.ps1 -SkipBuild

# Use existing builds
.\build-installer.ps1 -SkipBuild -SkipUI
```

## Troubleshooting

### "IExpress not found"

IExpress should be at `C:\Windows\System32\iexpress.exe`. If missing:

- Check Windows version (Windows 10/11 required)
- Try running `where iexpress` to locate it
- Reinstall Windows if necessary

### "Binary not found"

Run without `-SkipBuild` to build binaries first:

```powershell
.\build-installer.ps1
```

### "Access denied" during installation

Run the installer as Administrator (right-click → Run as administrator).

### Installation fails

Check the installation log (displayed in PowerShell window) for specific errors.

## IExpress Overview

**IExpress** is a built-in Windows tool that creates self-extracting executables:

- **Included with Windows** - no installation needed
- **Creates .exe files** that extract and run commands
- **Professional appearance** with custom prompts and license display
- **Silent installation support** for enterprise deployment

### Why IExpress?

| Alternative | Issue |
|-------------|-------|
| WiX Toolset | Requires installation and learning curve |
| Inno Setup | Requires separate download |
| NSIS | External dependency |
| **IExpress** | **Built into Windows, no dependencies** |

## Distribution

The generated `keyrx-installer-v0.1.5.exe` is:

- **Completely self-contained** - no external files needed
- **Safe to distribute** - standard Windows executable
- **Easy to use** - just run the .exe
- **Professional** - Windows-native installer experience

Users can download and run the installer without installing any tools or dependencies.

## Security

The installer:

- Requires administrator privileges (standard for system-level installation)
- Only modifies its own installation directory
- Registers properly in Windows Programs list
- Includes a clean uninstaller

## Future Enhancements

Possible improvements:

- [ ] Code signing certificate for verified publisher
- [ ] MSI package option for enterprise deployment
- [ ] Auto-updater functionality
- [ ] Custom installation wizard (Windows Forms UI)
- [ ] Multi-language support
- [ ] Installation customization options

## References

- [IExpress Documentation](https://docs.microsoft.com/en-us/windows-hardware/drivers/devtest/iexpress)
- [PowerShell Installation Scripts](https://docs.microsoft.com/en-us/powershell/scripting/samples/creating-a-custom-powershell-tab)

## License

AGPL-3.0-or-later (same as KeyRx project)

---

**Built with IExpress - No external tools required!**
