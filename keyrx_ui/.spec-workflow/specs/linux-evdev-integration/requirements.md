# Requirements Document

## Introduction

The Linux evdev Integration spec implements real keyboard interception and injection on Linux, enabling keyrx to function as a production-ready keyboard remapper. This phase bridges the gap between the mock platform layer (Phase 2) and actual hardware interaction, using Linux's evdev subsystem for input capture and uinput for output injection.

**Current State:** The runtime engine (keyrx_core/runtime) provides event processing with <1ms latency. Platform traits (InputDevice, OutputDevice) are defined with mock implementations for testing. The daemon can process events end-to-end using mock devices.

**This Spec Delivers:** A fully functional Linux daemon that intercepts real keyboard input via evdev, applies configured remappings, and injects output via uinput—achieving the "CapsLock → Escape" end-to-end workflow on real hardware.

## Alignment with Product Vision

This spec directly enables the core value proposition from product.md:

**From product.md - Core Value Proposition:**
> "keyrx delivers firmware-class performance (<1ms latency) with software-level flexibility"

**How This Spec Delivers:**
- **Real hardware interaction:** Direct evdev kernel interface with minimal userspace overhead
- **Exclusive device grab:** EVIOCGRAB ensures no events leak to other applications
- **Virtual device injection:** uinput creates seamless virtual keyboard for output
- **Sub-millisecond latency:** Efficient event processing with zero-copy where possible

**From product.md - Cross-Platform OS Integration:**
> "Linux: evdev/uinput with EVIOCGRAB for kernel-level interception"
> "Device-specific configuration via serial number matching"

**How This Spec Serves Them:**
- **evdev input capture:** Implement InputDevice trait using evdev crate
- **uinput output injection:** Implement OutputDevice trait using uinput crate
- **Device identification:** Match devices by serial number pattern for per-keyboard configs
- **Proper permissions:** udev rules for non-root usage

**From product.md - Product Principles:**
> "AI Coding Agent First: CLI-first design, structured logging"

**How This Spec Enables:**
- **CLI daemon:** `keyrx_daemon --config config.krx` for scriptable deployment
- **Structured logging:** JSON logs for daemon lifecycle and event processing
- **Signal handling:** Graceful shutdown on SIGTERM/SIGINT
- **systemd integration:** Service file for production deployment

## Requirements

### Requirement 1: evdev Input Device Discovery and Capture

**User Story:** As a keyrx daemon, I want to discover and capture keyboard input devices via evdev, so that I can intercept physical keyboard events before they reach applications.

#### Acceptance Criteria

1. **WHEN** daemon starts **THEN** system **SHALL** enumerate input devices in `/dev/input/`
   - Scan `/dev/input/event*` devices
   - Filter for keyboard devices (EV_KEY capability with alphanumeric keys)
   - Log discovered devices with name and path

2. **WHEN** device matches configuration pattern **THEN** system **SHALL** grab device exclusively
   - Use EVIOCGRAB ioctl for exclusive access
   - Prevent events from reaching other applications
   - Return DeviceError::PermissionDenied if grab fails (usually permission issue)

3. **WHEN** device has serial number **THEN** system **SHALL** extract and match against config
   - Read serial from `/sys/class/input/eventX/device/id/serial` or via ioctl
   - Support glob patterns (e.g., `"USB\\VID_04D9*"`) for matching
   - Support wildcard `"*"` to match all devices

4. **WHEN** reading events from grabbed device **THEN** system **SHALL** convert to KeyEvent
   - Map evdev event codes to KeyCode enum
   - Distinguish EV_KEY value 1 (press) from 0 (release)
   - Handle key repeat (value 2) appropriately (ignore or pass through)

5. **WHEN** device is disconnected **THEN** system **SHALL** handle gracefully
   - Detect read errors indicating device removal
   - Log device disconnection at WARN level
   - Continue processing other devices if available

6. **WHEN** no matching devices found **THEN** system **SHALL** return informative error
   - List available devices in error message
   - Suggest pattern fixes or permission checks

### Requirement 2: uinput Virtual Device Creation and Injection

**User Story:** As a keyrx daemon, I want to inject remapped key events via uinput, so that applications receive the transformed input as if from a real keyboard.

#### Acceptance Criteria

1. **WHEN** daemon initializes **THEN** system **SHALL** create uinput virtual device
   - Open `/dev/uinput` (requires appropriate permissions)
   - Configure device with keyboard capabilities (all EV_KEY codes)
   - Set device name to "keyrx virtual keyboard"
   - Return DeviceError::PermissionDenied if creation fails

