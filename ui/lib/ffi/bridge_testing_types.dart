/// Result types for testing and diagnostic FFI methods.
library;

import 'dart:convert';

/// Discovered test function.
class DiscoveredTest {
  const DiscoveredTest({required this.name, required this.file, this.line});

  final String name;
  final String file;
  final int? line;
}

/// Test discovery result.
class TestDiscoveryResult {
  const TestDiscoveryResult({required this.tests, this.errorMessage});

  factory TestDiscoveryResult.error(String message) =>
      TestDiscoveryResult(tests: const [], errorMessage: message);

  factory TestDiscoveryResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return TestDiscoveryResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return TestDiscoveryResult.error('invalid test list payload');
      }

      final tests = decoded
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return DiscoveredTest(
              name: e['name']?.toString() ?? '',
              file: e['file']?.toString() ?? '',
              line: (e['line'] as num?)?.toInt(),
            );
          })
          .whereType<DiscoveredTest>()
          .toList();

      return TestDiscoveryResult(tests: tests);
    } catch (e) {
      return TestDiscoveryResult.error('$e');
    }
  }

  final List<DiscoveredTest> tests;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Individual test result.
class TestResult {
  const TestResult({
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

/// Test run result.
class TestRunResult {
  const TestRunResult({
    required this.total,
    required this.passed,
    required this.failed,
    required this.durationMs,
    required this.results,
    this.errorMessage,
  });

  factory TestRunResult.error(String message) => TestRunResult(
    total: 0,
    passed: 0,
    failed: 0,
    durationMs: 0,
    results: const [],
    errorMessage: message,
  );

  factory TestRunResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return TestRunResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return TestRunResult.error('invalid test run payload');
      }

      final resultsList = decoded['results'] as List<dynamic>? ?? [];
      final results = resultsList
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return TestResult(
              name: e['name']?.toString() ?? '',
              passed: e['passed'] as bool? ?? false,
              error: e['error']?.toString(),
              durationMs: (e['durationMs'] as num?)?.toDouble() ?? 0,
            );
          })
          .whereType<TestResult>()
          .toList();

      return TestRunResult(
        total: (decoded['total'] as num?)?.toInt() ?? 0,
        passed: (decoded['passed'] as num?)?.toInt() ?? 0,
        failed: (decoded['failed'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        results: results,
      );
    } catch (e) {
      return TestRunResult.error('$e');
    }
  }

  final int total;
  final int passed;
  final int failed;
  final double durationMs;
  final List<TestResult> results;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Key input for simulation.
class KeyInput {
  const KeyInput({required this.code, this.holdMs});

  final String code;
  final int? holdMs;

  Map<String, dynamic> toJson() => {
    'code': code,
    if (holdMs != null) 'holdMs': holdMs,
  };
}

/// Key mapping from simulation.
class SimulationMapping {
  const SimulationMapping({
    required this.input,
    required this.output,
    required this.decision,
  });

  final String input;
  final String output;
  final String decision;
}

/// Simulation result.
class SimulationResult {
  const SimulationResult({
    required this.mappings,
    required this.activeLayers,
    required this.pending,
    this.errorMessage,
  });

  factory SimulationResult.error(String message) => SimulationResult(
    mappings: const [],
    activeLayers: const [],
    pending: const [],
    errorMessage: message,
  );

  factory SimulationResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SimulationResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return SimulationResult.error('invalid simulation payload');
      }

      final mappingsList = decoded['mappings'] as List<dynamic>? ?? [];
      final mappings = mappingsList
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return SimulationMapping(
              input: e['input']?.toString() ?? '',
              output: e['output']?.toString() ?? '',
              decision: e['decision']?.toString() ?? '',
            );
          })
          .whereType<SimulationMapping>()
          .toList();

      final layers =
          (decoded['activeLayers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final pending =
          (decoded['pending'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];

      return SimulationResult(
        mappings: mappings,
        activeLayers: layers,
        pending: pending,
      );
    } catch (e) {
      return SimulationResult.error('$e');
    }
  }

  final List<SimulationMapping> mappings;
  final List<String> activeLayers;
  final List<String> pending;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Benchmark result.
class BenchmarkResult {
  const BenchmarkResult({
    required this.minNs,
    required this.maxNs,
    required this.meanNs,
    required this.p99Ns,
    required this.iterations,
    required this.hasWarning,
    this.warning,
    this.errorMessage,
  });

  factory BenchmarkResult.error(String message) => BenchmarkResult(
    minNs: 0,
    maxNs: 0,
    meanNs: 0,
    p99Ns: 0,
    iterations: 0,
    hasWarning: false,
    errorMessage: message,
  );

  factory BenchmarkResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return BenchmarkResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return BenchmarkResult.error('invalid benchmark payload');
      }

      return BenchmarkResult(
        minNs: (decoded['minNs'] as num?)?.toInt() ?? 0,
        maxNs: (decoded['maxNs'] as num?)?.toInt() ?? 0,
        meanNs: (decoded['meanNs'] as num?)?.toInt() ?? 0,
        p99Ns: (decoded['p99Ns'] as num?)?.toInt() ?? 0,
        iterations: (decoded['iterations'] as num?)?.toInt() ?? 0,
        hasWarning: decoded['hasWarning'] as bool? ?? false,
        warning: decoded['warning']?.toString(),
      );
    } catch (e) {
      return BenchmarkResult.error('$e');
    }
  }

  final int minNs;
  final int maxNs;
  final int meanNs;
  final int p99Ns;
  final int iterations;
  final bool hasWarning;
  final String? warning;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Diagnostic check result.
class DiagnosticCheck {
  const DiagnosticCheck({
    required this.name,
    required this.status,
    this.details,
    this.remediation,
  });

  final String name;
  final String status;
  final String? details;
  final String? remediation;
}

/// Doctor result.
class DoctorResult {
  const DoctorResult({
    required this.checks,
    required this.passed,
    required this.failed,
    required this.warned,
    this.errorMessage,
  });

  factory DoctorResult.error(String message) => DoctorResult(
    checks: const [],
    passed: 0,
    failed: 0,
    warned: 0,
    errorMessage: message,
  );

  factory DoctorResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DoctorResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DoctorResult.error('invalid doctor payload');
      }

      final checksList = decoded['checks'] as List<dynamic>? ?? [];
      final checks = checksList
          .map((e) {
            if (e is! Map<String, dynamic>) return null;
            return DiagnosticCheck(
              name: e['name']?.toString() ?? '',
              status: e['status']?.toString() ?? '',
              details: e['details']?.toString(),
              remediation: e['remediation']?.toString(),
            );
          })
          .whereType<DiagnosticCheck>()
          .toList();

      return DoctorResult(
        checks: checks,
        passed: (decoded['passed'] as num?)?.toInt() ?? 0,
        failed: (decoded['failed'] as num?)?.toInt() ?? 0,
        warned: (decoded['warned'] as num?)?.toInt() ?? 0,
      );
    } catch (e) {
      return DoctorResult.error('$e');
    }
  }

  final List<DiagnosticCheck> checks;
  final int passed;
  final int failed;
  final int warned;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}
