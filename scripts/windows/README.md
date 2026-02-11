# KeyRx Windows Scripts

**MECE** (Mutually Exclusive, Collectively Exhaustive) PowerShell scripts for KeyRx development on Windows.

## Principles

- **MECE**: Each script does one thing, no overlap
- **SLAP**: Single Level of Abstraction - scripts call utilities, not inline logic
- **KISS**: Keep It Simple - one purpose per script
- **SSOT**: Single Source of Truth - shared code in `common/Utils.ps1`
- **SOLID**: Single Responsibility - each script has one clear purpose

## Quick Start

```powershell
# First time setup
.\scripts\windows\dev\Setup.ps1

# Build and run
.\scripts\windows\build\Build.ps1 -Release
.\scripts\windows\daemon\Start.ps1 -Release -Background -Wait

# Run tests
.\scripts\windows\test\UAT.ps1

# Check status
.\scripts\windows\daemon\Status.ps1
```

## Directory Structure

```
scripts/windows/
├── common/          # Shared utilities (SSOT)
│   └── Utils.ps1    # Common functions, paths, colors
├── build/           # Build operations
│   ├── Build.ps1    # Full build
│   └── Clean.ps1    # Clean artifacts
├── daemon/          # Daemon management
│   ├── Start.ps1    # Start daemon
│   ├── Stop.ps1     # Stop daemon
│   ├── Restart.ps1  # Restart daemon
│   └── Status.ps1   # Check status
├── test/            # Testing
│   └── UAT.ps1      # User acceptance tests
└── dev/             # Development workflows
    └── Setup.ps1    # First-time setup
```

## Common Utilities (`common/Utils.ps1`)

All scripts import shared utilities:

```powershell
. (Join-Path $PSScriptRoot "..\common\Utils.ps1")
```

**Functions**:
- `Write-Success`, `Write-Info`, `Write-Warning-Custom`, `Write-Error-Custom`, `Write-Step`
- `Test-DaemonRunning`, `Get-DaemonPid`, `Stop-Daemon`
- `Wait-DaemonReady`, `Test-PortInUse`, `Test-ApiEndpoint`
- `Set-ProjectRoot`, `Test-CargoInstalled`, `Test-NpmInstalled`

**Variables** (SSOT paths):
- `$ProjectRoot`, `$TargetDir`, `$ReleaseDir`, `$DaemonExe`, `$UiDir`

## Build Scripts

### Build.ps1
Full rebuild of daemon.

```powershell
# Debug build (default)
.\scripts\windows\build\Build.ps1

# Release build (optimized)
.\scripts\windows\build\Build.ps1 -Release

# Clean + build
.\scripts\windows\build\Build.ps1 -Release -Clean

# Verbose output
.\scripts\windows\build\Build.ps1 -Verbose
```

### Clean.ps1
Clean build artifacts.

```powershell
# Clean cargo only (default)
.\scripts\windows\build\Clean.ps1

# Clean everything
.\scripts\windows\build\Clean.ps1 -All

# Clean npm only
.\scripts\windows\build\Clean.ps1 -Npm

# No confirmation
.\scripts\windows\build\Clean.ps1 -All -Force
```

## Daemon Scripts

### Start.ps1
Start the daemon.

```powershell
# Start debug build in foreground
.\scripts\windows\daemon\Start.ps1

# Start release build in background
.\scripts\windows\daemon\Start.ps1 -Release -Background

# Start and wait for ready
.\scripts\windows\daemon\Start.ps1 -Release -Background -Wait

# Start with specific profile
.\scripts\windows\daemon\Start.ps1 -Release -Background -Profile "gaming"

# Custom port
.\scripts\windows\daemon\Start.ps1 -Port 8080
```

### Stop.ps1
Stop the daemon.

```powershell
# Graceful stop
.\scripts\windows\daemon\Stop.ps1

# Force kill
.\scripts\windows\daemon\Stop.ps1 -Force
```

### Restart.ps1
Restart the daemon.

```powershell
# Restart with release build
.\scripts\windows\daemon\Restart.ps1 -Release

# Restart and wait for ready
.\scripts\windows\daemon\Restart.ps1 -Release -Wait

# Restart with specific profile
.\scripts\windows\daemon\Restart.ps1 -Release -Profile "default"
```

### Status.ps1
Check daemon status.

```powershell
# Show status
.\scripts\windows\daemon\Status.ps1

# JSON output
.\scripts\windows\daemon\Status.ps1 -Json

# Custom port
.\scripts\windows\daemon\Status.ps1 -Port 8080
```

## Test Scripts

### UAT.ps1
User acceptance testing.

```powershell
# Run standard UAT tests
.\scripts\windows\test\UAT.ps1

# Quick tests only
.\scripts\windows\test\UAT.ps1 -Quick

# Full comprehensive tests
.\scripts\windows\test\UAT.ps1 -Full

# Custom port
.\scripts\windows\test\UAT.ps1 -Port 8080
```

**Test Scenarios**:
1. API Health Check
2. Daemon Status API
3. List Profiles
4. Get Profile Config
5. Activate Profile
6. Verify Active Profile
7. Diagnostics Endpoint (Full mode)
8. Diagnostics Routes (Full mode)
9. Web UI Accessible (Full mode)

## Dev Scripts

### Setup.ps1
First-time development environment setup.

```powershell
# Full setup
.\scripts\windows\dev\Setup.ps1

# Skip Rust check
.\scripts\windows\dev\Setup.ps1 -SkipRust

# Skip Node.js check
.\scripts\windows\dev\Setup.ps1 -SkipNode

# Skip initial build
.\scripts\windows\dev\Setup.ps1 -SkipBuild
```

**What it does**:
- Checks Rust toolchain
- Checks Node.js/npm
- Installs cargo-watch, cargo-tarpaulin
- Installs UI dependencies (npm install)
- Runs initial release build
- Creates config directory

## Common Workflows

### Full Rebuild + UAT
```powershell
.\scripts\windows\build\Build.ps1 -Release -Clean
.\scripts\windows\daemon\Restart.ps1 -Release -Wait
.\scripts\windows\test\UAT.ps1
```

### Quick Development Cycle
```powershell
# Build
.\scripts\windows\build\Build.ps1

# Restart
.\scripts\windows\daemon\Restart.ps1 -Wait

# Check status
.\scripts\windows\daemon\Status.ps1
```

### Clean Start
```powershell
.\scripts\windows\build\Clean.ps1 -All -Force
.\scripts\windows\dev\Setup.ps1
.\scripts\windows\daemon\Start.ps1 -Release -Background -Wait
.\scripts\windows\test\UAT.ps1
```

## Exit Codes

- `0` - Success
- `1` - Error

## Error Handling

All scripts use:
- `Set-StrictMode -Version Latest`
- `$ErrorActionPreference = "Stop"`
- Proper exit codes
- Colored output for errors/warnings/success

## Extensibility

To add new scripts:

1. Create in appropriate category directory
2. Import utilities: `. (Join-Path $PSScriptRoot "..\common\Utils.ps1")`
3. Use common functions and variables
4. Follow MECE principle - one clear purpose
5. Add help/documentation at top
6. Update this README

## Requirements

- PowerShell 5.1+
- Rust 1.70+
- Node.js 18+ (for UI)
- Windows 10+

## Contributing

When adding scripts:
- ✅ Single purpose (MECE)
- ✅ Use common utilities (SSOT)
- ✅ Keep logic simple (KISS)
- ✅ One abstraction level (SLAP)
- ✅ Proper error handling
- ✅ Colored output for UX
- ✅ Document parameters
- ✅ Update README

## License

See root LICENSE file.
