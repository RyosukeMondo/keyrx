# Requirements Document: CLI-First Configuration Management

## Introduction

KeyRx2 currently provides powerful keyboard remapping through Rhai DSL compilation, but lacks comprehensive configuration management. Users must manually edit Rhai scripts and manage configuration files through the filesystem.

This specification adds **profile management, device naming, and configuration tooling** via CLI commands. The philosophy: **CLI provides complete functionality autonomously, enabling deterministic testing and automation.**

**Key Principle**: Every feature must be testable via CLI commands in automated scripts, enabling AI agents to verify functionality deterministically without human interaction.

**Web UI**: A separate spec (`web-ui-configuration-editor`) will add optional web-based visual configuration after CLI v1.0 is complete.

## Alignment with Product Vision

From `product.md`:
- **AI-First Development**: "Zero Manual Testing" - all features must be verifiable via CLI automation
- **Single Source of Truth (SSOT)**: `.krx` binary files remain the only runtime config source
- **Deterministic Simulation Testing**: All configuration changes must be verifiable with DST
- **CLI First, GUI Later**: Development guidelines explicitly prioritize CLI interfaces

From `tech.md`:
- **Browser-based UI**: Optional WASM-based configuration editor (Phase 6, not core requirement)
- **JSON Structured Logging**: All CLI commands output machine-parseable JSON for AI consumption

## Requirements

### Requirement 1: Device Registry Management

**User Story:** As a user, I want to assign persistent names to my keyboards, so that I can distinguish "Left Numpad" from "Main Keyboard" in configurations.

#### Acceptance Criteria

1. WHEN user runs `keyrx devices list` THEN system SHALL output JSON with all connected keyboards (id, name, serial, path, status)
2. WHEN user runs `keyrx devices rename <id> "Left Numpad"` THEN system SHALL persist name to `~/.config/keyrx/devices.json`
3. WHEN daemon restarts AND device reconnects with same serial THEN system SHALL use persisted name
4. WHEN user runs `keyrx devices set-scope <id> device-specific` THEN system SHALL mark device for serial-specific configuration
5. WHEN user runs `keyrx devices set-scope <id> global` THEN system SHALL apply configurations globally (any device)
6. WHEN device has never been named THEN `keyrx devices list` SHALL show auto-detected name with `unnamed: true` flag
7. WHEN user runs `keyrx devices forget <id>` THEN system SHALL remove from registry and delete device-specific configs

#### Input Validation

8. WHEN device name exceeds 64 characters THEN system SHALL reject with error "Device name too long (max 64 characters)"
9. WHEN device name contains invalid characters (non-alphanumeric except space, dash, underscore) THEN system SHALL reject with error "Invalid characters in name"
10. WHEN device ID exceeds 256 characters THEN system SHALL reject (prevent DoS)
11. WHEN device ID not found THEN system SHALL return error code 1001 "Device not found" with available device IDs

#### Error Scenarios

12. WHEN `devices.json` is corrupted THEN system SHALL log warning, backup corrupted file to `devices.json.bak`, create new empty registry
13. WHEN `devices.json` is read-only THEN system SHALL return error "Permission denied: cannot write to ~/.config/keyrx/devices.json"
14. WHEN disk is full during save THEN system SHALL return error "Disk full: cannot save device registry" and keep previous version
15. WHEN two CLIs call rename simultaneously THEN system SHALL serialize writes (second waits for first) via file lock

**CLI Examples:**
```bash
# List all devices (JSON output for AI parsing)
keyrx devices list --json
# Output: {"devices": [{"id": "USB123", "name": "Left Numpad", "serial": "ABC", "scope": "device-specific", "unnamed": false}]}

# Rename device
keyrx devices rename USB123 "Stream Deck Clone"

# Change scope
keyrx devices set-scope USB123 device-specific

# Forget device
keyrx devices forget USB123 --confirm
```

---

### Requirement 2: Profile Management

**User Story:** As a user, I want to create and switch between multiple keyboard configurations (gaming, work, streaming), so that I can adapt my keyboard behavior to different contexts.

#### Acceptance Criteria

