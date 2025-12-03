# Requirements Document

## Introduction

This spec bridges the gap between KeyRx's powerful CLI capabilities and the Flutter UI. Currently, only 3 of 12 CLI commands are fully wired to the UI. This leaves 9 essential features inaccessible from the graphical interface.

**Goal:** Expose all CLI functionality through the Flutter UI with **clear separation between User and Developer interfaces** - keeping the user experience simple while containing advanced complexity in a dedicated developer area.

## Alignment with Product Vision

From product.md:
- **CLI First, GUI Later**: CLI is complete; now it's time for "GUI Later"
- **Developer Experience**: Full feature parity between CLI and UI
- **Visual Debugging**: UI should provide enhanced visualization over CLI

From tech.md:
- **FFI Bridge**: Extend existing `ui/lib/ffi/` bindings for new functionality
- **Flutter Architecture**: Follow established page/widget patterns
- **Cross-Platform**: UI must work on Linux and Windows

## UI Organization Philosophy

### Two-Tier Interface

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                               │
│  Simple, essential features for daily keyboard usage                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐               │
│  │Training │  │ Editor  │  │ Devices │  │  Run    │               │
│  │         │  │         │  │         │  │Controls │               │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘               │
├─────────────────────────────────────────────────────────────────────┤
│                      DEVELOPER TOOLS                                 │
│  Advanced features for script development, debugging, testing        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │Debugger │  │ Console │  │  Test   │  │Simulate │  │ Analyze │  │
│  │         │  │  (REPL) │  │ Runner  │  │         │  │         │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                            │
│  │  Bench  │  │ Doctor  │  │ Replay  │                            │
│  │         │  │         │  │         │                            │
│  └─────────┘  └─────────┘  └─────────┘                            │
└─────────────────────────────────────────────────────────────────────┘
```

### User Interface (Main Navigation)
Features everyday users need:
- **Training** (existing) - Audio-based key training
- **Editor** (existing) - Visual keymap creation
- **Devices** (new) - Select keyboard device
- **Run Controls** (enhanced) - Start/stop with simple options

### Developer Tools (Secondary Navigation / Drawer)
Features for script developers and power users:
- **Debugger** (existing) - Real-time state visualization
- **Console** (existing) - REPL for Rhai evaluation
- **Test Runner** (new) - Run script tests
- **Simulator** (new) - Test key sequences
- **Analyzer** (new) - Session analysis and timing
- **Benchmark** (new) - Latency measurements
- **Doctor** (new) - System diagnostics
- **Replay** (new) - Session replay

---

## User Interface Requirements

### Requirement 1: Device Selection (devices)

**User Story:** As a user, I want to see and select my keyboard device, so that KeyRx works with the right hardware.

#### Acceptance Criteria

1. WHEN I open the Devices screen, THEN the UI SHALL list all detected keyboard devices
2. WHEN devices are listed, THEN each SHALL show: device name and status indicator
3. WHEN I tap a device, THEN it SHALL become the active input source
4. WHEN no devices are found, THEN the UI SHALL show simple troubleshooting message
5. WHEN I pull to refresh, THEN the UI SHALL re-scan for devices
6. WHEN a device has a saved profile, THEN it SHALL show a checkmark badge

### Requirement 2: Enhanced Run Controls (run options)

**User Story:** As a user, I want simple controls to start/stop KeyRx with optional session recording.

#### Acceptance Criteria

1. WHEN I view the main screen, THEN I SHALL see a prominent Start/Stop button
2. WHEN I tap Start, THEN the engine SHALL begin with selected device and script
3. WHEN I toggle "Record Session", THEN the session SHALL be saved for later review
4. WHEN engine is running, THEN status indicators SHALL show: active script, device, recording status
5. WHEN I tap Stop, THEN the engine SHALL gracefully shut down
6. WHEN recording is enabled, THEN the saved file path SHALL be shown

### Requirement 3: Script Validation in Editor (check)

**User Story:** As a user, I want immediate feedback when my script has errors, so I can fix them before loading.

#### Acceptance Criteria

1. WHEN I modify a script in the Editor, THEN syntax SHALL be validated automatically
2. WHEN syntax errors exist, THEN an error banner SHALL appear with message
3. WHEN I tap the error, THEN the problematic line SHALL be highlighted
4. WHEN script is valid, THEN a success indicator SHALL appear
5. WHEN I tap "Load Script", THEN validation SHALL run first and block if errors exist

---

## Developer Tools Requirements

### Requirement 4: Developer Tools Navigation

**User Story:** As a developer, I want access to advanced tools without cluttering the main user interface.

#### Acceptance Criteria

1. WHEN I tap "Developer Tools" button/menu, THEN the developer navigation SHALL appear
2. WHEN in developer mode, THEN I SHALL see all developer-only screens
3. WHEN I tap "Back to User View", THEN I SHALL return to simplified navigation
4. WHEN developer tools are hidden, THEN they SHALL not appear in main navigation
5. WHEN I'm in developer mode, THEN it SHALL persist across app restarts (preference)

### Requirement 5: Test Runner (test)

**User Story:** As a developer, I want to run script tests from the UI, so I can verify my mappings work.

#### Acceptance Criteria

1. WHEN I open Test Runner, THEN the UI SHALL discover all `test_*` functions
2. WHEN tests are discovered, THEN each SHALL show: name, file, status (pending/pass/fail)
3. WHEN I tap "Run All", THEN all tests SHALL execute with progress indicator
4. WHEN I tap a specific test, THEN only that test SHALL run
5. WHEN I enter a filter pattern, THEN only matching tests SHALL be shown
6. WHEN a test fails, THEN the UI SHALL show: error message, line number
7. WHEN I enable "Watch Mode", THEN tests SHALL re-run on script changes

### Requirement 6: Key Sequence Simulator (simulate)

**User Story:** As a developer, I want to simulate key sequences, so I can test mappings without physical key presses.

#### Acceptance Criteria

1. WHEN I open Simulator, THEN I SHALL see key input area and virtual keyboard
2. WHEN I tap keys on virtual keyboard, THEN they SHALL be added to sequence
3. WHEN I tap "Simulate", THEN the engine SHALL process the sequence
4. WHEN simulation completes, THEN I SHALL see: input → output for each key
5. WHEN I specify hold duration, THEN simulation SHALL respect timing
6. WHEN I enable "Combo Mode", THEN keys SHALL be treated as simultaneous
7. WHEN simulation runs, THEN pending decisions and active layers SHALL display

### Requirement 7: Session Analyzer (analyze)

**User Story:** As a developer, I want to analyze recorded sessions to understand timing and decisions.

#### Acceptance Criteria

1. WHEN I open Analyzer, THEN I SHALL see list of recorded sessions (.krx files)
2. WHEN I select a session, THEN statistics SHALL display: event count, duration, avg latency
3. WHEN analysis loads, THEN decision breakdown SHALL show (remap, block, tap, hold, combo, layer)
4. WHEN I tap "Timing Diagram", THEN visual timeline of events SHALL render
5. WHEN viewing timeline, THEN each event SHALL show: input, decision, output, latency
6. WHEN I tap an event, THEN detailed information SHALL appear

### Requirement 8: Latency Benchmark (bench)

**User Story:** As a developer, I want to run latency benchmarks to verify performance requirements.

#### Acceptance Criteria

1. WHEN I open Benchmark, THEN I SHALL see configuration options
2. WHEN I set iterations, THEN that value SHALL be used (default: 10,000)
3. WHEN I tap "Run Benchmark", THEN measurements SHALL execute with progress
4. WHEN benchmark completes, THEN results SHALL show: min, max, mean, p99 latencies
5. WHEN mean latency exceeds 1ms, THEN a warning SHALL display
6. WHEN previous results exist, THEN trend comparison SHALL be available

### Requirement 9: System Diagnostics (doctor)

**User Story:** As a developer, I want to run system diagnostics to verify setup is correct.

#### Acceptance Criteria

1. WHEN I open Doctor, THEN all system checks SHALL run automatically
2. WHEN checks complete, THEN each SHALL show: name, status (pass/fail/warn), details
3. WHEN a check fails, THEN remediation steps SHALL display
4. WHEN on Linux, THEN checks SHALL include: /dev/uinput, input group membership
5. WHEN on Windows, THEN checks SHALL include: keyboard hook API, user32.dll
6. WHEN I tap "Re-run", THEN all checks SHALL execute again

### Requirement 10: Session Replay (replay)

**User Story:** As a developer, I want to replay sessions to debug and verify behavior.

#### Acceptance Criteria

1. WHEN I open Replay, THEN I SHALL see list of recorded sessions
2. WHEN I select a session, THEN metadata SHALL show: duration, event count, timestamp
3. WHEN I tap "Play", THEN session SHALL replay through engine
4. WHEN replay runs, THEN events SHALL visualize in real-time
5. WHEN I adjust speed slider, THEN replay speed SHALL change (0x, 1x, 2x)
6. WHEN I enable "Verify Mode", THEN outputs SHALL compare against recorded
7. WHEN verification fails, THEN mismatched events SHALL highlight

### Requirement 11: Device Profile Discovery (discover)

**User Story:** As a developer, I want to create device profiles through guided discovery.

#### Acceptance Criteria

1. WHEN I tap "Start Discovery" in Developer Tools, THEN discovery wizard SHALL begin
2. WHEN discovery starts, THEN prompts SHALL guide me to press specific keys
3. WHEN I press keys, THEN progress SHALL update in real-time
4. WHEN discovery completes, THEN detected layout SHALL display for confirmation
5. WHEN I confirm, THEN profile SHALL save to device registry
6. WHEN I cancel, THEN partial progress SHALL discard
7. WHEN emergency exit pattern detected, THEN discovery SHALL abort safely

---

## Non-Functional Requirements

### UI/UX Standards

- **Separation**: Clear visual distinction between User and Developer interfaces
- **Simplicity**: User interface contains only essential features
- **Discoverability**: Developer tools accessible but not prominent
- **Consistency**: All screens follow existing navigation patterns
- **Feedback**: Async operations show loading indicators
- **Errors**: User-friendly messages with remediation hints

### Performance

- **Benchmark UI**: Must not introduce >1% overhead to measurements
- **Simulation**: Key processing visible within 16ms (60fps)
- **Device Scan**: Complete within 2 seconds

### Architecture

- **FFI Bindings**: All new functionality via existing bridge pattern
- **State Management**: Use existing AppState patterns
- **Service Layer**: Follow existing EngineService/AudioService patterns
- **Navigation**: User screens in main NavigationRail; Developer screens in drawer/secondary nav
