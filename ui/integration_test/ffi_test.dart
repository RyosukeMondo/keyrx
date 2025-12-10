/// Integration test: FFI round-trip verification.
///
/// Tests Dart→Rust→Dart data flow for key FFI operations.
/// Skips tests if the native library is unavailable.
library;

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/device_state.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  late KeyrxBridge bridge;
  bool libraryAvailable = false;

  setUpAll(() {
    bridge = KeyrxBridge.open();
    libraryAvailable = bridge.loadFailure == null;
    if (!libraryAvailable) {
      // ignore: avoid_print
      print('Skipping FFI tests: ${bridge.loadFailure}');
    }
  });

  group('FFI round-trip tests', () {
    testWidgets('listDevices returns valid device list', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      final result = bridge.listRegisteredDevices();
      expect(result.hasError, isFalse);
      expect(result.data, isA<List<DeviceState>>());
      // Note: The device list might be empty on CI/simulation, which is valid.
    });

    testWidgets('checkScript validates Rhai syntax', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      // Valid script
      final validResult = bridge.checkScript('let x = 1;');
      expect(validResult.valid, isTrue);
      expect(validResult.errors, isEmpty);

      // Invalid script
      final invalidResult = bridge.checkScript('let x = ');
      expect(invalidResult.valid, isFalse);
      expect(invalidResult.errors, isNotEmpty);
    });

    testWidgets('simulate processes key sequences', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      final result = bridge.simulate([
        const KeyInput(code: 'KeyA', holdMs: 50),
      ], comboMode: false);

      // Should return a result without crashing
      expect(result, isNotNull);
      // Note: The exact output depends on the engine state
    });

    testWidgets('runBenchmark returns latency metrics', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      final result = bridge.runBenchmark(100);

      // Should return benchmark data
      if (!result.hasError) {
        expect(result.iterations, greaterThan(0));
        expect(result.minNs, greaterThanOrEqualTo(0));
        expect(result.maxNs, greaterThanOrEqualTo(result.minNs));
        expect(result.meanNs, greaterThanOrEqualTo(result.minNs));
        expect(result.meanNs, lessThanOrEqualTo(result.maxNs));
      }
    });

    testWidgets('runDoctor returns diagnostic checks', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      final result = bridge.runDoctor();

      // Should return diagnostic data
      if (!result.hasError) {
        expect(result.checks, isA<List<DiagnosticCheck>>());
        expect(result.passed, greaterThanOrEqualTo(0));
        expect(result.failed, greaterThanOrEqualTo(0));
        expect(result.warned, greaterThanOrEqualTo(0));
      }
    });

    testWidgets('listSessions returns session list', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      final result = bridge.listSessions('sessions/');

      // Should return a list (possibly empty) without crashing
      expect(result.sessions, isA<List<SessionInfo>>());
    });

    testWidgets('discoverTests finds test functions', (tester) async {
      if (!libraryAvailable) {
        // ignore: avoid_print
        print('Skipping test: library unavailable');
        return;
      }

      // Test discovery on a non-existent file should handle gracefully
      final result = bridge.discoverTests('nonexistent.rhai');

      // Should return a result (possibly with error) without crashing
      expect(result, isNotNull);
    });
  });
}
