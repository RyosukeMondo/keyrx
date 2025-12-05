/// Device identity data model for revolutionary mapping.
///
/// Represents the unique identifier for a physical device instance,
/// matching the Rust DeviceIdentity struct.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

part 'device_identity.freezed.dart';
part 'device_identity.g.dart';

/// Unique identifier for a physical device instance.
///
/// Unlike simple VID:PID identification, DeviceIdentity includes
/// serial number to distinguish between multiple identical devices.
@freezed
class DeviceIdentity with _$DeviceIdentity {
  const factory DeviceIdentity({
    /// USB Vendor ID (e.g., 0x046D for Logitech)
    @JsonKey(name: 'vendor_id') required int vendorId,

    /// USB Product ID (e.g., 0xC52B for specific device model)
    @JsonKey(name: 'product_id') required int productId,

    /// Device serial number extracted from USB descriptors or generated
    @JsonKey(name: 'serial_number') required String serialNumber,

    /// Optional user-assigned label for easier identification in UI
    @JsonKey(name: 'user_label', includeIfNull: false) String? userLabel,
  }) = _DeviceIdentity;

  const DeviceIdentity._();

  /// Create from JSON
  factory DeviceIdentity.fromJson(Map<String, dynamic> json) =>
      _$DeviceIdentityFromJson(json);

  /// Convert to a string key suitable for HashMap keys and file storage.
  ///
  /// Format: `{vendor_id:04x}:{product_id:04x}:{serial_number}`
  /// Example: `046d:c52b:ABC123456`
  String toKey() {
    final vid = vendorId.toRadixString(16).padLeft(4, '0');
    final pid = productId.toRadixString(16).padLeft(4, '0');
    return '$vid:$pid:$serialNumber';
  }

  /// Parse a DeviceIdentity from a key string.
  ///
  /// Expected format: `{vendor_id:04x}:{product_id:04x}:{serial_number}`
  static DeviceIdentity? fromKey(String key) {
    final parts = key.split(':');
    if (parts.length < 3) {
      return null;
    }

    final vendorId = int.tryParse(parts[0], radix: 16);
    final productId = int.tryParse(parts[1], radix: 16);

    if (vendorId == null || productId == null) {
      return null;
    }

    // Serial number is everything after the second colon
    // (in case serial contains colons)
    final serialNumber = parts.sublist(2).join(':');

    return DeviceIdentity(
      vendorId: vendorId,
      productId: productId,
      serialNumber: serialNumber,
    );
  }

  /// Get a display name for the device.
  /// Uses user label if set, otherwise falls back to VID:PID.
  String get displayName {
    if (userLabel != null && userLabel!.isNotEmpty) {
      return userLabel!;
    }
    final vid = vendorId.toRadixString(16).padLeft(4, '0');
    final pid = productId.toRadixString(16).padLeft(4, '0');
    return '$vid:$pid';
  }

  /// Get a full display string with all information.
  String get fullDisplayString {
    if (userLabel != null && userLabel!.isNotEmpty) {
      return '$userLabel (${toKey()})';
    }
    return toKey();
  }
}
