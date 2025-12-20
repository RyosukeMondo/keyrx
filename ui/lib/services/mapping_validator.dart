/// UI-specific service for validating key mapping user input.
///
/// **UI LAYER RESPONSIBILITY - NOT A SSOT VIOLATION**
///
/// This validator handles UI-level validation of user input before submitting
/// to core_logic services. It is legitimately UI-specific because:
///
/// 1. **User Input Validation**: Validates form data from UI components
///    (checking for empty fields, required fields, etc.)
///
/// 2. **User-Facing Error Messages**: Provides human-readable error messages
///    specifically designed for display in UI dialogs and forms.
///
/// 3. **UI-Specific Key List**: Uses the runtime-updatable UI key list from
///    KeyMappings.allowedKeys, which is populated from the native bridge
///    and may differ from core validation rules.
///
/// 4. **No Core Duplication**: Core_logic does NOT contain equivalent
///    validation logic. Core layer trusts UI to validate input.
///
/// This follows proper layering: UI validates user input, core_logic
/// handles business logic and domain rules.
library;

import 'package:core_logic/models/key_mapping.dart';
import 'key_mappings_util.dart';

/// Result of a mapping validation operation.
class MappingValidationResult {
  const MappingValidationResult._({required this.isValid, this.errorMessage});

  /// Creates a successful validation result.
  const MappingValidationResult.valid() : this._(isValid: true);

  /// Creates a failed validation result with an error message.
  const MappingValidationResult.invalid(String message)
    : this._(isValid: false, errorMessage: message);

  /// Whether the validation passed.
  final bool isValid;

  /// Error message if validation failed.
  final String? errorMessage;
}

/// Validates key mappings and combo configurations from UI forms.
///
/// **UI-SPECIFIC INPUT VALIDATION**
///
/// This class validates user input from UI forms before submitting to core_logic.
/// It checks for:
/// - Required fields are filled
/// - Keys exist in the UI's allowed key list
/// - Combos have minimum required keys
/// - Appropriate error messages for UI display
///
/// This is NOT business logic - it's input validation at the UI boundary.
/// Core_logic does not duplicate this validation.
///
/// Pure validation logic with no Flutter dependencies, making it testable in isolation.
class MappingValidator {
  const MappingValidator();

  /// Validates a key mapping before applying.
  ///
  /// Checks:
  /// - fromKey is not null/empty and is a known key
  /// - For remap actions, the target key is provided and known
  MappingValidationResult validateMapping(String? fromKey, KeyMapping mapping) {
    if (fromKey == null || fromKey.isEmpty) {
      return const MappingValidationResult.invalid('No key selected.');
    }

    if (!KeyMappings.isKnownKey(fromKey)) {
      return MappingValidationResult.invalid('Unknown key: $fromKey');
    }

    if (mapping.type == KeyActionType.remap) {
      final targetKey = mapping.to?.trim();
      if (targetKey == null || targetKey.isEmpty) {
        return const MappingValidationResult.invalid(
          'Provide a target key for remap.',
        );
      }
      if (!KeyMappings.isKnownKey(targetKey)) {
        return MappingValidationResult.invalid('Unknown target key: $targetKey');
      }
    }

    return const MappingValidationResult.valid();
  }

  /// Validates a combo configuration before adding.
  ///
  /// Checks:
  /// - At least 2 keys are provided
  /// - Output is not empty
  /// - All keys are known
  MappingValidationResult validateCombo(List<String> keys, String output) {
    if (keys.length < 2) {
      return const MappingValidationResult.invalid(
        'Provide at least 2 keys for a combo.',
      );
    }

    if (output.trim().isEmpty) {
      return const MappingValidationResult.invalid('Provide an output for the combo.');
    }

    final unknownKeys = keys.where((k) => !KeyMappings.isKnownKey(k)).toList();
    if (unknownKeys.isNotEmpty) {
      return MappingValidationResult.invalid(
        'Unknown key(s) in combo: ${unknownKeys.join(", ")}',
      );
    }

    return const MappingValidationResult.valid();
  }
}
