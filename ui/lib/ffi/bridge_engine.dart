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

/// State stream payload containing a delta and optional full snapshot.
class BridgeStateUpdate {
  const BridgeStateUpdate({
    required this.delta,
    this.fullSnapshot,
    this.event,
    this.latencyUs,
    this.timing,
  });

  final BridgeStateDelta delta;
  final BridgeSnapshot? fullSnapshot;
  final String? event;
  final int? latencyUs;
  final BridgeTiming? timing;
}

/// Incremental delta describing changes to engine state.
class BridgeStateDelta {
  const BridgeStateDelta({
    required this.fromVersion,
    required this.toVersion,
    required this.changes,
  });

  factory BridgeStateDelta.fromJson(Map<String, dynamic> json) {
    final rawChanges = json['changes'] as List<dynamic>? ?? const [];
    return BridgeStateDelta(
      fromVersion: (json['from_version'] as num?)?.toInt() ?? 0,
      toVersion: (json['to_version'] as num?)?.toInt() ?? 0,
      changes: rawChanges
          .map(BridgeDeltaChange.fromJson)
          .whereType<BridgeDeltaChange>()
          .toList(),
    );
  }

  final int fromVersion;
  final int toVersion;
  final List<BridgeDeltaChange> changes;
}

/// Types of delta changes emitted by the core.
enum BridgeDeltaChangeType {
  keyPressed,
  keyReleased,
  allKeysReleased,
  layerActivated,
  layerDeactivated,
  layerStackChanged,
  modifierChanged,
  allModifiersCleared,
  pendingAdded,
  pendingResolved,
  allPendingCleared,
  versionChanged,
}

/// One change entry inside a delta.
class BridgeDeltaChange {
  const BridgeDeltaChange(this.type, {this.payload});

  final BridgeDeltaChangeType type;
  final Object? payload;

  static BridgeDeltaChange? fromJson(dynamic raw) {
    if (raw is String) {
      switch (raw) {
        case 'AllKeysReleased':
          return const BridgeDeltaChange(BridgeDeltaChangeType.allKeysReleased);
        case 'AllModifiersCleared':
          return const BridgeDeltaChange(
            BridgeDeltaChangeType.allModifiersCleared,
          );
        case 'AllPendingCleared':
          return const BridgeDeltaChange(
            BridgeDeltaChangeType.allPendingCleared,
          );
        default:
          return null;
      }
    }

    if (raw is Map<String, dynamic> && raw.length == 1) {
      final entry = raw.entries.first;
      final key = entry.key;
      final value = entry.value;

      switch (key) {
        case 'KeyPressed':
          return BridgeDeltaChange(
            BridgeDeltaChangeType.keyPressed,
            payload: value,
          );
        case 'KeyReleased':
          return BridgeDeltaChange(
            BridgeDeltaChangeType.keyReleased,
            payload: value,
          );
        case 'LayerActivated':
          return BridgeDeltaChange(
            BridgeDeltaChangeType.layerActivated,
            payload: (value as num?)?.toInt(),
          );
        case 'LayerDeactivated':
          return BridgeDeltaChange(
            BridgeDeltaChangeType.layerDeactivated,
            payload: (value as num?)?.toInt(),
          );
        case 'LayerStackChanged':
          final layers = (value is Map<String, dynamic>)
              ? value['layers']
              : (value is List<dynamic> ? value : null);
          return BridgeDeltaChange(
            BridgeDeltaChangeType.layerStackChanged,
            payload: layers is List<dynamic>
                ? layers
                      .map((e) => (e as num?)?.toInt())
                      .whereType<int>()
                      .toList()
                : const <int>[],
          );
        case 'ModifierChanged':
          if (value is Map<String, dynamic>) {
            return BridgeDeltaChange(
              BridgeDeltaChangeType.modifierChanged,
              payload: {
                'id': (value['id'] as num?)?.toInt(),
                'active': value['active'] == true,
              },
            );
          }
          break;
        case 'PendingAdded':
          if (value is Map<String, dynamic>) {
            return BridgeDeltaChange(
              BridgeDeltaChangeType.pendingAdded,
              payload: (value['id'] as num?)?.toInt(),
            );
          }
          break;
        case 'PendingResolved':
          if (value is Map<String, dynamic>) {
            return BridgeDeltaChange(
              BridgeDeltaChangeType.pendingResolved,
              payload: (value['id'] as num?)?.toInt(),
            );
          }
          break;
        case 'VersionChanged':
          if (value is Map<String, dynamic>) {
            return BridgeDeltaChange(
              BridgeDeltaChangeType.versionChanged,
              payload: (value['version'] as num?)?.toInt(),
            );
          }
          break;
        default:
          break;
      }
    }

    return null;
  }
}

