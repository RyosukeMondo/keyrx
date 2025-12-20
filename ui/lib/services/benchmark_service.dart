/// UI benchmark service that delegates to core BenchmarkService.
///
/// Provides UI-specific BenchmarkServiceResult wrapper around core service.
library;

import 'package:core_logic/services/benchmark_service.dart' as core;

/// Re-export BenchmarkData from core for UI compatibility.
typedef BenchmarkData = core.BenchmarkData;

/// UI-specific result wrapper for benchmark operations.
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

  /// Dispose any held resources.
  Future<void> dispose();
}

/// UI BenchmarkService that delegates to core service.
class BenchmarkServiceImpl implements BenchmarkService {
  BenchmarkServiceImpl({required core.BenchmarkService coreService})
    : _coreService = coreService;

  final core.BenchmarkService _coreService;

  @override
  Future<BenchmarkServiceResult> runBenchmark(
    int iterations, {
    String? scriptPath,
  }) async {
    final result = await _coreService.runBenchmark(
      iterations,
      scriptPath: scriptPath,
    );

    if (result.hasError) {
      return BenchmarkServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    return BenchmarkServiceResult(data: result.data);
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose
  }
}
