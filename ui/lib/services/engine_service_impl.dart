import 'dart:async';
import 'dart:collection';

import '../ffi/bridge.dart';
import 'engine_service.dart';

/// Real EngineService that wraps the KeyrxBridge.
class EngineServiceImpl implements EngineService {
  EngineServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<EngineSnapshot> _stateController =
      StreamController<EngineSnapshot>.broadcast();
  StreamSubscription<BridgeStateUpdate>? _stateSub;
  final _EngineStateAccumulator _accumulator = _EngineStateAccumulator();
  String? _lastEvent;

  @override
  bool get isInitialized => _bridge.isInitialized;

  @override
  String get version => _bridge.version;

  @override
  Future<bool> initialize() async {
    if (_bridge.loadFailure != null) {
      return false;
    }
    final initialized = _bridge.initialize();

    final stateStream = _bridge.stateStream;
    if (initialized && stateStream != null && _stateSub == null) {
      _stateSub = stateStream.listen(_handleStateUpdate, onError: (_) {});
    }

    // Emit a heartbeat even if native streams are absent so the UI has data.
    _emitSnapshot();

    return initialized;
  }

  @override
  Future<bool> loadScript(String path) async {
    if (!isInitialized) return false;
    return _bridge.loadScript(path);
  }

  @override
  Future<ConsoleEvalResult> eval(String command) async {
    if (command.trim().isEmpty) {
      return const ConsoleEvalResult(
        success: false,
        output: 'No command provided.',
      );
    }

    if (_bridge.loadFailure != null) {
      return ConsoleEvalResult(
        success: false,
        output: 'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    if (!isInitialized) {
      return const ConsoleEvalResult(
        success: false,
        output: 'Engine not initialized.',
      );
    }

    final response = await _bridge.eval(command);
    if (response == null) {
      return const ConsoleEvalResult(
        success: false,
        output: 'Console evaluation is not available.',
      );
    }

    // If the native side returns a prefixed error, treat as failure.
    final isError = response.toLowerCase().startsWith('error:');
    return ConsoleEvalResult(success: !isError, output: response);
  }

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async {
    if (_bridge.loadFailure != null) {
      return KeyRegistryResult.fallback(
        'error: engine unavailable: ${_bridge.loadFailure}',
      );
    }
    return _bridge.listKeys();
  }

  @override
  Future<void> dispose() async {
    await _stateSub?.cancel();
    await _bridge.dispose();
    await _stateController.close();
  }

  @override
  Future<void> stop() async {
    if (_bridge.isInitialized) {
      _bridge.shutdown();
    }
  }

  void _handleStateUpdate(BridgeStateUpdate update) {
    final timing = _mapTiming(update.timing);

    if (update.fullSnapshot != null) {
      _accumulator.applySnapshot(update.fullSnapshot!);
      _lastEvent = update.event ?? _lastEvent;
      _emitSnapshot(
        lastEvent: update.event,
        latencyUs: update.latencyUs,
        timing: timing,
      );
      return;
    }

    final applied = _accumulator.applyDelta(update.delta);
    if (!applied) {
      _requestFullSync();
      return;
    }

    _lastEvent = update.event ?? _lastEvent;
    _emitSnapshot(
      lastEvent: update.event,
      latencyUs: update.latencyUs,
      timing: timing,
    );
  }

  void _emitSnapshot({
    String? lastEvent,
    int? latencyUs,
    EngineTiming? timing,
  }) {
    if (_stateController.isClosed) return;
    _stateController.add(
      _accumulator.toSnapshot(
        lastEvent: lastEvent ?? _lastEvent,
        latencyUs: latencyUs,
        timing: timing,
      ),
    );
  }

  void _requestFullSync() {
    _accumulator.clearPendingPlaceholders();
    _bridge.requestFullStateResubscribe();
  }

  EngineTiming? _mapTiming(BridgeTiming? timing) {
    if (timing == null) return null;
    return EngineTiming(
      tapTimeoutMs: timing.tapTimeoutMs,
      comboTimeoutMs: timing.comboTimeoutMs,
      holdDelayMs: timing.holdDelayMs,
      eagerTap: timing.eagerTap,
      permissiveHold: timing.permissiveHold,
      retroTap: timing.retroTap,
    );
  }
}

class _EngineStateAccumulator {
  int? version;
  int? _pendingCount;
  final LinkedHashSet<int> _layers = LinkedHashSet<int>();
  final Set<int> _virtualModifiers = <int>{};
  final Set<_StandardModifier> _standardModifiers = <_StandardModifier>{};
  final Set<String> _heldKeys = <String>{};
  final Set<int> _pendingIds = <int>{};
  List<LayoutState> _layouts = const [];
  List<int> _sharedModifiers = const [];

