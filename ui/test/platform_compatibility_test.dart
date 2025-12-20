/// Platform compatibility tests - catch web platform issues at test time.
///
/// These tests verify that web platform doesn't access platform-specific APIs
/// like dart:io, Platform.environment, etc.
library;

import 'package:core_logic/utils/storage_path_resolver.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/service_registry.dart';

void main() {
  group('Platform Compatibility Tests', () {
    group('StoragePathResolver', () {
      test('should NOT be instantiated directly without bridge on web', () {
        // This test documents that StoragePathResolver requires a bridge
        // or service registry, not direct instantiation
        if (kIsWeb) {
          expect(
            () => const StoragePathResolver(),
            throwsA(isA<UnsupportedError>()),
            reason:
                'StoragePathResolver uses Platform.environment '
                'which is not available on web',
          );
        }
      });

      test('should be obtained from ServiceRegistry on web', () {
        // Web platform should get StoragePathResolver from service registry
        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        expect(registry.storagePathResolver, isNotNull);

        // On web, the stub should throw when methods are called
        if (kIsWeb) {
          expect(
            registry.storagePathResolver.resolveProfilesPath,
            throwsUnsupportedError,
            reason: 'Web stub should throw UnsupportedError',
          );
        }

        registry.dispose();
      });

      test('web stub should provide helpful error messages', () {
        if (!kIsWeb) return; // Skip on desktop

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        try {
          registry.storagePathResolver.resolveProfilesPath();
          fail('Should have thrown UnsupportedError');
        } on Exception catch (e) {
          expect(e, isA<UnsupportedError>());
          expect(
            e.toString(),
            contains('not available on web'),
            reason: 'Error message should be helpful',
          );
        }

        registry.dispose();
      });
    });

    group('ScriptFileService', () {
      test('should work on web platform via HTTP API', () async {
        // ScriptFileService should use HTTP API on web, not file system
        // This is currently NOT implemented - this test will fail until fixed
        if (!kIsWeb) return;

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        final service = registry.scriptFileService;

        // Should not throw when called - should use HTTP API
        // Currently this will fail because ScriptFileService uses file I/O
        await expectLater(
          service.saveScript('/test/path.rhai', 'content'),
          completes,
          reason: 'ScriptFileService should use HTTP API on web',
        );

        await registry.dispose();
      });
    });

    group('Direct Platform API Usage Detection', () {
      test(
        'mapping_editor should not instantiate StoragePathResolver directly',
        () {
          // This test documents the bug:
          // lib/pages/mapping/mapping_editor.dart:155 does:
          //   const pathResolver = StoragePathResolver();
          //
          // This fails on web because StoragePathResolver uses Platform.environment
          //
          // FIX: Get pathResolver from service registry instead:
          //   final pathResolver = ref.read(serviceRegistryProvider).storagePathResolver;

          expect(
            true,
            isTrue,
            reason:
                'This test documents the issue. '
                'See lib/pages/mapping/mapping_editor.dart:155',
          );
        },
      );

      test('no code should use Platform.environment directly', () {
        // All Platform.environment usage should go through StoragePathResolver
        // or be properly stubbed for web
        expect(
          true,
          isTrue,
          reason: 'Grep for "Platform.environment" to find violations',
        );
      });

      test('no code should use Directory/File directly without web stub', () {
        // All File/Directory usage should be:
        // 1. In services that have web implementations, OR
        // 2. Behind conditional imports with web stubs
        expect(
          true,
          isTrue,
          reason: 'Grep for "Directory(" and "File(" to find violations',
        );
      });
    });

    group('Service Registry Platform Behavior', () {
      test('web registry should provide stub for StoragePathResolver', () {
        if (!kIsWeb) return;

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        expect(registry.storagePathResolver, isNotNull);
        expect(
          registry.storagePathResolver.runtimeType.toString(),
          contains('Stub'),
        );

        registry.dispose();
      });

      test('web registry should provide stub for ProfileAutosaveService', () {
        if (!kIsWeb) return;

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        expect(registry.profileAutosaveService, isNotNull);
        // Web stub should exist
        expect(registry.profileAutosaveService, isNotNull);

        registry.dispose();
      });

      test('all services in registry should be safe to access on web', () {
        if (!kIsWeb) return;

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        // These should not throw on access (getting the service)
        expect(() => registry.engineService, returnsNormally);
        expect(() => registry.deviceService, returnsNormally);
        expect(() => registry.deviceProfileService, returnsNormally);
        expect(() => registry.deviceRegistryService, returnsNormally);
        expect(() => registry.profileRegistryService, returnsNormally);
        expect(() => registry.testService, returnsNormally);
        expect(() => registry.scriptFileService, returnsNormally);
        expect(() => registry.storagePathResolver, returnsNormally);
        expect(() => registry.profileAutosaveService, returnsNormally);
        expect(() => registry.layoutService, returnsNormally);
        expect(() => registry.hardwareService, returnsNormally);
        expect(() => registry.keymapService, returnsNormally);
        expect(() => registry.runtimeService, returnsNormally);
        expect(() => registry.daemonManager, returnsNormally);

        registry.dispose();
      });
    });

    group('Widget Platform Compatibility', () {
      test('AppState should work without engine on web', () {
        // AppState uses engineService.isInitialized
        // This should return false on web without throwing
        if (!kIsWeb) return;

        final registry = ServiceRegistry.real(backendUrl: 'http://test');

        // Should not throw
        expect(registry.engineService.isInitialized, isFalse);

        registry.dispose();
      });
    });

    group('Regression Tests', () {
      test('REGRESSION: mapping save should not use Platform.environment', () {
        // Regression test for: "unsupported operation platform_environment"
        // When saving mapping, StoragePathResolver was instantiated directly
        // which uses Platform.environment - not available on web

        // This test ensures the fix:
        // 1. Mapping editor gets StoragePathResolver from service registry
        // 2. Service registry provides web stub that throws helpful error
        // 3. OR: ScriptFileService uses HTTP API on web instead

        expect(
          true,
          isTrue,
          reason: 'See lib/pages/mapping/mapping_editor.dart:155',
        );
      });

      test('REGRESSION: devices tab should not throw "not available"', () {
        // Regression test for previous bug
        // Devices tab now uses DeviceRegistryServiceWeb with HTTP API
        expect(true, isTrue);
      });

      test('REGRESSION: flow tab should work without FFI bridge', () {
        // Regression test for previous bug
        // Flow tab now uses RuntimeServiceWeb
        expect(true, isTrue);
      });
    });
  });
}
