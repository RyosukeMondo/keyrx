// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'runtime_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_ProfileSlot _$ProfileSlotFromJson(Map<String, dynamic> json) => _ProfileSlot(
  id: json['id'] as String,
  hardwareProfileId: json['hardware_profile_id'] as String,
  keymapId: json['keymap_id'] as String,
  active: json['active'] as bool? ?? false,
  priority: (json['priority'] as num?)?.toInt() ?? 0,
);

Map<String, dynamic> _$ProfileSlotToJson(_ProfileSlot instance) =>
    <String, dynamic>{
      'id': instance.id,
      'hardware_profile_id': instance.hardwareProfileId,
      'keymap_id': instance.keymapId,
      'active': instance.active,
      'priority': instance.priority,
    };

_DeviceSlots _$DeviceSlotsFromJson(Map<String, dynamic> json) => _DeviceSlots(
  device: DeviceInstanceId.fromJson(json['device'] as Map<String, dynamic>),
  slots:
      (json['slots'] as List<dynamic>?)
          ?.map((e) => ProfileSlot.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <ProfileSlot>[],
);

Map<String, dynamic> _$DeviceSlotsToJson(_DeviceSlots instance) =>
    <String, dynamic>{'device': instance.device, 'slots': instance.slots};

_RuntimeConfig _$RuntimeConfigFromJson(Map<String, dynamic> json) =>
    _RuntimeConfig(
      devices:
          (json['devices'] as List<dynamic>?)
              ?.map((e) => DeviceSlots.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const <DeviceSlots>[],
    );

Map<String, dynamic> _$RuntimeConfigToJson(_RuntimeConfig instance) =>
    <String, dynamic>{'devices': instance.devices};
