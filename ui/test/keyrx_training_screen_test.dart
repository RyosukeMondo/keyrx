import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/keyrx_training_screen.dart';
import 'package:keyrx_ui/pages/training_lessons.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';

// Mock implementations

class _ControllableEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController<EngineSnapshot>.broadcast();

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
      const KeyRegistryResult(entries: []);

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
  Stream<ClassificationResult> get classificationStream =>
      const Stream.empty();

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

Widget _buildTestApp({
  required _ControllableEngineService engineService,
}) {
  final registry = ServiceRegistry.withOverrides(
    permissionService: _FakePermissionService(),
    audioService: _FakeAudioService(),
    errorTranslator: _FakeErrorTranslator(),
    engineService: engineService,
  );

  return MultiProvider(
    providers: [
      Provider<ServiceRegistry>.value(value: registry),
    ],
    child: const MaterialApp(home: KeyrxTrainingScreen()),
  );
}

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('Lesson display', () {
    testWidgets('displays lesson carousel with all lessons', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Should display all 5 lesson titles
      expect(find.text('Basic Remapping'), findsOneWidget);
      expect(find.text('Layers'), findsOneWidget);
      expect(find.text('Modifiers'), findsOneWidget);
      expect(find.text('Tap-Hold'), findsOneWidget);
      expect(find.text('Combos'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('displays current step instruction', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // First lesson, first step instruction
      expect(
        find.text('Press the CapsLock key to see it remapped to Escape'),
        findsOneWidget,
      );
      expect(find.text('Step 1 of 2'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('shows expected output when defined', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      expect(find.textContaining('Expected: Escape key event'), findsOneWidget);

      await engine.dispose();
    });
  });

  group('Step progression', () {
    testWidgets('advances to next step when validator passes', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Initial: Step 1 of 2
      expect(find.text('Step 1 of 2'), findsOneWidget);

      // Emit snapshot that satisfies first step validator (escape in lastEvent)
      engine.emit(
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

      // Should advance to step 2
      expect(find.text('Step 2 of 2'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('advances to next lesson when all steps complete',
        (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Complete step 1 of first lesson
      engine.emit(
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

      // Complete step 2 (any lastEvent triggers)
      engine.emit(
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

      // Completing last step auto-advances to next lesson (Layers)
      // and shows first step of the new lesson
      expect(
        find.text('Hold your layer key to activate a layer'),
        findsOneWidget,
      );
      expect(find.text('Step 1 of 3'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('selects lesson via carousel tap', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Tap on Layers lesson
      await tester.tap(find.text('Layers'));
      await tester.pumpAndSettle();

      // Should show Layers lesson step
      expect(
        find.text('Hold your layer key to activate a layer'),
        findsOneWidget,
      );

      await engine.dispose();
    });
  });

  group('Exercise validation', () {
    testWidgets('shows exercise prompt', (tester) async {
      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      expect(
        find.text('Press CapsLock to produce Escape'),
        findsOneWidget,
      );

      await engine.dispose();
    });

    testWidgets('shows success feedback when exercise passes and step advances',
        (tester) async {
      // Start at modifier lesson step 1 (validator: Shift in activeModifiers)
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2', 'layer:3'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // We're at modifier lesson step 1
      expect(find.text('Press and hold the Shift key'), findsOneWidget);

      // Emit correct input (Shift modifier)
      engine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeModifiers: const ['Shift'],
          activeLayers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // When exercise passes, step advances automatically
      // Should now be on step 2 (Ctrl key prompt)
      expect(find.text('Press and hold the Ctrl key'), findsOneWidget);
      expect(find.text('Step 2 of 3'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('shows error feedback on wrong input', (tester) async {
      // Increase window size to avoid overflow
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      // Start at modifier lesson step 1 (validator: Shift in activeModifiers)
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2', 'layer:3'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // We're at modifier lesson step 1 (expects Shift)
      expect(find.text('Press and hold the Shift key'), findsOneWidget);

      // Emit wrong input (Ctrl instead of Shift)
      engine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeModifiers: const ['Ctrl'], // Wrong modifier
          activeLayers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Step should NOT advance (Shift not detected)
      // Still on step 1, exercise shows failure feedback
      expect(find.text('Step 1 of 3'), findsOneWidget);
      // Error indicator should be present (cancel icon)
      expect(find.byIcon(Icons.cancel), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('validates layer exercise and advances step', (tester) async {
      // Increase window size to avoid overflow
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      // Start at layer lesson step 1
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // We're at layer lesson step 1
      expect(
        find.text('Hold your layer key to activate it'),
        findsOneWidget,
      );

      // Emit snapshot with active layers - this will pass both exercise and step
      engine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['nav'],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Step should advance to step 2
      expect(
        find.text('While holding the layer key, press another key'),
        findsOneWidget,
      );
      expect(find.text('Step 2 of 3'), findsOneWidget);

      await engine.dispose();
    });
  });

  group('Hint system', () {
    testWidgets('shows hint button initially', (tester) async {
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      expect(find.text('Show Hint'), findsOneWidget);

      await engine.dispose();
    });

    testWidgets('reveals hint on button press', (tester) async {
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Show Hint'));
      await tester.pumpAndSettle();

      expect(
        find.text('CapsLock is often remapped to Escape for easier Vim usage'),
        findsOneWidget,
      );

      await engine.dispose();
    });
  });

  group('Progress persistence', () {
    testWidgets('persists progress to SharedPreferences', (tester) async {
      // Pre-set progress to step 1 of remap lesson so next completion saves
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:1'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // We're at step 2 (index 1) of remap, complete it
      engine.emit(
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

      // Verify progress saved
      final prefs = await SharedPreferences.getInstance();
      final progress = prefs.getStringList('keyrx_training_progress');
      expect(progress, contains('remap:2'));

      await engine.dispose();
    });

    testWidgets('loads saved progress on startup', (tester) async {
      // Pre-populate progress
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2', 'layer:1'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // First lesson complete, should show on second lesson step 2
      // (layer lesson has progress of 1, meaning step 1 complete)
      expect(
        find.text('While holding the layer key, press another key'),
        findsOneWidget,
      );

      await engine.dispose();
    });

    testWidgets('reset progress clears SharedPreferences', (tester) async {
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2', 'layer:3'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Tap reset button
      await tester.tap(find.byIcon(Icons.refresh));
      await tester.pumpAndSettle();

      // Confirm reset
      await tester.tap(find.text('Reset'));
      await tester.pumpAndSettle();

      // Should be back at first lesson, first step
      expect(find.text('Step 1 of 2'), findsOneWidget);
      expect(
        find.text('Press the CapsLock key to see it remapped to Escape'),
        findsOneWidget,
      );

      // Progress should be cleared
      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getStringList('keyrx_training_progress'), isNull);

      await engine.dispose();
    });
  });

  group('Certificate display', () {
    testWidgets('shows certificate when all lessons complete', (tester) async {
      // Complete all lessons except last step of combo
      // combo has 3 steps, so combo:2 means step 3 remains
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': [
          'remap:2',
          'layer:3',
          'modifier:3',
          'taphold:3',
          'combo:2', // One step remaining (step 3: release all keys)
        ],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Auto-navigates to first incomplete lesson step
      // Should be at combo step 3: "Release all keys to complete the combo"
      expect(
        find.text('Release all keys to complete the combo'),
        findsOneWidget,
      );

      // Complete the remaining combo step (step 3: release keys)
      // Step validator: heldKeys.isEmpty && pendingDecisions.isEmpty
      engine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Certificate dialog should appear
      expect(find.text('CERTIFICATE'), findsOneWidget);
      expect(find.text('of Completion'), findsOneWidget);
      expect(find.textContaining('You mastered all 5 KeyRx lessons'), findsOneWidget);
      // emoji_events appears in both certificate (72px) and lesson complete view (64px)
      expect(find.byIcon(Icons.emoji_events), findsWidgets);

      await engine.dispose();
    });

    testWidgets('certificate has continue button to dismiss', (tester) async {
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': [
          'remap:2',
          'layer:3',
          'modifier:3',
          'taphold:3',
          'combo:2',
        ],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Complete final combo step
      engine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
        ),
      );
      await tester.pumpAndSettle();

      // Dismiss certificate
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();

      // Certificate should be dismissed
      expect(find.text('CERTIFICATE'), findsNothing);

      await engine.dispose();
    });
  });

  group('State preview', () {
    testWidgets('shows state preview bar after receiving snapshot',
        (tester) async {
      // Increase window size to avoid overflow
      tester.view.physicalSize = const Size(1200, 1000);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Initially no state preview (visibility icon only appears after snapshot)
      expect(find.byIcon(Icons.visibility), findsNothing);

      // Emit a snapshot
      engine.emit(
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

      // State preview should appear
      expect(find.byIcon(Icons.visibility), findsOneWidget);
      expect(find.textContaining('Event: KeyA pressed'), findsOneWidget);
      expect(find.textContaining('Layers: nav'), findsOneWidget);
      expect(find.textContaining('Mods: Ctrl'), findsOneWidget);
      expect(find.textContaining('Held: A'), findsOneWidget);

      await engine.dispose();
    });
  });

  group('Lesson completion markers', () {
    testWidgets('shows checkmark for completed lessons', (tester) async {
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:2', 'layer:3'],
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Check that completed lessons show check_circle icons
      // There should be 2 check_circle icons (for remap and layer)
      final checkIcons = find.byIcon(Icons.check_circle);
      expect(checkIcons, findsNWidgets(2));

      await engine.dispose();
    });

    testWidgets('shows progress indicator for incomplete lessons',
        (tester) async {
      SharedPreferences.setMockInitialValues({
        'keyrx_training_progress': ['remap:1'], // 1 of 2 steps done
      });

      final engine = _ControllableEngineService();

      await tester.pumpWidget(_buildTestApp(engineService: engine));
      await tester.pumpAndSettle();

      // Should show 1/2 for remap lesson progress
      expect(find.text('1/2'), findsOneWidget);

      await engine.dispose();
    });
  });
}
