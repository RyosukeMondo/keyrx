import 'dart:io';
import 'package:path/path.dart' as path;
import '../models.dart';

class DuplicateImplementationAnalyzer {
  Future<List<SSOTViolation>> analyze(String projectPath) async {
    final violations = <SSOTViolation>[];

    // Check for duplicate logic between core_logic and UI
    violations.addAll(await _checkDuplicateClasses(projectPath));
    violations.addAll(await _checkDuplicateConstants(projectPath));

    return violations;
  }

  Future<List<SSOTViolation>> _checkDuplicateClasses(String projectPath) async {
    final violations = <SSOTViolation>[];

    final coreLogicPath = path.join(projectPath, 'packages', 'core_logic', 'lib');
    final uiPath = path.join(projectPath, 'ui', 'lib');

    if (!await Directory(coreLogicPath).exists() ||
        !await Directory(uiPath).exists()) {
      return violations;
    }

    final coreLogicClasses = await _extractClasses(coreLogicPath);
    final uiClasses = await _extractClasses(uiPath);

    // Find classes with same name in both locations
    final duplicateClasses = coreLogicClasses.keys.toSet()
        .intersection(uiClasses.keys.toSet());

    for (final className in duplicateClasses) {
      // Skip UI wrapper classes (those ending in UI or ChangeNotifier wrappers)
      if (className.endsWith('UI') ||
          className.endsWith('Notifier') ||
          className.endsWith('Provider')) {
        continue;
      }

      final uiFile = uiClasses[className]!;
      final relativePath = path.relative(uiFile, from: projectPath);

      violations.add(SSOTViolation(
        type: ViolationType.duplicateImplementation,
        file: relativePath,
        line: null,
        message: 'Class "$className" duplicated in UI layer (also exists in core_logic)',
        suggestion: 'Remove duplicate, import from core_logic instead',
      ));
    }

    return violations;
  }

  Future<List<SSOTViolation>> _checkDuplicateConstants(String projectPath) async {
    final violations = <SSOTViolation>[];

    final coreLogicPath = path.join(projectPath, 'packages', 'core_logic', 'lib');
    final uiPath = path.join(projectPath, 'ui', 'lib');

    if (!await Directory(coreLogicPath).exists() ||
        !await Directory(uiPath).exists()) {
      return violations;
    }

    final coreLogicConstants = await _extractConstants(coreLogicPath);
    final uiConstants = await _extractConstants(uiPath);

    // Find constants with same name and value
    for (final entry in uiConstants.entries) {
      final constName = entry.key;
      final uiFile = entry.value;

      if (coreLogicConstants.containsKey(constName)) {
        final relativePath = path.relative(uiFile, from: projectPath);
        violations.add(SSOTViolation(
          type: ViolationType.duplicateImplementation,
          file: relativePath,
          line: null,
          message: 'Constant "$constName" duplicated in UI layer (also exists in core_logic)',
          suggestion: 'Remove duplicate, import from core_logic instead',
        ));
      }
    }

    return violations;
  }

  Future<Map<String, String>> _extractClasses(String directoryPath) async {
    final classes = <String, String>{};
    final dir = Directory(directoryPath);

    if (!await dir.exists()) {
      return classes;
    }

    await for (final entity in dir.list(recursive: true)) {
      if (entity is File && entity.path.endsWith('.dart')) {
        final content = await entity.readAsString();
        final classPattern = RegExp(r'class\s+(\w+)(?:\s+extends|\s+implements|\s*\{)');

        for (final match in classPattern.allMatches(content)) {
          final className = match.group(1)!;
          classes[className] = entity.path;
        }
      }
    }

    return classes;
  }

  Future<Map<String, String>> _extractConstants(String directoryPath) async {
    final constants = <String, String>{};
    final dir = Directory(directoryPath);

    if (!await dir.exists()) {
      return constants;
    }

    await for (final entity in dir.list(recursive: true)) {
      if (entity is File && entity.path.endsWith('.dart')) {
        final content = await entity.readAsString();
        final constPattern = RegExp(r'const\s+(\w+)\s*=');

        for (final match in constPattern.allMatches(content)) {
          final constName = match.group(1)!;
          constants[constName] = entity.path;
        }
      }
    }

    return constants;
  }
}
