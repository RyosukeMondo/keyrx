/// Profile data model for revolutionary mapping.
///
/// Defines key mapping profiles with physical positions and actions,
/// matching the Rust Profile struct.
library;

import 'package:freezed_annotation/freezed_annotation.dart';
import 'layout_type.dart';

part 'profile.freezed.dart';
part 'profile.g.dart';

/// Physical position of a key on a device layout.
///
/// Used to identify keys by their physical location (row, column)
/// rather than their scancode, enabling layout-aware remapping.
@freezed
class PhysicalPosition with _$PhysicalPosition {
  const factory PhysicalPosition({
    /// Row index (0-based)
    required int row,

    /// Column index (0-based)
    required int col,
  }) = _PhysicalPosition;

  const PhysicalPosition._();

  /// Create from JSON
  factory PhysicalPosition.fromJson(Map<String, dynamic> json) =>
      _$PhysicalPositionFromJson(json);

  /// Convert to a string key for serialization (format: "row,col")
  String toKey() => '$row,$col';

  /// Parse from a string key (format: "row,col")
  static PhysicalPosition? fromKey(String key) {
    final parts = key.split(',');
    if (parts.length != 2) {
      return null;
    }
    final row = int.tryParse(parts[0]);
    final col = int.tryParse(parts[1]);
    if (row == null || col == null) {
      return null;
    }
    return PhysicalPosition(row: row, col: col);
  }

  @override
  String toString() => '$row,$col';
}

/// Action to perform when a key is pressed.
///
/// Defines the behavior of a mapped key, supporting simple remaps,
/// chords, scripts, blocking, and passthrough.
@Freezed(unionKey: 'type')
sealed class KeyAction with _$KeyAction {
  /// Remap to a single key
  const factory KeyAction.key({
    /// The output key to emit
    required String key,
  }) = KeyActionKey;

  /// Remap to a chord (multiple keys pressed simultaneously)
  const factory KeyAction.chord({
    /// Keys to press together
    required List<String> keys,
  }) = KeyActionChord;

  /// Execute a script/command
  const factory KeyAction.script({
    /// Script identifier or command to run
    required String script,
  }) = KeyActionScript;

  /// Block the key (no output)
  const factory KeyAction.block() = KeyActionBlock;

  /// Pass through unchanged (default behavior)
  const factory KeyAction.pass() = KeyActionPass;

  /// Create from JSON
  factory KeyAction.fromJson(Map<String, dynamic> json) =>
      _$KeyActionFromJson(json);
}

/// A key mapping profile.
///
/// Profiles define how keys are remapped for devices with a specific layout.
/// They are layout-aware and can be assigned to multiple devices with the
/// same layout type.
@freezed
class Profile with _$Profile {
  const factory Profile({
    /// Unique identifier (UUID v4)
    required String id,

    /// Human-readable profile name
    required String name,

    /// Layout type this profile is designed for
    @JsonKey(name: 'layout_type') required LayoutType layoutType,

    /// Key mappings: physical position key → action
    /// Only contains entries for remapped keys (sparse map)
    /// Serialized as {"row,col": action} in JSON
    @Default({}) Map<String, KeyAction> mappings,

    /// Creation timestamp (ISO 8601)
    @JsonKey(name: 'created_at') required String createdAt,

    /// Last modification timestamp (ISO 8601)
    @JsonKey(name: 'updated_at') required String updatedAt,
  }) = _Profile;

  const Profile._();

  /// Create from JSON
  factory Profile.fromJson(Map<String, dynamic> json) =>
      _$ProfileFromJson(json);

  /// Get the action for a physical position
  KeyAction? getAction(PhysicalPosition pos) {
    return mappings[pos.toKey()];
  }

  /// Check if this profile is compatible with a given layout type
  bool isCompatibleWith(LayoutType other) {
    return layoutType.isCompatibleWith(other);
  }

  /// Get the number of mapped keys
  int get mappingCount => mappings.length;

  /// Check if the profile has any mappings
  bool get hasMapping => mappings.isNotEmpty;
}
