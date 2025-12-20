/// Shared widget testing harness with Riverpod + Provider wiring.
///
/// Provides a consistent shell for widget tests that need app-level
/// dependencies without touching real FFI or disk state. Use
/// [WidgetTestHarness] to wrap widgets in a `ProviderScope`, bridge
/// legacy `provider` adapters, and inject fakes via overrides.
library;

import 'dart:io';

import 'package:core_logic/core_logic.dart' as core;
import 'package:core_logic/core_logic.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/repositories/mapping_repository_ui.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/hardware_service_ui.dart';
import 'package:keyrx_ui/services/layout_service_ui.dart';
import 'package:keyrx_ui/services/profile_autosave_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:keyrx_ui/state/providers.dart';
import 'package:keyrx_ui/state/providers_setup.dart';
import 'package:provider/provider.dart';
import 'package:provider/single_child_widget.dart';

import '../mocks/mock_keyrx_facade.dart';
import 'fake_services.dart';

/// Harness that builds a minimal application shell for widget tests.
class WidgetTestHarness {
  factory WidgetTestHarness({
    ServiceRegistry? registry,
    KeyrxFacade? facade,
    FakeBridge? bridge,
    AppState? appState,
    ThemeData? theme,
    List<Override> overrides = const [],
    List<SingleChildWidget>? providerOverrides,
    bool wrapWithScaffold = true,
  }) {
    final effectiveBridge = bridge ?? FakeBridge();
    final effectiveRegistry = registry ?? _buildRegistry(effectiveBridge);
    final effectiveFacade = facade ?? MockKeyrxFacade.withDefaults();
    final effectiveAppState =
        appState ??
        AppState(
          engineService: effectiveRegistry.engineService,
          errorTranslator: effectiveRegistry.errorTranslator,
        );
    final providers =
        providerOverrides ??
        createProviders(
          registry: effectiveRegistry,
          facade: effectiveFacade,
          apiDocsService: effectiveRegistry.apiDocsService,
          appState: effectiveAppState,
        );
    final mergedOverrides = [
      serviceRegistryProvider.overrideWithValue(effectiveRegistry),
      keyrxFacadeProvider.overrideWithValue(effectiveFacade),
      apiDocsServiceProvider.overrideWithValue(
        effectiveRegistry.apiDocsService,
      ),
      appStateProvider.overrideWith((ref) => effectiveAppState),
      ...overrides,
    ];

    return WidgetTestHarness._(
      registry: effectiveRegistry,
      facade: effectiveFacade,
      bridge: effectiveBridge,
      appState: effectiveAppState,
      theme: theme ?? ThemeData.dark(useMaterial3: true),
      providerAdapters: providers,
      overrides: mergedOverrides,
      wrapWithScaffold: wrapWithScaffold,
    );
  }
  WidgetTestHarness._({
    required this.registry,
    required this.facade,
    required this.bridge,
    required this.appState,
    required this.theme,
    required this.providerAdapters,
    required this.overrides,
    required this.wrapWithScaffold,
  });

  final ServiceRegistry registry;
  final KeyrxFacade facade;
  final FakeBridge bridge;
  final AppState appState;
  final ThemeData theme;
  final bool wrapWithScaffold;
  final List<Override> overrides;
  final List<SingleChildWidget> providerAdapters;

  /// Build the full widget tree with ProviderScope + MultiProvider shell.
  Widget build(Widget child, {bool? withScaffold}) {
    final body = (withScaffold ?? wrapWithScaffold)
        ? Scaffold(body: child)
        : child;

    return ProviderScope(
      overrides: overrides,
      child: MultiProvider(
        providers: providerAdapters,
        child: MaterialApp(theme: theme, home: body),
      ),
    );
  }

  /// Pump a widget wrapped with the harness shell.
  Future<void> pumpWidget(
    WidgetTester tester,
    Widget widget, {
    bool settle = true,
    bool? withScaffold,
  }) async {
    await tester.pumpWidget(build(widget, withScaffold: withScaffold));
    await tester.pump();
    if (settle) {
      await tester.pumpAndSettle();
    }
  }

  /// Pump a widget intended to behave like a full page.
  Future<void> pumpPage(
    WidgetTester tester,
    Widget widget, {
    bool settle = true,
  }) => pumpWidget(tester, widget, settle: settle, withScaffold: true);

  static ServiceRegistry _buildRegistry(FakeBridge bridge) {
    final profileRegistry = FakeProfileRegistryService();
    final storageResolver = StoragePathResolver(
      environment: {'HOME': Directory.systemTemp.path},
      bridge: bridge,
    );

    return ServiceRegistry.withOverrides(
      errorTranslator: FakeErrorTranslator(),
      engineService: FakeEngineService(),
      mappingRepository: MappingRepositoryUI(core.MappingRepository()),
      deviceService: FakeDeviceService(),
      deviceProfileService: FakeDeviceProfileService(),
      deviceRegistryService: FakeDeviceRegistryService(),
      profileRegistryService: profileRegistry,
      scriptFileService: FakeScriptFileService(),
      testService: FakeTestService(),
      bridge: bridge,
      apiDocsService: FakeApiDocsService(),
      storagePathResolver: storageResolver,
      profileAutosaveService: ProfileAutosaveService(
        profileRegistryService: profileRegistry,
        storagePathResolver: storageResolver,
        delayFn: (_) async {},
      ),
      layoutService: LayoutServiceUI(core.LayoutServiceFFI(bridge: bridge)),
      hardwareService: HardwareServiceUI(
        core.HardwareServiceFFI(bridge: bridge),
      ),
      keymapService: KeymapServiceFFI(bridge: bridge),
      runtimeService: FakeRuntimeService(),
      metricsService: FakeMetricsService(),
      benchmarkService: FakeBenchmarkService(),
      daemonManager: MockDaemonManager(),
    );
  }
}

/// Common tester conveniences for widget tests.
extension WidgetTesterX on WidgetTester {
  /// Tap a finder and settle a single frame.
  Future<void> tapAndSettle(
    Finder finder, {
    Duration settleDuration = const Duration(milliseconds: 50),
  }) async {
    await tap(finder);
    await pumpAndSettle(settleDuration);
  }

  /// Enter text and settle quickly.
  Future<void> enterTextAndSettle(
    Finder finder,
    String text, {
    Duration settleDuration = const Duration(milliseconds: 50),
  }) async {
    await enterText(finder, text);
    await pumpAndSettle(settleDuration);
  }

  /// Expect exactly one widget and return the finder for chaining.
  Finder expectOne(Finder finder) {
    expect(finder, findsOneWidget);
    return finder;
  }

  /// Expect no widgets and return the finder for chaining.
  Finder expectNone(Finder finder) {
    expect(finder, findsNothing);
    return finder;
  }
}
