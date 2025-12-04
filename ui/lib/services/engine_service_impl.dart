import 'dart:async';

import '../ffi/bridge.dart';
import 'engine_service.dart';

/// Real EngineService that wraps the KeyrxBridge.
class EngineServiceImpl implements EngineService {
  EngineServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<EngineSnapshot> _stateController =
      StreamController<EngineSnapshot>.broadcast();
  StreamSubscription<BridgeState>? _stateSub;

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
      _stateSub = stateStream.listen(
        (state) {
          _stateController.add(
            EngineSnapshot(
              timestamp: state.timestamp,
              activeLayers: state.layers,
              activeModifiers: state.modifiers,
              heldKeys: state.heldKeys,
              pendingDecisions: state.pendingDecisions,
              lastEvent: state.lastEvent,
              latencyUs: state.latencyUs,
              timing: _mapTiming(state.timing),
            ),
          );
        },
        onError: (_) {},
      );
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

  void _emitSnapshot({String? lastEvent}) {
    if (_stateController.isClosed) return;
    _stateController.add(
      EngineSnapshot(
        timestamp: DateTime.now(),
        activeLayers: const ['base'], // Placeholder until core exposes layers.
        activeModifiers: const [],
        heldKeys: const [],
        pendingDecisions: const [],
        lastEvent: lastEvent,
        timing: const EngineTiming(),
      ),
    );
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
