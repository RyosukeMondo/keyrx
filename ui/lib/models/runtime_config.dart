/// Runtime configuration models mirrored from Rust.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

import 'config_ids.dart';
import 'hardware_profile.dart';

part 'runtime_config.freezed.dart';
part 'runtime_config.g.dart';

/// Runtime assignment for a device slot.
@freezed
class ProfileSlot with _$ProfileSlot {
  const factory ProfileSlot({
    required String id,
    @JsonKey(name: 'hardware_profile_id')
    required HardwareProfileId hardwareProfileId,
    @JsonKey(name: 'keymap_id') required KeymapId keymapId,
    @Default(false) bool active,
    @Default(0) int priority,
  }) = _ProfileSlot;

  const ProfileSlot._();

  factory ProfileSlot.fromJson(Map<String, dynamic> json) =>
      _$ProfileSlotFromJson(json);
}

/// Runtime slots associated to a single device instance.
@freezed
class DeviceSlots with _$DeviceSlots {
  const factory DeviceSlots({
    required DeviceInstanceId device,
    @Default(<ProfileSlot>[]) List<ProfileSlot> slots,
  }) = _DeviceSlots;

  const DeviceSlots._();

  factory DeviceSlots.fromJson(Map<String, dynamic> json) =>
      _$DeviceSlotsFromJson(json);
}

/// Live runtime configuration for all connected devices.
@freezed
class RuntimeConfig with _$RuntimeConfig {
  const factory RuntimeConfig({
    @Default(<DeviceSlots>[]) List<DeviceSlots> devices,
  }) = _RuntimeConfig;

  const RuntimeConfig._();

  factory RuntimeConfig.fromJson(Map<String, dynamic> json) =>
      _$RuntimeConfigFromJson(json);
}
