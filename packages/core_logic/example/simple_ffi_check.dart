import 'dart:io';
import 'package:core_logic/ffi/bridge.dart';

void main() {
  print('Attempting to load KeyRxBridge...');
  try {
    final bridge = KeyrxBridge.open();
    if (bridge.loadFailure != null) {
      print('Load Failure: ${bridge.loadFailure}');
      exit(1);
    }
    
    print('Bridge loaded successfully.');
    print('Protocol Version: ${bridge.version}'); // Version is from BridgeCoreMixin
    
    // Check if initialized (requires setConfigRoot usually)
    // bridge.setConfigRoot('/tmp');
    // bridge.initialize();
    // print('Initialized: ${bridge.initialized}');
    
    // Check if we can get a device profile (will fail/empty but exercises FFI)
    final profile = bridge.getDeviceProfile(0x1234, 0x5678);
    print('Get Profile Result: ${profile.errorMessage ?? "Success"}');

  } catch (e, st) {
    print('Exception: $e');
    print(st);
    exit(1);
  }
}
