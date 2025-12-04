/// Engine control FFI methods.
///
/// Provides script loading, evaluation, key listing, and bypass control
/// for the KeyRx bridge.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings.dart';

/// State snapshot payload from the bridge.
class BridgeState {
  const BridgeState({
    required this.layers,
    required this.modifiers,
    required this.heldKeys,
    required this.pendingDecisions,
    required this.timestamp,
    this.lastEvent,
    this.latencyUs,
    this.timing,
  });

  final List<String> layers;
  final List<String> modifiers;
  final List<String> heldKeys;
  final List<String> pendingDecisions;
  final DateTime timestamp;
  final String? lastEvent;
  final int? latencyUs;
  final BridgeTiming? timing;
}

/// Timing configuration snapshot from the engine.
class BridgeTiming {
  const BridgeTiming({
    this.tapTimeoutMs,
    this.comboTimeoutMs,
    this.holdDelayMs,
    this.eagerTap,
    this.permissiveHold,
    this.retroTap,
  });

  final int? tapTimeoutMs;
  final int? comboTimeoutMs;
  final int? holdDelayMs;
  final bool? eagerTap;
  final bool? permissiveHold;
  final bool? retroTap;
}

/// Canonical key definition entry returned by the FFI registry.
class KeyRegistryEntry {
  const KeyRegistryEntry({
    required this.name,
    this.aliases = const [],
    this.evdev,
    this.vk,
  });

  final String name;
  final List<String> aliases;
  final int? evdev;
  final int? vk;
}

/// Result of requesting the key registry (with fallback on errors).
class KeyRegistryResult {
  const KeyRegistryResult({
    required this.entries,
    this.error,
    this.usedFallback = false,
  });

  factory KeyRegistryResult.fallback(String error) => KeyRegistryResult(
        entries: const [],
        error: error,
        usedFallback: true,
      );

  factory KeyRegistryResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return KeyRegistryResult.fallback(trimmed);
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return KeyRegistryResult.fallback('error: invalid registry payload');
      }

      final entries = decoded.map((entry) {
        if (entry is! Map) {
          return const KeyRegistryEntry(name: '');
        }
        final name = entry['name']?.toString() ?? '';
        final aliases = (entry['aliases'] as List<dynamic>?)
                ?.map((e) => e.toString())
                .toList() ??
            const <String>[];
        final evdev = (entry['evdev'] as num?)?.toInt();
        final vk = (entry['vk'] as num?)?.toInt();
        return KeyRegistryEntry(
          name: name,
          aliases: aliases,
          evdev: evdev,
          vk: vk,
        );
      }).where((entry) => entry.name.isNotEmpty).toList();

      if (entries.isEmpty) {
        return KeyRegistryResult.fallback('error: empty registry payload');
      }

      return KeyRegistryResult(entries: entries);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    }
  }

  final List<KeyRegistryEntry> entries;
  final String? error;
  final bool usedFallback;

  List<String> get names => entries.map((e) => e.name).toList();
}

/// Script validation error detail.
class ScriptValidationError {
  const ScriptValidationError({
    this.line,
    this.column,
    required this.message,
  });

  final int? line;
  final int? column;
  final String message;
}

/// Script validation result.
class ScriptValidationResult {
  const ScriptValidationResult({
    required this.valid,
    required this.errors,
    this.errorMessage,
  });

  factory ScriptValidationResult.error(String message) =>
      ScriptValidationResult(
        valid: false,
        errors: const [],
        errorMessage: message,
      );

  factory ScriptValidationResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ScriptValidationResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ScriptValidationResult.error('invalid validation payload');
      }

      final valid = decoded['valid'] as bool? ?? false;
      final errorsList = decoded['errors'] as List<dynamic>? ?? [];
      final errors = errorsList.map((e) {
        if (e is! Map<String, dynamic>) {
          return const ScriptValidationError(message: 'unknown error');
        }
        return ScriptValidationError(
          line: (e['line'] as num?)?.toInt(),
          column: (e['column'] as num?)?.toInt(),
          message: e['message']?.toString() ?? 'unknown error',
        );
      }).toList();

      return ScriptValidationResult(valid: valid, errors: errors);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    }
  }

  final bool valid;
  final List<ScriptValidationError> errors;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Mixin providing engine control FFI methods.
