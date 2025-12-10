/// Service for validating key mappings and combos.
///
/// Extracts validation logic from UI components for testability.
library;

import '../models/key_mapping.dart';
import 'key_mappings_util.dart';

/// Result of a mapping validation operation.
class ValidationResult {
  const ValidationResult._({required this.isValid, this.errorMessage});

  /// Creates a successful validation result.
  const ValidationResult.valid() : this._(isValid: true);

  /// Creates a failed validation result with an error message.
  const ValidationResult.invalid(String message)
    : this._(isValid: false, errorMessage: message);

  /// Whether the validation passed.
  final bool isValid;

  /// Error message if validation failed.
  final String? errorMessage;
}

/// Validates key mappings and combo configurations.
///
/// Pure validation logic with no UI dependencies, making it testable in isolation.
class MappingValidator {
  const MappingValidator();

  /// Validates a key mapping before applying.
  ///
  /// Checks:
  /// - fromKey is not null/empty and is a known key
  /// - For remap actions, the target key is provided and known
  ValidationResult validateMapping(String? fromKey, KeyMapping mapping) {
    if (fromKey == null || fromKey.isEmpty) {
      return const ValidationResult.invalid('No key selected.');
    }

    if (!KeyMappings.isKnownKey(fromKey)) {
      return ValidationResult.invalid('Unknown key: $fromKey');
    }

    if (mapping.type == KeyActionType.remap) {
      final targetKey = mapping.to?.trim();
      if (targetKey == null || targetKey.isEmpty) {
        return const ValidationResult.invalid(
          'Provide a target key for remap.',
        );
      }
      if (!KeyMappings.isKnownKey(targetKey)) {
        return ValidationResult.invalid('Unknown target key: $targetKey');
      }
    }

    return const ValidationResult.valid();
  }

  /// Validates a combo configuration before adding.
  ///
  /// Checks:
  /// - At least 2 keys are provided
  /// - Output is not empty
  /// - All keys are known
  ValidationResult validateCombo(List<String> keys, String output) {
    if (keys.length < 2) {
      return const ValidationResult.invalid(
        'Provide at least 2 keys for a combo.',
      );
    }

    if (output.trim().isEmpty) {
      return const ValidationResult.invalid('Provide an output for the combo.');
    }

    final unknownKeys = keys.where((k) => !KeyMappings.isKnownKey(k)).toList();
    if (unknownKeys.isNotEmpty) {
      return ValidationResult.invalid(
        'Unknown key(s) in combo: ${unknownKeys.join(", ")}',
      );
    }

    return const ValidationResult.valid();
  }
}
