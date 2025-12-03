# Design Document

## Overview

This design implements a two-tier Flutter UI that exposes all CLI functionality while maintaining a clean separation between User and Developer interfaces. The User interface provides essential features for daily usage, while Developer Tools contains advanced debugging, testing, and analysis capabilities.

## Steering Document Alignment

### Technical Standards (tech.md)

- **FFI Bridge**: Extends `ui/lib/ffi/bridge.dart` with new bindings
- **Flutter Architecture**: Follows existing page/widget/service patterns
- **Cross-Platform**: All features work on Linux and Windows
- **State Management**: Uses existing AppState patterns

### Project Structure (structure.md)

```
ui/lib/
├── main.dart                       # MODIFY: Add developer mode toggle
├── ffi/
│   ├── bindings.dart              # MODIFY: Add new FFI signatures
│   └── bridge.dart                # MODIFY: Add new bridge methods
├── pages/
│   ├── devices_page.dart          # NEW: Device selection
│   ├── run_controls_page.dart     # NEW: Engine control panel
│   ├── developer/                 # NEW: Developer tools directory
│   │   ├── test_runner_page.dart
│   │   ├── simulator_page.dart
│   │   ├── analyzer_page.dart
│   │   ├── benchmark_page.dart
│   │   ├── doctor_page.dart
│   │   ├── replay_page.dart
│   │   └── discovery_page.dart
│   └── editor_page.dart           # MODIFY: Add validation
├── services/
│   ├── device_service.dart        # NEW: Device management
│   ├── test_service.dart          # NEW: Test runner
│   ├── simulation_service.dart    # NEW: Key simulation
│   ├── session_service.dart       # NEW: Replay/analyze
│   ├── benchmark_service.dart     # NEW: Benchmarking
│   └── doctor_service.dart        # NEW: Diagnostics
├── state/
│   └── app_state.dart             # MODIFY: Add developer mode
└── widgets/
    ├── developer_drawer.dart      # NEW: Developer navigation
    └── virtual_keyboard.dart      # NEW: For simulator
```

## Code Reuse Analysis

### Existing Components to Leverage

- **`ui/lib/ffi/bridge.dart`**: FFI bridge pattern - extend with new methods
- **`ui/lib/services/engine_service.dart`**: Service pattern - create similar services
- **`ui/lib/state/app_state.dart`**: State management - add developer mode flag
- **`ui/lib/pages/debugger_page.dart`**: Page layout pattern - follow for new pages
- **`ui/lib/widgets/keyboard.dart`**: Virtual keyboard - extend for simulator

### Integration Points

- **Rust CLI Commands**: Each CLI command maps to an FFI function
- **Existing Navigation**: Main NavigationRail extended with Devices, Run Controls
- **Developer Drawer**: New drawer component for developer tools

## Architecture

### Navigation Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Main App Shell                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         AppBar                                          ││
│  │  [KeyRx Logo]                              [Developer Tools ⚙️]          ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│  ┌────────────┐  ┌──────────────────────────────────────────────────────────┐│
│  │            │  │                                                          ││
│  │ Navigation │  │                    Content Area                          ││
│  │   Rail     │  │                                                          ││
│  │            │  │  ┌────────────────────────────────────────────────────┐ ││
│  │ ┌────────┐ │  │  │                                                    │ ││
│  │ │Training│ │  │  │            Active Page Content                     │ ││
│  │ └────────┘ │  │  │                                                    │ ││
│  │ ┌────────┐ │  │  │                                                    │ ││
│  │ │ Editor │ │  │  └────────────────────────────────────────────────────┘ ││
│  │ └────────┘ │  │                                                          ││
│  │ ┌────────┐ │  └──────────────────────────────────────────────────────────┘│
│  │ │Devices │ │                                                              │
│  │ └────────┘ │  Developer Drawer (slides from right when toggled)          │
│  │ ┌────────┐ │  ┌──────────────────────────────────────────────────────────┐│
│  │ │  Run   │ │  │ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     ││
│  │ └────────┘ │  │ │ Debugger │ │ Console  │ │  Test    │ │ Simulate │     ││
│  │            │  │ └──────────┘ └──────────┘ └──────────┘ └──────────┘     ││
│  └────────────┘  │ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     ││
│                  │ │ Analyze  │ │  Bench   │ │  Doctor  │ │  Replay  │     ││
│                  │ └──────────┘ └──────────┘ └──────────┘ └──────────┘     ││
│                  │ ┌──────────┐                                             ││
│                  │ │ Discover │                                             ││
│                  │ └──────────┘                                             ││
│                  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### FFI Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Flutter UI Layer                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ DevicePage  │  │ TestRunner  │  │ Simulator   │  │ Analyzer    │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                │                │
│         ▼                ▼                ▼                ▼                │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         Service Layer                                   ││
│  │  DeviceService  TestService  SimulationService  SessionService  ...    ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                      FFI Bridge (bridge.dart)                           ││
│  │  listDevices()  runTests()  simulate()  analyzeSession()  ...          ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                        │
│                                    ▼ FFI                                    │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
┌────────────────────────────────────┼────────────────────────────────────────┐
│                           Rust Core (libkeyrx)                              │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                       FFI Exports (ffi/mod.rs)                          ││
│  │  keyrx_list_devices()  keyrx_run_tests()  keyrx_simulate()  ...        ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                        CLI Command Implementations                       ││
│  │  devices.rs  test.rs  simulate.rs  analyze.rs  bench.rs  doctor.rs     ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Modular Design Principles

