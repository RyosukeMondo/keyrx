/// Test discovery and execution service for Rhai scripts.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import '../ffi/bridge.dart';

/// Result of a test discovery operation.
class TestDiscoveryServiceResult {
  const TestDiscoveryServiceResult({required this.tests, this.errorMessage});

  factory TestDiscoveryServiceResult.error(String message) =>
      TestDiscoveryServiceResult(tests: const [], errorMessage: message);

  final List<TestCase> tests;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Test case discovered from a script.
class TestCase {
  const TestCase({required this.name, required this.file, this.line});

  final String name;
  final String file;
  final int? line;
}

/// Result of running tests.
class TestRunServiceResult {
  const TestRunServiceResult({
    required this.total,
    required this.passed,
    required this.failed,
    required this.durationMs,
    required this.results,
    this.errorMessage,
  });

  factory TestRunServiceResult.error(String message) => TestRunServiceResult(
    total: 0,
    passed: 0,
    failed: 0,
    durationMs: 0,
    results: const [],
    errorMessage: message,
  );

  final int total;
  final int passed;
  final int failed;
  final double durationMs;
  final List<TestCaseResult> results;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get allPassed => failed == 0 && !hasError;
}

/// Result of a single test case.
class TestCaseResult {
  const TestCaseResult({
    required this.name,
    required this.passed,
    this.error,
    required this.durationMs,
  });

  final String name;
  final bool passed;
  final String? error;
  final double durationMs;
}

/// Abstraction for test discovery and execution operations.
abstract class TestService {
  /// Discover test functions in a Rhai script.
  Future<TestDiscoveryServiceResult> discoverTests(String scriptPath);

  /// Run tests in a Rhai script with optional filter pattern.
  Future<TestRunServiceResult> runTests(String scriptPath, {String? filter});

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real TestService that wraps the KeyrxBridge.
class TestServiceImpl implements TestService {
  TestServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<TestDiscoveryServiceResult> discoverTests(String scriptPath) async {
    if (_bridge.loadFailure != null) {
      return TestDiscoveryServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.discoverTests(scriptPath);

    if (result.hasError) {
      return TestDiscoveryServiceResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    final tests = result.tests.map((t) {
      return TestCase(name: t.name, file: t.file, line: t.line);
    }).toList();

    return TestDiscoveryServiceResult(tests: tests);
  }

  @override
  Future<TestRunServiceResult> runTests(
    String scriptPath, {
    String? filter,
  }) async {
    if (_bridge.loadFailure != null) {
      return TestRunServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.runTests(scriptPath, filter: filter);

    if (result.hasError) {
      return TestRunServiceResult.error(result.errorMessage ?? 'Unknown error');
    }

    final results = result.results.map((r) {
      return TestCaseResult(
        name: r.name,
        passed: r.passed,
        error: r.error,
        durationMs: r.durationMs,
      );
    }).toList();

    return TestRunServiceResult(
      total: result.total,
      passed: result.passed,
      failed: result.failed,
      durationMs: result.durationMs,
      results: results,
    );
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose
  }
}
