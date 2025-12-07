// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'keymap.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$KeymapLayerImpl _$$KeymapLayerImplFromJson(Map<String, dynamic> json) =>
    _$KeymapLayerImpl(
      name: json['name'] as String,
      bindings:
          (json['bindings'] as Map<String, dynamic>?)?.map(
            (k, e) =>
                MapEntry(k, ActionBinding.fromJson(e as Map<String, dynamic>)),
          ) ??
          const <VirtualKeyId, ActionBinding>{},
    );

Map<String, dynamic> _$$KeymapLayerImplToJson(_$KeymapLayerImpl instance) =>
    <String, dynamic>{'name': instance.name, 'bindings': instance.bindings};

_$KeymapImpl _$$KeymapImplFromJson(Map<String, dynamic> json) => _$KeymapImpl(
  id: json['id'] as String,
  name: json['name'] as String,
  virtualLayoutId: json['virtual_layout_id'] as String,
  layers:
      (json['layers'] as List<dynamic>?)
          ?.map((e) => KeymapLayer.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <KeymapLayer>[],
);

Map<String, dynamic> _$$KeymapImplToJson(_$KeymapImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'virtual_layout_id': instance.virtualLayoutId,
      'layers': instance.layers,
    };
