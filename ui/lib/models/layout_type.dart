/// Layout type data model for revolutionary mapping.
///
/// Defines the physical layout structure of a device,
/// matching the Rust LayoutType enum.
library;

import 'package:json_annotation/json_annotation.dart';

/// Layout type for device profiles.
///
/// Defines the physical layout structure of a device, which determines
/// how keys are organized and mapped.
@JsonEnum(fieldRename: FieldRename.none)
enum LayoutType {
  /// Standard keyboard layout (ANSI/ISO)
  /// Keys are identified by standard positions
  @JsonValue('standard')
  standard,

  /// Matrix layout (e.g., macro pads, Stream Deck)
  /// Keys are organized in a grid (row, col)
  @JsonValue('matrix')
  matrix,

  /// Split keyboard layout
  /// Two independent halves with separate coordinate systems
  @JsonValue('split')
  split,
}

/// Extension methods for LayoutType
extension LayoutTypeExtension on LayoutType {
  /// Get a display label for this layout type
  String get label {
    switch (this) {
      case LayoutType.standard:
        return 'Standard';
      case LayoutType.matrix:
        return 'Matrix';
      case LayoutType.split:
        return 'Split';
    }
  }

  /// Get a description for this layout type
  String get description {
    switch (this) {
      case LayoutType.standard:
        return 'Standard keyboard layout (ANSI/ISO)';
      case LayoutType.matrix:
        return 'Matrix layout (macro pads, Stream Deck)';
      case LayoutType.split:
        return 'Split keyboard layout';
    }
  }

  /// Check if this layout type is compatible with another
  bool isCompatibleWith(LayoutType other) {
    return this == other;
  }

  /// Convert to JSON string representation
  String toJsonString() {
    switch (this) {
      case LayoutType.standard:
        return 'standard';
      case LayoutType.matrix:
        return 'matrix';
      case LayoutType.split:
        return 'split';
    }
  }
}
