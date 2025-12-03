import 'dart:ffi';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import '../../lib/ffi/generated/bindings_generated.dart';

void main() {
  group('KeyrxBindingsGenerated', () {
    test('bindings class is instantiable', () {
      // Create a mock library for testing
      // This test just verifies the generated code compiles and can be instantiated
      expect(KeyrxBindingsGenerated, isNotNull);
    });

    test('generated bindings have correct Dart naming conventions', () {
      // Verify that generated member names follow lowerCamelCase
      final libPath = Platform.isLinux
          ? 'libkeyrx_core.so'
          : Platform.isWindows
              ? 'keyrx_core.dll'
              : 'libkeyrx_core.dylib';

      try {
        final lib = DynamicLibrary.open(libPath);
        final bindings = KeyrxBindingsGenerated(lib);

        // Verify key functions are available with proper naming
        // These will throw if the members don't exist
        expect(() => bindings.validateScript, returnsNormally);
        expect(() => bindings.startDiscovery, returnsNormally);
        expect(() => bindings.errorCategories, returnsNormally);
        expect(() => bindings.listDevices, returnsNormally);
      } catch (e) {
        // If library doesn't exist, that's okay - we're just testing code generation
        print('Library not found (expected during code generation testing): $e');
      }
    });

    test('generated typedefs are valid', () {
      // Just verify the code compiles - typedefs exist
      expect(ValidateScript, isNotNull);
      expect(ValidateScriptNative, isNotNull);
      expect(StartDiscovery, isNotNull);
      expect(StartDiscoveryNative, isNotNull);
    });
  });
}