2. **WHEN** injecting key event **THEN** system **SHALL** write correct uinput event
   - Convert KeyEvent::Press to EV_KEY with value 1
   - Convert KeyEvent::Release to EV_KEY with value 0
   - Write EV_SYN/SYN_REPORT after each key event
   - Return DeviceError::InjectionFailed on write error

3. **WHEN** injecting modified output (Shift+Key) **THEN** system **SHALL** sequence correctly
   - Inject modifier press, then key press, then sync
   - Inject key release, then modifier release, then sync
   - Maintain correct timing (no artificial delays needed)

4. **WHEN** daemon shuts down **THEN** system **SHALL** clean up uinput device
   - Release any held keys (inject release events)
   - Destroy virtual device properly
   - Close file descriptor

5. **WHEN** uinput device creation fails **THEN** system **SHALL** provide actionable error
   - Check `/dev/uinput` existence
   - Check permission issues
   - Suggest udev rules or running as root

### Requirement 3: Device Pattern Matching and Multi-Device Support

**User Story:** As a user with multiple keyboards, I want to apply different configurations to different devices, so that each keyboard behaves according to its specific mapping.

#### Acceptance Criteria

1. **WHEN** config has device-specific patterns **THEN** system **SHALL** match devices correctly
   - Parse DeviceConfig.identifier.pattern field
   - Match against device serial number or name
   - Support multiple DeviceConfigs with different patterns

2. **WHEN** multiple devices match same pattern **THEN** system **SHALL** apply config to all
   - Create separate DeviceState per physical device
   - Maintain independent modifier/lock state per device
   - Log each matched device with its applied configuration

3. **WHEN** device pattern is wildcard `"*"` **THEN** system **SHALL** match all keyboards
   - Apply as fallback configuration
   - Lower priority than specific patterns

4. **WHEN** device matches no pattern **THEN** system **SHALL** pass through unchanged
   - Log at INFO level that device has no matching config
   - Do not grab device (let events pass to system)

5. **WHEN** config has cross-device modifiers (future) **THEN** system **SHALL** stub behavior
   - Log TODO for cross-device modifier sharing
   - Document as Phase 4 enhancement

### Requirement 4: Daemon Lifecycle Management

**User Story:** As a system administrator, I want the keyrx daemon to run reliably as a service, so that keyboard remapping is always active and recoverable from errors.

#### Acceptance Criteria

1. **WHEN** daemon starts **THEN** system **SHALL** initialize in order
   - Parse CLI arguments (--config path, --debug flag)
   - Load and validate .krx configuration
   - Discover and grab matching input devices
   - Create uinput virtual device
   - Enter event processing loop
   - Log startup completion with device count

2. **WHEN** receiving SIGTERM or SIGINT **THEN** system **SHALL** shutdown gracefully
   - Stop event processing loop
   - Release grabbed devices (EVIOCGRAB ioctl with 0)
   - Destroy uinput virtual device
   - Log shutdown at INFO level
   - Exit with code 0

3. **WHEN** receiving SIGHUP **THEN** system **SHALL** reload configuration
   - Load new .krx file
   - Rebuild lookup tables
   - Continue processing with new mappings
   - Log reload at INFO level

4. **WHEN** fatal error occurs **THEN** system **SHALL** exit with meaningful code
   - Exit code 1: Configuration error (file not found, parse error)
   - Exit code 2: Permission error (cannot grab device, cannot create uinput)
   - Exit code 3: Runtime error (device disconnected with no fallback)
   - Log error with full context before exit

5. **WHEN** running with --debug flag **THEN** system **SHALL** enable verbose logging
   - Log every key event at DEBUG level
   - Log state transitions (modifier/lock changes)
   - Log performance metrics (processing latency)

### Requirement 5: Permission Management and udev Rules

**User Story:** As a user, I want to run keyrx without root privileges, so that I can use it safely without elevated permissions.

#### Acceptance Criteria

1. **WHEN** user is in `input` group **THEN** system **SHALL** access evdev devices
   - udev rule grants read access to `/dev/input/event*` for input group
   - Document group addition: `sudo usermod -aG input $USER`

2. **WHEN** user is in `uinput` group **THEN** system **SHALL** create virtual devices
   - udev rule grants write access to `/dev/uinput` for uinput group
   - Create uinput group if not exists

3. **WHEN** udev rules not installed **THEN** daemon **SHALL** provide helpful error
   - Detect permission denied errors
   - Print instructions for installing udev rules
   - Provide path to rules file in package

