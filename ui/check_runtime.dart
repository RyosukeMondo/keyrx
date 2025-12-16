import 'dart:convert';
import 'dart:io';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/runtime_config.dart';

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
  stderr.writeln('Raw config JSON:');
  stderr.writeln(configJson);
  stderr.writeln('\n');

  final config = RuntimeConfig.fromJson(jsonDecode(configJson) as Map<String, dynamic>);

  stderr.writeln('Parsed Runtime Config:');
  stderr.writeln('- Devices: ${config.devices.length}');

  if (config.devices.isEmpty) {
    print('\n⚠️  No devices configured!');
    print('\nTo make the flow visible, you need to add a runtime slot.');
    print('Example:');
    print('  Device: 2be8:0213:/dev/input/event24');
    print('  Hardware Profile: hardware_1765788359604');
    print('  Keymap: keymap_1765788388188');
  } else {
    for (final deviceSlots in config.devices) {
      stderr.writeln('\nDevice: ${deviceSlots.device}');
      stderr.writeln('  Slots: ${deviceSlots.slots.length}');
      for (final slot in deviceSlots.slots) {
        stderr.writeln('    - Slot ${slot.id}:');
        stderr.writeln('      Hardware Profile: ${slot.hardwareProfileId}');
        stderr.writeln('      Keymap: ${slot.keymapId}');
        stderr.writeln('      Active: ${slot.active}');
        stderr.writeln('      Priority: ${slot.priority}');
      }
    }
  }
}
