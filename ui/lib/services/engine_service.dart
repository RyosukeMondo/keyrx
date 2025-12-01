/// Engine-level operations exposed to the UI layer.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import '../ffi/bridge.dart';

/// Snapshot of current engine state for UI/debugger consumption.
class EngineSnapshot {
  const EngineSnapshot({
    required this.timestamp,
    this.activeLayers = const [],
    this.activeModifiers = const [],
    this.heldKeys = const [],
    this.pendingDecisions = const [],
    this.lastEvent,
    this.latencyUs,
    this.timing,
  });

  final DateTime timestamp;
  final List<String> activeLayers;
  final List<String> activeModifiers;
  final List<String> heldKeys;
  final List<String> pendingDecisions;
  final String? lastEvent;
  final int? latencyUs;
  final EngineTiming? timing;
}

/// Timing configuration for decision-making.
class EngineTiming {
  const EngineTiming({
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

/// Result of evaluating a console command via the engine.
class ConsoleEvalResult {
  const ConsoleEvalResult({
    required this.success,
    required this.output,
  });

  final bool success;
  final String output;

  bool get isError => !success;
}

/// Abstraction for interacting with the engine core.
abstract class EngineService {
  /// Whether the engine has been initialized.
  bool get isInitialized;

  /// Core library version string.
  String get version;

  /// Initialize the engine and underlying bridge.
  Future<bool> initialize();

  /// Load a script path into the engine.
  Future<bool> loadScript(String path);

  /// Evaluate a command in the engine context (REPL).
  Future<ConsoleEvalResult> eval(String command);

  /// Stream of engine snapshots for real-time UI consumption.
  Stream<EngineSnapshot> get stateStream;

  /// Fetch canonical key registry definitions (with graceful fallback).
  Future<KeyRegistryResult> fetchKeyRegistry();

  /// Dispose any held resources.
  Future<void> dispose();
}