4. **WHEN** installing keyrx **THEN** package **SHALL** include udev rules
   - File: `/etc/udev/rules.d/99-keyrx.rules`
   - Rules for both input device access and uinput creation
   - Reload command: `sudo udevadm control --reload-rules`

5. **WHEN** running as root **THEN** system **SHALL** work without additional setup
   - Root bypasses udev permission checks
   - Log warning recommending non-root usage for security

### Requirement 6: systemd Service Integration

**User Story:** As a system administrator, I want to manage keyrx via systemd, so that it starts automatically and can be monitored like other services.

#### Acceptance Criteria

1. **WHEN** installing keyrx **THEN** package **SHALL** include systemd service file
   - File: `/etc/systemd/system/keyrx.service` or user service
   - Type=simple (daemon runs in foreground)
   - ExecStart with --config path
   - Restart=on-failure for automatic recovery

2. **WHEN** service starts **THEN** system **SHALL** log to journald
   - Use syslog or stdout (captured by journald)
   - Include structured JSON for parsing
   - Enable filtering: `journalctl -u keyrx -f`

3. **WHEN** service fails **THEN** systemd **SHALL** restart automatically
   - RestartSec=1 for quick recovery
   - Limit restart attempts to prevent loop
   - Log restart reason

4. **WHEN** user runs `systemctl status keyrx` **THEN** output **SHALL** show health
   - Active/inactive status
   - Recent log entries
   - PID and uptime

5. **WHEN** config file changes **THEN** user **SHALL** be able to reload
   - `systemctl reload keyrx` sends SIGHUP
   - Service continues running with new config
   - Log reload success/failure

### Requirement 7: Error Handling and Recovery

**User Story:** As a user, I want keyrx to handle errors gracefully and recover when possible, so that my keyboard doesn't become unusable due to transient issues.

#### Acceptance Criteria

1. **WHEN** device read fails transiently **THEN** system **SHALL** retry
   - Distinguish transient errors (EAGAIN) from fatal (device removed)
   - Log warning and continue for transient errors
   - Only exit on unrecoverable errors

2. **WHEN** uinput write fails **THEN** system **SHALL** attempt recovery
   - Log error with event details
   - Continue processing (don't drop subsequent events)
   - Consider recreating uinput device on repeated failures

3. **WHEN** all input devices lost **THEN** system **SHALL** wait for reconnection
   - Enter polling mode for device discovery
   - Check for new devices every second
   - Resume normal operation when device appears
   - Timeout after configurable period (default: infinite wait)

4. **WHEN** panic occurs in event processing **THEN** system **SHALL** NOT crash
   - Catch panics at event loop boundary
   - Log panic with backtrace
   - Continue processing other events
   - Note: This is defense-in-depth; panics should not occur

5. **WHEN** event processing exceeds latency threshold **THEN** system **SHALL** log warning
   - Threshold: 10ms (10x normal budget)
   - Log at WARN level with processing details
   - Help identify performance regressions

## Non-Functional Requirements

### Performance

- **Event Processing Latency**: <1ms end-to-end (maintained from Phase 2)
- **Device Discovery**: <100ms to enumerate and match devices
- **Event Read Latency**: <100μs from kernel to daemon (evdev is efficient)
- **Event Write Latency**: <100μs from daemon to kernel (uinput is efficient)
- **Memory Usage**: <50MB total (including all device states and lookup tables)
- **CPU Usage**: <1% idle, <5% under heavy typing load

### Security

- **Least Privilege**: Run as non-root user with minimal group memberships
- **No Network Access**: Daemon should not open any network connections
- **Input Validation**: Validate all evdev event codes before processing
- **Fail-Safe Mode**: If daemon crashes, grabbed devices are auto-released by kernel
- **No Key Logging**: Never log actual key content (only keycodes for debugging)

### Reliability

- **Graceful Degradation**: Continue with available devices if some fail
- **Auto-Recovery**: Reconnect to devices after transient disconnection
- **Clean Shutdown**: Always release resources (no orphaned virtual devices)
- **Watchdog**: Optional systemd watchdog integration for health monitoring
- **No Event Loss**: Every input event must produce output (even if passthrough)

### Usability

- **Clear Error Messages**: All errors include actionable remediation steps
- **Verbose Logging**: --debug flag enables detailed diagnostics
- **Device Listing**: Subcommand to list available devices for configuration
- **Dry-Run Mode**: Subcommand to validate config without grabbing devices
- **Status Reporting**: API or command to query current daemon state
