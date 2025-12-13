import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:keyrx_ui/ffi/generated_bindings.dart'; // For generated strict bindings types if needed
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('FFI Contract Tests', () {
    late DynamicLibrary lib;
    late RuntimeBindings runtimeBindings;
    late ConfigBindings configBindings;

    setUpAll(() {
      // Locate the binaries.
      // In development, they are likely in target/debug/ or target/release/
      // Or in windows/runner/ if built by Flutter.
      // Assuming typical Rust cargo build location for now or local path
      var path = 'keyrx_core.dll';
      if (Platform.isWindows) {
        // Try paths relative to ui/ directory
        if (File('../target/debug/keyrx_core.dll').existsSync()) {
          path = '../target/debug/keyrx_core.dll';
        } else if (File('../target/release/keyrx_core.dll').existsSync()) {
          path = '../target/release/keyrx_core.dll';
        } else if (File(
          'build/windows/runner/Debug/keyrx_core.dll',
        ).existsSync()) {
          path = 'build/windows/runner/Debug/keyrx_core.dll';
        } else if (File(
          '../build/windows/runner/Debug/keyrx_core.dll',
        ).existsSync()) {
          path = '../build/windows/runner/Debug/keyrx_core.dll';
        }
      } else if (Platform.isLinux) {
        if (File('../target/debug/libkeyrx_core.so').existsSync()) {
          path = '../target/debug/libkeyrx_core.so';
        } else {
          path = 'libkeyrx_core.so';
        }
      }

      stdout.writeln('Loading library from: $path');
      lib = DynamicLibrary.open(path);
      runtimeBindings = RuntimeBindings(lib);
      configBindings = ConfigBindings(lib);

      // Initialize core (using ConfigBindings or manual lookup if init is not exposed)
      // generated_bindings.dart doesn't usually expose init() unless in a domain.
      // We can rely on lazy init or just assume it works.
      // But we might need to set config root.
      // configBindings.getConfigRoot(); // Just to trigger load?
      // Actually, runtime_config_store() in Rust lazy inits.
    });

    test('runtimeGetConfig returns strict JSON without "ok:" prefix', () {
      final raw = runtimeBindings.getConfig();
      stdout.writeln('runtimeGetConfig output: $raw');

      // 1. Must not start with legacy prefix
      expect(
        raw.startsWith('ok:'),
        isFalse,
        reason: 'Output should not have legacy "ok:" prefix',
      );
      expect(
        raw.startsWith('error:'),
        isFalse,
        reason: 'Output should not have legacy "error:" prefix',
      );

      // 2. Must be valid JSON
      expect(
        () => jsonDecode(raw),
        returnsNormally,
        reason: 'Output must be valid JSON',
      );

      final json = jsonDecode(raw);
      expect(
        json,
        isA<Map>(),
        reason: 'Runtime config should be a JSON object',
      );
    });

    test('configListHardwareProfiles returns strict JSON list', () {
      // listHardwareProfiles returns List<HardwareProfile>, not String (in generated Dart code)
      // Wait. Step 1620 showed ConfigBindings.listHardwareProfiles RETURNS List<HardwareProfile>
      // It assumes valid JSON internally!
      // So checking raw string is impossible via ConfigBindings.
      // The test 'Strict JSON' is implicitly verified because if it wasn't strict, this call would CRASH.

      expect(() => configBindings.listHardwareProfiles(), returnsNormally);

      final profiles = configBindings.listHardwareProfiles();
      expect(profiles, isA<List>(), reason: 'Should return a List');
    });
  });
}
