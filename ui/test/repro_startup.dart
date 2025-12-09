// ignore_for_file: avoid_print
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:logging/logging.dart';

void main() {
  Logger.root.level = Level.ALL;
  Logger.root.onRecord.listen((record) {
    print('${record.level.name}: ${record.time}: ${record.message}');
  });

  test('Reproduction: Engine Initialization Recovery', () async {
    print('1. Opening Bridge...');
    final bridge = KeyrxBridge.open();

    print('2. Initializing Bridge (should be idempotent and robust)...');
    final initResult = bridge.initialize();
    print('   Bridge initialize result: $initResult');
    expect(initResult, isTrue, reason: 'Bridge should initialize successfully');

    if (!initResult) return;

    // Simulate "Hot Restart" condition where Dart side might be fresh but checking native state
    print('3. Attempting to load script...');

    // We need a dummy script
    final scriptFile = File('test_monitor.rhai');
    await scriptFile.writeAsString('// Test script\n');

    // Check if we can load it.
    // loadScript call involves:
    // - _bridge.bindings.engineLoadScript(path)
    // We expect this to SUCCEED if _initRevolutionaryRuntime was called.

    try {
      // We use the raw binding or bridge method if available.
      // BridgeEngineMixin has loadScript
      final loaded = bridge.loadScript(scriptFile.absolute.path);
      print('   Load script result (bool): $loaded');

      // If false, it might be due to "Engine not initialized" logged on native side
      // Note: bridge.loadScript returns bool, catches internal errors?
      // Let's check bridge_engine.dart implementation.
      // It returns false on error.

      expect(
        loaded,
        isTrue,
        reason: 'Script should load if engine is initialized',
      );
    } finally {
      if (scriptFile.existsSync()) {
        scriptFile.deleteSync();
      }
    }
  });
}
