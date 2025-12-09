// Widget tests for console page styling.
//
// Tests ok: prefix shows green, error: prefix shows red,
// and quick action button appears for initialization errors.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/console.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/hardware_service.dart';
import 'package:keyrx_ui/services/keymap_service.dart';
import 'package:keyrx_ui/services/layout_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';
import 'package:provider/provider.dart';

import 'helpers/fake_services.dart';

class _FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController.broadcast();

  Future<ConsoleEvalResult> Function(String command)? onEval;
  Future<bool> Function()? onInitialize;

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async =>
      onInitialize != null ? await onInitialize!() : true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async => onEval != null
      ? await onEval!(command)
      : ConsoleEvalResult(success: true, output: 'ok: $command');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }

  @override
  Future<void> stop() async {
    // No-op
  }
}

Widget _buildTestWidget({required _FakeEngineService engine}) {
  final registry = ServiceRegistry.withOverrides(
    engineService: engine,
    testService: FakeTestService(),
    scriptFileService: FakeScriptFileService(),
    deviceService: FakeDeviceService(),
    bridge: FakeBridge(),
    apiDocsService: FakeApiDocsService(),
    deviceProfileService: FakeDeviceProfileService(),
    deviceRegistryService: FakeDeviceRegistryService(),
    profileRegistryService: FakeProfileRegistryService(),
    runtimeService: FakeRuntimeService(),
    errorTranslator: FakeErrorTranslator(),
    mappingRepository: MappingRepository(),
    storagePathResolver: const StoragePathResolver(),
    layoutService: LayoutService(bridge: FakeBridge()),
    hardwareService: HardwareService(bridge: FakeBridge()),
    keymapService: KeymapService(bridge: FakeBridge()),
  );
  final facade = KeyrxFacade.real(registry);

  return MultiProvider(
    providers: [Provider<KeyrxFacade>.value(value: facade)],
    child: MaterialApp(home: ConsolePage()),
  );
}

void main() {
  group('OK response styling', () {
    testWidgets('ok: prefix displays OK badge', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async => const ConsoleEvalResult(
          success: true,
          output: 'ok: command executed',
        );

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'test');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('OK'), findsWidgets);
    });

    testWidgets('ok: response shows check icon', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async =>
            const ConsoleEvalResult(success: true, output: 'ok: success');

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'cmd');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.check_circle_outline), findsWidgets);
    });

    testWidgets('ok: prefix is stripped from display text', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async =>
            const ConsoleEvalResult(success: true, output: 'ok: result value');

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'test');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      // The stripped text should be displayed
      expect(find.textContaining('result value'), findsOneWidget);
    });
  });

  group('Error response styling', () {
    testWidgets('error: prefix displays ERROR badge', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async =>
            const ConsoleEvalResult(success: false, output: 'error: failed');

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'bad');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('ERROR'), findsWidgets);
    });

    testWidgets('error: response shows warning icon', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async =>
            const ConsoleEvalResult(success: false, output: 'error: oops');

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'fail');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.warning_amber_rounded), findsWidgets);
    });

    testWidgets('error: prefix is stripped from display text', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async => const ConsoleEvalResult(
          success: false,
          output: 'error: something went wrong',
        );

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'test');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.textContaining('something went wrong'), findsOneWidget);
    });

    testWidgets('isError flag triggers error styling', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async => const ConsoleEvalResult(
          success: false,
          output: 'plain error message',
        );

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'err');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('ERROR'), findsWidgets);
    });
  });

  group('Quick action button', () {
    testWidgets('shows Initialize Engine button for not initialized error', (
      tester,
    ) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async => const ConsoleEvalResult(
          success: false,
          output: 'error: engine not initialized',
        );

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'status');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('Initialize Engine'), findsOneWidget);
      expect(find.byIcon(Icons.power_settings_new), findsOneWidget);
    });

    testWidgets('Initialize Engine button calls engine.initialize()', (
      tester,
    ) async {
      var initializeCalled = false;
      final fakeEngine = _FakeEngineService();
      fakeEngine.onEval = (_) async => const ConsoleEvalResult(
        success: false,
        output: 'error: not initialized please init',
      );
      fakeEngine.onInitialize = () async {
        initializeCalled = true;
        return true;
      };

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'check');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Initialize Engine'));
      await tester.pumpAndSettle();

      expect(initializeCalled, isTrue);
    });

    testWidgets('successful initialization shows ok message', (tester) async {
      final fakeEngine = _FakeEngineService();
      fakeEngine.onEval = (_) async => const ConsoleEvalResult(
        success: false,
        output: 'error: not initialized',
      );
      fakeEngine.onInitialize = () async => true;

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'test');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Initialize Engine'));
      await tester.pumpAndSettle();

      expect(find.textContaining('Engine initialized'), findsOneWidget);
    });

    testWidgets('no quick action button for other errors', (tester) async {
      final fakeEngine = _FakeEngineService()
        ..onEval = (_) async => const ConsoleEvalResult(
          success: false,
          output: 'error: syntax error in script',
        );

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'bad code');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('Initialize Engine'), findsNothing);
    });
  });

  group('Command input styling', () {
    testWidgets('input command shows CMD badge', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'my_command');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.text('CMD'), findsOneWidget);
    });

    testWidgets('input command shows chevron icon', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'command');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.chevron_right), findsOneWidget);
    });
  });

  group('Console structure', () {
    testWidgets('has Rhai Console title', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      expect(find.text('Rhai Console'), findsOneWidget);
    });

    testWidgets('has clear button in app bar', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.clear_all), findsOneWidget);
    });

    testWidgets('clear button removes history', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      // Add some history
      await tester.enterText(find.byType(TextField), 'first');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(find.textContaining('first'), findsWidgets);

      // Clear history
      await tester.tap(find.byIcon(Icons.clear_all));
      await tester.pumpAndSettle();

      expect(find.textContaining('first'), findsNothing);
    });

    testWidgets('shows input prompt', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      expect(find.text('> '), findsOneWidget);
    });

    testWidgets('has text input field', (tester) async {
      final fakeEngine = _FakeEngineService();

      await tester.pumpWidget(_buildTestWidget(engine: fakeEngine));
      await tester.pumpAndSettle();

      expect(find.byType(TextField), findsOneWidget);
    });
  });
}
