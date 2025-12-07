/// Layout type for virtual key definitions.
library;

import 'package:json_annotation/json_annotation.dart';

/// Supported virtual layout representations.
@JsonEnum(fieldRename: FieldRename.snake)
enum VirtualLayoutType {
  /// Grid-style layout keyed by coordinates.
  matrix,

  /// Semantic/named layout keyed by labels.
  semantic,
}

extension VirtualLayoutTypeX on VirtualLayoutType {
  /// Human-friendly label for UI.
  String get label {
    switch (this) {
      case VirtualLayoutType.matrix:
        return 'Matrix';
      case VirtualLayoutType.semantic:
        return 'Semantic';
    }
  }

  /// Description for editors.
  String get description {
    switch (this) {
      case VirtualLayoutType.matrix:
        return 'Grid-based layout addressed by row/col coordinates.';
      case VirtualLayoutType.semantic:
        return 'Named keys (e.g., F1, ESC) independent of physical position.';
    }
  }

  /// Serialize to snake_case string.
  String toJsonString() {
    switch (this) {
      case VirtualLayoutType.matrix:
        return 'matrix';
      case VirtualLayoutType.semantic:
        return 'semantic';
    }
  }
}
