# Design: Move Config Services to core_logic

## Architecture Overview

Move RuntimeService (177 lines) and KeymapService (67 lines) from `ui/lib/services/` to `packages/core_logic/lib/services/` without modifying their implementation. Update UI imports to reference the new location.

###Before Architecture
```
ui/lib/services/
  ├── runtime_service.dart          (177 lines, no Flutter deps)
  ├── keymap_service.dart            (67 lines, no Flutter deps)
  ├── config_result.dart             (shared result type)
  └── service_registry.dart          (imports above services)

packages/core_logic/lib/
  ├── services/
  │   └── device_registry_service.dart  (existing)
  └── core_logic.dart                 (exports)
```

### After Architecture
```
packages/core_logic/lib/
  ├── services/
  │   ├── device_registry_service.dart  ✅ Existing
  │   ├── runtime_service.dart          🆕 Moved from ui/
  │   ├── keymap_service.dart            🆕 Moved from ui/
  │   └── config_result.dart             🆕 Moved from ui/
  └── core_logic.dart                   (exports new services)

ui/lib/services/
  ├── service_registry.dart          (updated imports)
  └── ... (other UI-specific services)
```

## Component Design

### 1. RuntimeService

**Current Location**: `ui/lib/services/runtime_service.dart`
**Target Location**: `packages/core_logic/lib/services/runtime_service.dart`

**Dependencies** (all available in core_logic):
- `dart:convert` ✅ (Dart SDK)
- `package:core_logic/ffi/bridge.dart` ✅
- `package:core_logic/ffi/generated_bindings.dart` ✅
- `package:core_logic/models/hardware_profile.dart` ✅
- `package:core_logic/models/runtime_config.dart` ✅
- `config_result.dart` (needs to be moved)

**Interface** (unchanged):
```dart
abstract class RuntimeService {
  Future<ConfigOperationResult<RuntimeConfig>> getConfig();
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  );
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  );
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  );
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  );
  RuntimeConfig? get lastConfig;
}
```

**Migration**: Copy file, update imports, verify no Flutter deps.

### 2. KeymapService

**Current Location**: `ui/lib/services/keymap_service.dart`
**Target Location**: `packages/core_logic/lib/services/keymap_service.dart`

**Dependencies** (all available in core_logic):
- `package:core_logic/ffi/bridge.dart` ✅
- `package:core_logic/ffi/generated_bindings.dart` ✅
- `package:core_logic/models/keymap.dart` ✅
- `config_result.dart` (needs to be moved)

**Interface** (unchanged):
```dart
class KeymapService {
  KeymapService({required KeyrxBridge bridge});

  Future<ConfigOperationResult<List<Keymap>>> listKeymaps();
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap);
  Future<ConfigOperationResult<void>> deleteKeymap(String id);
}
```

**Migration**: Copy file, update imports, verify no Flutter deps.

### 3. ConfigOperationResult (Shared Type)

**Current Location**: `ui/lib/services/config_result.dart`
**Target Location**: `packages/core_logic/lib/services/config_result.dart`

**Purpose**: Shared result type for all config operations.

**Interface**:
```dart
class ConfigOperationResult<T> {
  const ConfigOperationResult.success(T data);
  const ConfigOperationResult.error(String errorMessage);

  final T? data;
  final String? errorMessage;
  bool get hasError => errorMessage != null;
}
```

**Migration**: Copy file, no imports needed (pure Dart).

## Implementation Strategy

### Phase 1: Move Core Files
1. Create `packages/core_logic/lib/services/` directory if needed
2. Copy `config_result.dart` to core_logic
3. Copy `runtime_service.dart` to core_logic
4. Copy `keymap_service.dart` to core_logic
5. Update imports in moved files to reference core_logic

### Phase 2: Update Exports
1. Add exports to `packages/core_logic/lib/core_logic.dart`:
   ```dart
   export 'services/config_result.dart';
   export 'services/runtime_service.dart';
   export 'services/keymap_service.dart';
   ```

