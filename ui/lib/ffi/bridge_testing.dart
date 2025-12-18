/// Testing and diagnostic FFI methods.
///
/// Provides test discovery, test execution, simulation, benchmarking,
/// and system diagnostics for the KeyRx bridge.
library;

import 'dart:convert';

import 'bindings.dart';
import 'bridge_testing_types.dart';

export 'bridge_testing_types.dart';

/// Mixin providing testing and diagnostic FFI methods.
mixin BridgeTestingMixin {
  KeyrxBindings? get bindings;

  /// Discover test functions in a Rhai script.
  ///
  /// Returns list of discovered test functions.
  TestDiscoveryResult discoverTests(String path) {
    if (bindings == null) {
      return TestDiscoveryResult.error('discoverTests not available');
    }

    try {
      final raw = bindings!.compat.discoverTests(path);
      return TestDiscoveryResult.parse(raw);
    } catch (e) {
      return TestDiscoveryResult.error('$e');
    }
  }

  /// Run tests in a Rhai script with optional filter.
  ///
  /// [path] - Path to the script file.
  /// [filter] - Optional pattern to filter test names (null for all tests).
  TestRunResult runTests(String path, {String? filter}) {
    if (bindings == null) {
      return TestRunResult.error('runTests not available');
    }

    try {
      final raw = bindings!.compat.runTests(path, filter ?? '');
      return TestRunResult.parse(raw);
    } catch (e) {
      return TestRunResult.error('$e');
    }
  }

  /// Simulate key sequences through the engine.
  ///
  /// [keys] - List of key inputs to simulate.
  /// [scriptPath] - Optional path to Rhai script (null uses active script).
  /// [comboMode] - If true, keys are pressed simultaneously; otherwise
  ///   sequentially.
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) {
    if (bindings == null) {
      return SimulationResult.error('simulate not available');
    }

    final keysJson = json.encode(keys.map((k) => k.toJson()).toList());
    try {
      final raw = bindings!.compat.simulate(
        keysJson,
        scriptPath ?? '',
        comboMode,
      );
      return SimulationResult.parse(raw);
    } catch (e) {
      return SimulationResult.error('$e');
    }
  }

  /// Run benchmark with specified iterations.
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) {
    if (bindings == null) {
      return BenchmarkResult.error('runBenchmark not available');
    }

    try {
      final raw = bindings!.compat.runBenchmark(iterations, scriptPath ?? '');
      return BenchmarkResult.parse(raw);
    } catch (e) {
      return BenchmarkResult.error('$e');
    }
  }

  /// Run system diagnostics.
  DoctorResult runDoctor() {
    if (bindings == null) {
      return DoctorResult.error('runDoctor not available');
    }

    try {
      final raw = bindings!.compat.runDoctor();
      return DoctorResult.parse(raw);
    } catch (e) {
      return DoctorResult.error('$e');
    }
  }
}
