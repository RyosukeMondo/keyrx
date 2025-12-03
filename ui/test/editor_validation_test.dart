import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/editor_page.dart';
import 'package:keyrx_ui/pages/editor_widgets.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:provider/provider.dart';

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
        const KeyMapping(
          from: 'Q',
          type: KeyActionType.block,
        ),
      ];
      final combos = [
        const ComboMapping(keys: ['A', 'S'], output: 'Ctrl'),
      ];

      final script =
          ScriptGenerator.build(mappings: mappings, combos: combos);

      expect(script, contains('remap("CapsLock", "Escape");'));
      expect(script, contains('tap_hold("CapsLock", tap: "Escape", hold: "Ctrl");'));
      expect(script, contains('layer("navigation", "CapsLock", "Escape");'));
      expect(script, contains('block("Q");'));
      expect(script, contains('combo(["A", "S"], "Ctrl");'));
    });
  });

  group('EditorPage', () {
    testWidgets('uses fetched key registry for validation and badges', (tester) async {
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
      addTearDown(() => engine.dispose());
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: engine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: EditorPage(engineService: engine)),
        ),
      );

      await tester.pumpAndSettle();

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
      await tester.pumpAndSettle();

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
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(
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

class _FakeAudioService implements AudioService {
  @override
  AudioState get state => AudioState.idle;

  @override
  Stream<ClassificationResult> get classificationStream => const Stream.empty();

  @override
  Future<AudioOperationResult> start({required int bpm}) async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> stop() async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> setBpm(int bpm) async =>
      const AudioOperationResult(success: true);

  @override
  Future<void> dispose() async {}
}

class _FakePermissionService implements PermissionService {
  @override
  Future<PermissionResult> checkMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);

  @override
  Future<PermissionResult> requestMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);
}

class _FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'err', body: 'error');
}
