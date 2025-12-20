import 'dart:convert';
import 'dart:io';

import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/models/runtime_config.dart';

void main() async {
  stderr.writeln('Checking runtime configuration...\n');

  final bridge = KeyrxBridge.open();
  final bindings = bridge.runtimeBindings;

  if (bindings == null) {
    stderr.writeln('ERROR: Runtime bindings not available');
    exit(1);
  }

  // Get current runtime config
  final configJson = bindings.getConfig();
  stderr
    ..writeln('Raw config JSON:')
    ..writeln(configJson)
    ..writeln('\n');

  final config = RuntimeConfig.fromJson(
    jsonDecode(configJson) as Map<String, dynamic>,
  );

  stderr
    ..writeln('Parsed Runtime Config:')
    ..writeln('- Devices: ${config.devices.length}');

  if (config.devices.isEmpty) {
    stdout
      ..writeln('\n⚠️  No devices configured!')
      ..writeln('\nTo make the flow visible, you need to add a runtime slot.')
      ..writeln('Example:')
      ..writeln('  Device: 2be8:0213:/dev/input/event24')
      ..writeln('  Hardware Profile: hardware_1765788359604')
      ..writeln('  Keymap: keymap_1765788388188');
  } else {
    for (final deviceSlots in config.devices) {
      stderr
        ..writeln('\nDevice: ${deviceSlots.device}')
        ..writeln('  Slots: ${deviceSlots.slots.length}');
      for (final slot in deviceSlots.slots) {
        stderr
          ..writeln('    - Slot ${slot.id}:')
          ..writeln('      Hardware Profile: ${slot.hardwareProfileId}')
          ..writeln('      Keymap: ${slot.keymapId}')
          ..writeln('      Active: ${slot.active}')
          ..writeln('      Priority: ${slot.priority}');
      }
    }
  }
}
