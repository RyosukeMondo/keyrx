# Platform-Specific Scripts

This directory contains platform-specific utility scripts organized by operating system.

## Directory Structure

```
platform/
├── linux/           # Linux-specific scripts
├── macos/           # macOS-specific scripts
└── windows/         # Windows-specific scripts
```

## Linux Scripts

Linux-specific scripts are currently integrated into the main cross-platform scripts (setup.sh, uat.sh, install.sh). Future Linux-only utilities can be added to `platform/linux/`.

## macOS Scripts

| Script | Purpose |
|--------|---------|
| `macos/check_permission.sh` | Check if Accessibility permission is granted |
| `macos/test_full.sh` | Comprehensive test runner (mock, E2E, benchmarks) |
| `macos/setup_accessibility.sh` | Interactive guide to grant Accessibility permission |
| `macos/install_launchd.sh` | Install daemon as a LaunchAgent for auto-start |

### Usage Examples

**Check Accessibility Permission:**
```bash
./scripts/platform/macos/check_permission.sh
echo "Exit code: $?"  # 0 = granted, 1 = denied
```

**Run Full Test Suite:**
```bash
./scripts/platform/macos/test_full.sh
```

**Interactive Permission Setup:**
```bash
./scripts/platform/macos/setup_accessibility.sh
```

**Install LaunchAgent:**
```bash
# First build the release binary
cargo build --release

# Then install the LaunchAgent
./scripts/platform/macos/install_launchd.sh
```

## Windows Scripts

| Script | Purpose |
|--------|---------|
| `windows/test_vm.sh` | Run tests in Vagrant Windows VM |
| `windows/deploy.sh` | Deploy Windows binaries to remote PC via SSH |
| `windows/build_msi_wrapper.ps1` | Build MSI installer wrapper |
| `windows/build_windows_installer.ps1` | Build Windows installer |
| `windows/build_windows_simple.ps1` | Simple Windows build script |
| `windows/clean_test_profiles.ps1` | Clean test profiles |

### Usage Examples

**Test in Windows VM:**
```bash
# Automated testing
./scripts/platform/windows/test_vm.sh

# With UAT
./scripts/platform/windows/test_vm.sh --uat
```

**Deploy to Windows PC:**
```bash
./scripts/platform/windows/deploy.sh --host 192.168.1.100 --user developer
```

## Cross-Platform Scripts

The main scripts directory (`scripts/`) contains cross-platform scripts that work on all supported platforms:

- `build.sh` - Build WASM, UI, and daemon
- `test.sh` - Run unit, integration, and fuzz tests
- `verify.sh` - Quality gates (clippy, fmt, tests, coverage)
- `setup.sh` - Environment setup (with platform-specific logic)
- `uat.sh` - User acceptance testing (with platform-specific logic)
- `install.sh` - System installation (with platform-specific logic)
- `launch.sh` - Start daemon (cross-platform)
- `dev.sh` - Development server (cross-platform)

These scripts automatically detect the operating system and adjust behavior accordingly.

## Platform Detection

Scripts use `$OSTYPE` for platform detection:

```bash
case "$OSTYPE" in
    darwin*)
        # macOS-specific logic
        ;;
    linux*)
        # Linux-specific logic
        ;;
    *)
        echo "Unsupported OS: $OSTYPE"
        exit 1
        ;;
esac
```

Common `$OSTYPE` values:
- `darwin*` - macOS
- `linux-gnu*` - Linux
- `msys*` or `cygwin*` - Windows (Git Bash/Cygwin)

## Migration from Old Paths

If you have scripts or documentation referencing the old paths, update them:

| Old Path | New Path |
|----------|----------|
| `scripts/check_macos_permission.sh` | `scripts/platform/macos/check_permission.sh` |
| `scripts/test_macos_full.sh` | `scripts/platform/macos/test_full.sh` |
| `scripts/windows_test_vm.sh` | `scripts/platform/windows/test_vm.sh` |
| `scripts/deploy_windows.sh` | `scripts/platform/windows/deploy.sh` |

## Adding New Platform-Specific Scripts

When adding new platform-specific scripts:

1. **Determine if it's truly platform-specific:**
   - If it can work on multiple platforms with conditional logic, add it to the main `scripts/` directory
   - Only add to `platform/` if it's fundamentally tied to one OS

2. **Place in correct subdirectory:**
   - Linux-only → `platform/linux/`
   - macOS-only → `platform/macos/`
   - Windows-only → `platform/windows/`

3. **Make executable:**
   ```bash
   chmod +x scripts/platform/[os]/script_name.sh
   ```

4. **Update documentation:**
   - Add entry to this README
   - Update `scripts/CLAUDE.md` if it's a commonly-used script

5. **Use absolute paths:**
   ```bash
   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
   PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"  # Adjust based on depth
   ```

## Testing

All platform-specific scripts should be tested on their target platform before committing:

- **macOS scripts**: Test on macOS 12+ (Monterey or later)
- **Linux scripts**: Test on Ubuntu 20.04+ or equivalent
- **Windows scripts**: Test in Windows 10/11 environment or Vagrant VM