1. WHEN user runs `keyrx profiles list` THEN system SHALL output all profiles with metadata (name, devices, layers, modified_at)
2. WHEN user runs `keyrx profiles create gaming --template qmk-layers` THEN system SHALL create `~/.config/keyrx/profiles/gaming.rhai` from template
3. WHEN user runs `keyrx profiles activate gaming` THEN system SHALL compile to .krx and hot-reload daemon without restart
4. WHEN compilation fails THEN system SHALL keep previous profile active and output JSON error with line numbers
5. WHEN user runs `keyrx profiles duplicate default gaming` THEN system SHALL copy default.rhai → gaming.rhai
6. WHEN user runs `keyrx profiles delete gaming` THEN system SHALL require `--confirm` flag and delete both .rhai and .krx files
7. WHEN user runs `keyrx profiles export gaming /path/to/backup.rhai` THEN system SHALL copy profile with metadata
8. WHEN user runs `keyrx profiles import /path/to/backup.rhai` THEN system SHALL validate and install profile

#### Input Validation

9. WHEN profile name exceeds 32 characters THEN system SHALL reject with error "Profile name too long (max 32 characters)"
10. WHEN profile name contains invalid characters (non-alphanumeric except dash, underscore) THEN system SHALL reject with error "Invalid characters in profile name (use a-z, 0-9, -, _)"
11. WHEN profile name is reserved word (default, system, temp) THEN system SHALL reject with error "Reserved profile name"
12. WHEN user creates more than 100 profiles THEN system SHALL reject with error "Maximum 100 profiles allowed"
13. WHEN layer count exceeds 16 THEN system SHALL reject with error "Maximum 16 layers per profile"

#### Error Scenarios

14. WHEN profile compilation times out (>30 seconds) THEN system SHALL kill compilation process, return error "Compilation timeout", keep previous profile active
15. WHEN daemon crashes during hot-reload THEN system SHALL auto-restart daemon with previous working profile
16. WHEN two CLIs call activate simultaneously THEN system SHALL serialize (second waits for first compilation) via RwLock
17. WHEN .rhai file is corrupted THEN system SHALL return error with Rhai parser error message and line number
18. WHEN importing profile with same name as existing THEN system SHALL return error "Profile 'gaming' already exists. Use --overwrite flag to replace"
19. WHEN disk full during compilation THEN system SHALL return error "Disk full: cannot write .krx file" and keep previous profile

**CLI Examples:**
```bash
# List profiles
keyrx profiles list --json
# Output: {"profiles": [{"name": "default", "active": true, "devices": ["USB123"], "layers": 3, "modified_at": 1735459200}]}

# Create new profile
keyrx profiles create gaming --template qmk-layers --layers 5

# Switch profile (hot-reload)
keyrx profiles activate gaming
# Output: {"success": true, "compile_time_ms": 45, "reload_time_ms": 12}

# Export/import
keyrx profiles export gaming ~/backup/gaming-2025.rhai
keyrx profiles import ~/downloads/streamer-setup.rhai --name streaming
```

---

### Requirement 3: Key Mapping Configuration via CLI

**User Story:** As a user, I want to configure key mappings via CLI commands, so that I can script configuration changes and test them in CI/CD.

#### Acceptance Criteria

1. WHEN user runs `keyrx config set-key <profile> <layer> <key> --tap <output>` THEN system SHALL update Rhai source and recompile
2. WHEN user runs `keyrx config set-key <profile> base CapsLock --tap Escape --hold MD_00 --threshold 200` THEN system SHALL create tap-hold mapping
3. WHEN user runs `keyrx config set-key <profile> base A --macro "Ctrl+C,wait:100,Ctrl+V"` THEN system SHALL generate macro Rhai code
4. WHEN user runs `keyrx config get-key <profile> <layer> <key>` THEN system SHALL output current mapping as JSON
5. WHEN user runs `keyrx config delete-key <profile> <layer> <key>` THEN system SHALL remove mapping and revert to passthrough
6. WHEN recompilation fails THEN system SHALL output error JSON with line number and keep old .krx active
7. WHEN user runs `keyrx config validate <profile>` THEN system SHALL dry-run compilation and report errors without applying

#### Input Validation

