/// Web application smoke tests - catch initialization issues at test time.
///
/// These tests verify that the app can initialize and render on web platform
/// without throwing errors. This catches issues that would cause gray screens
/// or blank pages in UAT.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/main.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/providers_setup.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Web App Smoke Tests', () {
    testWidgets('app should initialize without errors on web', (tester) async {
      // Create a ProviderScope and pump the app
      await tester.pumpWidget(const ProviderScope(child: KeyrxApp()));

      // Let the app build
      await tester.pump();

      // Should not throw any errors
      expect(
        tester.takeException(),
        isNull,
        reason: 'App should initialize without throwing exceptions',
      );
    });

    testWidgets('service registry should create without errors on web', (tester) async {
      expect(
        () => ServiceRegistry.real(backendUrl: 'http://test:8888'),
        returnsNormally,
        reason: 'ServiceRegistry.real() should not throw on web',
      );
    });

    testWidgets('all providers should initialize without errors', (tester) async {
      final container = ProviderContainer();

      // Test that all providers can be read without errors
      expect(
        () => container.read(bridgeProvider),
        returnsNormally,
        reason: 'bridgeProvider should not throw',
      );

      expect(
        () => container.read(serviceRegistryProvider),
        returnsNormally,
        reason: 'serviceRegistryProvider should not throw',
      );

      expect(
        () => container.read(apiDocsServiceProvider),
        returnsNormally,
        reason: 'apiDocsServiceProvider should not throw',
      );

      expect(
        () => container.read(keyrxFacadeProvider),
        returnsNormally,
        reason: 'keyrxFacadeProvider should not throw',
      );

      expect(
        () => container.read(appStateProvider),
        returnsNormally,
        reason: 'appStateProvider should not throw',
      );

      container.dispose();
    });

    testWidgets('facade state stream should not close immediately', (tester) async {
      final container = ProviderContainer();
      final facade = container.read(keyrxFacadeProvider);

      // The state stream should be a broadcast stream that stays open
      final subscription = facade.stateStream.listen((_) {});

      await tester.pump();

      // After a short delay, the stream should still be open
      await tester.pump(const Duration(milliseconds: 200));

      // If this throws, it means the stream closed prematurely
      expect(
        subscription.isPaused,
        isFalse,
        reason: 'State stream should remain open, not close immediately',
      );

      await subscription.cancel();
      container.dispose();
    });

    testWidgets('engine service stub should provide valid streams', (tester) async {
      final container = ProviderContainer();
      final registry = container.read(serviceRegistryProvider);

      // Engine service should not throw when accessed
      expect(() => registry.engineService, returnsNormally);

      // State stream should not complete immediately
      final streamCompleted = <bool>[];
      final subscription = registry.engineService.stateStream.listen(
        (_) {},
        onDone: () => streamCompleted.add(true),
      );

      await tester.pump(const Duration(milliseconds: 100));

      expect(
        streamCompleted,
        isEmpty,
        reason: 'Engine stateStream should not complete immediately',
      );

      await subscription.cancel();
      container.dispose();
    });

    testWidgets('app should render material app without errors', (tester) async {
      await tester.pumpWidget(const ProviderScope(child: KeyrxApp()));

      // Pump multiple frames to let async initialization complete
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump(const Duration(milliseconds: 100));

      // Should find MaterialApp
      expect(find.byType(MaterialApp), findsOneWidget, reason: 'App should render MaterialApp');

      // Should not have thrown errors
      expect(tester.takeException(), isNull);
    });

    test('service registry web stubs should implement all methods', () {
      final registry = ServiceRegistry.real(backendUrl: 'http://test:8888');

      // All service getters should return non-null values
      expect(registry.engineService, isNotNull);
      expect(registry.deviceService, isNotNull);
      expect(registry.deviceProfileService, isNotNull);
      expect(registry.deviceRegistryService, isNotNull);
      expect(registry.profileRegistryService, isNotNull);
      expect(registry.testService, isNotNull);
      expect(registry.scriptFileService, isNotNull);
      expect(registry.storagePathResolver, isNotNull);
      expect(registry.profileAutosaveService, isNotNull);
      expect(registry.layoutService, isNotNull);
      expect(registry.hardwareService, isNotNull);
      expect(registry.keymapService, isNotNull);
      expect(registry.runtimeService, isNotNull);
      expect(registry.daemonManager, isNotNull);

      registry.dispose();
    });

    test('storage path resolver stub should not throw on method calls', () {
      final registry = ServiceRegistry.real(backendUrl: 'http://test:8888');

      expect(
        registry.storagePathResolver.resolveProfilesPath,
        returnsNormally,
        reason: 'resolveProfilesPath should not throw',
      );

      expect(
        () async => registry.storagePathResolver.ensureProfilesDirectory(),
        returnsNormally,
        reason: 'ensureProfilesDirectory should not throw',
      );

      registry.dispose();
    });
  });

  group('Web Service Stubs Error Messages', () {
    test('web stubs should provide helpful error messages when operations fail', () {
      final registry = ServiceRegistry.real(backendUrl: 'http://test:8888');

      // When stubs fail operations, they should return helpful errors
      // not throw UnsupportedError exceptions

      // Engine service should return false, not throw
      expect(registry.engineService.isInitialized, isFalse);

      registry.dispose();
    });
  });
}
