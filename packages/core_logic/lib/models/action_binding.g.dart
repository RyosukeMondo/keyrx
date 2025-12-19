// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'action_binding.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ActionBindingStandardKey _$ActionBindingStandardKeyFromJson(
  Map<String, dynamic> json,
) => ActionBindingStandardKey(
  value: json['value'] as String? ?? '',
  $type: json['type'] as String?,
);

Map<String, dynamic> _$ActionBindingStandardKeyToJson(
  ActionBindingStandardKey instance,
) => <String, dynamic>{'value': instance.value, 'type': instance.$type};

ActionBindingMacro _$ActionBindingMacroFromJson(Map<String, dynamic> json) =>
    ActionBindingMacro(
      value: json['value'] as String? ?? '',
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$ActionBindingMacroToJson(ActionBindingMacro instance) =>
    <String, dynamic>{'value': instance.value, 'type': instance.$type};

ActionBindingLayerToggle _$ActionBindingLayerToggleFromJson(
  Map<String, dynamic> json,
) => ActionBindingLayerToggle(
  value: json['value'] as String? ?? '',
  $type: json['type'] as String?,
);

Map<String, dynamic> _$ActionBindingLayerToggleToJson(
  ActionBindingLayerToggle instance,
) => <String, dynamic>{'value': instance.value, 'type': instance.$type};

ActionBindingTapHold _$ActionBindingTapHoldFromJson(
  Map<String, dynamic> json,
) => ActionBindingTapHold(
  value:
      (json['value'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const [],
  $type: json['type'] as String?,
);

Map<String, dynamic> _$ActionBindingTapHoldToJson(
  ActionBindingTapHold instance,
) => <String, dynamic>{'value': instance.value, 'type': instance.$type};

ActionBindingTransparent _$ActionBindingTransparentFromJson(
  Map<String, dynamic> json,
) => ActionBindingTransparent($type: json['type'] as String?);

Map<String, dynamic> _$ActionBindingTransparentToJson(
  ActionBindingTransparent instance,
) => <String, dynamic>{'type': instance.$type};
