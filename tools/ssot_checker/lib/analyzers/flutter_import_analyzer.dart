import 'dart:io';
import 'package:path/path.dart' as path;
import '../models.dart';

class FlutterImportAnalyzer {
  Future<List<SSOTViolation>> analyze(String projectPath) async {
    final violations = <SSOTViolation>[];
    final coreLogicPath = path.join(projectPath, 'packages', 'core_logic', 'lib');
    final coreLogicDir = Directory(coreLogicPath);

    if (!await coreLogicDir.exists()) {
      return violations;
    }

    await for (final entity in coreLogicDir.list(recursive: true)) {
      if (entity is File && entity.path.endsWith('.dart')) {
        violations.addAll(await _checkFile(entity, projectPath));
      }
    }

    return violations;
  }

  Future<List<SSOTViolation>> _checkFile(File file, String projectPath) async {
    final violations = <SSOTViolation>[];
    final content = await file.readAsString();
    final lines = content.split('\n');

    for (var i = 0; i < lines.length; i++) {
      final line = lines[i].trim();

      if (line.startsWith('import ') && line.contains('package:flutter/')) {
        final relativePath = path.relative(file.path, from: projectPath);
        violations.add(SSOTViolation(
          type: ViolationType.flutterInCoreLogic,
          file: relativePath,
          line: i + 1,
          message: 'Flutter import detected in core_logic',
          suggestion: 'Remove Flutter dependency, use pure Dart',
        ));
      }
    }

    return violations;
  }
}
