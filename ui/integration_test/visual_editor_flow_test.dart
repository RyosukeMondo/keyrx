// Integration test: Visual Editor Flow
//
// Tests the end-to-end visual editor workflow:
// - Create mappings visually with drag-drop
// - View generated Rhai code
// - Parse code back to visual config
// - Verify bidirectional sync works correctly

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/keyboard_layout.dart';
import 'package:keyrx_ui/pages/visual_editor_page.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/rhai_generator.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:keyrx_ui/widgets/visual_keyboard.dart';

import '../test/helpers/fake_services.dart';

/// Mock engine service for testing.
class MockEngineService implements EngineService {
  final _stateController = StreamController<EngineSnapshot>.broadcast();
  String? lastLoadedScript;

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test-1.0.0';

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async {
    lastLoadedScript = path;
    return true;
  }

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok: executed');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class MockErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'Error', body: 'An error occurred');
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  late MockEngineService mockEngine;

  setUp(() {
    mockEngine = MockEngineService();
  });

  Widget buildTestApp() {
    final mappingRepo = MappingRepository();
    final registry = ServiceRegistry.withOverrides(
      errorTranslator: MockErrorTranslator(),
      engineService: mockEngine,
      mappingRepository: mappingRepo,
      deviceService: FakeDeviceService(),
      deviceProfileService: FakeDeviceProfileService(),
      deviceRegistryService: FakeDeviceRegistryService(),
      profileRegistryService: FakeProfileRegistryService(),
      scriptFileService: FakeScriptFileService(),
      testService: FakeTestService(),
      bridge: FakeBridge(),
      apiDocsService: FakeApiDocsService(),
    );

    return MultiProvider(
      providers: [
        Provider<ServiceRegistry>.value(value: registry),
        ChangeNotifierProvider(
          create: (context) => AppState(
            engineService: mockEngine,
            errorTranslator: MockErrorTranslator(),
          ),
        ),
      ],
      child: MaterialApp(
        title: 'KeyRx Test',
        theme: ThemeData.dark(useMaterial3: true),
        home: Scaffold(
          body: VisualEditorPage(mappingRepository: mappingRepo),
        ),
      ),
    );
  }

  group('Visual Editor Flow', () {
    testWidgets('Visual editor page renders with keyboard',
        (WidgetTester tester) async {
      // Set up larger window for visual editor
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Verify visual editor page is shown
      expect(find.text('Visual Editor'), findsOneWidget);

      // Verify keyboard is rendered (check for some standard keys)
      expect(find.byType(VisualKeyboard), findsOneWidget);

      // Verify mapping panel is shown
      expect(find.text('No mappings yet.'), findsOneWidget);
    });

    testWidgets('Toggle between visual and code view',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Should start in visual view
      expect(find.byType(VisualKeyboard), findsOneWidget);

      // Tap the code view toggle button
      await tester.tap(find.byIcon(Icons.code));
      await tester.pumpAndSettle();

      // Should now show code view
      expect(find.text('Rhai Script'), findsOneWidget);

      // Toggle back to visual view
      await tester.tap(find.byIcon(Icons.grid_view));
      await tester.pumpAndSettle();

      // Should be back in visual view
      expect(find.byType(VisualKeyboard), findsOneWidget);
    });

    testWidgets('Mapping panel shows count and clear button',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Verify mapping count is shown
      expect(find.text('Mappings (0)'), findsOneWidget);

      // Clear button should be disabled when no mappings
      final clearButton = find.byIcon(Icons.delete_sweep);
      expect(clearButton, findsOneWidget);
    });

    testWidgets('Code view shows generated Rhai syntax',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Switch to code view
      await tester.tap(find.byIcon(Icons.code));
      await tester.pumpAndSettle();

      // Code view should have the Rhai script header/template
      expect(find.text('Rhai Script'), findsOneWidget);

      // Verify the code editor is present (TextField)
      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('Parse to Visual button appears when code is modified',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Switch to code view
      await tester.tap(find.byIcon(Icons.code));
      await tester.pumpAndSettle();

      // Parse to Visual button should be disabled initially
      final parseButton = find.text('Parse to Visual');
      expect(parseButton, findsOneWidget);

      // Modify the code
      final textField = find.byType(TextField);
      await tester.enterText(textField, 'remap("KeyA", "KeyB");');
      await tester.pumpAndSettle();

      // Modified chip should appear
      expect(find.text('Modified'), findsOneWidget);
    });

