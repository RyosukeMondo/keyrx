/// Device profile FFI methods.
///
/// Provides device profile access for viewing row-column mappings.
library;

import '../models/bridge_models.dart';
import 'bindings.dart';

/// Mixin providing device profile FFI methods.
mixin BridgeDeviceProfileMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Get device profile for a specific device.
  ///
  /// Returns the complete device profile including keymap and layout.
  DeviceProfileResult getDeviceProfile(int vendorId, int productId) {
    if (bindings == null) {
      return DeviceProfileResult.error('getDeviceProfile not available');
    }

    try {
      final raw = bindings!.compat.getDeviceProfile(vendorId, productId);
      return DeviceProfileResult.parse(raw);
    } catch (e) {
      return DeviceProfileResult.error('$e');
    }
  }

  /// Check if a device profile exists.
  ///
  /// Returns true if a profile has been created for this device.
  bool hasDeviceProfile(int vendorId, int productId) {
    if (bindings == null) {
      return false;
    }

    try {
      final raw = bindings!.compat.hasDeviceProfile(vendorId, productId);
      final trimmed = raw.trim();

      // Parse "ok:true" or "ok:false" response
      if (trimmed.toLowerCase().startsWith('ok:')) {
        final payload = trimmed.substring(trimmed.indexOf(':') + 1).trim();
        return payload.toLowerCase() == 'true';
      }

      return false;
    } catch (e) {
      return false;
    }
  }

  /// Save a device profile to disk.
  ///
  /// [profileJson] - JSON representation of the DeviceProfile.
  ///
  /// Returns true if successful.
  bool saveDeviceProfile(String profileJson) {
    if (bindings == null) return false;

    try {
      final raw = bindings!.compat.saveDeviceProfile(profileJson);
      final trimmed = raw.trim();

      // Parse "ok:" response (or empty ok for void result)
      if (trimmed.toLowerCase().startsWith('ok:')) {
        return true;
      }
      print('saveDeviceProfile failed. Rust returned: $trimmed');
      return false;
    } catch (e) {
      print('saveDeviceProfile exception: $e');
      return false;
    }
  }
}
