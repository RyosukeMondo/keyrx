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
        child: const MaterialApp(home: DebuggerPage()),
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

    expect(chipLabels, containsAll(['base', 'nav', 'Ctrl', 'tap_hold A']));
    expect(find.textContaining('Latency'), findsWidgets);
    expect(find.textContaining('120µs'), findsWidgets);
  });
}