/// Full snapshot payload attached to a delta when resync is needed.
class BridgeSnapshot {
  const BridgeSnapshot({
    required this.version,
    required this.pressedKeys,
    required this.activeLayers,
    required this.baseLayer,
    required this.standardModifierBits,
    required this.virtualModifiers,
    required this.pendingCount,
    this.layouts = const [],
    this.sharedModifiers = const [],
  });

  factory BridgeSnapshot.fromJson(Map<String, dynamic> json) {
    final pressed =
        (json['pressed_keys'] as List<dynamic>?)
            ?.map((e) {
              if (e is Map<String, dynamic>) {
                final key = e['key']?.toString() ?? '';
                return key;
              }
              return e.toString();
            })
            .where((k) => k.isNotEmpty)
            .toList() ??
        const <String>[];

    var standardBits = 0;
    final standardRaw = json['standard_modifiers'];
    if (standardRaw is Map<String, dynamic>) {
      if (standardRaw['bits'] is num) {
        standardBits = (standardRaw['bits'] as num).toInt();
      } else if (standardRaw.values.isNotEmpty &&
          standardRaw.values.first is num) {
        standardBits = (standardRaw.values.first as num).toInt();
      }
    }

    List<BridgeLayoutSnapshot> layouts = const [];
    List<int> sharedModifiers = const [];
    final compositor = json['layout_compositor'];
    if (compositor is Map<String, dynamic>) {
      final rawLayouts = compositor['layouts'] as List<dynamic>? ?? const [];
      layouts = rawLayouts
          .map(
            (entry) => entry is Map<String, dynamic>
                ? BridgeLayoutSnapshot.fromJson(entry)
                : null,
          )
          .whereType<BridgeLayoutSnapshot>()
          .toList();
      sharedModifiers =
          (compositor['shared_modifiers'] as List<dynamic>? ?? const [])
              .map((e) => (e as num?)?.toInt())
              .whereType<int>()
              .toList();
    }

    return BridgeSnapshot(
      version: (json['version'] as num?)?.toInt() ?? 0,
      pressedKeys: pressed,
      activeLayers:
          (json['active_layers'] as List<dynamic>?)
              ?.map((e) => (e as num?)?.toInt() ?? 0)
              .toList() ??
          const <int>[],
      baseLayer: (json['base_layer'] as num?)?.toInt() ?? 0,
      standardModifierBits: standardBits,
      virtualModifiers:
          (json['virtual_modifiers'] as List<dynamic>?)
              ?.map((e) => (e as num?)?.toInt() ?? 0)
              .toList() ??
          const <int>[],
      pendingCount: (json['pending_count'] as num?)?.toInt() ?? 0,
      layouts: layouts,
      sharedModifiers: sharedModifiers,
    );
  }

  final int version;
  final List<String> pressedKeys;
  final List<int> activeLayers;
  final int baseLayer;
  final int standardModifierBits;
  final List<int> virtualModifiers;
  final int pendingCount;
  final List<BridgeLayoutSnapshot> layouts;
  final List<int> sharedModifiers;
}

/// Snapshot of a layout as reported by the native compositor.
class BridgeLayoutSnapshot {
  const BridgeLayoutSnapshot({
    required this.id,
    required this.name,
    required this.priority,
    required this.enabled,
    this.activeLayers = const [],
    this.description,
    this.tags = const [],
    this.modifiers = const [],
  });

  factory BridgeLayoutSnapshot.fromJson(Map<String, dynamic> json) {
    return BridgeLayoutSnapshot(
      id: json['id']?.toString() ?? '',
      name: json['name']?.toString() ?? '',
      priority: (json['priority'] as num?)?.toInt() ?? 0,
      enabled: json['enabled'] == true,
      activeLayers: (json['active_layers'] as List<dynamic>? ?? const [])
          .map((e) => (e as num?)?.toInt() ?? 0)
          .toList(),
      description: json['description']?.toString(),
      tags: (json['tags'] as List<dynamic>? ?? const [])
          .map((e) => e.toString())
          .where((e) => e.isNotEmpty)
          .toList(),
      modifiers: (json['modifiers'] as List<dynamic>? ?? const [])
          .map((e) => (e as num?)?.toInt())
          .whereType<int>()
          .toList(),
    );
  }

