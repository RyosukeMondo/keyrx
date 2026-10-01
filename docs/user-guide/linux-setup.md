# Linux Setup Guide

Complete guide for setting up KeyRx keyboard remapping daemon on Linux.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Setup](#quick-setup)
- [Installation](#installation)
  - [Building from Source](#building-from-source)
  - [Installing the Binary](#installing-the-binary)
- [Permissions Setup](#permissions-setup)
  - [udev Rules](#udev-rules)
  - [User Groups](#user-groups)
- [Running the Daemon](#running-the-daemon)
  - [Manual Execution](#manual-execution)
  - [systemd Service (System-wide)](#systemd-service-system-wide)
  - [systemd Service (Per-user)](#systemd-service-per-user)
- [Configuration Management](#configuration-management)
  - [Hot Reload](#hot-reload)
  - [Multiple Devices](#multiple-devices)
- [Troubleshooting](#troubleshooting)
- [Security Considerations](#security-considerations)

## Prerequisites

- Linux kernel 2.6+ (evdev and uinput support)
- Rust 1.70+ (for building from source)
- systemd (optional, for service management)

### Verify Kernel Support

```bash
# Check evdev module
lsmod | grep evdev
# If not loaded: sudo modprobe evdev

# Check uinput module
lsmod | grep uinput
# If not loaded: sudo modprobe uinput

# Verify input devices exist
ls /dev/input/event*
```

## Quick Setup

For users who want to get started quickly:

```bash
# 1. Build the daemon
cargo build --release -p keyrx_daemon --features linux

# 2. Install udev rules
sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger

# 3. Create uinput group and add your user
sudo groupadd -f uinput
sudo usermod -aG input,uinput $USER

# 4. Log out and back in (or reboot)

# 5. Verify setup
./target/release/keyrx_daemon list-devices

# 6. Run the daemon
./target/release/keyrx_daemon run --config your-config.krx
```

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/keyrx/keyrx.git
cd keyrx

# Build with Linux features enabled
cargo build --release -p keyrx_daemon --features linux

# The binary is at: target/release/keyrx_daemon
```

### Installing the Binary

**System-wide installation (recommended for systemd service):**

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
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

## Permissions Setup

KeyRx requires access to input devices and the ability to create virtual keyboards.

### udev Rules

Install the provided udev rules for non-root operation:

```bash
# Copy rules file
sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

The rules grant:
- `input` group: Read access to `/dev/input/event*` devices
- `uinput` group: Read/write access to `/dev/uinput`

### User Groups

Add your user to the required groups:

```bash
# Create uinput group if it doesn't exist
sudo groupadd -f uinput

# Add user to both groups
sudo usermod -aG input,uinput $USER

# Verify group membership
groups $USER
```

**Important:** You must log out and log back in for group changes to take effect. Alternatively, use:

```bash
newgrp input
newgrp uinput
```

### Verify Permissions

```bash
# Check device permissions
ls -la /dev/input/event* /dev/uinput

# Should show:
# crw-rw---- 1 root input  ... /dev/input/event*
# crw-rw---- 1 root uinput ... /dev/uinput

# Test read access
cat /dev/input/event0
# (Press a key, you should see binary data, Ctrl+C to exit)

# List available keyboards
keyrx_daemon list-devices
```

## Running the Daemon

### Manual Execution

**List available devices:**

```bash
keyrx_daemon list-devices
```

Output:
```
Available keyboard devices:

PATH                           NAME                      READABLE   SERIAL
------------------------------------------------------------------------------------------
/dev/input/event3              AT Translated Set 2 kbd   yes        -
/dev/input/event18             USB Keyboard              yes        USB-12345

Found 2 keyboard device(s).

Tip: Use patterns in your configuration to match devices:
  - "*" matches all keyboards
  - "USB*" matches devices with USB in name/serial
  - Exact name match for specific devices
```

The `READABLE` column is a cheap permission check (no device is opened just
to list it). A device listed as `no` cannot be grabbed by the daemon even if
a `device_start` pattern matches it - see `keyrx_daemon doctor` for the fix.

**Validate configuration (dry-run):**

```bash
keyrx_daemon validate --config my-config.krx
```

Output:
```
Validating configuration: my-config.krx

1. Loading configuration...
   Configuration loaded: 1 device pattern(s)
   [ 1] Pattern: "*" (5 mapping(s))

2. Enumerating keyboard devices...
   Found 2 keyboard device(s)

3. Matching devices to configuration patterns...

   [MATCH] /dev/input/event3 -> pattern "*"
           Name: AT Translated Set 2 keyboard
   [MATCH] /dev/input/event18 -> pattern "*"
           Name: USB Keyboard
           Serial: USB-12345

============================================================
RESULT: Configuration is valid. 2 of 2 device(s) matched.

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

### systemd Service (System-wide)

For system-wide operation with automatic startup:

**1. Create keyrx user:**

```bash
sudo useradd -r -s /usr/sbin/nologin -M -d /nonexistent keyrx
sudo usermod -aG input,uinput keyrx
```

**2. Create configuration directory:**

```bash
sudo mkdir -p /etc/keyrx
sudo cp my-config.krx /etc/keyrx/config.krx
```

**3. Install the service:**

```bash
sudo cp keyrx_daemon/systemd/keyrx.service /etc/systemd/system/
sudo systemctl daemon-reload
```

**4. Enable and start:**

```bash
sudo systemctl enable keyrx.service
sudo systemctl start keyrx.service
```

**Service management commands:**

```bash
# Start/stop/restart
sudo systemctl start keyrx
sudo systemctl stop keyrx
sudo systemctl restart keyrx

# Reload configuration (without restart)
sudo systemctl reload keyrx

# Check status
sudo systemctl status keyrx

# View logs
journalctl -u keyrx -f
```

### systemd Service (Per-user)

For single-user operation without root privileges:

**1. Create directories:**

```bash
mkdir -p ~/.config/systemd/user
mkdir -p ~/.config/keyrx
mkdir -p ~/.local/bin
```

**2. Install files:**

```bash
cp target/release/keyrx_daemon target/release/keyrx_compiler ~/.local/bin/
cp keyrx_daemon/systemd/keyrx-user.service ~/.config/systemd/user/keyrx.service
# The service runs the ACTIVE profile (web UI http://127.0.0.1:9867, or CLI):
keyrx_daemon profiles create my --template blank   # then edit ~/.config/keyrx/profiles/my.rhai
keyrx_daemon profiles activate my
```

Scope the profile to the keyboard you mean to remap: `device_start("*")`
also grabs virtual keyboards (remote-desktop/Sunshine) and anything plugged
in later. Use the name shown by `keyrx_daemon list-devices`, e.g.
`device_start("USB Keyboard");`, and check it with
`keyrx_daemon validate --config ~/.config/keyrx/profiles/my.krx`.

**3. Enable and start:**

```bash
systemctl --user daemon-reload
systemctl --user enable keyrx.service
systemctl --user start keyrx.service
```

**4. (Optional) Start at boot without login:**

```bash
loginctl enable-linger $USER
```

With lingering the service starts at boot (before login, so without a tray
icon - the web UI works; `systemctl --user restart keyrx` after login brings
the tray up). Without it, the service starts at login.

**Emergency stop:** hold Left Ctrl + Right Ctrl + Escape (works on the raw
keys even if the config is broken), or from another TTY / SSH session run
`systemctl --user stop keyrx` (SIGKILL is also safe: the kernel releases the
grab and the keyboard types normally again).

**Service management commands:**

```bash
# Start/stop/restart
systemctl --user start keyrx
systemctl --user stop keyrx
systemctl --user restart keyrx

# Reload configuration
systemctl --user reload keyrx

# Check status
systemctl --user status keyrx

# View logs
journalctl --user -u keyrx -f
```

## Configuration Management

### Hot Reload

Reload configuration without restarting the daemon:

```bash
# For systemd (system service)
sudo systemctl reload keyrx

# For systemd (user service)
systemctl --user reload keyrx

# For manual daemon (send SIGHUP)
kill -HUP $(pgrep keyrx_daemon)
```

Hot reload:
- Preserves current modifier and lock states
- Keeps device grabs active (no interruption)
- Applies new mappings immediately

### Multiple Devices

Configure different mappings for different keyboards:

```rhai
// Built-in laptop keyboard
device_start("AT Translated Set 2");
    map("CapsLock", "VK_Escape");
device_end();

// External USB keyboard with different layout
device_start("USB Keyboard");
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

Only devices a `device_start` pattern matches are grabbed - a keyboard
matched by no block is left alone entirely (not remapped, not intercepted).
Plugging in a keyboard (USB, or a Bluetooth keyboard reconnecting after
suspend) is picked up automatically, without restarting the daemon, as soon
as it matches a pattern; unplugging one is detected and released the same
way. Both react within about a poll cycle of `/dev/input` changing, with no
added input latency while typing.

### When no valid profile is live

If the active profile is missing, fails to compile, or its `.krx` cannot be
loaded, the daemon grabs **no keyboard at all** - it never falls back to
capturing everything. It keeps running and serving the web UI, IPC and REST so
you can fix the profile (or activate another), logs the reason, and reports it
as `config_error` in `keyrx_daemon status` (also `GET /api/status` and the MCP
status tool); `keyrx_daemon doctor` flags it. A reload that fails (bad edit,
SIGHUP) keeps the previous working config and reports the error the same way.

## Troubleshooting

Run `keyrx_daemon doctor` first - it checks input/uinput group membership
(both configured in `/etc/group` *and* active in your current login session,
since a group added by `usermod` needs a logout/login to take effect),
`/dev/uinput` access, whether the udev rule is installed, whether the config
directory and active profile resolve and compile, whether the daemon is
reachable over IPC, and whether the web UI is listening - with the fix for
whatever fails. `keyrx_daemon doctor --json` for scripting.

### Permission Denied

**Symptom:** `Error: Permission denied when accessing /dev/input/eventX`

**Solutions:**

1. Verify group membership:
   ```bash
   groups $USER  # Should include 'input' and 'uinput'
   ```

2. Check if you logged out after adding groups:
   ```bash
   # Quick fix without logout:
   newgrp input && newgrp uinput
   ```

3. Verify udev rules are installed:
   ```bash
   ls -la /etc/udev/rules.d/99-keyrx.rules
   ```

4. Reload udev rules:
   ```bash
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

5. Check device permissions:
   ```bash
   ls -la /dev/input/event* /dev/uinput
   ```

### No Devices Found

**Symptom:** `keyrx_daemon list-devices` shows no keyboards

**Solutions:**

1. Check if input devices exist:
   ```bash
   ls /dev/input/event*
   ```

2. Verify evdev module is loaded:
   ```bash
   lsmod | grep evdev
   sudo modprobe evdev
   ```

3. Check device permissions (may need udev rules).

### uinput Not Available

**Symptom:** `Error: Failed to open /dev/uinput`

**Solutions:**

1. Load uinput module:
   ```bash
   sudo modprobe uinput
   ```

2. Make it persistent (load at boot):
   ```bash
   echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
   ```

3. Verify device exists:
   ```bash
   ls -la /dev/uinput
   ```

### Daemon Starts but Keys Not Remapped

**Solutions:**

1. Check if device is matched:
   ```bash
   keyrx_daemon validate --config your-config.krx
   ```

2. Verify configuration syntax:
   ```bash
   keyrx_compiler verify your-config.krx
   ```

3. Run with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```
   `--debug` never logs which keys you press. If you need the key names to
   debug a mapping, add `--log-keys` (this makes the log a keylogger; turn it
   off again afterwards and do not share such logs).

### Editing a Profile While the Daemon Runs

The daemon watches the `.rhai` source of the profile it has loaded
(`~/.config/keyrx/profiles/<name>.rhai`, or under `$KEYRX_CONFIG_DIR`). When
you save it, the profile is recompiled and applied within a couple of seconds
without `profiles activate` or a restart. If the new source does not compile,
the daemon logs the error with its `file:line:column` and keeps running the
previous mapping:

```bash
journalctl --user -u keyrx -f
```

Pass `--no-watch` to `keyrx_daemon run` to turn this off. A daemon started
with `run --config FILE` (a pinned file, not a profile) is not watched.
`keyrx_daemon profiles activate NAME` returns only after the daemon is running
the new profile and exits non-zero if the daemon refuses it.

### Running a Second (Test) Instance Safely

To try a build or a profile without touching the keyboards your real daemon
manages, start the scratch daemon with its own config dir, port and runtime
dir, and narrow what it may grab:

```bash
KEYRX_CONFIG_DIR=/tmp/kx-test KEYRX_PORT=9890 XDG_RUNTIME_DIR=/tmp/kx-rt \
KEYRX_DEVICE_SCOPE='My Test Keyboard*' KEYRX_OUTPUT_NAME=keyrx-test \
  keyrx_daemon run
```

`KEYRX_DEVICE_SCOPE` (a glob on the keyboard name, default `*`) can only
narrow what the profile's `device_start()` patterns select, and
`KEYRX_OUTPUT_NAME` names the virtual output keyboard (default `keyrx`) so
tools can tell the instances apart.

### Keyboard Input Overflow

If keystrokes arrive much faster than the daemon can process them (for
example a stuck macro tool injecting thousands of events), the kernel drops
events for the grabbed keyboard (`SYN_DROPPED`). The daemon notices, resyncs
with the keyboard's real key state, releases any key it believed was held and
logs a warning ("lost input events"). The number of recoveries is shown by
`keyrx_daemon status` as "Input overflows" and by `/api/status`
(`input_overflows`); if it keeps growing, find what is flooding the keyboard.

### Emergency Escape: Keyboard Unusable From a Bad Config

If an active profile remaps a key you need (e.g. Escape) badly enough that
you cannot type, hold **Left Ctrl + Right Ctrl + Escape** together. The
daemon releases (ungrabs) every keyboard it holds and stops immediately, on
the raw physical keys, regardless of what the broken config maps them to.
Your keyboard goes back to normal system input right away; fix the config
(e.g. via the web UI or `keyrx_daemon profiles`) and start the daemon again
(`keyrx_daemon run`, or `systemctl --user restart keyrx`).

### Keys Stuck After Crash

If the daemon crashes while a key is held, it may appear "stuck."

**Solutions:**

1. Press and release the stuck key.
2. If using modifiers, press and release all modifier keys.
3. As last resort: `xdotool key --clearmodifiers Return`

### Service Fails to Start

**Symptom:** `systemctl status keyrx` shows failed

**Solutions:**

1. Check logs for details:
   ```bash
   journalctl -u keyrx --no-pager -n 50
   ```

2. Verify configuration path exists:
   ```bash
   ls -la /etc/keyrx/config.krx  # system service
   ls -la ~/.config/keyrx/config.krx  # user service
   ```

3. Test configuration manually:
   ```bash
   /usr/local/bin/keyrx_daemon validate --config /etc/keyrx/config.krx
   ```

4. Check keyrx user permissions (system service):
   ```bash
   sudo -u keyrx groups  # Should show input, uinput
   ```

## Security Considerations

### Group Access Risks

Users in the `input` group can:
- Read all input devices (keyboards, mice, etc.)
- Potentially capture sensitive input (passwords)

Only add trusted users to the `input` group.

### Virtual Keyboard Risks

Users in the `uinput` group can:
- Create virtual input devices
- Inject arbitrary keyboard/mouse events

This could be used to:
- Automate actions
- Simulate user input

Only add trusted users to the `uinput` group.

### systemd Security Hardening

The system-wide unit (`keyrx_daemon/systemd/keyrx.service`) includes security
hardening. The per-user unit (`keyrx_daemon/systemd/keyrx-user.service`) does
**not** set these — it runs as your own login user, so most of them would be
redundant or would block access to `~/.config/keyrx`:

- `NoNewPrivileges=yes` - Prevents privilege escalation
- `ProtectSystem=strict` - Read-only filesystem
- `ProtectHome=yes` - No home directory access
- `DeviceAllow=...` - Restricted device access
- `SystemCallFilter=...` - Limited system calls

Note: `PrivateTmp` is deliberately *not* set on `keyrx.service` either, since
the daemon's IPC socket lives at `/tmp/keyrx-daemon.sock` and must stay
visible to `keyrx_daemon status`/`profiles activate` run from a normal shell.

### Best Practices

1. Use a dedicated `keyrx` user for the system service
2. Keep the daemon binary read-only
3. Store configuration in `/etc/keyrx/` (system) or `~/.config/keyrx/` (user)
4. Review and test configurations before deploying
5. Monitor daemon logs for unusual activity
