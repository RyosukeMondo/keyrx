/// Engine control FFI methods.
///
/// Provides script loading, evaluation, key listing, and bypass control
/// for the KeyRx bridge.
library;

import 'dart:async';
import 'dart:convert';

import '../models/bridge_models.dart';
import 'bindings.dart';
import 'debug_tools.dart';

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
    return FfiDebugTools.traceSync<bool>(
      operation: 'engine.load_script',
      context: {'path': path},
      call: () {
        final result = bindings!.compat.loadScript(path);
        return result == 0;
      },
    );
  }

  /// Evaluate a console command if the native binding is available.
  ///
  /// Returns stdout/stderr text. Caller interprets success.
  Future<String?> eval(String command) async {
    if (bindings == null) return 'error: eval not available';

    return FfiDebugTools.traceAsync<String?>(
      operation: 'engine.eval',
      context: {'command_preview': command.length > 40 ? '${command.substring(0, 40)}…' : command},
      includeTimeline: false,
      call: () async {
        try {
          final raw = bindings!.compat.eval(command);
          return _normalizeEval(raw);
        } catch (e) {
          return 'error: $e';
        }
      },
    );
  }

  /// List canonical key names from the core definition table.
  KeyRegistryResult listKeys() {
    if (bindings == null) {
      return KeyRegistryResult.fallback('error: listKeys not available');
    }

    return FfiDebugTools.traceSync<KeyRegistryResult>(
      operation: 'engine.list_keys',
      includeTimeline: false,
      call: () {
        try {
          final jsonStr = bindings!.compat.listKeys();
          return KeyRegistryResult.parse(jsonStr);
        } catch (e) {
          return KeyRegistryResult.fallback('error: $e');
        }
      },
    );
  }

  /// Check if emergency bypass mode is currently active.
  ///
  /// When bypass mode is active, all key remapping is disabled.
  bool isBypassActive() {
    if (bindings == null) return false;
    return FfiDebugTools.traceSync<bool>(
      operation: 'engine.bypass.is_active',
      includeTimeline: false,
      call: () => bindings!.compat.isBypassActive(),
    );
  }

  /// Set the emergency bypass mode state.
  ///
  /// [active] - If true, enable bypass mode (disable remapping).
  ///            If false, disable bypass mode (re-enable remapping).
  void setBypass(bool active) {
    if (bindings == null) return;
    FfiDebugTools.traceSync<void>(
      operation: 'engine.bypass.set',
      includeTimeline: false,
      context: {'active': active},
      call: () => bindings!.compat.setBypass(active),
    );
  }

  /// Validate a Rhai script without executing it.
  ///
  /// Returns validation result with any syntax errors.
  ScriptValidationResult checkScript(String path) {
    if (bindings == null) {
      return ScriptValidationResult.error('checkScript not available');
    }

    return FfiDebugTools.traceSync<ScriptValidationResult>(
      operation: 'engine.check_script',
      context: {'path': path},
      call: () {
        try {
          final raw = bindings!.compat.checkScript(path);
          return ScriptValidationResult.parse(raw);
        } catch (e) {
          return ScriptValidationResult.error('$e');
        }
      },
    );
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
      final payload = decoded.containsKey('eventType') && decoded.containsKey('payload')
          ? decoded['payload'] as Map<String, dynamic>
          : decoded;

      if (payload['delta'] is Map<String, dynamic>) {
        final delta = BridgeStateDelta.fromJson(payload['delta'] as Map<String, dynamic>);
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
          layers.isNotEmpty || modifiers.isNotEmpty || held.isNotEmpty || pending.isNotEmpty;

      if (!hasLegacyData) return null;

      final snapshot = BridgeSnapshot(
        version: 0,
        pressedKeys: held,
        activeLayers: layers.map((e) => int.tryParse(e) ?? 0).toList(growable: false),
        baseLayer: 0,
        standardModifierBits: 0,
        virtualModifiers: const [],
        pendingCount: pending.length,
      );

      return BridgeStateUpdate(
        delta: const BridgeStateDelta(fromVersion: 0, toVersion: 0, changes: []),
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
