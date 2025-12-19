// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'virtual_layout.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_KeyPosition _$KeyPositionFromJson(Map<String, dynamic> json) => _KeyPosition(
  x: (json['x'] as num).toDouble(),
  y: (json['y'] as num).toDouble(),
);

Map<String, dynamic> _$KeyPositionToJson(_KeyPosition instance) =>
    <String, dynamic>{'x': instance.x, 'y': instance.y};

_KeySize _$KeySizeFromJson(Map<String, dynamic> json) => _KeySize(
  width: (json['width'] as num).toDouble(),
  height: (json['height'] as num).toDouble(),
);

Map<String, dynamic> _$KeySizeToJson(_KeySize instance) => <String, dynamic>{
  'width': instance.width,
  'height': instance.height,
};

_VirtualKeyDef _$VirtualKeyDefFromJson(Map<String, dynamic> json) =>
    _VirtualKeyDef(
      id: json['id'] as String,
      label: json['label'] as String,
      position: json['position'] == null
          ? null
          : KeyPosition.fromJson(json['position'] as Map<String, dynamic>),
      size: json['size'] == null
          ? null
          : KeySize.fromJson(json['size'] as Map<String, dynamic>),
      row: (json['row'] as num?)?.toInt(),
      column: (json['column'] as num?)?.toInt(),
    );

Map<String, dynamic> _$VirtualKeyDefToJson(_VirtualKeyDef instance) =>
    <String, dynamic>{
      'id': instance.id,
      'label': instance.label,
      'position': instance.position,
      'size': instance.size,
      'row': instance.row,
      'column': instance.column,
    };

_VirtualLayout _$VirtualLayoutFromJson(Map<String, dynamic> json) =>
    _VirtualLayout(
      id: json['id'] as String,
      name: json['name'] as String,
      layoutType: $enumDecode(_$VirtualLayoutTypeEnumMap, json['layout_type']),
      keys:
          (json['keys'] as List<dynamic>?)
              ?.map((e) => VirtualKeyDef.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const <VirtualKeyDef>[],
    );

Map<String, dynamic> _$VirtualLayoutToJson(_VirtualLayout instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'layout_type': _$VirtualLayoutTypeEnumMap[instance.layoutType]!,
      'keys': instance.keys,
    };

const _$VirtualLayoutTypeEnumMap = {
  VirtualLayoutType.matrix: 'matrix',
  VirtualLayoutType.semantic: 'semantic',
};