8. WHEN key name is invalid (not in key code table) THEN system SHALL reject with error "Unknown key: <name>. Run 'keyrx keys list' for valid keys"
9. WHEN modifier ID exceeds MD_254 THEN system SHALL reject with error "Invalid modifier (max MD_254)"
10. WHEN lock ID exceeds LK_254 THEN system SHALL reject with error "Invalid lock (max LK_254)"
11. WHEN tap-hold threshold < 50ms THEN system SHALL reject with error "Threshold too low (min 50ms)"
12. WHEN tap-hold threshold > 5000ms THEN system SHALL warn but accept
13. WHEN macro sequence exceeds 100 actions THEN system SHALL reject with error "Macro too long (max 100 actions)"
14. WHEN macro wait exceeds 10000ms THEN system SHALL reject with error "Macro wait too long (max 10s)"

#### Error Scenarios

15. WHEN layer ID does not exist THEN system SHALL return error "Layer <id> not found in profile <name>"
16. WHEN setting key on non-existent profile THEN system SHALL return error "Profile <name> not found"
17. WHEN Rhai code generation fails THEN system SHALL return error with specific cause (circular dependency, syntax error, etc.)
18. WHEN validation detects infinite loop THEN system SHALL return error "Circular mapping detected: A→B→C→A"

**CLI Examples:**
```bash
# Simple remap
keyrx config set-key default base CapsLock --remap Escape

# Tap-hold
keyrx config set-key default base CapsLock --tap Escape --hold MD_00 --threshold 200

# Macro
keyrx config set-key default base Numpad7 --macro "Ctrl+C,wait:100,Ctrl+V"

# Query current mapping
keyrx config get-key default base CapsLock --json
# Output: {"key": "CapsLock", "action": {"type": "tap_hold", "tap": "Escape", "hold": "MD_00", "threshold_ms": 200}}

# Validate without applying
keyrx config validate gaming --json
# Output: {"valid": false, "errors": [{"line": 42, "message": "Undefined modifier MD_99"}]}
```

---

### Requirement 4: Layer Management

**User Story:** As a user, I want to create and manage layers (QMK-style), so that I can organize complex key mappings hierarchically.

#### Acceptance Criteria

1. WHEN user runs `keyrx layers list <profile>` THEN system SHALL output all layers with IDs, names, activation modes
2. WHEN user runs `keyrx layers create <profile> "Numlock" --modifier MD_01` THEN system SHALL add layer to Rhai config
3. WHEN user runs `keyrx layers rename <profile> MD_01 "Symbols"` THEN system SHALL update layer name in Rhai
4. WHEN user runs `keyrx layers delete <profile> MD_01` THEN system SHALL remove layer and all its mappings with `--confirm`
5. WHEN user runs `keyrx layers show <profile> MD_01` THEN system SHALL output all keys configured in that layer

#### Input Validation

6. WHEN layer name exceeds 32 characters THEN system SHALL reject with error "Layer name too long (max 32 characters)"
7. WHEN creating layer MD_00 (reserved for base) THEN system SHALL reject with error "MD_00 is reserved for base layer"
8. WHEN creating layer with ID > MD_15 THEN system SHALL warn "More than 16 layers may impact performance"
9. WHEN deleting base layer (MD_00) THEN system SHALL reject with error "Cannot delete base layer"

#### Error Scenarios

10. WHEN layer ID already exists THEN system SHALL return error "Layer <id> already exists in profile <name>"
11. WHEN renaming non-existent layer THEN system SHALL return error "Layer <id> not found"
12. WHEN deleting layer referenced by mappings THEN system SHALL return error "Cannot delete layer with active mappings. Delete mappings first or use --force"

**CLI Examples:**
```bash
# List layers
keyrx layers list default --json
# Output: {"layers": [{"id": "MD_00", "name": "Base", "mode": "always_active"}, {"id": "MD_01", "name": "Numlock", "mode": "modifier"}]}

# Create layer
keyrx layers create default "Function" --modifier MD_02

# Show layer mappings
keyrx layers show default MD_01 --json
# Output: {"layer": "MD_01", "keys": [{"key": "Numpad7", "action": {"type": "simple", "output": "F7"}}]}
```

---

### Requirement 5: Configuration Inspection and Debugging

**User Story:** As a developer, I want to inspect active configuration state, so that I can debug mapping issues and verify behavior.

#### Acceptance Criteria