### Phase 3: Update UI References
1. Update `ui/lib/services/service_registry.dart`:
   ```dart
   // Before
   import 'runtime_service.dart';
   import 'keymap_service.dart';

   // After
   import 'package:core_logic/services/runtime_service.dart';
   import 'package:core_logic/services/keymap_service.dart';
   ```

2. Update any other UI files importing these services

### Phase 4: Delete Old Files
1. Delete `ui/lib/services/runtime_service.dart`
2. Delete `ui/lib/services/keymap_service.dart`
3. Delete `ui/lib/services/config_result.dart`

### Phase 5: Verification
1. Run `flutter pub get` in UI
2. Run all UI tests
3. Verify code metrics
4. Check that services have no Flutter dependencies

## Data Flow

### RuntimeService Data Flow (unchanged)
```
UI/CLI → RuntimeService → KeyrxBridge.runtimeBindings → Rust FFI
                                                         ↓
                                                    RuntimeConfig
                                                         ↓
                            ConfigOperationResult<RuntimeConfig> ← Parse JSON
```

### KeymapService Data Flow (unchanged)
```
UI/CLI → KeymapService → KeyrxBridge.configBindings → Rust FFI
                                                       ↓
                                                  List<Keymap>
                                                       ↓
                       ConfigOperationResult<List<Keymap>> ← Parse/Validate
```

## Error Handling

**Strategy**: Preserve existing error handling (no changes).

**Error Types** (from config_result.dart):
- FFI load failure: "Engine unavailable: {reason}"
- Bindings unavailable: "{Operation} bindings unavailable"
- JSON parse error: "Failed to parse runtime config: {error}"
- Backend error: "Runtime error: {error}"

**Example**:
```dart
Future<ConfigOperationResult<RuntimeConfig>> getConfig() async =>
    _guard('get runtime config', (bindings) {
      try {
        final raw = bindings.getConfig();
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('get runtime config failed: $e');
      }
    });
```

## Testing Strategy

### Unit Tests (if they exist)
- Move tests to `packages/core_logic/test/services/`
- Update test imports
- Verify all tests pass

### Integration Tests
- UI tests should pass without changes
- ServiceRegistry tests verify correct service usage

### Validation Tests
1. Import verification: `dart analyze packages/core_logic/`
2. Flutter dep check: `grep -r "package:flutter" packages/core_logic/lib/services/`
3. Metrics check: `dart run ui/scripts/check_metrics.dart`

## Dependencies

### Import Changes

**RuntimeService** (before):
```dart
import 'dart:convert';
import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/ffi/generated_bindings.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'config_result.dart';
```

**RuntimeService** (after):
```dart
import 'dart:convert';
import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/hardware_profile.dart';
import '../models/runtime_config.dart';
import 'config_result.dart';
```

### Export Updates

**core_logic.dart** (add):
```dart
// Config Services
export 'services/config_result.dart';
export 'services/runtime_service.dart';
export 'services/keymap_service.dart';
```

## Rollback Plan

If issues arise:
1. Revert commits (git revert)
2. Or manually restore:
   - Copy services back to `ui/lib/services/`
   - Revert `core_logic.dart` exports
   - Revert UI import changes

## Success Criteria

- ✅ Files exist at new locations in core_logic
- ✅ Zero Flutter dependencies in moved files
- ✅ All UI tests pass
- ✅ `dart analyze` shows no errors
- ✅ Services exported from core_logic.dart
- ✅ Code metrics maintained

## Performance Considerations

**No performance impact expected**:
- Services remain identical (copy-paste)
- Same FFI calls
- Same error handling
- Only import paths change

## Security Considerations

**No security changes**:
- Services don't access sensitive data
- FFI bridge handles security
- Error messages remain the same

## Future Enhancements (Out of Scope)

- CLI commands using these services (separate spec)
- Performance optimizations
- Additional service methods
- Improved error messages
