// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'hardware_profile.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_DeviceInstanceId _$DeviceInstanceIdFromJson(Map<String, dynamic> json) =>
    _DeviceInstanceId(
      vendorId: (json['vendor_id'] as num).toInt(),
      productId: (json['product_id'] as num).toInt(),
      serial: json['serial'] as String?,
    );

Map<String, dynamic> _$DeviceInstanceIdToJson(_DeviceInstanceId instance) =>
    <String, dynamic>{
      'vendor_id': instance.vendorId,
      'product_id': instance.productId,
      'serial': instance.serial,
    };

_HardwareProfile _$HardwareProfileFromJson(Map<String, dynamic> json) =>
    _HardwareProfile(
      id: json['id'] as String,
      virtualLayoutId: json['virtual_layout_id'] as String,
      vendorId: (json['vendor_id'] as num?)?.toInt() ?? 0,
      productId: (json['product_id'] as num?)?.toInt() ?? 0,
      serialNumber: json['serial_number'] as String?,
      name: json['name'] as String?,
      wiring:
          (json['wiring'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(int.parse(k), e as String),
          ) ??
          const <int, VirtualKeyId>{},
    );

Map<String, dynamic> _$HardwareProfileToJson(_HardwareProfile instance) =>
    <String, dynamic>{
      'id': instance.id,
      'virtual_layout_id': instance.virtualLayoutId,
      'vendor_id': instance.vendorId,
      'product_id': instance.productId,
      'serial_number': instance.serialNumber,
      'name': instance.name,
      'wiring': instance.wiring.map((k, e) => MapEntry(k.toString(), e)),
    };
