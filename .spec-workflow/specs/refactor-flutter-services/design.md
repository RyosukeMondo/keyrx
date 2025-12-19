# Design: Refactor Flutter Services for core_logic

## Architecture Overview

Refactor HardwareService (90 lines) and LayoutService (84 lines) to remove ChangeNotifier, move to core_logic, and create UI wrappers.

### Before
```
ui/lib/services/
  ├── hardware_service.dart    (extends ChangeNotifier) ❌
  └── layout_service.dart      (extends ChangeNotifier) ❌
```

### After
```
packages/core_logic/lib/services/
  ├── hardware_service.dart    (StreamController) ✅
  └── layout_service.dart      (StreamController) ✅

ui/lib/services/
  ├── hardware_service_ui.dart (ChangeNotifier wrapper) ✅
  └── layout_service_ui.dart   (ChangeNotifier wrapper) ✅
```

## Refactoring Strategy

### Phase 1: Create Core Services
1. Copy service to core_logic
2. Remove `with ChangeNotifier`
3. Add `StreamController<void> _changesController`
4. Add `Stream<void> get changes`
5. Replace `notifyListeners()` with `_changesController.add(null)`
6. Add `dispose()` to close stream
7. Update imports to relative paths

### Phase 2: Create UI Wrappers
1. Create new UI wrapper file
2. Extend ChangeNotifier
3. Accept core service in constructor
4. Subscribe to core service stream
5. Forward all method calls to core service
6. Call `notifyListeners()` on stream events
7. Dispose subscription properly

### Phase 3: Update ServiceRegistry
1. Import core services from core_logic
2. Create UI wrappers
3. Pass UI wrappers to consumers

## Component Design

### HardwareService (Core)
```dart
class HardwareService {
  HardwareService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<void> _changesController = StreamController.broadcast();

  Stream<void> get changes => _changesController.stream;

  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async =>
      _guard('list hardware profiles', (bindings) {
        try {
          final profiles = bindings.listHardwareProfiles();
          return ConfigOperationResult.success(profiles);
        } catch (e) {
          return ConfigOperationResult.error('list hardware profiles failed: $e');
        }
      });

  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async {
    final result = await _guard<HardwareProfile>('save hardware profile', (bindings) {
      try {
        final saved = bindings.saveHardwareProfile(profile);
        return ConfigOperationResult.success(saved);
      } catch (e) {
        return ConfigOperationResult.error('save hardware profile failed: $e');
      }
    });

    if (!result.hasError) {
      _changesController.add(null);  // Notify listeners
    }
    return result;
  }

  Future<ConfigOperationResult<void>> deleteProfile(String id) async {
    final result = await _guard<void>('delete hardware profile', (bindings) {
      try {
        bindings.deleteHardwareProfile(id);
        return ConfigOperationResult.success();
      } catch (e) {
        return ConfigOperationResult.error('delete hardware profile failed: $e');
      }
    });

    if (!result.hasError) {
      _changesController.add(null);
    }
    return result;
  }

  Future<ConfigOperationResult<T>> _guard<T>(...) { /* unchanged */ }

  void dispose() {
    _changesController.close();
  }
}
```

### HardwareServiceUI (UI Wrapper)
```dart
import 'package:flutter/foundation.dart';
import 'package:core_logic/services/hardware_service.dart' as core;

class HardwareServiceUI extends ChangeNotifier {
  HardwareServiceUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.HardwareService _core;
  StreamSubscription<void>? _subscription;

  // Forward all methods
  Future<core.ConfigOperationResult<List<core.HardwareProfile>>> listProfiles() =>
      _core.listProfiles();

  Future<core.ConfigOperationResult<core.HardwareProfile>> saveProfile(
    core.HardwareProfile profile,
  ) => _core.saveProfile(profile);

  Future<core.ConfigOperationResult<void>> deleteProfile(String id) =>
      _core.deleteProfile(id);

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }
}
```

### ServiceRegistry Updates
```dart
factory ServiceRegistry.real({...}) {
  final bridge = KeyrxBridge.open();

  // Create core services
  final coreHardware = core.HardwareService(bridge: bridge);
  final coreLayout = core.LayoutService(bridge: bridge);

  // Create UI wrappers
  final hardware = HardwareServiceUI(coreHardware);
  final layout = LayoutServiceUI(coreLayout);

  return ServiceRegistry(
    hardwareService: hardware,
    layoutService: layout,
    // ... other services
  );
}
```

## Data Flow

### Before (ChangeNotifier)
```
UI Widget → HardwareService.saveProfile() → FFI → notifyListeners()
                                                          ↓
                                                    UI rebuilds
```

### After (Stream + Wrapper)
```
UI Widget → HardwareServiceUI.saveProfile()
                ↓
           core.HardwareService.saveProfile() → FFI → _changesController.add(null)
                                                              ↓
                                                         changes stream
                                                              ↓
                                              HardwareServiceUI hears event
                                                              ↓
                                                      notifyListeners()
                                                              ↓
                                                         UI rebuilds
```

## Testing Strategy

1. **Core Service Tests**: Verify stream emits on changes
2. **UI Wrapper Tests**: Verify ChangeNotifier notifications
3. **Integration Tests**: Verify UI continues to work
4. **Memory Tests**: Verify streams are disposed

## Rollback Plan

Revert commits or manually restore original files from git history.

## Success Criteria

- ✅ Core services have zero Flutter dependencies
- ✅ UI wrappers provide ChangeNotifier interface
- ✅ All tests pass
- ✅ Streams properly disposed
- ✅ No memory leaks
