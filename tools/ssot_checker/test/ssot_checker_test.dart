import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:test/test.dart';
import 'package:ssot_checker/ssot_checker.dart';
import 'package:ssot_checker/models.dart';
import 'package:ssot_checker/analyzers/flutter_import_analyzer.dart';

void main() {
  late Directory tempDir;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('ssot_test_');
  });

  tearDown(() async {
    if (await tempDir.exists()) {
      await tempDir.delete(recursive: true);
    }
  });

  group('FlutterImportAnalyzer', () {
    test('detects Flutter imports in core_logic', () async {
      final coreLogicDir = Directory(
        path.join(tempDir.path, 'packages', 'core_logic', 'lib'),
      );
      await coreLogicDir.create(recursive: true);

      final testFile = File(path.join(coreLogicDir.path, 'test.dart'));
      await testFile.writeAsString('''
import 'package:flutter/material.dart';

class TestClass {
  void test() {}
}
''');

      final analyzer = FlutterImportAnalyzer();
      final violations = await analyzer.analyze(tempDir.path);

      expect(violations, hasLength(1));
      expect(violations.first.type, ViolationType.flutterInCoreLogic);
      expect(violations.first.line, 1);
    });

    test('allows pure Dart imports in core_logic', () async {
      final coreLogicDir = Directory(
        path.join(tempDir.path, 'packages', 'core_logic', 'lib'),
      );
      await coreLogicDir.create(recursive: true);

      final testFile = File(path.join(coreLogicDir.path, 'test.dart'));
      await testFile.writeAsString('''
import 'dart:async';
import 'package:some_dart_package/lib.dart';

class TestClass {
  void test() {}
}
''');

      final analyzer = FlutterImportAnalyzer();
      final violations = await analyzer.analyze(tempDir.path);

      expect(violations, isEmpty);
    });
  });

  group('SSOTChecker', () {
    test('returns true when no violations found', () async {
      final checker = SSOTChecker();
      final success = await checker.check(tempDir.path);

      expect(success, isTrue);
      expect(checker.violations, isEmpty);
    });

    test('returns false when violations found', () async {
      final coreLogicDir = Directory(
        path.join(tempDir.path, 'packages', 'core_logic', 'lib'),
      );
      await coreLogicDir.create(recursive: true);

      final testFile = File(path.join(coreLogicDir.path, 'test.dart'));
      await testFile.writeAsString('''
import 'package:flutter/widgets.dart';

class TestClass {}
''');

      final checker = SSOTChecker();
      final success = await checker.check(tempDir.path);

      expect(success, isFalse);
      expect(checker.violations, isNotEmpty);
    });
  });
}