- **Single Responsibility**: Each page handles one feature
- **Service Isolation**: Each CLI command gets a dedicated service
- **Component Reuse**: Virtual keyboard shared between Editor and Simulator
- **State Separation**: Developer mode state separate from engine state

## Components and Interfaces

### Component 1: FFI Bindings Extension

**Purpose:** Expose CLI commands to Flutter via FFI

**File:** `ui/lib/ffi/bindings.dart` (extend)

**New Signatures:**
```dart
// Device management
typedef KeyrxListDevices = Pointer<Utf8> Function();
typedef KeyrxSelectDevice = Int32 Function(Pointer<Utf8> devicePath);

// Script validation
typedef KeyrxCheckScript = Pointer<Utf8> Function(Pointer<Utf8> scriptPath);

// Test runner
typedef KeyrxDiscoverTests = Pointer<Utf8> Function(Pointer<Utf8> scriptPath);
typedef KeyrxRunTests = Pointer<Utf8> Function(Pointer<Utf8> scriptPath, Pointer<Utf8> filter);

// Simulation
typedef KeyrxSimulate = Pointer<Utf8> Function(Pointer<Utf8> keys, Int32 comboMode);

// Session management
typedef KeyrxListSessions = Pointer<Utf8> Function();
typedef KeyrxAnalyzeSession = Pointer<Utf8> Function(Pointer<Utf8> sessionPath);
typedef KeyrxReplaySession = Pointer<Utf8> Function(Pointer<Utf8> sessionPath, Int32 verify);

// Benchmarking
typedef KeyrxRunBenchmark = Pointer<Utf8> Function(Int32 iterations);

// Diagnostics
typedef KeyrxRunDoctor = Pointer<Utf8> Function();

// Discovery
typedef KeyrxStartDiscovery = Pointer<Utf8> Function(Pointer<Utf8> deviceId);
typedef KeyrxOnDiscoveryProgress = Void Function(Pointer<Utf8>);

// Recording control
typedef KeyrxStartRecording = Int32 Function(Pointer<Utf8> outputPath);
typedef KeyrxStopRecording = Pointer<Utf8> Function();
```

**Dependencies:** Rust FFI exports
**Reuses:** Existing FFI patterns from bindings.dart

### Component 2: Developer Navigation Drawer

**Purpose:** Provide access to developer tools without cluttering main navigation

**File:** `ui/lib/widgets/developer_drawer.dart` (new)

**Interface:**
```dart
class DeveloperDrawer extends StatelessWidget {
  final int selectedIndex;
  final Function(int) onDestinationSelected;

  // Destinations:
  // 0: Debugger (existing)
  // 1: Console (existing)
  // 2: Test Runner
  // 3: Simulator
  // 4: Analyzer
  // 5: Benchmark
  // 6: Doctor
  // 7: Replay
  // 8: Discovery
}
```

**Dependencies:** AppState for developer mode
**Reuses:** NavigationRail styling patterns

