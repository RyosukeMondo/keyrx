import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/key_mapping.dart';
import 'package:keyrx_ui/pages/editor_page.dart';
import 'package:keyrx_ui/services/key_mappings_util.dart';
import 'package:keyrx_ui/services/script_generator.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:provider/provider.dart';

import 'helpers/fake_services.dart';

void main() {
  group('KeyMappings', () {
    test('recognizes known keys case-insensitively', () {
      expect(KeyMappings.isKnownKey('Esc'), isTrue);
      expect(KeyMappings.isKnownKey('space'), isTrue);
      expect(KeyMappings.isKnownKey('UnknownKey'), isFalse);
      expect(KeyMappings.isKnownKey(''), isFalse);
    });
  });

  group('ScriptGenerator', () {
    test('renders remap, tap-hold, and combos', () {
      final mappings = [
        const KeyMapping(
          from: 'CapsLock',
          type: KeyActionType.remap,
          to: 'Escape',
          tapHoldTap: 'Escape',
          tapHoldHold: 'Ctrl',
          layer: 'navigation',
        ),
        const KeyMapping(from: 'Q', type: KeyActionType.block),
      ];
      final combos = [
        const ComboMapping(keys: ['A', 'S'], output: 'Ctrl'),
      ];

      final script = ScriptGenerator.build(mappings: mappings, combos: combos);

      expect(script, contains('remap("CapsLock", "Escape");'));
      expect(
        script,
        contains('tap_hold("CapsLock", tap: "Escape", hold: "Ctrl");'),
      );
      expect(script, contains('layer("navigation", "CapsLock", "Escape");'));
      expect(script, contains('block("Q");'));
      expect(script, contains('combo(["A", "S"], "Ctrl");'));
    });
  });

  group('EditorPage', () {
    testWidgets('uses fetched key registry for validation and badges', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(1400, 1800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      final originalAllowedKeys = KeyMappings.allowedKeys;
      addTearDown(() => KeyMappings.allowedKeys = originalAllowedKeys);
      KeyMappings.allowedKeys = [];

      final engine = _FakeEngineService();
      final bridge = FakeBridge();
      addTearDown(() async {
        await engine.dispose();
        await bridge.dispose();
      });
      final registry = ServiceRegistry.withOverrides(
        errorTranslator: _FakeErrorTranslator(),
        engineService: engine,
        deviceService: FakeDeviceService(),
        testService: FakeTestService(),
        bridge: bridge,
        mappingRepository: MappingRepository(),
        scriptFileService: FakeScriptFileService(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: const StoragePathResolver(),
      );

      final facade = KeyrxFacade.real(registry);
      addTearDown(() => facade.dispose());

      await tester.pumpWidget(
        MultiProvider(
          providers: [
            Provider<ServiceRegistry>.value(value: registry),
            Provider<KeyrxFacade>.value(value: facade),
            ChangeNotifierProvider<AppState>(
              create: (_) => AppState(
                engineService: engine,
                errorTranslator: registry.errorTranslator,
              ),
            ),
          ],
          child: MaterialApp(
            home: EditorPage(
              facade: facade,
              mappingRepository: registry.mappingRepository,
            ),
          ),
        ),
      );

      // Use pump with duration instead of pumpAndSettle to avoid timeout
      // from validation debounce timer
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));

      expect(find.textContaining('Loaded 3 canonical keys'), findsOneWidget);

      await tester.tap(find.text('Q'));
      await tester.pump();

      final outputField = find.byWidgetPredicate(
        (widget) =>
            widget is TextField &&
            widget.decoration?.labelText == 'Remap to key',
      );
      await tester.enterText(outputField, 'Space');
      await tester.pump();

      await tester.tap(find.text('Apply'));
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));

      expect(find.text('Q → Space'), findsOneWidget);
      expect(find.text('from Q'), findsOneWidget);
      expect(find.text('to Space'), findsOneWidget);
    });
  });
}

class _FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController.broadcast();

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async => const KeyRegistryResult(
    entries: [
      KeyRegistryEntry(name: 'Space', aliases: ['Q', 'Enter']),
    ],
  );

  void emit(EngineSnapshot snapshot) {
    _stateController.add(snapshot);
  }

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class _FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'err', body: 'error');
}
