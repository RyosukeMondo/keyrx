/// Virtual layout definitions mirrored from Rust.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

import 'config_ids.dart';
import 'virtual_layout_type.dart';

part 'virtual_layout.freezed.dart';
part 'virtual_layout.g.dart';

/// Visual position for editor rendering.
@freezed
class KeyPosition with _$KeyPosition {
  const factory KeyPosition({required double x, required double y}) =
      _KeyPosition;

  const KeyPosition._();

  factory KeyPosition.fromJson(Map<String, dynamic> json) =>
      _$KeyPositionFromJson(json);
}

/// Visual size for editor rendering.
@freezed
class KeySize with _$KeySize {
  const factory KeySize({required double width, required double height}) =
      _KeySize;

  const KeySize._();

  factory KeySize.fromJson(Map<String, dynamic> json) =>
      _$KeySizeFromJson(json);
}

/// Definition of a single virtual key within a layout.
@freezed
class VirtualKeyDef with _$VirtualKeyDef {
  const factory VirtualKeyDef({
    required VirtualKeyId id,
    required String label,
    KeyPosition? position,
    KeySize? size,
    int? row,
    int? column,
  }) = _VirtualKeyDef;

  const VirtualKeyDef._();

  factory VirtualKeyDef.fromJson(Map<String, dynamic> json) =>
      _$VirtualKeyDefFromJson(json);
}

/// Layout-agnostic representation of keys used by wiring and mapping.
@freezed
class VirtualLayout with _$VirtualLayout {
  const factory VirtualLayout({
    required VirtualLayoutId id,
    required String name,
    @JsonKey(name: 'layout_type') required VirtualLayoutType layoutType,
    @Default(<VirtualKeyDef>[]) List<VirtualKeyDef> keys,
  }) = _VirtualLayout;

  const VirtualLayout._();

  factory VirtualLayout.fromJson(Map<String, dynamic> json) =>
      _$VirtualLayoutFromJson(json);
}