### Component 3: Device Service

**Purpose:** Manage keyboard device listing and selection

**File:** `ui/lib/services/device_service.dart` (new)

**Interface:**
```dart
class DeviceService {
  Future<List<KeyboardDevice>> listDevices();
  Future<void> selectDevice(String devicePath);
  Future<bool> hasProfile(String deviceId);
  Stream<DeviceEvent> get deviceEvents;
}

class KeyboardDevice {
  final String name;
  final String vendorId;
  final String productId;
  final String path;
  final bool hasProfile;
}
```

**Dependencies:** KeyrxBridge
**Reuses:** EngineService patterns

### Component 4: Test Service

**Purpose:** Discover and run script tests

**File:** `ui/lib/services/test_service.dart` (new)

**Interface:**
```dart
class TestService {
  Future<List<TestCase>> discoverTests(String scriptPath);
  Future<TestResults> runTests(String scriptPath, {String? filter});
  Stream<TestProgress> runTestsStreaming(String scriptPath);
}

class TestCase {
  final String name;
  final String file;
  final int lineNumber;
}

class TestResult {
  final String name;
  final bool passed;
  final String? error;
  final Duration duration;
}
```

**Dependencies:** KeyrxBridge
**Reuses:** Service async patterns

### Component 5: Simulation Service

**Purpose:** Simulate key sequences through the engine

**File:** `ui/lib/services/simulation_service.dart` (new)

**Interface:**
```dart
class SimulationService {
  Future<SimulationResult> simulate(List<KeyInput> keys, {bool comboMode = false});
}

class KeyInput {
  final String keyCode;
  final int? holdMs;
}

class SimulationResult {
  final List<KeyMapping> mappings;
  final List<String> activeLayers;
  final List<PendingDecision> pending;
}

class KeyMapping {
  final String input;
  final String output;
  final String decisionType;
}
```

**Dependencies:** KeyrxBridge
**Reuses:** Existing key registry

### Component 6: Session Service

**Purpose:** Manage session replay and analysis

**File:** `ui/lib/services/session_service.dart` (new)

**Interface:**
```dart
class SessionService {
  Future<List<SessionInfo>> listSessions();
  Future<SessionAnalysis> analyze(String sessionPath);
  Stream<ReplayEvent> replay(String sessionPath, {double speed = 0, bool verify = false});
}

class SessionInfo {
  final String path;
  final String name;
  final DateTime created;
  final int eventCount;
  final Duration duration;
}

class SessionAnalysis {
  final int eventCount;
  final Duration duration;
  final Duration avgLatency;
  final Map<String, int> decisionBreakdown;
  final List<TimingEvent> timeline;
}
```

**Dependencies:** KeyrxBridge
**Reuses:** Session format from core

### Component 7: Devices Page

**Purpose:** List and select keyboard devices

**File:** `ui/lib/pages/devices_page.dart` (new)

**Interface:**
```dart
class DevicesPage extends StatefulWidget {
  // Displays list of detected keyboards
  // Allows selection of active device
  // Shows profile status badges
  // Refresh button for re-scan
}
```

**Dependencies:** DeviceService
**Reuses:** ListTile patterns from existing pages

### Component 8: Run Controls Page

**Purpose:** Central engine control panel

**File:** `ui/lib/pages/run_controls_page.dart` (new)

**Interface:**
```dart
class RunControlsPage extends StatefulWidget {
  // Large Start/Stop button
  // Device selector dropdown
  // Script selector
  // Recording toggle
  // Status indicators
}
```

**Dependencies:** EngineService, DeviceService
**Reuses:** Existing engine state patterns

### Component 9: Test Runner Page

**Purpose:** Discover and run script tests with results

**File:** `ui/lib/pages/developer/test_runner_page.dart` (new)

**Interface:**
```dart
class TestRunnerPage extends StatefulWidget {
  // Test list with status indicators
  // Run All / Run Selected buttons
  // Filter input
  // Watch mode toggle
  // Results panel with error details
}
```

**Dependencies:** TestService
**Reuses:** List patterns from debugger

### Component 10: Simulator Page

**Purpose:** Simulate key sequences visually

**File:** `ui/lib/pages/developer/simulator_page.dart` (new)