  void applySnapshot(BridgeSnapshot snapshot) {
    version = snapshot.version;
    _layers
      ..clear()
      ..addAll(
        snapshot.activeLayers.isNotEmpty
            ? snapshot.activeLayers
            : [snapshot.baseLayer],
      );
    if (_layers.isEmpty) {
      _layers.add(snapshot.baseLayer);
    }

    _standardModifiers
      ..clear()
      ..addAll(
        _StandardModifierParsing.fromBits(snapshot.standardModifierBits),
      );
    _virtualModifiers
      ..clear()
      ..addAll(snapshot.virtualModifiers);
    _heldKeys
      ..clear()
      ..addAll(snapshot.pressedKeys);
    _pendingIds.clear();
    _pendingCount = snapshot.pendingCount;
    _layouts = snapshot.layouts
        .map(
          (layout) => LayoutState(
            id: layout.id,
            name: layout.name,
            priority: layout.priority,
            enabled: layout.enabled,
            activeLayers: layout.activeLayers,
            tags: layout.tags,
            description: layout.description,
            modifiers: layout.modifiers,
          ),
        )
        .toList(growable: false);
    _sharedModifiers = List<int>.from(snapshot.sharedModifiers);
  }

  bool applyDelta(BridgeStateDelta delta) {
    if (version == null) return false;
    if (version != delta.fromVersion) return false;

    for (final change in delta.changes) {
      switch (change.type) {
        case BridgeDeltaChangeType.keyPressed:
          final key = change.payload?.toString();
          if (key != null) _heldKeys.add(key);
          break;
        case BridgeDeltaChangeType.keyReleased:
          final key = change.payload?.toString();
          if (key != null) _heldKeys.remove(key);
          break;
        case BridgeDeltaChangeType.allKeysReleased:
          _heldKeys.clear();
          break;
        case BridgeDeltaChangeType.layerActivated:
          final id = change.payload as int?;
          if (id != null) _layers.add(id);
          break;
        case BridgeDeltaChangeType.layerDeactivated:
          final id = change.payload as int?;
          if (id != null) _layers.remove(id);
          break;
        case BridgeDeltaChangeType.layerStackChanged:
          final layers = change.payload;
          if (layers is List<int>) {
            _layers
              ..clear()
              ..addAll(layers);
          }
          break;
        case BridgeDeltaChangeType.modifierChanged:
          final data = change.payload;
          final id = data is Map ? (data['id'] as num?)?.toInt() : null;
          final active = data is Map ? data['active'] == true : null;
          if (id != null) {
            if (active ?? false) {
              _virtualModifiers.add(id);
            } else {
              _virtualModifiers.remove(id);
            }
          }
          break;
        case BridgeDeltaChangeType.allModifiersCleared:
          _virtualModifiers.clear();
          _standardModifiers.clear();
          break;
        case BridgeDeltaChangeType.pendingAdded:
          final id = change.payload as int?;
          if (id != null) {
            _pendingIds.add(id);
            if (_pendingCount != null) {
              _pendingCount = _pendingCount! + 1;
            } else {
              _pendingCount = _pendingIds.length;
            }
          }
          break;
        case BridgeDeltaChangeType.pendingResolved:
          final id = change.payload as int?;
          if (id != null) {
            _pendingIds.remove(id);
            if (_pendingCount != null && _pendingCount! > 0) {
              _pendingCount = _pendingCount! - 1;
            }
          }
          break;
        case BridgeDeltaChangeType.allPendingCleared:
          _pendingIds.clear();
          _pendingCount = 0;
          break;
        case BridgeDeltaChangeType.versionChanged:
          final next = change.payload as int?;
          if (next != null) {
            version = next;
          }
          break;
      }
    }

    version = delta.toVersion;
    return true;
  }

  void clearPendingPlaceholders() {
    _pendingCount = null;
  }

  EngineSnapshot toSnapshot({
    String? lastEvent,
    int? latencyUs,
    EngineTiming? timing,
  }) {
    final layers = _layers.isNotEmpty
        ? _layers.map((l) => l.toString()).toList(growable: false)
        : const ['base'];
    final modifiers = <String>[
      ..._standardModifiers.map((m) => m.label),
      ..._virtualModifiers.map((id) => 'mod$id'),
    ];
    final effectivePendingCount = _pendingCount ?? _pendingIds.length;
    final pending = <String>[..._pendingIds.map((id) => 'pending:$id')];
    if (effectivePendingCount > _pendingIds.length) {
      final placeholders = effectivePendingCount - _pendingIds.length;
      for (var i = 0; i < placeholders; i++) {
        pending.add('pending-${_pendingIds.length + i + 1}');
      }
    }

    return EngineSnapshot(
      timestamp: DateTime.now(),
      activeLayers: layers,
      activeModifiers: modifiers,
      heldKeys: _heldKeys.toList(growable: false),
      pendingDecisions: pending,
      layouts: _layouts,
      sharedModifiers: _sharedModifiers,
      lastEvent: lastEvent,
      latencyUs: latencyUs,
      timing: timing,
      version: version,
    );
  }
}

enum _StandardModifier { shift, control, alt, meta }

extension _StandardModifierParsing on _StandardModifier {
  static Iterable<_StandardModifier> fromBits(int bits) sync* {
    if (bits & 1 != 0) yield _StandardModifier.shift;
    if (bits & 2 != 0) yield _StandardModifier.control;
    if (bits & 4 != 0) yield _StandardModifier.alt;
    if (bits & 8 != 0) yield _StandardModifier.meta;
  }

  String get label {
    switch (this) {
      case _StandardModifier.shift:
        return 'Shift';
      case _StandardModifier.control:
        return 'Control';
      case _StandardModifier.alt:
        return 'Alt';
      case _StandardModifier.meta:
        return 'Meta';
    }
  }
}
