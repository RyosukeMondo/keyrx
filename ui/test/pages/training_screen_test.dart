import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/training_screen.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/app_state.dart';

class _FakePermissionService implements PermissionService {
  const _FakePermissionService([this.result = const PermissionResult(
    state: PermissionState.granted,
  )]);

  final PermissionResult result;

  @override
  Future<PermissionResult> checkMicrophone() async => result;

  @override
  Future<PermissionResult> requestMicrophone() async => result;
}

class _FakeEngineService implements EngineService {
  const _FakeEngineService();

  @override
  Future<void> dispose() async {}

  @override
  bool get isInitialized => true;

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  String get version => 'test';

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok');

  @override
  Stream<EngineSnapshot> get stateStream =>
      const Stream<EngineSnapshot>.empty();

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);
}

class _RecordingTranslator implements ErrorTranslator {
  const _RecordingTranslator(this.message);

  final UserMessage message;

  @override
  UserMessage translate(Object error) => message;
}

class _ControllableAudioService implements AudioService {
  _ControllableAudioService({
    AudioOperationResult startResult = const AudioOperationResult(success: true),
    AudioOperationResult stopResult = const AudioOperationResult(success: true),
  })  : _startResult = startResult,
        _stopResult = stopResult;

  final StreamController<ClassificationResult> _controller =
      StreamController<ClassificationResult>.broadcast();
  AudioState _state = AudioState.idle;
  AudioOperationResult _startResult;
  AudioOperationResult _stopResult;
  Completer<AudioOperationResult>? startCompleter;
  Completer<AudioOperationResult>? stopCompleter;
  int startCalls = 0;
  int stopCalls = 0;
  int setBpmCalls = 0;

  @override
  AudioState get state => _state;

  @override
  Stream<ClassificationResult> get classificationStream => _controller.stream;

  @override
  Future<AudioOperationResult> start({required int bpm}) {
    startCalls += 1;
    _state = _startResult.success ? AudioState.running : AudioState.idle;
    if (startCompleter != null) {
      return startCompleter!.future;
    }
    return Future.value(_startResult);
  }

  @override
  Future<AudioOperationResult> stop() {
    stopCalls += 1;
    _state = AudioState.idle;
    if (stopCompleter != null) {
      return stopCompleter!.future;
    }
    return Future.value(_stopResult);
  }

  @override
  Future<AudioOperationResult> setBpm(int bpm) async {
    setBpmCalls += 1;
    return const AudioOperationResult(success: true);
  }

  void emitClassification(ClassificationResult result) {
    _controller.add(result);
  }

  void emitError(Object error) {
    _controller.addError(error);
  }

  @override
  Future<void> dispose() async {
    await _controller.close();
  }
}

void main() {
  testWidgets('shows loading overlay while starting and disables start on run',
      (tester) async {
    final audio = _ControllableAudioService();
    audio.startCompleter = Completer<AudioOperationResult>();

    final registry = ServiceRegistry.withOverrides(
      permissionService: const _FakePermissionService(),
      audioService: audio,
      errorTranslator: const _RecordingTranslator(
        UserMessage(title: 'Unused', body: 'Unused'),
      ),
      engineService: const _FakeEngineService(),
    );

    addTearDown(registry.dispose);

    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<ServiceRegistry>.value(value: registry),
          ChangeNotifierProvider<AppState>(
            create: (_) => AppState(
              engineService: const _FakeEngineService(),
              errorTranslator: const _RecordingTranslator(
                UserMessage(title: 'Unused', body: 'Unused'),
              ),
            ),
          ),
        ],
        child: const MaterialApp(home: TrainingScreen()),
      ),
    );

    await tester.tap(find.text('Start'));
    await tester.pump();

    expect(find.text('Starting audio...'), findsOneWidget);

    audio.startCompleter!.complete(const AudioOperationResult(success: true));
    await tester.pump();
    await tester.pumpAndSettle();

    expect(find.text('Starting audio...'), findsNothing);
    expect(audio.startCalls, 1);
  });

  testWidgets('renders classification results from the audio stream',
      (tester) async {
    final audio = _ControllableAudioService();
    final registry = ServiceRegistry.withOverrides(
      permissionService: const _FakePermissionService(),
      audioService: audio,
      errorTranslator: const _RecordingTranslator(
        UserMessage(title: 'Unused', body: 'Unused'),
      ),
      engineService: const _FakeEngineService(),
    );

    addTearDown(registry.dispose);

    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<ServiceRegistry>.value(value: registry),
          ChangeNotifierProvider<AppState>(
            create: (_) => AppState(
              engineService: const _FakeEngineService(),
              errorTranslator: const _RecordingTranslator(
                UserMessage(title: 'Unused', body: 'Unused'),
              ),
            ),
          ),
        ],
        child: const MaterialApp(home: TrainingScreen()),
      ),
    );

    audio.emitClassification(
      ClassificationResult(
        label: 'kick',
        confidence: 0.91,
        timestamp: DateTime(2025, 1, 1, 12, 0),
      ),
    );

    await tester.pump();

    expect(find.text('kick'), findsOneWidget);
    expect(find.textContaining('91.0%'), findsOneWidget);
  });

  testWidgets('shows translated error when classification stream errors',
      (tester) async {
    const translated = UserMessage(
      title: 'Stream failed',
      body: 'Check microphone',
    );

    final audio = _ControllableAudioService();
    final registry = ServiceRegistry.withOverrides(
      permissionService: const _FakePermissionService(),
      audioService: audio,
      errorTranslator: const _RecordingTranslator(translated),
      engineService: const _FakeEngineService(),
    );

    addTearDown(registry.dispose);

    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<ServiceRegistry>.value(value: registry),
          ChangeNotifierProvider<AppState>(
            create: (_) => AppState(
              engineService: const _FakeEngineService(),
              errorTranslator: const _RecordingTranslator(
                UserMessage(title: 'Unused', body: 'Unused'),
              ),
            ),
          ),
        ],
        child: const MaterialApp(home: TrainingScreen()),
      ),
    );

    audio.emitError(StateError('boom'));
    await tester.pumpAndSettle();

    expect(find.text('Stream failed'), findsOneWidget);
    expect(find.text('Check microphone'), findsOneWidget);
  });
}
