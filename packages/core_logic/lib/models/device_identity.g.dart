// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'device_identity.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_DeviceIdentity _$DeviceIdentityFromJson(Map<String, dynamic> json) =>
    _DeviceIdentity(
      vendorId: (json['vendor_id'] as num).toInt(),
      productId: (json['product_id'] as num).toInt(),
      serialNumber: json['serial_number'] as String,
      userLabel: json['user_label'] as String?,
    );

Map<String, dynamic> _$DeviceIdentityToJson(_DeviceIdentity instance) =>
    <String, dynamic>{
      'vendor_id': instance.vendorId,
      'product_id': instance.productId,
      'serial_number': instance.serialNumber,
      'user_label': ?instance.userLabel,
    };
