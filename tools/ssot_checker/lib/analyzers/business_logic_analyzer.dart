import 'dart:io';
import 'package:path/path.dart' as path;
import '../models.dart';

class BusinessLogicAnalyzer {
  Future<List<SSOTViolation>> analyze(String projectPath) async {
    final violations = <SSOTViolation>[];
    final uiPath = path.join(projectPath, 'ui', 'lib');
    final uiDir = Directory(uiPath);

    if (!await uiDir.exists()) {
      return violations;
    }

    await for (final entity in uiDir.list(recursive: true)) {
      if (entity is File && entity.path.endsWith('.dart')) {
        violations.addAll(await _checkFile(entity, projectPath));
      }
    }

    return violations;
  }

  Future<List<SSOTViolation>> _checkFile(File file, String projectPath) async {
    final violations = <SSOTViolation>[];
    final fileName = path.basename(file.path);

    // Skip service_registry and UI wrappers
    if (fileName == 'service_registry.dart' || fileName.endsWith('_ui.dart')) {
      return violations;
    }

    final content = await file.readAsString();
    final lines = content.split('\n');

    // Check for direct FFI bridge usage
    for (var i = 0; i < lines.length; i++) {
      final line = lines[i].trim();

      if (line.contains('KeyrxBridge') && !line.startsWith('//')) {
        final relativePath = path.relative(file.path, from: projectPath);
        violations.add(SSOTViolation(
          type: ViolationType.businessLogicInUI,
          file: relativePath,
          line: i + 1,
          message: 'Direct FFI bridge usage detected in UI layer',
          suggestion: 'Use services from core_logic instead',
        ));
      }
    }

    // Check for business logic patterns
    violations.addAll(_detectBusinessLogicPatterns(content, file, projectPath));

    return violations;
  }

  List<SSOTViolation> _detectBusinessLogicPatterns(
    String content,
    File file,
    String projectPath,
  ) {
    final violations = <SSOTViolation>[];
    final lines = content.split('\n');

    // Pattern: Non-widget classes with business logic methods
    final classPattern = RegExp(
      r'class\s+(\w+)(?!\s*extends\s*(StatelessWidget|StatefulWidget|ChangeNotifier|Widget))',
    );
    final businessMethodPattern = RegExp(
      r'\b(validate|calculate|transform|process|compute|analyze)\w*\s*\(',
    );

    var currentClass = '';
    var isWidget = false;

    for (var i = 0; i < lines.length; i++) {
      final line = lines[i];

      // Check for class declaration
      final classMatch = classPattern.firstMatch(line);
      if (classMatch != null) {
        currentClass = classMatch.group(1)!;
        isWidget = line.contains('extends') &&
            (line.contains('Widget') ||
                line.contains('State<') ||
                line.contains('ChangeNotifier'));
      }

      // If we're in a non-widget class, check for business logic
      if (currentClass.isNotEmpty &&
          !isWidget &&
          businessMethodPattern.hasMatch(line) &&
          !line.trim().startsWith('//')) {
        final relativePath = path.relative(file.path, from: projectPath);
        violations.add(SSOTViolation(
          type: ViolationType.businessLogicInUI,
          file: relativePath,
          line: i + 1,
          message: 'Business logic detected in UI layer (class: $currentClass)',
          suggestion: 'Move business logic to core_logic service',
        ));
      }

      // Reset class context on closing brace at start of line
      if (line.trim().startsWith('}') && currentClass.isNotEmpty) {
        currentClass = '';
        isWidget = false;
      }
    }

    return violations;
  }
}
