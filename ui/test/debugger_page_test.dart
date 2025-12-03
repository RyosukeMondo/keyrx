import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/debugger.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:provider/provider.dart';

// Test helpers

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

void main() {
  testWidgets('Debugger renders incoming engine state', (tester) async {
    final fakeEngine = _FakeEngineService();
    final registry = ServiceRegistry.withOverrides(
      permissionService: _FakePermissionService(),
      audioService: _FakeAudioService(),
      errorTranslator: _FakeErrorTranslator(),
      engineService: fakeEngine,
    );

    await tester.pumpWidget(
      MultiProvider(
        providers: [Provider<ServiceRegistry>.value(value: registry)],
        child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
      ),
    );

    fakeEngine.emit(
      EngineSnapshot(
        timestamp: DateTime.now(),
        activeLayers: const ['base', 'nav'],
        activeModifiers: const ['Ctrl'],
        heldKeys: const ['A'],
        pendingDecisions: const ['tap_hold A'],
        lastEvent: 'A pressed',
        latencyUs: 120,
      ),
    );

    await tester.pumpAndSettle();

    final chipLabels = tester
        .widgetList<Chip>(find.byType(Chip))
        .map((chip) {
          final label = chip.label;
          if (label is Text) {
            return label.data ?? '';
          }
          return '';
        })
        .where((label) => label.isNotEmpty)
        .toList();

    // Layers, modifiers, and held keys are still in chips
    expect(chipLabels, containsAll(['base', 'nav', 'Ctrl']));
    // Pending tap-hold decisions are now shown in a separate widget with countdown
    expect(find.textContaining('tap_hold A'), findsWidgets);
    expect(find.textContaining('Latency'), findsWidgets);
    expect(find.textContaining('120µs'), findsWidgets);
  });

  testWidgets('Debugger shows timing and pending decisions with latency timeline', (tester) async {
    final fakeEngine = _FakeEngineService();
    final registry = ServiceRegistry.withOverrides(
      permissionService: _FakePermissionService(),
      audioService: _FakeAudioService(),
      errorTranslator: _FakeErrorTranslator(),
      engineService: fakeEngine,
    );

    await tester.pumpWidget(
      MultiProvider(
        providers: [Provider<ServiceRegistry>.value(value: registry)],
        child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
      ),
    );

    fakeEngine.emit(
      EngineSnapshot(
        timestamp: DateTime.now(),
        activeLayers: const ['base'],
        activeModifiers: const ['Shift'],
        heldKeys: const ['K'],
        pendingDecisions: const ['combo A+B'],
        lastEvent: 'combo triggered',
        latencyUs: 25000,
        timing: const EngineTiming(
          tapTimeoutMs: 150,
          comboTimeoutMs: 250,
          holdDelayMs: 40,
          eagerTap: true,
        ),
      ),
    );

    await tester.pumpAndSettle();

    // Combo decisions are now shown with individual key chips (A and B extracted from "combo A+B")
    expect(find.text('A'), findsWidgets);
    expect(find.text('B'), findsWidgets);
    expect(find.text('Timing'), findsOneWidget);
    expect(find.textContaining('Tap timeout: 150ms'), findsOneWidget);
    expect(find.textContaining('Hold delay: 40ms'), findsOneWidget);
    expect(find.textContaining('Eager tap: true'), findsOneWidget);
    expect(find.textContaining('Latency Timeline'), findsOneWidget);
    expect(find.textContaining('Avg:'), findsOneWidget);
  });

  group('State stream subscription', () {
    testWidgets('subscribes to state stream on init', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // Initially shows waiting state
      expect(find.textContaining('Waiting'), findsWidgets);

      // Emit a snapshot
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['layer1'],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          lastEvent: 'test event',
          latencyUs: 100,
        ),
      );

      await tester.pumpAndSettle();

      // Now shows the event data
      expect(find.text('layer1'), findsOneWidget);
      // Event appears in both last event card and event log
      expect(find.textContaining('test event'), findsWidgets);
    });

    testWidgets('updates UI on new state snapshots', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // First snapshot
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['initial'],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 50,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('initial'), findsOneWidget);

      // Second snapshot with different data
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['updated'],
          activeModifiers: const ['Shift'],
          heldKeys: const ['X'],
          pendingDecisions: const [],
          latencyUs: 200,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('initial'), findsNothing);
      expect(find.text('updated'), findsOneWidget);
      expect(find.text('Shift'), findsOneWidget);
    });

  });

  group('Latency meter', () {
    testWidgets('displays latency value correctly', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 5000,
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('5000'), findsOneWidget);
      expect(find.textContaining('µs per event'), findsOneWidget);
      expect(find.textContaining('Healthy'), findsOneWidget);
    });

    testWidgets('shows caution for medium latency', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // 15000 µs is between caution (10k) and warning (20k)
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 15000,
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('15000'), findsOneWidget);
      expect(find.textContaining('Caution'), findsOneWidget);
    });

    testWidgets('shows warning for high latency', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // 25000 µs is above warning threshold (20k)
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 25000,
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('25000'), findsOneWidget);
      expect(find.textContaining('High'), findsOneWidget);
    });

    testWidgets('updates on latency change', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // Start with low latency
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 1000,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('1000'), findsOneWidget);

      // Update to high latency
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 30000,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('1000'), findsNothing);
      expect(find.text('30000'), findsOneWidget);
    });

    testWidgets('shows waiting state when no latency', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.textContaining('Waiting for samples'), findsOneWidget);
    });
  });

  group('Pending tap-hold countdown', () {
    testWidgets('shows countdown timer for tap-hold decisions', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['tap_hold KeyA'],
          latencyUs: 100,
          timing: const EngineTiming(tapTimeoutMs: 200),
        ),
      );

      await tester.pump();

      // Should show the CircularProgressIndicator for countdown
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      // Should show the tap-hold decision text
      expect(find.textContaining('tap_hold KeyA'), findsOneWidget);
      // Should show the touch_app icon
      expect(find.byIcon(Icons.touch_app), findsOneWidget);
    });

    testWidgets('countdown progress decreases over time', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['tap_hold X'],
          latencyUs: 100,
          timing: const EngineTiming(tapTimeoutMs: 200),
        ),
      );

      await tester.pump();

      // Get initial progress indicator
      var indicator = tester.widget<CircularProgressIndicator>(
        find.byType(CircularProgressIndicator),
      );
      final initialValue = indicator.value;

      // Wait for some time (100ms out of 200ms)
      await tester.pump(const Duration(milliseconds: 100));

      indicator = tester.widget<CircularProgressIndicator>(
        find.byType(CircularProgressIndicator),
      );
      final laterValue = indicator.value;

      // Progress should have decreased
      expect(laterValue, lessThan(initialValue ?? 1.0));
    });

    testWidgets('categorizes taphold and hold keywords correctly', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['TapHold on KeyJ', 'hold action KeyK'],
          latencyUs: 100,
        ),
      );

      await tester.pump();

      // Should find both tap-hold style decisions with countdown indicators
      expect(find.byType(CircularProgressIndicator), findsNWidgets(2));
      expect(find.byIcon(Icons.touch_app), findsNWidgets(2));
    });
  });

  group('Combo key highlighting', () {
    testWidgets('extracts and displays combo keys from plus notation', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['combo Ctrl+Shift+P'],
          latencyUs: 100,
        ),
      );

      await tester.pumpAndSettle();

      // Should extract and display individual keys
      expect(find.text('Ctrl'), findsWidgets);
      expect(find.text('Shift'), findsWidgets);
      expect(find.text('P'), findsWidgets);
      // Should show keyboard icon for combos
      expect(find.byIcon(Icons.keyboard), findsOneWidget);
    });

    testWidgets('extracts combo keys from bracket notation', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['combo [A, B, C]'],
          latencyUs: 100,
        ),
      );

      await tester.pumpAndSettle();

      // Should extract and display individual keys
      expect(find.text('A'), findsOneWidget);
      expect(find.text('B'), findsOneWidget);
      expect(find.text('C'), findsOneWidget);
    });

    testWidgets('combo keys have blue styling', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['combo X+Y'],
          latencyUs: 100,
        ),
      );

      await tester.pumpAndSettle();

      // Find the keyboard icon (combo indicator) and verify it's blue
      final keyboardIcon = tester.widget<Icon>(find.byIcon(Icons.keyboard));
      expect(keyboardIcon.color, equals(Colors.blue));
    });

    testWidgets('differentiates combo from tap-hold decisions', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const [],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const ['tap_hold KeyA', 'combo B+C'],
          latencyUs: 100,
        ),
      );

      await tester.pumpAndSettle();

      // tap_hold should show touch_app icon with countdown
      expect(find.byIcon(Icons.touch_app), findsOneWidget);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      // combo should show keyboard icon without countdown
      expect(find.byIcon(Icons.keyboard), findsOneWidget);
      expect(find.text('B'), findsWidgets);
      expect(find.text('C'), findsWidgets);
    });
  });

  group('Recording controls', () {
    testWidgets('pause button stops recording new events', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // Initially live
      expect(find.text('LIVE'), findsOneWidget);

      // Tap pause button
      await tester.tap(find.byIcon(Icons.pause));
      await tester.pumpAndSettle();

      // Should show paused
      expect(find.text('PAUSED'), findsOneWidget);

      // Emit an event while paused
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['should_not_appear'],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          latencyUs: 100,
        ),
      );
      await tester.pumpAndSettle();

      // Event should not appear since paused
      expect(find.text('should_not_appear'), findsNothing);

      // Resume recording
      await tester.tap(find.byIcon(Icons.play_arrow));
      await tester.pumpAndSettle();

      // Should show live again
      expect(find.text('LIVE'), findsOneWidget);
    });

    testWidgets('clear button removes all logged events', (tester) async {
      final fakeEngine = _FakeEngineService();
      final registry = ServiceRegistry.withOverrides(
        permissionService: _FakePermissionService(),
        audioService: _FakeAudioService(),
        errorTranslator: _FakeErrorTranslator(),
        engineService: fakeEngine,
      );

      await tester.pumpWidget(
        MultiProvider(
          providers: [Provider<ServiceRegistry>.value(value: registry)],
          child: MaterialApp(home: DebuggerPage(engineService: fakeEngine)),
        ),
      );

      // Emit some events
      fakeEngine.emit(
        EngineSnapshot(
          timestamp: DateTime.now(),
          activeLayers: const ['layer_to_clear'],
          activeModifiers: const [],
          heldKeys: const [],
          pendingDecisions: const [],
          lastEvent: 'event_to_clear',
          latencyUs: 100,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('layer_to_clear'), findsOneWidget);
      expect(find.textContaining('event_to_clear'), findsWidgets);

      // Tap clear button
      await tester.tap(find.byIcon(Icons.clear_all));
      await tester.pumpAndSettle();

      // Event log should be cleared
      expect(find.textContaining('event_to_clear'), findsNothing);
      expect(find.textContaining('Waiting'), findsWidgets);
    });
  });
}
