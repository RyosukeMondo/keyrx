import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/console.dart';
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

  Future<ConsoleEvalResult> Function(String command)? onEval;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      onEval != null
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
  testWidgets('Console executes commands via EngineService', (tester) async {
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
        child: const MaterialApp(home: ConsolePage()),
      ),
    );

    await tester.enterText(find.byType(TextField), 'print("hi")');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.textContaining('print("hi")'), findsWidgets);
    expect(find.text('OK'), findsWidgets);
  });

  testWidgets('Console renders error styling for error-prefixed output', (tester) async {
    final fakeEngine = _FakeEngineService()
      ..onEval = (_) async => const ConsoleEvalResult(
            success: false,
            output: 'error: engine unavailable',
          );
    final registry = ServiceRegistry.withOverrides(
      permissionService: _FakePermissionService(),
      audioService: _FakeAudioService(),
      errorTranslator: _FakeErrorTranslator(),
      engineService: fakeEngine,
    );

    await tester.pumpWidget(
      MultiProvider(
        providers: [Provider<ServiceRegistry>.value(value: registry)],
        child: const MaterialApp(home: ConsolePage()),
      ),
    );

    await tester.enterText(find.byType(TextField), 'status');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.textContaining('status'), findsWidgets);
    expect(find.text('ERROR'), findsWidgets);
    expect(find.textContaining('engine unavailable'), findsWidgets);
  });
}
