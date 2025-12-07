/// Keymap models mirrored from Rust.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

import 'action_binding.dart';
import 'config_ids.dart';

part 'keymap.freezed.dart';
part 'keymap.g.dart';

/// A single logical layer of a keymap (virtual key -> action).
@freezed
class KeymapLayer with _$KeymapLayer {
  const factory KeymapLayer({
    required String name,
    @Default(<VirtualKeyId, ActionBinding>{})
    Map<VirtualKeyId, ActionBinding> bindings,
  }) = _KeymapLayer;

  const KeymapLayer._();

  factory KeymapLayer.fromJson(Map<String, dynamic> json) =>
      _$KeymapLayerFromJson(json);
}

/// Logical mapping definition attached to a virtual layout.
@freezed
class Keymap with _$Keymap {
  const factory Keymap({
    required KeymapId id,
    required String name,
    @JsonKey(name: 'virtual_layout_id')
    required VirtualLayoutId virtualLayoutId,
    @Default(<KeymapLayer>[]) List<KeymapLayer> layers,
  }) = _Keymap;

  const Keymap._();

  factory Keymap.fromJson(Map<String, dynamic> json) => _$KeymapFromJson(json);
}
