/// Validation FFI methods.
///
/// Provides script validation and key name suggestion methods for the bridge.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

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

    final validateFn = bindings?.validateScript;
    if (validateFn == null) {
      return ValidationResult.error('validateScript not available');
    }

    final scriptPtr = script.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = validateFn(scriptPtr);
      if (ptr == nullptr) {
        return ValidationResult.error('validateScript returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return _parseValidationResult(raw);
    } catch (e) {
      return ValidationResult.error('$e');
    } finally {
      calloc.free(scriptPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  ValidationResult _validateWithOptions(
      String script, ValidationOptions options) {
    final validateFn = bindings?.validateScriptWithOptions;
    if (validateFn == null) {
      // Fall back to basic validation if options variant not available
      return validateScript(script);
    }

    final scriptPtr = script.toNativeUtf8();
    final optionsJson = json.encode(options.toJson());
    final optionsPtr = optionsJson.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = validateFn(scriptPtr, optionsPtr);
      if (ptr == nullptr) {
        return ValidationResult.error(
            'validateScriptWithOptions returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return _parseValidationResult(raw);
    } catch (e) {
      return ValidationResult.error('$e');
    } finally {
      calloc.free(scriptPtr);
      calloc.free(optionsPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Suggest similar key names for autocomplete.
  ///
  /// [partial] is the partial key name typed by the user.
  /// Returns a list of similar valid key names.
  List<String> suggestKeys(String partial) {
    final suggestFn = bindings?.suggestKeys;
    if (suggestFn == null) {
      return const [];
    }

    final partialPtr = partial.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = suggestFn(partialPtr);
      if (ptr == nullptr) {
        return const [];
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return _parseSuggestions(raw);
    } catch (_) {
      return const [];
    } finally {
      calloc.free(partialPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Get all valid key names for autocomplete.
  ///
  /// Returns a list of all recognized key names.
  List<String> allKeyNames() {
    final allNamesFn = bindings?.allKeyNames;
    if (allNamesFn == null) {
      return const [];
    }

    Pointer<Char>? ptr;
    try {
      ptr = allNamesFn();
      if (ptr == nullptr) {
        return const [];
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return _parseSuggestions(raw);
    } catch (_) {
      return const [];
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  static ValidationResult _parseValidationResult(String raw) {
    final trimmed = raw.trim();

    if (trimmed.toLowerCase().startsWith('error:')) {
      return ValidationResult.error(
          trimmed.substring('error:'.length).trim());
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
