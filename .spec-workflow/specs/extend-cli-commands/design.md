# Design: Extend CLI Commands

## Architecture

Add command handlers to `apps/cli_tool/lib/command_processor.dart` that use services from core_logic.

### New Commands (12 total)

**Runtime (4)**:
- `listRuntime` → `runtimeService.getConfig()`
- `addSlot` → `runtimeService.addSlot(device, slot)`
- `removeSlot` → `runtimeService.removeSlot(device, slotId)`
- `activateSlot` → `runtimeService.setSlotActive(device, slotId, active)`

**Hardware (3)**:
- `listHardware` → `hardwareService.listProfiles()`
- `saveHardware` → `hardwareService.saveProfile(profile)`
- `deleteHardware` → `hardwareService.deleteProfile(id)`

**Layout (3)**:
- `listLayouts` → `layoutService.listLayouts()`
- `saveLayout` → `layoutService.saveLayout(layout)`
- `deleteLayout` → `layoutService.deleteLayout(id)`

**Keymap (3)**:
- `listKeymaps` → `keymapService.listKeymaps()`
- `saveKeymap` → `keymapService.saveKeymap(keymap)`
- `deleteKeymap` → `keymapService.deleteKeymap(id)`

## Implementation Pattern

```dart
// Add services to CommandProcessor
class CommandProcessor {
  final KeyrxBridge bridge;
  final DeviceRegistryService deviceRegistry;
  final RuntimeService runtimeService;
  final HardwareService hardwareService;
  final LayoutService layoutService;
  final KeymapService keymapService;

  CommandProcessor(
    this.bridge,
    this.deviceRegistry,
    this.runtimeService,
    this.hardwareService,
    this.layoutService,
    this.keymapService,
  );
}

// Add command handlers
switch (command) {
  case 'listRuntime':
    final result = await runtimeService.getConfig();
    if (result.hasError) {
      response['error'] = result.errorMessage;
    } else {
      response['result'] = result.data?.toJson();
    }
    break;

  case 'addSlot':
    if (payload is! Map<String, dynamic>) {
      response['error'] = 'Invalid payload';
      break;
    }
    final device = DeviceInstanceId.fromJson(payload['device']);
    final slot = ProfileSlot.fromJson(payload['slot']);
    final result = await runtimeService.addSlot(device, slot);
    if (result.hasError) {
      response['error'] = result.errorMessage;
    } else {
      response['result'] = 'ok';
    }
    break;

  // ... similar for other commands
}
```

## JSON Protocol

### Request Format (unchanged)
```json
{
  "command": "commandName",
  "id": 1,
  "payload": { ... }
}
```

### Response Format (unchanged)
```json
{
  "id": 1,
  "result": { ... }
}
// or
{
  "id": 1,
  "error": "error message"
}
```

## Testing Strategy

1. Manual testing via echo/pipe
2. Shell script test suite
3. Error case coverage