1. WHEN user runs `keyrx status` THEN system SHALL output daemon status, active profile, device count, uptime
2. WHEN user runs `keyrx config show <profile>` THEN system SHALL output compiled .krx metadata (hash, size, device mappings)
3. WHEN user runs `keyrx config diff <profile1> <profile2>` THEN system SHALL show key mapping differences
4. WHEN user runs `keyrx state inspect` THEN system SHALL output current modifier/lock state (255-bit vectors)
5. WHEN user runs `keyrx metrics latency` THEN system SHALL output latency statistics (min, avg, max, p95, p99)
6. WHEN user runs `keyrx events tail -n 100` THEN system SHALL stream last 100 keyboard events with timestamps

#### Input Validation

7. WHEN events tail -n exceeds 10000 THEN system SHALL reject with error "Cannot tail more than 10K events (memory limit)"
8. WHEN metrics query on non-running daemon THEN system SHALL return error "Daemon not running. Start with 'keyrx daemon start'"

#### Error Scenarios

9. WHEN IPC socket not found (/tmp/keyrx-daemon.sock) THEN system SHALL return error "Cannot connect to daemon. Is it running?"
10. WHEN IPC timeout (>5 seconds) THEN system SHALL return error "Daemon not responding (timeout)"
11. WHEN daemon crashes mid-query THEN system SHALL return error "Daemon connection lost"

**CLI Examples:**
```bash
# Daemon status
keyrx status --json
# Output: {"running": true, "uptime_seconds": 86400, "active_profile": "default", "device_count": 2}

# Inspect state
keyrx state inspect --json
# Output: {"modifiers": [0, 5], "locks": [2], "active_layers": ["MD_00", "MD_05"]}

# Latency metrics
keyrx metrics latency --json
# Output: {"min_us": 120, "avg_us": 350, "max_us": 980, "p95_us": 650, "p99_us": 820}

# Event tail
keyrx events tail -n 100 --follow --json
# Output: [{"timestamp_us": 1735459200000, "device_id": "USB123", "key": "CapsLock", "event": "press"}]
```

---

### Requirement 6: Keyboard Layout Presets

**User Story:** As a user, I want to use keyboard layout presets (ANSI, ISO, JIS, numpad), so that I can visualize my keyboard correctly.

#### Acceptance Criteria

1. WHEN user runs `keyrx layouts list` THEN system SHALL show built-in layouts (ansi_104, iso_105, jis_109, hhkb, numpad)
2. WHEN user runs `keyrx layouts show ansi_104` THEN system SHALL output keyboard-layout-editor (KLE) JSON format
3. WHEN user runs `keyrx layouts import ~/custom.json` THEN system SHALL validate KLE JSON and save to `~/.config/keyrx/layouts/`
4. WHEN user runs `keyrx devices set-layout <id> ansi_104` THEN system SHALL associate layout with device
5. WHEN layout is invalid JSON THEN system SHALL reject with validation error

#### Input Validation

6. WHEN layout name exceeds 32 characters THEN system SHALL reject with error "Layout name too long (max 32 characters)"
7. WHEN layout file exceeds 1MB THEN system SHALL reject with error "Layout file too large (max 1MB)"
8. WHEN KLE JSON has >200 keys THEN system SHALL reject with error "Layout too complex (max 200 keys)"
9. WHEN importing builtin layout name THEN system SHALL reject with error "Cannot overwrite builtin layout '<name>'"

#### Error Scenarios

10. WHEN KLE JSON missing required fields THEN system SHALL return error "Invalid KLE format: missing '<field>' field"
11. WHEN layout file not found THEN system SHALL return error "Layout file not found: <path>"
12. WHEN user has >50 custom layouts THEN system SHALL reject with error "Maximum 50 custom layouts allowed"

**CLI Examples:**
```bash
# List layouts
keyrx layouts list --json
# Output: {"layouts": ["ansi_104", "iso_105", "jis_109", "hhkb", "numpad", "custom_numpad"]}

# Show layout
keyrx layouts show ansi_104 --json > ansi.json

# Import custom layout
keyrx layouts import ~/Downloads/ergodox.json --name ergodox

# Assign layout to device
keyrx devices set-layout USB123 numpad
```

---

### Requirement 7: Simulation and Testing

**User Story:** As an AI agent, I want to simulate keyboard events and verify output, so that I can test configurations without physical keyboards.

#### Acceptance Criteria

