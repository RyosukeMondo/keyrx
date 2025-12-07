/// Hardware wiring profile for a physical device.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

import 'config_ids.dart';

part 'hardware_profile.freezed.dart';
part 'hardware_profile.g.dart';

/// Identifier for a concrete device instance.
@freezed
class DeviceInstanceId with _$DeviceInstanceId {
  const factory DeviceInstanceId({
    @JsonKey(name: 'vendor_id') required int vendorId,
    @JsonKey(name: 'product_id') required int productId,
    String? serial,
  }) = _DeviceInstanceId;

  const DeviceInstanceId._();

  factory DeviceInstanceId.fromJson(Map<String, dynamic> json) =>
      _$DeviceInstanceIdFromJson(json);
}

/// Wiring definition from physical scancodes to virtual keys.
@freezed
class HardwareProfile with _$HardwareProfile {
  const factory HardwareProfile({
    required HardwareProfileId id,
    @JsonKey(name: 'vendor_id') required int vendorId,
    @JsonKey(name: 'product_id') required int productId,
    String? name,
    @JsonKey(name: 'virtual_layout_id')
    required VirtualLayoutId virtualLayoutId,
    @Default(<int, VirtualKeyId>{}) Map<int, VirtualKeyId> wiring,
  }) = _HardwareProfile;

  const HardwareProfile._();

  factory HardwareProfile.fromJson(Map<String, dynamic> json) =>
      _$HardwareProfileFromJson(json);
}
