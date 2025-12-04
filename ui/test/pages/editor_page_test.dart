import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/key_mapping.dart';
import 'package:keyrx_ui/models/validation.dart' as validation_models;
import 'package:keyrx_ui/pages/editor_page.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/facade/result.dart';
import 'package:keyrx_ui/services/mapping_validator.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/app_state.dart';

import '../mocks/mock_keyrx_facade.dart';

/// Mock service registry for tests
class MockServiceRegistry extends Mock implements ServiceRegistry {}

/// Mock engine service for direct access tests
class MockEngineService extends Mock implements EngineService {}

/// Mock bridge for validation tests
class MockBridge extends Mock implements KeyrxBridge {}

class _RecordingTranslator implements ErrorTranslator {
  const _RecordingTranslator(this.message);

  final UserMessage message;

  @override
  UserMessage translate(Object error) => message;
}

void main() {
  late MockKeyrxFacade mockFacade;
  late MappingRepository mappingRepository;
  late MockServiceRegistry mockServices;
  late MockEngineService mockEngine;
  late MockBridge mockBridge;

  setUpAll(() {
    // Register fallback values for mocktail
    registerFallbackValue(const validation_models.ValidationOptions(includeCoverage: true));
    // Register facade fallback values
    registerKeyrxFacadeFallbackValues();
  });

  setUp(() {
    // Create mocks
    mockFacade = MockKeyrxFacade.withDefaults();
    mockServices = MockServiceRegistry();
    mockEngine = MockEngineService();
    mockBridge = MockBridge();
    mappingRepository = MappingRepository();

    // Setup facade to return mock services
    when(() => mockFacade.services).thenReturn(mockServices);
    when(() => mockServices.engineService).thenReturn(mockEngine);
    when(() => mockServices.bridge).thenReturn(mockBridge);

    // Setup default engine behavior
    when(() => mockEngine.isInitialized).thenReturn(true);
    when(() => mockEngine.fetchKeyRegistry()).thenAnswer(
      (_) async => const KeyRegistryResult(entries: []),
    );

    // Setup default bridge behavior for validation
    when(() => mockBridge.validateScript(
      any(),
      any(),
    )).thenReturn(
      const validation_models.ValidationResult(
        isValid: true,
        errors: [],
        warnings: [],
        coverage: null,
      ),
    );
  });

  tearDown(() {
    mappingRepository.dispose();
  });

  Widget buildTestWidget({
    MappingValidator validator = const MappingValidator(),
  }) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider<AppState>(
          create: (_) => AppState(
            engineService: mockEngine,
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
          facade: mockFacade,
          mappingRepository: mappingRepository,
          validator: validator,
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

    // Setup engine to accept loadScript
    when(() => mockEngine.loadScript(any()))
        .thenAnswer((_) async => true);

    mappingRepository.setMapping(
      'A',
      const KeyMapping(from: 'A', type: KeyActionType.remap, to: 'b'),
    );

    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.save));
    await tester.pumpAndSettle();

    // Verify facade's saveScript was called
    verify(() => mockFacade.saveScript(any(), any())).called(1);
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

    // Override facade to return error on save
    when(() => mockFacade.saveScript(any(), any())).thenAnswer(
      (_) async => Result.err(
        FacadeError.fileError('/path/to/file', 'Permission denied'),
      ),
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
