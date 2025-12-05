// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'device_state.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$DeviceStateImpl _$$DeviceStateImplFromJson(Map<String, dynamic> json) =>
    _$DeviceStateImpl(
      identity: DeviceIdentity.fromJson(
        json['identity'] as Map<String, dynamic>,
      ),
      remapEnabled: json['remap_enabled'] as bool,
      profileId: json['profile_id'] as String?,
      connectedAt: json['connected_at'] as String,
      updatedAt: json['updated_at'] as String,
    );

Map<String, dynamic> _$$DeviceStateImplToJson(_$DeviceStateImpl instance) =>
    <String, dynamic>{
      'identity': instance.identity,
      'remap_enabled': instance.remapEnabled,
      'profile_id': instance.profileId,
      'connected_at': instance.connectedAt,
      'updated_at': instance.updatedAt,
    };
