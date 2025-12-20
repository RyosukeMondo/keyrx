/// Device discovery FFI methods.
///
/// Provides device discovery and listing functionality for the KeyRx bridge.
library;

import 'bindings.dart';

/// Mixin providing device discovery and listing FFI methods.
mixin BridgeDiscoveryMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Select a device by path for the engine to use.
  ///
  /// Returns 0 on success, negative on error:
  /// - -1: Null pointer
  /// - -2: Invalid UTF-8
  /// - -3: Device path does not exist
  /// - -4: Lock error
  int selectDevice(String path) {
    if (bindings == null) return -1;

    try {
      return bindings!.compat.selectDevice(path);
    } catch (_) {
      return -1;
    }
  }

  // NOTE: Legacy discovery methods (startDiscovery, etc.) have been removed.
  // Use DeviceRegistryService for device management.
}
