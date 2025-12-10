/// Key sequence simulation service.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import '../ffi/bridge.dart';

/// Input key for simulation.
class SimulationKeyInput {
  const SimulationKeyInput({required this.code, this.holdMs});

  final String code;
  final int? holdMs;
}

/// Key mapping from simulation showing input to output transformation.
class SimulationKeyMapping {
  const SimulationKeyMapping({
    required this.input,
    required this.output,
    required this.decision,
  });

  final String input;
  final String output;
  final String decision;
}

/// Result of a simulation operation.
class SimulationServiceResult {
  const SimulationServiceResult({
    required this.mappings,
    required this.activeLayers,
    required this.pending,
    this.errorMessage,
  });

  factory SimulationServiceResult.error(String message) =>
      SimulationServiceResult(
        mappings: const [],
        activeLayers: const [],
        pending: const [],
        errorMessage: message,
      );

  final List<SimulationKeyMapping> mappings;
  final List<String> activeLayers;
  final List<String> pending;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Abstraction for key sequence simulation operations.
abstract class SimulationService {
  /// Simulate key sequences through the engine.
  ///
  /// [keys] - List of key inputs to simulate.
  /// [scriptPath] - Optional path to Rhai script (null uses active script).
  /// [comboMode] - If true, keys are pressed simultaneously; otherwise sequentially.
  Future<SimulationServiceResult> simulate(
    List<SimulationKeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  });

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real SimulationService that wraps the KeyrxBridge.
class SimulationServiceImpl implements SimulationService {
  SimulationServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<SimulationServiceResult> simulate(
    List<SimulationKeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) async {
    if (_bridge.loadFailure != null) {
      return SimulationServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final bridgeKeys = keys.map((k) {
      return KeyInput(code: k.code, holdMs: k.holdMs);
    }).toList();

    final result = _bridge.simulate(
      bridgeKeys,
      scriptPath: scriptPath,
      comboMode: comboMode,
    );

    if (result.hasError) {
      return SimulationServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    final mappings = result.mappings.map((m) {
      return SimulationKeyMapping(
        input: m.input,
        output: m.output,
        decision: m.decision,
      );
    }).toList();

    return SimulationServiceResult(
      mappings: mappings,
      activeLayers: result.activeLayers,
      pending: result.pending,
    );
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose
  }
}
