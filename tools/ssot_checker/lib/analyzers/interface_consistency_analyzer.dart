import 'dart:io';
import 'package:path/path.dart' as path;
import '../models.dart';

class InterfaceConsistencyAnalyzer {
  final List<String> services = [
    'device_registry_service',
    'profile_registry_service',
    'hardware_service',
    'mapping_service',
    'config_service',
    'profile_manager',
    'device_manager',
    'notification_service',
    'history_service',
    'export_service',
    'import_service',
  ];

  Future<List<SSOTViolation>> analyze(String projectPath) async {
    final violations = <SSOTViolation>[];
    final servicesPath = path.join(projectPath, 'packages', 'core_logic', 'lib', 'services');

    if (!await Directory(servicesPath).exists()) {
      return violations;
    }

    for (final service in services) {
      final baseFile = File(path.join(servicesPath, '$service.dart'));
      final ffiFile = File(path.join(servicesPath, '${service}_ffi.dart'));
      final webFile = File(path.join(servicesPath, '${service}_web.dart'));

      // Check if base file exists
      if (!await baseFile.exists()) {
        continue;
      }

      // Check if base file has conditional export
      final baseContent = await baseFile.readAsString();
      if (!_hasConditionalExport(baseContent)) {
        final relativePath = path.relative(baseFile.path, from: projectPath);
        violations.add(SSOTViolation(
          type: ViolationType.interfaceInconsistency,
          file: relativePath,
          line: null,
          message: 'Service missing conditional export statement',
          suggestion: "Add: export '${service}_ffi.dart' if (dart.library.html) '${service}_web.dart';",
        ));
      }

      // Check interface consistency between FFI and Web implementations
      if (await ffiFile.exists() && await webFile.exists()) {
        final ffiMethods = await _extractMethods(ffiFile);
        final webMethods = await _extractMethods(webFile);

        final missingInWeb = ffiMethods.difference(webMethods);
        final missingInFfi = webMethods.difference(ffiMethods);

        if (missingInWeb.isNotEmpty) {
          final relativePath = path.relative(webFile.path, from: projectPath);
          violations.add(SSOTViolation(
            type: ViolationType.interfaceInconsistency,
            file: relativePath,
            line: null,
            message: 'Web implementation missing methods: ${missingInWeb.join(", ")}',
            suggestion: 'Implement missing methods from FFI interface',
          ));
        }

        if (missingInFfi.isNotEmpty) {
          final relativePath = path.relative(ffiFile.path, from: projectPath);
          violations.add(SSOTViolation(
            type: ViolationType.interfaceInconsistency,
            file: relativePath,
            line: null,
            message: 'FFI implementation missing methods: ${missingInFfi.join(", ")}',
            suggestion: 'Implement missing methods from Web interface',
          ));
        }
      }
    }

    return violations;
  }

  bool _hasConditionalExport(String content) {
    return content.contains(RegExp(r"export\s+'.+'\s+if\s+\(dart\.library\..+\)"));
  }

  Future<Set<String>> _extractMethods(File file) async {
    final content = await file.readAsString();
    final methods = <String>{};

    // Match method declarations (public methods)
    final methodPattern = RegExp(
      r'^\s+(?:Future<[^>]+>|Stream<[^>]+>|void|[A-Z]\w*)\s+(\w+)\s*\(',
      multiLine: true,
    );

    // Match getters
    final getterPattern = RegExp(
      r'^\s+(?:Future<[^>]+>|Stream<[^>]+>|[A-Z]\w*)\s+get\s+(\w+)',
      multiLine: true,
    );

    for (final match in methodPattern.allMatches(content)) {
      final methodName = match.group(1)!;
      if (!methodName.startsWith('_')) {
        methods.add(methodName);
      }
    }

    for (final match in getterPattern.allMatches(content)) {
      final getterName = match.group(1)!;
      if (!getterName.startsWith('_')) {
        methods.add('get $getterName');
      }
    }

    return methods;
  }
}
