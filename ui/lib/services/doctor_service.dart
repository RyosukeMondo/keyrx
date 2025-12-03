/// Doctor service for system diagnostics.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import '../ffi/bridge.dart';

/// Individual diagnostic check result.
class DiagnosticCheckData {
  const DiagnosticCheckData({
    required this.name,
    required this.status,
    this.details,
    this.remediation,
  });

  final String name;
  final String status;
  final String? details;
  final String? remediation;

  bool get passed => status == 'pass';
  bool get failed => status == 'fail';
  bool get warned => status == 'warn';
}

/// Full diagnostic report.
class DiagnosticReport {
  const DiagnosticReport({
    required this.checks,
    required this.passed,
    required this.failed,
    required this.warned,
  });

  final List<DiagnosticCheckData> checks;
  final int passed;
  final int failed;
  final int warned;

  bool get allPassed => failed == 0;
  bool get hasWarnings => warned > 0;
}

/// Result of a diagnostics operation.
class DoctorServiceResult {
  const DoctorServiceResult({
    this.report,
    this.errorMessage,
  });

  factory DoctorServiceResult.error(String message) =>
      DoctorServiceResult(errorMessage: message);

  final DiagnosticReport? report;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Abstraction for diagnostics operations.
abstract class DoctorService {
  /// Run system diagnostics.
  Future<DoctorServiceResult> runDiagnostics();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real DoctorService that wraps the KeyrxBridge.
class DoctorServiceImpl implements DoctorService {
  DoctorServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<DoctorServiceResult> runDiagnostics() async {
    if (_bridge.loadFailure != null) {
      return DoctorServiceResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.runDoctor();

    if (result.hasError) {
      return DoctorServiceResult.error(result.errorMessage ?? 'Unknown error');
    }

    final checks = result.checks.map((c) {
      return DiagnosticCheckData(
        name: c.name,
        status: c.status,
        details: c.details,
        remediation: c.remediation,
      );
    }).toList();

    return DoctorServiceResult(
      report: DiagnosticReport(
        checks: checks,
        passed: result.passed,
        failed: result.failed,
        warned: result.warned,
      ),
    );
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose
  }
}