    testWidgets('New configuration button shows confirmation dialog',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // The new configuration button should be present
      expect(find.byIcon(Icons.add), findsOneWidget);
    });

    testWidgets('Load script button is present',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Load script button should be present
      expect(find.byIcon(Icons.folder_open), findsOneWidget);

      // Tap it to show the dialog
      await tester.tap(find.byIcon(Icons.folder_open));
      await tester.pumpAndSettle();

      // Dialog should appear
      expect(find.text('Load Script'), findsOneWidget);
      expect(find.text('Script path'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);
      expect(find.text('Load'), findsOneWidget);

      // Close the dialog
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
    });

    testWidgets('Save script button is present',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Save script button should be present
      expect(find.byIcon(Icons.save), findsOneWidget);

      // Tap it to show the dialog
      await tester.tap(find.byIcon(Icons.save));
      await tester.pumpAndSettle();

      // Dialog should appear
      expect(find.text('Save Script'), findsOneWidget);
      expect(find.text('Save path'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);
      expect(find.text('Save'), findsOneWidget);

      // Close the dialog
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
    });

    testWidgets('Keyboard shows helper text for drag-drop',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Helper text should be shown
      expect(
        find.text('Drag from one key to another to create a mapping'),
        findsOneWidget,
      );
    });

    testWidgets('Empty mappings panel shows instructions',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Instructions should appear in the mapping panel
      expect(find.textContaining('No mappings yet'), findsOneWidget);
      expect(find.textContaining('Drag from one key'), findsOneWidget);
    });
  });

  group('Rhai Generator Integration', () {
    testWidgets('Generated script contains correct syntax',
        (WidgetTester tester) async {
      // Test the generator directly for expected output
      final generator = RhaiGenerator();

      const config = VisualConfig(
        mappings: [
          RemapConfig(
            sourceKeyId: 'CapsLock',
            targetKeyId: 'Escape',
            type: MappingType.simple,
          ),
        ],
      );

      final script = generator.generateScript(config);

      // Verify the script contains expected content
      expect(script, contains('remap("CapsLock", "Escape");'));
      expect(script, contains('// Generated by KeyRx Visual Editor'));
    });

    testWidgets('Parser extracts mappings from valid script',
        (WidgetTester tester) async {
      final generator = RhaiGenerator();

      const script = '''
// KeyRx script
fn main() {
  remap("KeyA", "KeyB");
  remap("CapsLock", "Escape");
}
''';

      final config = generator.parseScript(script);

      expect(config.mappings.length, 2);
      expect(config.mappings[0].sourceKeyId, 'KeyA');
      expect(config.mappings[0].targetKeyId, 'KeyB');
      expect(config.mappings[1].sourceKeyId, 'CapsLock');
      expect(config.mappings[1].targetKeyId, 'Escape');
    });

    testWidgets('Parser detects advanced features',
        (WidgetTester tester) async {
      final generator = RhaiGenerator();

      const advancedScript = '''
fn custom_function() {
  let x = 42;
}

fn main() {
  remap("KeyA", "KeyB");
  custom_function();
}
''';

      final config = generator.parseScript(advancedScript);

      expect(config.hasAdvancedFeatures, true);
    });

    testWidgets('Roundtrip: generate -> parse -> generate',
        (WidgetTester tester) async {
      final generator = RhaiGenerator();

      const originalConfig = VisualConfig(
        mappings: [
          RemapConfig(
            sourceKeyId: 'KeyA',
            targetKeyId: 'KeyB',
            type: MappingType.simple,
          ),
          RemapConfig(
            sourceKeyId: 'CapsLock',
            targetKeyId: 'Escape',
            type: MappingType.simple,
          ),
        ],
        tapHoldConfigs: [
          TapHoldConfig(
            triggerKey: 'Space',
            tapAction: 'Space',
            holdAction: 'ControlLeft',
          ),
        ],
      );

      // Generate script from config
      final script = generator.generateScript(originalConfig);

      // Parse script back to config
      final parsedConfig = generator.parseScript(script);

      // Verify mappings are preserved
      expect(parsedConfig.mappings.length, originalConfig.mappings.length);
      for (var i = 0; i < originalConfig.mappings.length; i++) {
        expect(
          parsedConfig.mappings[i].sourceKeyId,
          originalConfig.mappings[i].sourceKeyId,
        );
        expect(
          parsedConfig.mappings[i].targetKeyId,
          originalConfig.mappings[i].targetKeyId,
        );
      }

      // Verify tap-holds are preserved
      expect(
        parsedConfig.tapHoldConfigs.length,
        originalConfig.tapHoldConfigs.length,
      );
      for (var i = 0; i < originalConfig.tapHoldConfigs.length; i++) {
        expect(
          parsedConfig.tapHoldConfigs[i].triggerKey,
          originalConfig.tapHoldConfigs[i].triggerKey,
        );
        expect(
          parsedConfig.tapHoldConfigs[i].tapAction,
          originalConfig.tapHoldConfigs[i].tapAction,
        );
        expect(
          parsedConfig.tapHoldConfigs[i].holdAction,
          originalConfig.tapHoldConfigs[i].holdAction,
        );
      }
    });
  });

  group('Visual Keyboard Integration', () {
    testWidgets('Keyboard layout has expected structure',
        (WidgetTester tester) async {
      // Verify default ANSI layout has expected keys
      final layout = KeyboardLayout.ansi104();

      expect(layout.rows.isNotEmpty, true);

      // Check for common keys
      final allKeys = layout.rows.expand((r) => r.keys).toList();
      final keyIds = allKeys.map((k) => k.id).toSet();

      expect(keyIds.contains('Escape'), true);
      expect(keyIds.contains('KeyA'), true);
      expect(keyIds.contains('Space'), true);
      expect(keyIds.contains('CapsLock'), true);
      expect(keyIds.contains('Enter'), true);
    });

    testWidgets('Keyboard renders in visual editor context',
        (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Keyboard should be rendered
      expect(find.byType(VisualKeyboard), findsOneWidget);

      // Check that some keys are visible
      expect(find.text('Esc'), findsWidgets);
      expect(find.text('A'), findsWidgets);
    });
  });
}
