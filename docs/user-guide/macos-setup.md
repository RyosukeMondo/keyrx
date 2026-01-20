# macOS Setup Guide

Complete guide for setting up KeyRx keyboard remapping daemon on macOS.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Setup](#quick-setup)
- [Installation](#installation)
  - [Building from Source](#building-from-source)
  - [Installing the Binary](#installing-the-binary)
- [Accessibility Permission](#accessibility-permission)
  - [Granting Permission](#granting-permission)
  - [Verification](#verification)
  - [Troubleshooting Permission Issues](#troubleshooting-permission-issues)
- [Running the Daemon](#running-the-daemon)
  - [Manual Execution](#manual-execution)
  - [Launch Agent (Auto-start)](#launch-agent-auto-start)
- [Configuration Management](#configuration-management)
  - [Hot Reload](#hot-reload)
  - [Multiple Devices](#multiple-devices)
- [Troubleshooting](#troubleshooting)
- [Security Considerations](#security-considerations)

## Prerequisites

- macOS 10.9 (Mavericks) or later
- Rust 1.70+ (for building from source)
- Xcode Command Line Tools

### Verify Prerequisites

```bash
# Check macOS version
sw_vers

# Install Xcode Command Line Tools if needed
xcode-select --install

# Verify Rust installation
rustc --version
```

## Quick Setup

For users who want to get started quickly:

```bash
# 1. Build the daemon
cargo build --release -p keyrx_daemon --features macos

# 2. Install the binary
sudo cp target/release/keyrx_daemon /usr/local/bin/
sudo chmod 755 /usr/local/bin/keyrx_daemon

# 3. Grant Accessibility permission (see detailed instructions below)
#    Open: System Settings > Privacy & Security > Accessibility
#    Add keyrx_daemon (or Terminal/your IDE)

# 4. Verify setup
keyrx_daemon list-devices

# 5. Run the daemon
keyrx_daemon run --config your-config.krx
```

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/keyrx/keyrx.git
cd keyrx

# Build with macOS features enabled
cargo build --release -p keyrx_daemon --features macos

# The binary is at: target/release/keyrx_daemon
```

### Installing the Binary

**System-wide installation (recommended):**

```bash
sudo cp target/release/keyrx_daemon /usr/local/bin/
sudo chmod 755 /usr/local/bin/keyrx_daemon
```

**User installation:**

```bash
mkdir -p ~/.local/bin
cp target/release/keyrx_daemon ~/.local/bin/
chmod 755 ~/.local/bin/keyrx_daemon

# Ensure ~/.local/bin is in your PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
# Or for bash:
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

## Accessibility Permission

KeyRx requires **Accessibility permission** to capture and remap keyboard events on macOS. This is a security feature that prevents unauthorized applications from monitoring keyboard input.

### Granting Permission

**Method 1: Through System Settings (Recommended)**

1. **Open System Settings**
   - Click the Apple menu () → System Settings
   - Or search for "System Settings" in Spotlight (Cmd+Space)

2. **Navigate to Accessibility**
   - macOS Ventura (13.0+): Privacy & Security → Accessibility
   - macOS Monterey (12.0): Security & Privacy → Privacy → Accessibility
   - Older versions: Security & Privacy → Privacy → Accessibility

3. **Unlock Settings**
   - Click the lock icon (🔒) in the bottom-left corner
   - Enter your administrator password
   - The lock should change to unlocked (🔓)

4. **Add the Application**

   **If running the binary directly:**
   - Click the "+" button
   - Navigate to `/usr/local/bin/` (or `~/.local/bin/`)
   - Select `keyrx_daemon`
   - Enable the toggle next to `keyrx_daemon`

   **If running from Terminal:**
   - Find "Terminal" in the list
   - Enable the toggle next to "Terminal"

   **If running from an IDE (VS Code, IntelliJ, etc.):**
   - Find your IDE in the list
   - Enable the toggle next to your IDE

5. **Restart the Application**
   - Close and reopen Terminal/IDE
   - Or restart `keyrx_daemon` if running as a service

**Method 2: First-Run Prompt**

When you first run `keyrx_daemon`, macOS may display a permission dialog:

1. A dialog appears: "keyrx_daemon would like to receive keystrokes from any application"
2. Click "Open System Settings"
3. Follow steps 3-5 above

### Verification

Test that permission is granted:

```bash
# Check permission status
keyrx_daemon --version
# If permission is missing, you'll see a detailed error message

# List available devices (requires permission)
keyrx_daemon list-devices
# Should display connected keyboards
```

Expected output when permission is granted:

```
Available keyboard devices:
NAME                           VID:PID        SERIAL
Apple Internal Keyboard        05ac:027e      -
Magic Keyboard                 05ac:0267      A1B2C3D4E5F6

Tip: Use these names in your config with device_start("Magic Keyboard")
     or use device_start("*") to match all keyboards.
```

### Troubleshooting Permission Issues

#### Application Not Appearing in List

**Problem:** `keyrx_daemon` doesn't appear in the Accessibility list

**Solutions:**

1. Run the daemon first to trigger the prompt:
   ```bash
   keyrx_daemon run --config dummy.krx
   ```

2. Manually add the application:
   - In Accessibility settings, click "+"
   - Press Cmd+Shift+G to open "Go to folder"
   - Type `/usr/local/bin/` and press Enter
   - Select `keyrx_daemon`

3. If using Terminal/IDE, add that application instead

#### Permission Granted But Still Not Working

**Problem:** Toggle is enabled but daemon still reports permission denied

**Solutions:**

1. **Remove and re-add the application:**
   - In Accessibility settings, select the application
   - Click the "-" button to remove
   - Click "+" to add again
   - Enable the toggle

2. **Restart the application:**
   ```bash
   # If running manually
   killall keyrx_daemon
   keyrx_daemon run --config your-config.krx

   # If running as Launch Agent
   launchctl stop com.keyrx.daemon
   launchctl start com.keyrx.daemon
   ```

3. **Check for multiple instances:**
   ```bash
   # Kill all instances
   killall keyrx_daemon

   # Verify none are running
   pgrep keyrx_daemon
   ```

4. **Restart your Mac** (last resort)

#### Running from Terminal vs Standalone

**Important:** The application that needs permission is the one that **launches** `keyrx_daemon`:

- **Terminal:** Grant permission to Terminal
- **VS Code:** Grant permission to Code/Visual Studio Code
- **Standalone:** Grant permission to keyrx_daemon itself
- **Launch Agent:** Grant permission to keyrx_daemon

If you switch between these methods, you may need to grant permission multiple times.

## Running the Daemon

### Manual Execution

**List available devices:**

```bash
keyrx_daemon list-devices
```

Output:
```
Available keyboard devices:
NAME                           VID:PID        SERIAL
Apple Internal Keyboard        05ac:027e      -
Magic Keyboard                 05ac:0267      A1B2C3D4E5F6

Tip: Use these names in your config with device_start("Magic Keyboard")
     or use device_start("*") to match all keyboards.
```

**Validate configuration (dry-run):**

```bash
keyrx_daemon validate --config my-config.krx
```

Output:
```
Step 1/3: Loading configuration...
  [OK] Configuration loaded successfully

Step 2/3: Enumerating input devices...
  Found 2 keyboard device(s)

Step 3/3: Matching devices to configuration...
  [MATCH] Apple Internal Keyboard
          Matched pattern: "*" (5 mappings)
  [MATCH] Magic Keyboard
          Matched pattern: "Magic Keyboard" (12 mappings)

Validation successful! 2 device(s) will be remapped.
Run 'keyrx_daemon run --config my-config.krx' to start remapping.
```

**Run the daemon:**

```bash
# Normal operation
keyrx_daemon run --config my-config.krx

# With debug logging
keyrx_daemon run --config my-config.krx --debug
```

Press `Ctrl+C` to stop the daemon gracefully.

### Launch Agent (Auto-start)

For automatic startup at login, create a Launch Agent:

**1. Create the plist file:**

```bash
mkdir -p ~/Library/LaunchAgents
```

Create `~/Library/LaunchAgents/com.keyrx.daemon.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.keyrx.daemon</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/keyrx_daemon</string>
        <string>run</string>
        <string>--config</string>
        <string>/Users/YOUR_USERNAME/.config/keyrx/config.krx</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/Users/YOUR_USERNAME/Library/Logs/keyrx.log</string>

    <key>StandardErrorPath</key>
    <string>/Users/YOUR_USERNAME/Library/Logs/keyrx.error.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
</dict>
</plist>
```

**Important:** Replace `YOUR_USERNAME` with your actual username.

**2. Create configuration directory:**

```bash
mkdir -p ~/.config/keyrx
cp my-config.krx ~/.config/keyrx/config.krx
```

**3. Load the Launch Agent:**

```bash
launchctl load ~/Library/LaunchAgents/com.keyrx.daemon.plist
```

**4. Verify it's running:**

```bash
launchctl list | grep keyrx
```

**Launch Agent management commands:**

```bash
# Start
launchctl start com.keyrx.daemon

# Stop
launchctl stop com.keyrx.daemon

# Restart
launchctl stop com.keyrx.daemon
launchctl start com.keyrx.daemon

# Unload (disable auto-start)
launchctl unload ~/Library/LaunchAgents/com.keyrx.daemon.plist

# Reload (after editing plist)
launchctl unload ~/Library/LaunchAgents/com.keyrx.daemon.plist
launchctl load ~/Library/LaunchAgents/com.keyrx.daemon.plist

# View logs
tail -f ~/Library/Logs/keyrx.log
tail -f ~/Library/Logs/keyrx.error.log
```

## Configuration Management

### Hot Reload

Reload configuration without restarting the daemon:

```bash
# Send SIGHUP to reload
killall -HUP keyrx_daemon

# Or for Launch Agent
launchctl kickstart -k com.keyrx.daemon
```

Hot reload:
- Preserves current modifier and lock states
- Keeps device connections active (no interruption)
- Applies new mappings immediately

### Multiple Devices

Configure different mappings for different keyboards:

```rhai
// Built-in keyboard
device_start("Apple Internal Keyboard");
    map("CapsLock", "VK_Escape");
device_end();

// External Magic Keyboard with navigation layer
device_start("Magic Keyboard");
    map("CapsLock", "MD_00");  // Navigation layer
    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();
device_end();

// Fallback for any other keyboard
device_start("*");
    map("CapsLock", "VK_Escape");
device_end();
```

Use `keyrx_daemon list-devices` to find device names.

## Troubleshooting

### Permission Denied Error

**Symptom:**
```
Error: Accessibility permission required but not granted.

KeyRx requires Accessibility permission to capture and remap keyboard events.
...
```

**Solutions:**

1. Grant Accessibility permission (see [Accessibility Permission](#accessibility-permission) section)

2. Verify permission is granted:
   ```bash
   keyrx_daemon list-devices
   ```

3. If running from Terminal/IDE, grant permission to that application

4. Restart the application after granting permission

### No Devices Found

**Symptom:** `keyrx_daemon list-devices` shows no keyboards

**Solutions:**

1. Check Accessibility permission (see above)

2. Try unplugging and replugging USB keyboards

3. Check System Information:
   ```bash
   system_profiler SPUSBDataType | grep -i keyboard
   ```

4. Restart the daemon with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```

### Daemon Starts but Keys Not Remapped

**Solutions:**

1. Verify device matching:
   ```bash
   keyrx_daemon validate --config your-config.krx
   ```

2. Check configuration syntax:
   ```bash
   keyrx_compiler verify your-config.krx
   ```

3. Run with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```

4. Ensure only one instance is running:
   ```bash
   killall keyrx_daemon
   keyrx_daemon run --config your-config.krx
   ```

### Keys Stuck After Crash

If the daemon crashes while a key is held, it may appear "stuck."

**Solutions:**

1. Press and release the stuck key
2. Press and release all modifier keys (Cmd, Option, Control, Shift)
3. If using custom modifiers, press the keys mapped to those modifiers

### Launch Agent Not Starting

**Symptom:** Launch Agent loaded but daemon not running

**Solutions:**

1. Check logs for errors:
   ```bash
   tail -f ~/Library/Logs/keyrx.error.log
   ```

2. Verify plist syntax:
   ```bash
   plutil -lint ~/Library/LaunchAgents/com.keyrx.daemon.plist
   ```

3. Ensure paths are correct:
   - Replace `YOUR_USERNAME` with actual username
   - Verify binary exists: `ls -la /usr/local/bin/keyrx_daemon`
   - Verify config exists: `ls -la ~/.config/keyrx/config.krx`

4. Check Launch Agent status:
   ```bash
   launchctl list | grep keyrx
   # Should show: PID, Status, Label
   ```

5. Try manual load with verbose output:
   ```bash
   launchctl load -w ~/Library/LaunchAgents/com.keyrx.daemon.plist
   ```

### High CPU Usage

**Symptom:** Daemon using excessive CPU

**Solutions:**

1. Check for event loops with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```

2. Simplify configuration to isolate the issue

3. Report as a bug with reproduction steps

### Conflicts with Other Remapping Tools

**Problem:** Using Karabiner-Elements, BetterTouchTool, or similar tools

**Solutions:**

1. **Best practice:** Use only one remapping tool at a time

2. If you must use both:
   - Ensure they target different devices
   - Or use Karabiner for simple remaps, KeyRx for complex layers

3. Check for conflicts:
   ```bash
   # List processes accessing input devices
   lsof | grep input
   ```

## Security Considerations

### Accessibility Permission Risks

Applications with Accessibility permission can:
- Capture all keyboard and mouse input (including passwords)
- Inject keyboard and mouse events
- Control other applications

**Best Practices:**

1. Only grant permission to trusted applications
2. Review which applications have permission regularly:
   - System Settings → Privacy & Security → Accessibility
3. Revoke permission when not needed
4. Build KeyRx from source to verify code integrity

### Launch Agent Security

**Best Practices:**

1. Store the daemon binary in a protected location:
   ```bash
   sudo chown root:wheel /usr/local/bin/keyrx_daemon
   sudo chmod 755 /usr/local/bin/keyrx_daemon
   ```

2. Protect your configuration:
   ```bash
   chmod 600 ~/.config/keyrx/config.krx
   ```

3. Review configuration before deploying:
   ```bash
   keyrx_daemon validate --config ~/.config/keyrx/config.krx
   ```

4. Monitor daemon logs for unusual activity:
   ```bash
   tail -f ~/Library/Logs/keyrx.log
   ```

### Protecting Against Keyloggers

KeyRx itself could be misused as a keylogger. To prevent this:

1. **Only run trusted binaries** - build from source when possible
2. **Review the code** before building
3. **Keep the daemon updated** for security patches
4. **Use filesystem permissions** to protect the binary
5. **Monitor running processes** regularly:
   ```bash
   ps aux | grep keyrx_daemon
   ```

## Platform-Specific Notes

### Apple Silicon (M1/M2/M3) vs Intel

KeyRx supports both architectures:

- **Apple Silicon:** Build with default target (aarch64-apple-darwin)
- **Intel:** Build with default target (x86_64-apple-darwin)
- **Universal Binary:** Combine both architectures:
  ```bash
  cargo build --release --target=aarch64-apple-darwin
  cargo build --release --target=x86_64-apple-darwin
  lipo -create \
    target/aarch64-apple-darwin/release/keyrx_daemon \
    target/x86_64-apple-darwin/release/keyrx_daemon \
    -output keyrx_daemon-universal
  ```

### macOS Version Compatibility

| macOS Version | Status | Notes |
|--------------|--------|-------|
| 14.0+ (Sonoma) | ✅ Fully Supported | Latest tested version |
| 13.0-13.6 (Ventura) | ✅ Fully Supported | |
| 12.0-12.7 (Monterey) | ✅ Fully Supported | |
| 11.0-11.7 (Big Sur) | ✅ Supported | Limited testing |
| 10.15 (Catalina) | ⚠️ Should Work | Untested |
| 10.9-10.14 | ⚠️ Should Work | Accessibility API available |
| < 10.9 | ❌ Not Supported | No Accessibility API |

### Bluetooth Keyboards

Bluetooth keyboards work with KeyRx but may have:
- Slightly higher latency (typically <2ms additional)
- Intermittent connection issues
- Different device names (use `list-devices` to find exact names)

## Performance Notes

KeyRx on macOS is optimized for low latency:

- **Average latency:** <1ms (input capture to output injection)
- **Memory usage:** ~5-10 MB (depending on configuration size)
- **CPU usage:** <1% idle, <3% during typing

To verify performance on your system:

```bash
# Run with performance logging
keyrx_daemon run --config your-config.krx --debug

# Monitor resource usage
top -pid $(pgrep keyrx_daemon)
```

## Additional Resources

- **DSL Manual:** [docs/user-guide/dsl-manual.md](dsl-manual.md)
- **Configuration Examples:** [examples/](../../examples/)
- **Multi-Device Setup:** [docs/user-guide/multi-device-configuration.md](multi-device-configuration.md)
- **Architecture Details:** [docs/development/architecture.md](../development/architecture.md)
- **GitHub Issues:** https://github.com/keyrx/keyrx/issues

## Getting Help

If you encounter issues not covered in this guide:

1. Check existing GitHub issues
2. Run with `--debug` flag and review logs
3. Use `validate` command to check configuration
4. Create a new issue with:
   - macOS version (`sw_vers`)
   - Daemon version (`keyrx_daemon --version`)
   - Configuration file (if relevant)
   - Full error message and logs