1. WHEN user runs `keyrx simulate <profile> --events press:CapsLock,wait:50,release:CapsLock` THEN system SHALL use WASM core to simulate and output results
2. WHEN tap-hold threshold is 200ms AND wait is 50ms THEN simulation SHALL output tap action (Escape)
3. WHEN user runs `keyrx simulate <profile> --events-file scenario.json` THEN system SHALL replay event sequence deterministically
4. WHEN user runs `keyrx simulate <profile> --events-file scenario.json --seed 12345` THEN repeated runs SHALL produce identical output
5. WHEN simulation includes multiple devices THEN system SHALL support device IDs in event file
6. WHEN user runs `keyrx test <profile> --scenario tap-hold-under-threshold` THEN system SHALL run built-in test scenario

#### Input Validation

7. WHEN event sequence exceeds 100,000 events THEN system SHALL reject with error "Event sequence too long (max 100K events)"
8. WHEN event timestamp is negative THEN system SHALL reject with error "Invalid timestamp (must be >=0)"
9. WHEN event file exceeds 10MB THEN system SHALL reject with error "Event file too large (max 10MB)"
10. WHEN seed exceeds u64::MAX THEN system SHALL use seed % u64::MAX

#### Error Scenarios

11. WHEN profile not compiled (.krx missing) THEN system SHALL auto-compile from .rhai before simulating
12. WHEN event file has invalid JSON THEN system SHALL return error with JSON parser error and line number
13. WHEN simulation runs out of memory THEN system SHALL return error "Simulation memory limit exceeded (max 1GB)"
14. WHEN built-in scenario not found THEN system SHALL return error "Unknown scenario: <name>. Run 'keyrx test list-scenarios' for available scenarios"

**CLI Examples:**
```bash
# Inline simulation
keyrx simulate default --events "press:CapsLock,wait:50,release:CapsLock" --json
# Output: {"input": [...], "output": [{"key": "Escape", "event": "press"}, {"key": "Escape", "event": "release"}]}

# File-based replay (deterministic)
keyrx simulate default --events-file test-scenario.json --seed 42
# Output: {"deterministic": true, "hash": "abc123", "output": [...]}

# Built-in test scenarios
keyrx test default --scenario all --json
# Output: {"passed": 15, "failed": 0, "scenarios": ["tap-hold", "permissive-hold", "cross-device-modifiers"]}
```

---

**Note**: Web UI requirements have been moved to a separate spec: `web-ui-configuration-editor`. This spec focuses purely on CLI-first implementation.

---

## Non-Functional Requirements

### Code Architecture and Modularity

- **CLI-First Design**: Every web API endpoint must have CLI command equivalent
- **Single Responsibility**: Profile manager, device registry, layout manager as separate modules
- **Dependency Injection**: All file I/O abstracted via traits for testing
- **Clear Interfaces**: `ProfileManager::activate(&mut self, name: &str) -> Result<Profile, Error>`

### Autonomous Testing

- **Zero Manual UAT**: All features testable via CLI automation
- **Deterministic Simulation**: `keyrx simulate` produces identical output for same input+seed
- **JSON Output**: All CLI commands support `--json` for machine parsing
- **Exit Codes**: 0 (success), 1 (error), 2 (warning) for shell scripting

### Performance

- Hot-reload profile switch: < 100ms
- CLI command response: < 50ms (excluding compilation)
- Device listing: < 10ms
- Simulation throughput: > 10,000 events/sec

### Security

- Config files only writable by user (chmod 600)
- No remote access (localhost-only web server)
- Input validation on all CLI arguments (prevent path traversal)
- Rate limiting on hot-reload (max 10/minute)

### Reliability

- Atomic file writes (write to .tmp, rename)
- Rollback on compilation failure
- Graceful handling of missing files (create defaults)
- No daemon restart required for any configuration change

### Usability

- Clear error messages: `Error: Profile 'gaming' not found. Run 'keyrx profiles list' to see available profiles.`
- Progress indicators for long operations (compilation)
- `--help` on all commands with examples
- `--dry-run` option for destructive operations

### Compatibility

- Linux: Ubuntu 20.04+, Arch, Fedora
- Windows: 10/11
- File format: JSON (devices.json), Rhai (profiles/*.rhai), rkyv (.krx binary)
- KLE layout compatibility: keyboard-layout-editor.com JSON format
