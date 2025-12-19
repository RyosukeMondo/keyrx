import 'package:core_logic/ffi/bridge.dart';

void main() {
  final bridge = KeyrxBridge.open();
  if (bridge.loadFailure != null) {
    print('Failed to load KeyRX bridge: ${bridge.loadFailure}');
    return;
  }
  
  print('KeyRX Core Version: ${bridge.version}');
  
  if (bridge.initialize()) {
    print('Engine initialized successfully.');
  } else {
    print('Failed to initialize engine.');
  }
  
  bridge.dispose();
}