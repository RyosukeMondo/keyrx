# Windows Setup Guide

This guide explains how to install, configure, and run the KeyRx daemon on Windows.

## Prerequisites

- **Windows 10 or 11**
- **Rust Toolchain**: Required to build from source (see [rust-lang.org](https://www.rust-lang.org/))

## Installation

### 1. Build from Source

Clone the repository and build the daemon with the `windows` feature:

```powershell
git clone https://github.com/RyosukeMondo/keyrx
cd keyrx
cargo build --release -p keyrx_daemon --features windows
```

The binary will be available at `target/release/keyrx_daemon.exe`.

### 2. (Optional) Install to PATH

```powershell
cargo install --path keyrx_daemon --features windows
```

## Running the Daemon

To run the daemon with a compiled configuration (`.krx` file):

```powershell
keyrx_daemon run --config path\to\your-config.krx
```

### System Tray Integration

When the daemon starts, a KeyRx icon will appear in the system tray (notification area).

- **Reload Config**: Right-click the tray icon and select "Reload Config" to hot-reload your `.krx` file.
- **Exit**: Right-click the tray icon and select "Exit" to stop the daemon and release all keyboard hooks.

## Permissions

The KeyRx daemon requires administrative privileges to use the **Raw Input API** in background mode (`RIDEV_INPUTSINK`). This allows the daemon to capture and remap keys globally, even when it is not the active window.

- **Run as Administrator**: You must run your terminal (PowerShell or Command Prompt) as **Administrator** before starting the daemon.
- **Failures**: If started without admin rights, the daemon may fail to register devices or silently fail to capture input from background windows.

## Device Identification

KeyRx on Windows uses **Raw Input** to distinguish between multiple keyboards.

- **Enumeration**: On startup, the daemon logs all detected keyboards with their unique IDs (instance paths) and serial numbers (if available).
- **Log Output**: Check the logs (stdout) to find the ID of your specific device.
    ```
    INFO [keyrx_daemon::platform::windows::rawinput] Device arrived: ...
    ```
- **Web UI**: You can also view connected devices via the KeyRx web interface (if enabled) at `http://localhost:8080/settings/devices`.

## Troubleshooting

### Stuck Keys
If the daemon crashes or is terminated unexpectedly, keys might occasionally get "stuck" in a pressed state. Pressing the physical key again will usually clear the state. Using the "Exit" option from the tray icon ensures a clean release of all keys.

### Hook Conflicts
Other applications that use low-level keyboard hooks (like AutoHotkey or some gaming software) might conflict with KeyRx. If remappings are not working as expected, try closing other keyboard-related software. Raw Input usually coexists better than hooks, but conflicts can still occur if another app helps exclusive access.

### Missing Icon
If the tray icon is not visible, check the "Hidden icons" overflow menu in the taskbar. You can drag the KeyRx icon to the main taskbar area for better visibility.

