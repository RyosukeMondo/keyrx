// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'profile.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_PhysicalPosition _$PhysicalPositionFromJson(Map<String, dynamic> json) =>
    _PhysicalPosition(
      row: (json['row'] as num).toInt(),
      col: (json['col'] as num).toInt(),
    );

Map<String, dynamic> _$PhysicalPositionToJson(_PhysicalPosition instance) =>
    <String, dynamic>{'row': instance.row, 'col': instance.col};

KeyActionKey _$KeyActionKeyFromJson(Map<String, dynamic> json) =>
    KeyActionKey(key: json['key'] as String, $type: json['type'] as String?);

Map<String, dynamic> _$KeyActionKeyToJson(KeyActionKey instance) =>
    <String, dynamic>{'key': instance.key, 'type': instance.$type};

KeyActionChord _$KeyActionChordFromJson(Map<String, dynamic> json) =>
    KeyActionChord(
      keys: (json['keys'] as List<dynamic>).map((e) => e as String).toList(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$KeyActionChordToJson(KeyActionChord instance) =>
    <String, dynamic>{'keys': instance.keys, 'type': instance.$type};

KeyActionScript _$KeyActionScriptFromJson(Map<String, dynamic> json) =>
    KeyActionScript(
      script: json['script'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$KeyActionScriptToJson(KeyActionScript instance) =>
    <String, dynamic>{'script': instance.script, 'type': instance.$type};

KeyActionBlock _$KeyActionBlockFromJson(Map<String, dynamic> json) =>
    KeyActionBlock($type: json['type'] as String?);

Map<String, dynamic> _$KeyActionBlockToJson(KeyActionBlock instance) =>
    <String, dynamic>{'type': instance.$type};

KeyActionPass _$KeyActionPassFromJson(Map<String, dynamic> json) =>
    KeyActionPass($type: json['type'] as String?);

Map<String, dynamic> _$KeyActionPassToJson(KeyActionPass instance) =>
    <String, dynamic>{'type': instance.$type};

_Profile _$ProfileFromJson(Map<String, dynamic> json) => _Profile(
  id: json['id'] as String,
  name: json['name'] as String,
  layoutType: $enumDecode(_$LayoutTypeEnumMap, json['layout_type']),
  createdAt: json['created_at'] as String,
  updatedAt: json['updated_at'] as String,
  mappings:
      (json['mappings'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, KeyAction.fromJson(e as Map<String, dynamic>)),
      ) ??
      const {},
);

Map<String, dynamic> _$ProfileToJson(_Profile instance) => <String, dynamic>{
  'id': instance.id,
  'name': instance.name,
  'layout_type': _$LayoutTypeEnumMap[instance.layoutType]!,
  'created_at': instance.createdAt,
  'updated_at': instance.updatedAt,
  'mappings': instance.mappings,
};

const _$LayoutTypeEnumMap = {
  LayoutType.standard: 'standard',
  LayoutType.matrix: 'matrix',
  LayoutType.split: 'split',
};
