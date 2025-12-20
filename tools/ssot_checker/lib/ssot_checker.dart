import 'dart:io';
import 'analyzers/flutter_import_analyzer.dart';
import 'analyzers/business_logic_analyzer.dart';
import 'analyzers/interface_consistency_analyzer.dart';
import 'analyzers/duplicate_implementation_analyzer.dart';
import 'models.dart';

class SSOTChecker {
  final List<SSOTViolation> violations = [];
  final bool verbose;

  SSOTChecker({this.verbose = false});

  Future<bool> check(String projectPath) async {
    violations.clear();

    final startTime = DateTime.now();

    if (verbose) {
      print('Starting SSOT violation check...');
      print('Project path: $projectPath\n');
    }

    // Run all analyzers
    await _runAnalyzer(
      'Flutter imports in core_logic',
      FlutterImportAnalyzer(),
      projectPath,
    );

    await _runAnalyzer(
      'Business logic in UI',
      BusinessLogicAnalyzer(),
      projectPath,
    );

    await _runAnalyzer(
      'Interface consistency',
      InterfaceConsistencyAnalyzer(),
      projectPath,
    );

    await _runAnalyzer(
      'Duplicate implementations',
      DuplicateImplementationAnalyzer(),
      projectPath,
    );

    final duration = DateTime.now().difference(startTime);

    if (verbose) {
      print('\nCheck completed in ${duration.inMilliseconds}ms');
      print('Total violations: ${violations.length}');
    }

    return violations.isEmpty;
  }

  Future<void> _runAnalyzer(
    String name,
    dynamic analyzer,
    String projectPath,
  ) async {
    if (verbose) {
      stdout.write('Checking $name... ');
    }

    final startTime = DateTime.now();
    final results = await analyzer.analyze(projectPath);
    final duration = DateTime.now().difference(startTime);

    violations.addAll(results);

    if (verbose) {
      print('${results.length} violations (${duration.inMilliseconds}ms)');
    }
  }

  void printReport() {
    if (violations.isEmpty) {
      print('\n✅ No SSOT violations found!');
      return;
    }

    print('\n❌ SSOT Violations Detected:\n');

    final byType = <ViolationType, List<SSOTViolation>>{};
    for (final violation in violations) {
      byType.putIfAbsent(violation.type, () => []).add(violation);
    }

    for (final entry in byType.entries) {
      final type = entry.key;
      final typeViolations = entry.value;

      print('━━━ ${_typeToString(type)} (${typeViolations.length}) ━━━\n');

      for (final violation in typeViolations) {
        print(violation);
        print('');
      }
    }

    print('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    print('Total: ${violations.length} violation(s)');
    print('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  }

  String _typeToString(ViolationType type) {
    switch (type) {
      case ViolationType.flutterInCoreLogic:
        return 'Flutter Imports in core_logic';
      case ViolationType.businessLogicInUI:
        return 'Business Logic in UI';
      case ViolationType.interfaceInconsistency:
        return 'Interface Inconsistency';
      case ViolationType.duplicateImplementation:
        return 'Duplicate Implementation';
    }
  }
}
