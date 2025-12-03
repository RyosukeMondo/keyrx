# Design Document: Flutter UI Fixes

## Overview

This design addresses Flutter UI code quality issues:
1. Fix 8 failing tests by updating ServiceRegistry mock configurations
2. Split `bridge.dart` (1841 lines) into domain-specific modules
3. Refactor 3 oversized page/widget files

## Steering Document Alignment

### Technical Standards (tech.md)
- **Modular Design**: Split files by single responsibility
- **Testability**: All services mockable via dependency injection

### Project Structure (structure.md)
- **Max 500 lines/file**: Target for all refactored files
- **Re-exports**: Maintain public API through barrel files

## Code Reuse Analysis

### Existing Patterns to Follow
- Service interface pattern (`EngineService`, `AudioService`, etc.)
- `ServiceRegistry.withOverrides()` for test mocking
- Widget extraction pattern (see `debugger_widgets.dart`, `editor_widgets.dart`)

## Architecture

### Test Fix Strategy

The 8 failing tests all have the same root cause:
```dart
// Current (broken):
final registry = ServiceRegistry.withOverrides(
  engineService: _FakeEngineService(),
);

// Fixed (add required deviceService):
final registry = ServiceRegistry.withOverrides(
  engineService: _FakeEngineService(),
  deviceService: _FakeDeviceService(),  // Add this
);
```

### FFI Bridge Split Strategy

```
bridge.dart (1841 lines)
├── bridge.dart (~200 lines) - Re-exports, KeyrxBridge class shell
├── bridge_core.dart (~150 lines) - init, version, dispose
├── bridge_engine.dart (~300 lines) - engine control, state stream
├── bridge_audio.dart (~200 lines) - audio capture, classification
├── bridge_session.dart (~350 lines) - recording, replay, analysis
├── bridge_discovery.dart (~300 lines) - device discovery
└── bridge_testing.dart (~250 lines) - simulate, tests, benchmark
```

### Page/Widget Split Strategy

**run_controls_page.dart (542 lines)**
```
run_controls_page.dart (~300 lines) - Main page
run_controls_widgets.dart (~250 lines) - _StatusIndicator, cards
```

**visual_keyboard.dart (529 lines)**
```
visual_keyboard.dart (~300 lines) - Main widget
visual_keyboard_keys.dart (~230 lines) - Key rendering helpers
```

**editor_page.dart (509 lines)**
Already has `editor_widgets.dart` - just needs extraction of remaining helpers.

## Components

### Component 1: Test Fixes

| Test File | Fix Required |
|-----------|--------------|
| `keyrx_training_screen_test.dart` | Add `deviceService: _FakeDeviceService()` |
| `trade_off_test.dart` | Add `deviceService: _FakeDeviceService()` |
| `debugger_page_test.dart` | Add `deviceService: _FakeDeviceService()` |
| `training_screen_test.dart` | Add `deviceService: _FakeDeviceService()` |

### Component 2: Bridge Module Split

| Module | Contents | Est. Lines |
|--------|----------|------------|
| `bridge.dart` | KeyrxBridge class, re-exports | ~200 |
| `bridge_core.dart` | `_init()`, `version`, `dispose()`, `_freeString()` | ~150 |
| `bridge_engine.dart` | `loadScript()`, `eval()`, `stateStream`, `listKeys()` | ~300 |
| `bridge_audio.dart` | `startAudio()`, `stopAudio()`, `setBpm()`, classification | ~200 |
| `bridge_session.dart` | `startRecording()`, `stopRecording()`, `listSessions()`, `analyzeSession()`, `replaySession()` | ~350 |
| `bridge_discovery.dart` | `startDiscovery()`, `processDiscoveryEvent()`, `cancelDiscovery()` | ~300 |
| `bridge_testing.dart` | `simulate()`, `runTests()`, `discoverTests()`, `runBenchmark()`, `runDoctor()` | ~250 |

### Component 3: Page Widgets Extraction

Extract reusable widgets to separate files following existing pattern.

## Error Handling

- Test compilation errors will be fixed by providing required parameters
- Import path changes handled through re-exports in main files

## Testing Strategy

### Verification Steps
1. Run `flutter test` - expect 0 failures
2. Run `flutter analyze` - expect 0 errors
3. Verify no file exceeds 500 lines