mixin BridgeEngineMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;
  StreamController<BridgeState>? get stateController;

  /// Subscribe to engine state snapshots if exposed by the native layer.
  Stream<BridgeState>? get stateStream => stateController?.stream;

  /// Load a Rhai script file.
  bool loadScript(String path) {
    if (bindings == null) return false;
    final pathPtr = path.toNativeUtf8();
    try {
      final result = bindings!.loadScript(pathPtr.cast<Char>());
      return result == 0;
    } finally {
      calloc.free(pathPtr);
    }
  }

  /// Evaluate a console command if the native binding is available.
  ///
  /// Returns stdout/stderr text. Caller interprets success.
  Future<String?> eval(String command) async {
    final evalFn = bindings?.eval;
    if (evalFn == null) return 'error: eval not available';

    final cmdPtr = command.toNativeUtf8();
    Pointer<Char>? responsePtr;
    try {
      responsePtr = evalFn(cmdPtr.cast<Char>());
      if (responsePtr == nullptr) {
        return 'error: eval returned null';
      }

      final raw = responsePtr.cast<Utf8>().toDartString();
      return _normalizeEval(raw);
    } catch (e) {
      return 'error: $e';
    } finally {
      calloc.free(cmdPtr);
      if (responsePtr != null) {
        try {
          bindings?.freeString(responsePtr);
        } catch (_) {}
      }
    }
  }

  /// List canonical key names from the core definition table.
  KeyRegistryResult listKeys() {
    final listFn = bindings?.listKeys;
    if (listFn == null) {
      return KeyRegistryResult.fallback('error: listKeys not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = listFn();
      if (ptr == nullptr) {
        return KeyRegistryResult.fallback('error: listKeys returned null');
      }

      final jsonStr = ptr.cast<Utf8>().toDartString();
      return KeyRegistryResult.parse(jsonStr);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    } finally {
      if (ptr != null) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Check if emergency bypass mode is currently active.
  ///
  /// When bypass mode is active, all key remapping is disabled.
  bool isBypassActive() {
    final fn = bindings?.isBypassActive;
    if (fn == null) return false;
    return fn();
  }

  /// Set the emergency bypass mode state.
  ///
  /// [active] - If true, enable bypass mode (disable remapping).
  ///            If false, disable bypass mode (re-enable remapping).
  void setBypass(bool active) {
    final fn = bindings?.setBypass;
    if (fn == null) return;
    fn(active);
  }

  /// Validate a Rhai script without executing it.
  ///
  /// Returns validation result with any syntax errors.
  ScriptValidationResult checkScript(String path) {
    final checkFn = bindings?.checkScript;
    if (checkFn == null) {
      return ScriptValidationResult.error('checkScript not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = checkFn(pathPtr.cast<Char>());
      if (ptr == nullptr) {
        return ScriptValidationResult.error('checkScript returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return ScriptValidationResult.parse(raw);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  static String _normalizeEval(String raw) {
    final trimmed = raw.trim();
    final lower = trimmed.toLowerCase();
    if (lower.startsWith('ok:') || lower.startsWith('error:')) {
      return trimmed;
    }
    return 'ok:$trimmed';
  }
}

/// Extension for state stream setup and parsing.
extension BridgeEngineStateSetup on BridgeEngineMixin {
  /// Parse state payload from JSON bytes.
  static BridgeState? parseStatePayload(List<int> bytes) {
    try {
      final payload = json.decode(utf8.decode(bytes));
      if (payload is! Map<String, dynamic>) return null;

      final layers = (payload['layers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final modifiers = (payload['modifiers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final held = (payload['held'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final pending = (payload['pending'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final lastEvent = payload['event'] as String?;
      final latencyUs = (payload['latency_us'] as num?)?.toInt();
      final timing = _parseTiming(payload['timing']);

      return BridgeState(
        layers: layers,
        modifiers: modifiers,
        heldKeys: held,
        pendingDecisions: pending,
        lastEvent: lastEvent,
        latencyUs: latencyUs,
        timing: timing,
        timestamp: DateTime.now(),
      );
    } catch (_) {
      return null;
    }
  }

  static BridgeTiming? _parseTiming(dynamic raw) {
    if (raw is! Map<String, dynamic>) return null;
    try {
      return BridgeTiming(
        tapTimeoutMs: (raw['tap_timeout_ms'] as num?)?.toInt(),
        comboTimeoutMs: (raw['combo_timeout_ms'] as num?)?.toInt(),
        holdDelayMs: (raw['hold_delay_ms'] as num?)?.toInt(),
        eagerTap: raw['eager_tap'] as bool?,
        permissiveHold: raw['permissive_hold'] as bool?,
        retroTap: raw['retro_tap'] as bool?,
      );
    } catch (_) {
      return null;
    }
  }
}