**Interface:**
```dart
class SimulatorPage extends StatefulWidget {
  // Virtual keyboard for key selection
  // Key sequence display
  // Hold duration editor
  // Combo mode toggle
  // Results showing input → output
  // Layer/pending state display
}
```

**Dependencies:** SimulationService, VirtualKeyboard widget
**Reuses:** Keyboard widget from editor

## Data Models

### Device Model

```dart
class KeyboardDevice {
  final String name;
  final String vendorId;
  final String productId;
  final String path;
  final bool hasProfile;
  final bool isSelected;
}
```

### Test Models

```dart
class TestCase {
  final String name;
  final String file;
  final int lineNumber;
  TestStatus status; // pending, running, passed, failed
}

class TestResults {
  final int total;
  final int passed;
  final int failed;
  final Duration duration;
  final List<TestResult> results;
}
```

### Simulation Models

```dart
class SimulationInput {
  final List<KeyInput> keys;
  final bool comboMode;
}

class SimulationOutput {
  final List<KeyMapping> mappings;
  final EngineState finalState;
}
```

### Session Models

```dart
class SessionAnalysis {
  final SessionMetadata metadata;
  final LatencyStats latency;
  final Map<DecisionType, int> decisions;
  final List<TimelineEvent> timeline;
}

class TimelineEvent {
  final int sequence;
  final String input;
  final DecisionType decision;
  final String output;
  final Duration latency;
  final DateTime timestamp;
}
```

### Benchmark Models

```dart
class BenchmarkConfig {
  final int iterations;
  final String? scriptPath;
}

class BenchmarkResults {
  final Duration min;
  final Duration max;
  final Duration mean;
  final Duration p99;
  final bool hasWarning;
}
```

### Doctor Models

```dart
class DiagnosticCheck {
  final String name;
  final CheckStatus status; // pass, fail, warn
  final String details;
  final String? remediation;
}

class DiagnosticReport {
  final List<DiagnosticCheck> checks;
  final int passed;
  final int failed;
  final int warned;
}
```

## Error Handling

### Error Scenarios

1. **Device not found**
   - **Handling:** Show empty state with platform-specific troubleshooting
   - **User Impact:** "No keyboards detected. [Platform hints]"

2. **Script validation fails**
   - **Handling:** Parse error JSON, highlight line in editor
   - **User Impact:** Error banner with clickable line reference

3. **Test execution error**
   - **Handling:** Mark test as failed, show error details
   - **User Impact:** Red status with expandable error message

4. **Session file not found**
   - **Handling:** Remove from list, show toast notification
   - **User Impact:** "Session file no longer exists"

5. **Benchmark interrupted**
   - **Handling:** Show partial results with warning
   - **User Impact:** "Benchmark incomplete - partial results shown"

6. **Discovery cancelled**
   - **Handling:** Discard state, return to devices page
   - **User Impact:** "Discovery cancelled - no changes saved"

## Testing Strategy

### Unit Testing

- **Services**: Mock FFI bridge, test response parsing
- **State**: Test developer mode toggle, persistence
- **Models**: Test JSON serialization/deserialization

### Widget Testing

- **DevicesPage**: Test device list rendering, selection
- **TestRunnerPage**: Test status updates, filter functionality
- **SimulatorPage**: Test key input, results display
- **DeveloperDrawer**: Test navigation, state persistence

### Integration Testing

- **FFI Round-trip**: Call FFI → Verify response parsing
- **Navigation Flow**: User view → Developer tools → Back
- **Full Workflow**: Select device → Load script → Run → Record → Analyze

## Implementation Sequence

1. **FFI Bindings** - Foundation for all features
2. **Device Service + Page** - User-facing, enables device selection
3. **Run Controls Page** - User-facing, central control
4. **Script Validation** - Editor enhancement
5. **Developer Drawer** - Navigation for developer tools
6. **Test Service + Page** - Most commonly needed developer tool
7. **Simulation Service + Page** - Interactive testing
8. **Session Service** - Shared by Analyzer and Replay
9. **Analyzer Page** - Session analysis
10. **Replay Page** - Session playback
11. **Benchmark Service + Page** - Performance testing
12. **Doctor Service + Page** - Diagnostics
13. **Discovery Page** - Device profile creation
