/// Testing and diagnostic FFI methods.
///
/// Provides test discovery, test execution, simulation, benchmarking,
/// and system diagnostics for the KeyRx bridge.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

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
    final discoverFn = bindings?.discoverTests;
    if (discoverFn == null) {
      return TestDiscoveryResult.error('discoverTests not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = discoverFn(pathPtr.cast<Char>());
      if (ptr == nullptr) {
        return TestDiscoveryResult.error('discoverTests returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return TestDiscoveryResult.parse(raw);
    } catch (e) {
      return TestDiscoveryResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run tests in a Rhai script with optional filter.
  ///
  /// [path] - Path to the script file.
  /// [filter] - Optional pattern to filter test names (null for all tests).
  TestRunResult runTests(String path, {String? filter}) {
    final runFn = bindings?.runTests;
    if (runFn == null) {
      return TestRunResult.error('runTests not available');
    }

    final pathPtr = path.toNativeUtf8();
    final filterPtr = filter?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = runFn(pathPtr.cast<Char>(), filterPtr.cast<Char>());
      if (ptr == nullptr) {
        return TestRunResult.error('runTests returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return TestRunResult.parse(raw);
    } catch (e) {
      return TestRunResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (filterPtr != nullptr) {
        calloc.free(filterPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
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
    final simFn = bindings?.simulate;
    if (simFn == null) {
      return SimulationResult.error('simulate not available');
    }

    final keysJson = json.encode(keys.map((k) => k.toJson()).toList());
    final keysPtr = keysJson.toNativeUtf8();
    final scriptPtr = scriptPath?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = simFn(keysPtr.cast<Char>(), scriptPtr.cast<Char>(), comboMode);
      if (ptr == nullptr) {
        return SimulationResult.error('simulate returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SimulationResult.parse(raw);
    } catch (e) {
      return SimulationResult.error('$e');
    } finally {
      calloc.free(keysPtr);
      if (scriptPtr != nullptr) {
        calloc.free(scriptPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run benchmark with specified iterations.
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) {
    final benchFn = bindings?.runBenchmark;
    if (benchFn == null) {
      return BenchmarkResult.error('runBenchmark not available');
    }

    final scriptPtr = scriptPath?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = benchFn(iterations, scriptPtr.cast<Char>());
      if (ptr == nullptr) {
        return BenchmarkResult.error('runBenchmark returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return BenchmarkResult.parse(raw);
    } catch (e) {
      return BenchmarkResult.error('$e');
    } finally {
      if (scriptPtr != nullptr) {
        calloc.free(scriptPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run system diagnostics.
  DoctorResult runDoctor() {
    final doctorFn = bindings?.runDoctor;
    if (doctorFn == null) {
      return DoctorResult.error('runDoctor not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = doctorFn();
      if (ptr == nullptr) {
        return DoctorResult.error('runDoctor returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DoctorResult.parse(raw);
    } catch (e) {
      return DoctorResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
