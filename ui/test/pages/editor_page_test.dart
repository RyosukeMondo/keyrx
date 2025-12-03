import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/editor_page.dart';
import 'package:keyrx_ui/pages/editor_widgets.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/mapping_validator.dart';
import 'package:keyrx_ui/services/script_file_service.dart';
import 'package:keyrx_ui/state/app_state.dart';

import '../helpers/fake_services.dart';

class _FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController.broadcast();

  bool initializeResult = true;
  bool loadScriptResult = true;
  KeyRegistryResult registryResult = const KeyRegistryResult(entries: []);

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async => initializeResult;

  @override
  Future<bool> loadScript(String path) async => loadScriptResult;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      ConsoleEvalResult(success: true, output: 'ok: $command');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async => registryResult;

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class _FakeScriptFileService implements ScriptFileService {
  ScriptFileResult saveResult = const ScriptFileResult(success: true);
  String? lastSavedPath;
  String? lastSavedContent;

  @override
  Future<ScriptFileResult> saveScript(String path, String content) async {
    lastSavedPath = path;
    lastSavedContent = content;
    return saveResult;
  }

  @override
  Future<String?> loadScript(String path) async => null;
}

class _RecordingTranslator implements ErrorTranslator {
  const _RecordingTranslator(this.message);

  final UserMessage message;

  @override
  UserMessage translate(Object error) => message;
}

void main() {
  late _FakeEngineService fakeEngine;
  late MappingRepository mappingRepository;
  late _FakeScriptFileService fakeScriptFileService;
  late FakeBridge fakeBridge;

  setUp(() {
    fakeEngine = _FakeEngineService();
    mappingRepository = MappingRepository();
    fakeScriptFileService = _FakeScriptFileService();
    fakeBridge = FakeBridge();
  });

  tearDown(() async {
    await fakeEngine.dispose();
    mappingRepository.dispose();
    await fakeBridge.dispose();
  });

  Widget buildTestWidget({
    MappingValidator validator = const MappingValidator(),
  }) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider<AppState>(
          create: (_) => AppState(
            engineService: fakeEngine,
            errorTranslator: const _RecordingTranslator(
              UserMessage(title: 'Unused', body: 'Unused'),
            ),
          ),
        ),
        ChangeNotifierProvider<MappingRepository>.value(
          value: mappingRepository,
        ),
      ],
      child: MaterialApp(
        home: EditorPage(
          engineService: fakeEngine,
          mappingRepository: mappingRepository,
          validator: validator,
          scriptFileService: fakeScriptFileService,
          bridge: fakeBridge,
        ),
      ),
    );
  }

  testWidgets('displays initial UI with keyboard and config panel',
      (tester) async {
    // Set a larger surface size to prevent overflow
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    expect(find.text('Keymap Editor'), findsOneWidget);
    expect(find.text('Select a key to configure'), findsOneWidget);
    expect(find.byIcon(Icons.save), findsOneWidget);
    expect(find.byIcon(Icons.code), findsOneWidget);
  });

  testWidgets('selecting a key shows configuration panel', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Tap on the 'A' key
    await tester.tap(find.text('A'));
    await tester.pumpAndSettle();

    expect(find.text('Configuring: A'), findsOneWidget);
    expect(find.text('Remap'), findsWidgets);
  });

  testWidgets('invalid mapping shows error snackbar', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Select 'A' key
    await tester.tap(find.text('A'));
    await tester.pumpAndSettle();

    // Leave target empty and click Apply (should fail validation)
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();

    expect(find.text('Provide a target key for remap.'), findsOneWidget);
  });

  testWidgets('save script shows success message when mappings exist',
      (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    mappingRepository.setMapping(
      'A',
      const KeyMapping(from: 'A', type: KeyActionType.remap, to: 'b'),
    );

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.save));
    await tester.pumpAndSettle();

    expect(fakeScriptFileService.lastSavedPath, isNotNull);
    expect(
      fakeScriptFileService.lastSavedContent,
      contains('remap("A", "b")'),
    );
    expect(find.textContaining('Script saved'), findsOneWidget);
  });

  testWidgets('save script shows error when no mappings', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.save));
    await tester.pumpAndSettle();

    expect(
      find.text('Add at least one mapping before saving.'),
      findsOneWidget,
    );
  });

  testWidgets('save script shows error on file service failure',
      (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    fakeScriptFileService.saveResult = const ScriptFileResult(
      success: false,
      errorMessage: 'Permission denied',
    );

    mappingRepository.setMapping(
      'A',
      const KeyMapping(from: 'A', type: KeyActionType.remap, to: 'b'),
    );

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.save));
    await tester.pumpAndSettle();

    expect(find.textContaining('Permission denied'), findsOneWidget);
  });

  testWidgets('view script shows generated script dialog', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    mappingRepository.setMapping(
      'A',
      const KeyMapping(from: 'A', type: KeyActionType.remap, to: 'b'),
    );

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.code));
    await tester.pumpAndSettle();

    expect(find.text('Generated Script'), findsOneWidget);
    expect(find.textContaining('remap("A", "b")'), findsOneWidget);
  });

  testWidgets('adding combo validates keys correctly', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Find combo input fields
    final comboKeysField =
        find.widgetWithText(TextField, 'Keys (comma-separated)');
    final comboOutputField = find.widgetWithText(TextField, 'Output');

    // Enter invalid combo (single key)
    await tester.enterText(comboKeysField, 'a');
    await tester.enterText(comboOutputField, 'ctrl');
    await tester.pumpAndSettle();

    await tester.tap(find.text('Add Combo'));
    await tester.pumpAndSettle();

    expect(find.text('Provide at least 2 keys for a combo.'), findsOneWidget);
  });

  testWidgets('adding valid combo updates repository', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    final comboKeysField =
        find.widgetWithText(TextField, 'Keys (comma-separated)');
    final comboOutputField = find.widgetWithText(TextField, 'Output');

    await tester.enterText(comboKeysField, 'a,s');
    await tester.enterText(comboOutputField, 'ctrl');
    await tester.pumpAndSettle();

    await tester.tap(find.text('Add Combo'));
    await tester.pumpAndSettle();

    expect(mappingRepository.combos.length, equals(1));
    expect(mappingRepository.combos.first.keys, equals(['a', 's']));
    expect(mappingRepository.combos.first.output, equals('ctrl'));
  });

  testWidgets('key registry banner shows key icon', (tester) async {
    tester.view.physicalSize = const Size(1600, 1400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(buildTestWidget());
    // Pump once to start the fetch
    await tester.pump();

    // The banner should show the key icon
    expect(find.byIcon(Icons.key), findsOneWidget);
  });

}