  final String id;
  final String name;
  final int priority;
  final bool enabled;
  final List<int> activeLayers;
  final String? description;
  final List<String> tags;
  final List<int> modifiers;
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

  factory KeyRegistryResult.fallback(String error) =>
      KeyRegistryResult(entries: const [], error: error, usedFallback: true);

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

      final entries = decoded
          .map((entry) {
            if (entry is! Map) {
              return const KeyRegistryEntry(name: '');
            }
            final name = entry['name']?.toString() ?? '';
            final aliases =
                (entry['aliases'] as List<dynamic>?)
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
          })
          .where((entry) => entry.name.isNotEmpty)
          .toList();

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
  const ScriptValidationError({this.line, this.column, required this.message});

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
        trimmed.substring('error:'.length).trim(),
      );
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
  StreamController<BridgeStateUpdate>? get stateController;

  /// Subscribe to engine state snapshots if exposed by the native layer.
  Stream<BridgeStateUpdate>? get stateStream => stateController?.stream;

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
    final errorPtr = calloc<Pointer<Utf8>>();
    try {
      responsePtr = evalFn(cmdPtr.cast<Char>(), errorPtr);
      if (responsePtr == nullptr) {
        return 'error: eval returned null';
      }

      final raw = responsePtr.cast<Utf8>().toDartString();
      return _normalizeEval(raw);
    } catch (e) {
      return 'error: $e';
    } finally {
      calloc.free(errorPtr);
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
    final errorPtr = calloc<Pointer<Utf8>>();
    try {
      ptr = listFn(errorPtr);
      if (ptr == nullptr) {
        return KeyRegistryResult.fallback('error: listKeys returned null');
      }

      final jsonStr = ptr.cast<Utf8>().toDartString();
      return KeyRegistryResult.parse(jsonStr);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    } finally {
      calloc.free(errorPtr);
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
    final errorPtr = calloc<Pointer<Utf8>>();
    try {
      ptr = checkFn(pathPtr.cast<Char>(), errorPtr);
      if (ptr == nullptr) {
        return ScriptValidationResult.error('checkScript returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return ScriptValidationResult.parse(raw);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    } finally {
      calloc.free(errorPtr);
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
  static BridgeStateUpdate? parseStatePayload(List<int> bytes) {
    try {
      final decoded = json.decode(utf8.decode(bytes));
      if (decoded is! Map<String, dynamic>) return null;

      // Handle UnifiedEvent envelope
      final payload =
          decoded.containsKey('eventType') && decoded.containsKey('payload')
          ? decoded['payload'] as Map<String, dynamic>
          : decoded;

      if (payload['delta'] is Map<String, dynamic>) {
        final delta = BridgeStateDelta.fromJson(
          payload['delta'] as Map<String, dynamic>,
        );
        final snapshotRaw = payload['full_snapshot'];
        final snapshot = snapshotRaw is Map<String, dynamic>
            ? BridgeSnapshot.fromJson(snapshotRaw)
            : null;

        return BridgeStateUpdate(
          delta: delta,
          fullSnapshot: snapshot,
          event: payload['event']?.toString(),
          latencyUs: (payload['latency_us'] as num?)?.toInt(),
          timing: _parseTiming(payload['timing']),
        );
      }

      // Legacy fallback: interpret flat payloads with layers/modifiers.
      final layers = _asStringList(payload['layers']);
      final modifiers = _asStringList(payload['modifiers']);
      final held = _asStringList(payload['held']);
      final pending = _asStringList(payload['pending']);
      final hasLegacyData =
          layers.isNotEmpty ||
          modifiers.isNotEmpty ||
          held.isNotEmpty ||
          pending.isNotEmpty;

      if (!hasLegacyData) return null;

      final snapshot = BridgeSnapshot(
        version: 0,
        pressedKeys: held,
        activeLayers: layers
            .map((e) => int.tryParse(e) ?? 0)
            .toList(growable: false),
        baseLayer: 0,
        standardModifierBits: 0,
        virtualModifiers: const [],
        pendingCount: pending.length,
      );

      return BridgeStateUpdate(
        delta: const BridgeStateDelta(
          fromVersion: 0,
          toVersion: 0,
          changes: [],
        ),
        fullSnapshot: snapshot,
        event: payload['event']?.toString(),
        latencyUs: (payload['latency_us'] as num?)?.toInt(),
        timing: _parseTiming(payload['timing']),
      );
    } catch (_) {
      return null;
    }
  }

  static List<String> _asStringList(dynamic value) {
    if (value is List<dynamic>) {
      return value.map((e) => e.toString()).toList();
    }
    return const [];
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
