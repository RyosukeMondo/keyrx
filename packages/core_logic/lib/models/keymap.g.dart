// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'keymap.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_Combo _$ComboFromJson(Map<String, dynamic> json) => _Combo(
  keys: (json['keys'] as List<dynamic>).map((e) => e as String).toList(),
  output: json['output'] as String,
);

Map<String, dynamic> _$ComboToJson(_Combo instance) => <String, dynamic>{
  'keys': instance.keys,
  'output': instance.output,
};

_KeymapLayer _$KeymapLayerFromJson(Map<String, dynamic> json) => _KeymapLayer(
  name: json['name'] as String,
  bindings:
      (json['bindings'] as Map<String, dynamic>?)?.map(
        (k, e) =>
            MapEntry(k, ActionBinding.fromJson(e as Map<String, dynamic>)),
      ) ??
      const <VirtualKeyId, ActionBinding>{},
);

Map<String, dynamic> _$KeymapLayerToJson(_KeymapLayer instance) =>
    <String, dynamic>{'name': instance.name, 'bindings': instance.bindings};

_Keymap _$KeymapFromJson(Map<String, dynamic> json) => _Keymap(
  id: json['id'] as String,
  virtualLayoutId: json['virtual_layout_id'] as String,
  name: json['name'] as String? ?? 'Untitled Keymap',
  layers:
      (json['layers'] as List<dynamic>?)
          ?.map((e) => KeymapLayer.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <KeymapLayer>[],
  combos:
      (json['combos'] as List<dynamic>?)
          ?.map((e) => Combo.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <Combo>[],
);

Map<String, dynamic> _$KeymapToJson(_Keymap instance) => <String, dynamic>{
  'id': instance.id,
  'virtual_layout_id': instance.virtualLayoutId,
  'name': instance.name,
  'layers': instance.layers,
  'combos': instance.combos,
};
