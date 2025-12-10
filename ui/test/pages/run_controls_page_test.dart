import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/pages/run_controls_page.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/services/layout_service.dart';
import 'package:keyrx_ui/services/hardware_service.dart';
import 'package:keyrx_ui/services/keymap_service.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:provider/provider.dart';

import '../helpers/fake_services.dart';

void main() {
  group('RunControlsPage (Dashboard)', () {
    late FakeEngineService fakeEngineService;
    late FakeDeviceService fakeDeviceService;
    late FakeDeviceRegistryService fakeDeviceRegistryService;
    late KeyrxFacade facade;

    setUp(() {
      fakeEngineService = FakeEngineService();
      fakeDeviceService = FakeDeviceService();
      fakeDeviceRegistryService = FakeDeviceRegistryService();

      final registry = ServiceRegistry.withOverrides(
        engineService: fakeEngineService,
        deviceService: fakeDeviceService,
        scriptFileService: FakeScriptFileService(),
        bridge: FakeBridge(),
        testService: FakeTestService(),
        errorTranslator: FakeErrorTranslator(),
        mappingRepository: MappingRepository(),
        deviceProfileService: FakeDeviceProfileService(),
        deviceRegistryService: fakeDeviceRegistryService,
        profileRegistryService: FakeProfileRegistryService(),
        runtimeService: FakeRuntimeService(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: const StoragePathResolver(),
        layoutService: LayoutService(bridge: FakeBridge()),
        hardwareService: HardwareService(bridge: FakeBridge()),
        keymapService: KeymapService(bridge: FakeBridge()),
      );
      facade = KeyrxFacade.real(registry);
    });

    Widget buildSubject() {
      return MultiProvider(
        providers: [
          Provider<KeyrxFacade>.value(value: facade),
          ChangeNotifierProvider(
            create: (_) => AppState(
              engineService: fakeEngineService,
              errorTranslator: FakeErrorTranslator(),
            ),
          ),
        ],
        child: const MaterialApp(home: RunControlsPage()),
      );
    }

    testWidgets('shows device list and control buttons', (tester) async {
      await tester.pumpWidget(buildSubject());
      await tester.pumpAndSettle();

      // Initial empty state
      expect(find.text('Devices'), findsOneWidget);
      expect(find.textContaining('No devices found'), findsOneWidget);

      expect(find.byIcon(Icons.play_arrow), findsWidgets); // Start/Scan
      expect(find.byType(TabBar), findsOneWidget);
      expect(find.text('Monitor'), findsOneWidget);
      expect(find.text('Controls'), findsOneWidget);
    });

    /*
    testWidgets('switching tabs changes monitoring view', (tester) async {
      await tester.pumpWidget(buildSubject());

      expect(find.text('Status'), findsOneWidget); // Tab 1 content

      await tester.tap(find.byIcon(Icons.monitor_heart));
      await tester.pump();
      await tester.pump(const Duration(seconds: 1)); // Wait for animation
      await tester.pumpAndSettle();

      // Should show MonitorView content
      expect(find.text('INPUT'), findsOneWidget);
      expect(find.text('OUTPUT'), findsOneWidget);
      expect(find.text('TIME'), findsOneWidget);
    });
    */
  });
}
