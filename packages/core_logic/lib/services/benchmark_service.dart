/// Service for benchmark operations wrapping the KeyRx Core engine.
library;

import '../ffi/bridge.dart'
    if (dart.library.html) '../ffi/bridge_stub.dart';

/// Benchmark performance data.
class BenchmarkData {
  const BenchmarkData({
    required this.minNs,
    required this.maxNs,
    required this.meanNs,
    required this.p99Ns,
    required this.iterations,
    required this.hasWarning,
    this.warning,
  });

  final int minNs;
  final int maxNs;
  final int meanNs;
  final int p99Ns;
  final int iterations;
  final bool hasWarning;
  final String? warning;

  /// Minimum latency in microseconds.
  double get minUs => minNs / 1000.0;

  /// Maximum latency in microseconds.
  double get maxUs => maxNs / 1000.0;

  /// Mean latency in microseconds.
  double get meanUs => meanNs / 1000.0;

  /// P99 latency in microseconds.
  double get p99Us => p99Ns / 1000.0;
}

/// Service result wrapper for benchmark operations.
class BenchmarkServiceResult {
  const BenchmarkServiceResult({this.data, this.errorMessage});

  factory BenchmarkServiceResult.error(String message) =>
      BenchmarkServiceResult(errorMessage: message);

  final BenchmarkData? data;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Abstraction for benchmark operations.
abstract class BenchmarkService {
  /// Run benchmark with specified iterations.
  ///
  /// [iterations] - Number of iterations to run.
  /// [scriptPath] - Optional path to Rhai script (null uses active script).
  Future<BenchmarkServiceResult> runBenchmark(
    int iterations, {
    String? scriptPath,
  });
}

/// Implementation of BenchmarkService that wraps FFI bindings.
class BenchmarkServiceImpl implements BenchmarkService {
  BenchmarkServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<BenchmarkServiceResult> runBenchmark(
    int iterations, {
    String? scriptPath,
  }) async {
    if (_bridge.loadFailure != null) {
      return BenchmarkServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.runBenchmark(iterations, scriptPath: scriptPath);

    if (result.hasError) {
      return BenchmarkServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    return BenchmarkServiceResult(
      data: BenchmarkData(
        minNs: result.minNs,
        maxNs: result.maxNs,
        meanNs: result.meanNs,
        p99Ns: result.p99Ns,
        iterations: result.iterations,
        hasWarning: result.hasWarning,
        warning: result.warning,
      ),
    );
  }
}
