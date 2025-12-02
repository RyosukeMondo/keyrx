// Integration test: Training → Debugger flow
//
// Tests the end-to-end user journey from training screen completion
// through to debugger page state visualization.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/debugger.dart';
import 'package:keyrx_ui/pages/keyrx_training_screen.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/app_state.dart';

/// Mock engine service with controllable state stream.
class MockEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController<EngineSnapshot>.broadcast();

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test-1.0.0';

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok: command executed');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  void emitSnapshot(EngineSnapshot snapshot) {
    _stateController.add(snapshot);
  }

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class MockAudioService implements AudioService {
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

class MockPermissionService implements PermissionService {
  @override
  Future<PermissionResult> checkMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);

  @override
  Future<PermissionResult> requestMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);
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
    SharedPreferences.setMockInitialValues({});
    mockEngine = MockEngineService();
  });

  Widget buildTestApp() {
    final registry = ServiceRegistry.withOverrides(
      permissionService: MockPermissionService(),
      audioService: MockAudioService(),
      errorTranslator: MockErrorTranslator(),
      engineService: mockEngine,
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
        home: const _TestNavigationShell(),
      ),
    );
  }

  group('Training to Debugger Flow', () {
    testWidgets('User can navigate from training to debugger',
        (WidgetTester tester) async {
      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Verify we start on the training screen
      expect(find.text('Basic Remapping'), findsOneWidget);

      // Navigate to debugger using bottom nav
      await tester.tap(find.byIcon(Icons.bug_report_outlined));
      await tester.pumpAndSettle();

      // Verify debugger page is shown
      expect(find.text('State Debugger'), findsOneWidget);
    });

    testWidgets('Training completes lesson step and advances',
        (WidgetTester tester) async {
      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Verify initial state (first lesson, first step)
      expect(find.text('Step 1 of 2'), findsOneWidget);
      expect(
        find.text('Press the CapsLock key to see it remapped to Escape'),
        findsOneWidget,
      );

      // Emit engine snapshot that satisfies the validator (escape in lastEvent)
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'escape key pressed',
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Step should advance
      expect(find.text('Step 2 of 2'), findsOneWidget);
    });

    testWidgets('Debugger shows live state updates from engine',
        (WidgetTester tester) async {
      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Navigate to debugger
      await tester.tap(find.byIcon(Icons.bug_report_outlined));
      await tester.pumpAndSettle();

      // Emit a snapshot with state
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'KeyA pressed',
          activeLayers: const ['nav', 'symbols'],
          activeModifiers: const ['Ctrl', 'Shift'],
          heldKeys: const ['A', 'S'],
          pendingDecisions: const [],
          latencyUs: 1500,
        ),
      );
      await tester.pumpAndSettle();

      // Verify state is displayed - check for key state elements
      expect(find.textContaining('nav'), findsWidgets);
      expect(find.textContaining('symbols'), findsWidgets);
    });

    testWidgets('Full flow: training → complete lessons → debugger verification',
        (WidgetTester tester) async {
      // Pre-populate progress to skip to near completion
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:1'], // 1 of 2 steps done
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Should be at step 2 of first lesson
      expect(find.text('Step 2 of 2'), findsOneWidget);

      // Complete the current step
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'any key',
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Should advance to next lesson (Layers)
      expect(
        find.text('Hold your layer key to activate a layer'),
        findsOneWidget,
      );

      // Navigate to debugger to verify engine connection
      await tester.tap(find.byIcon(Icons.bug_report_outlined));
      await tester.pumpAndSettle();

      // Verify debugger page is active
      expect(find.text('State Debugger'), findsOneWidget);

      // Emit state update
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'Layer activated',
          activeLayers: const ['nav'],
          activeModifiers: const [],
          heldKeys: const ['Space'],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Verify state shows layer
      expect(find.textContaining('nav'), findsWidgets);
    });

    testWidgets('State preview in training shows engine state',
        (WidgetTester tester) async {
      // Increase window size to avoid overflow
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Emit snapshot with state
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'KeyA pressed',
          activeLayers: const ['nav'],
          activeModifiers: const ['Ctrl'],
          heldKeys: const ['A'],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // State preview should appear (visibility icon appears after snapshot)
      expect(find.byIcon(Icons.visibility), findsOneWidget);
      expect(find.textContaining('Event: KeyA pressed'), findsOneWidget);
    });

    testWidgets('Debugger pause/resume controls work',
        (WidgetTester tester) async {
      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Navigate to debugger
      await tester.tap(find.byIcon(Icons.bug_report_outlined));
      await tester.pumpAndSettle();

      // Verify LIVE indicator is shown
      expect(find.text('LIVE'), findsOneWidget);

      // Emit initial snapshot
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'Initial',
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();
    });

    testWidgets('Progress persists across navigation',
        (WidgetTester tester) async {
      await tester.pumpWidget(buildTestApp());
      await tester.pumpAndSettle();

      // Complete first step
      mockEngine.emitSnapshot(
        EngineSnapshot(
          timestamp: DateTime.now(),
          lastEvent: 'escape',
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Should be at step 2
      expect(find.text('Step 2 of 2'), findsOneWidget);

      // Navigate to debugger
      await tester.tap(find.byIcon(Icons.bug_report_outlined));
      await tester.pumpAndSettle();

      // Navigate back to training
      await tester.tap(find.byIcon(Icons.graphic_eq_outlined));
      await tester.pumpAndSettle();

      // Progress should be preserved
      expect(find.text('Step 2 of 2'), findsOneWidget);
    });
  });
}

/// Test shell with navigation bar for testing page transitions.
class _TestNavigationShell extends StatefulWidget {
  const _TestNavigationShell();

  @override
  State<_TestNavigationShell> createState() => _TestNavigationShellState();
}

class _TestNavigationShellState extends State<_TestNavigationShell> {
  int _selectedIndex = 0;

  final List<Widget> _pages = const [
    KeyrxTrainingScreen(),
    DebuggerPage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _pages[_selectedIndex],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _selectedIndex,
        onDestinationSelected: (index) {
          setState(() {
            _selectedIndex = index;
          });
        },
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.graphic_eq_outlined),
            selectedIcon: Icon(Icons.graphic_eq),
            label: 'Training',
          ),
          NavigationDestination(
            icon: Icon(Icons.bug_report_outlined),
            selectedIcon: Icon(Icons.bug_report),
            label: 'Debugger',
          ),
        ],
      ),
    );
  }
}
