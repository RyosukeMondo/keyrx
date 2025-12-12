/// Action binding mirrored from Rust ActionBinding enum.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

part 'action_binding.freezed.dart';
part 'action_binding.g.dart';

@Freezed(unionKey: 'type', unionValueCase: FreezedUnionCase.snake)
sealed class ActionBinding with _$ActionBinding {
  const factory ActionBinding.standardKey({
    @JsonKey(name: 'value') required String value,
  }) = ActionBindingStandardKey;

  const factory ActionBinding.macro({
    @JsonKey(name: 'value') required String value,
  }) = ActionBindingMacro;

  const factory ActionBinding.layerToggle({
    @JsonKey(name: 'value') required String value,
  }) = ActionBindingLayerToggle;

  const factory ActionBinding.tapHold({
    @JsonKey(name: 'value') required List<String> value,
  }) = ActionBindingTapHold;

  const factory ActionBinding.transparent() = ActionBindingTransparent;

  factory ActionBinding.fromJson(Map<String, dynamic> json) =>
      _$ActionBindingFromJson(json);
}
