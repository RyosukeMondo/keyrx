/// Validation FFI methods.
///
/// Provides script validation and key name suggestion methods for the bridge.
library;

import 'dart:convert';

import '../models/validation.dart';
import 'bindings.dart';

// Re-export validation types for public API compatibility.
export '../models/validation.dart';

/// Mixin providing validation FFI methods.
mixin BridgeValidationMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Validate a script and return structured results.
  ///
  /// [script] is the Rhai script content to validate.
  /// Returns a [ValidationResult] with errors, warnings, and optional coverage.
  ValidationResult validateScript(String script, [ValidationOptions? options]) {
    if (options != null) {
      return _validateWithOptions(script, options);
    }

    if (bindings == null) {
      return ValidationResult.error('validateScript not available');
    }

    try {
      final raw = bindings!.compat.validateScript(script);
      return _parseValidationResult(raw);
    } catch (e) {
      return ValidationResult.error('$e');
    }
  }

  ValidationResult _validateWithOptions(
    String script,
    ValidationOptions options,
  ) {
    if (bindings == null) {
      return ValidationResult.error('validateScriptWithOptions not available');
    }

    final optionsJson = json.encode(options.toJson());
    try {
      final raw = bindings!.compat.validateScriptWithOptions(
        script,
        optionsJson,
      );
      return _parseValidationResult(raw);
    } catch (e) {
      return ValidationResult.error('$e');
    }
  }

  /// Suggest similar key names for autocomplete.
  ///
  /// [partial] is the partial key name typed by the user.
  /// Returns a list of similar valid key names.
  List<String> suggestKeys(String partial) {
    if (bindings == null) return const [];

    try {
      final raw = bindings!.compat.suggestKeys(partial);
      return _parseSuggestions(raw);
    } catch (_) {
      return const [];
    }
  }

  /// Get all valid key names for autocomplete.
  ///
  /// Returns a list of all recognized key names.
  List<String> allKeyNames() {
    if (bindings == null) return const [];

    try {
      final raw = bindings!.compat.allKeyNames();
      return _parseSuggestions(raw);
    } catch (_) {
      return const [];
    }
  }

  static ValidationResult _parseValidationResult(String raw) {
    final trimmed = raw.trim();

    if (trimmed.toLowerCase().startsWith('error:')) {
      return ValidationResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ValidationResult.error('invalid validation payload');
      }
      return ValidationResult.fromJson(decoded);
    } catch (e) {
      return ValidationResult.error('$e');
    }
  }

  static List<String> _parseSuggestions(String raw) {
    final trimmed = raw.trim();

    if (trimmed.toLowerCase().startsWith('error:')) {
      return const [];
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return const [];
      }
      return decoded.map((e) => e.toString()).toList();
    } catch (_) {
      return const [];
    }
  }
}
