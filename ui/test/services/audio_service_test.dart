import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/audio_service_impl.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';

class _MockPermissionService extends Mock implements PermissionService {}

class _MockErrorTranslator extends Mock implements ErrorTranslator {}

class _FakeBridge implements KeyrxBridge {
  _FakeBridge({
    this.initializeReturn = true,
    this.startReturn = true,
    this.stopReturn = true,
    this.setBpmReturn = true,
    Stream<BridgeClassification>? classificationStream,
  }) : _classificationStream = classificationStream;

  bool initializeReturn;
  bool startReturn;
  bool stopReturn;
  bool setBpmReturn;
  int initializeCalls = 0;
  int startCalls = 0;
  int stopCalls = 0;
  int setBpmCalls = 0;
  bool _initialized = false;
  final Stream<BridgeClassification>? _classificationStream;

  @override
  bool initialize() {
    initializeCalls += 1;
    _initialized = initializeReturn;
    return initializeReturn;
  }

  @override
  Future<bool> startAudio({required int bpm}) async {
    startCalls += 1;
    return startReturn;
  }

  @override
  Future<bool> stopAudio() async {
    stopCalls += 1;
    return stopReturn;
  }

  @override
  Future<bool> setBpm(int bpm) async {
    setBpmCalls += 1;
    return setBpmReturn;
  }

  @override
  Stream<BridgeClassification>? get classificationStream =>
      _classificationStream;

  @override
  bool get isInitialized => _initialized;

  @override
  bool loadScript(String path) => true;

  @override
  String get version => 'test';
}

void main() {
  group('AudioServiceImpl', () {
    late _MockPermissionService permission;
    late _MockErrorTranslator translator;
    late _FakeBridge bridge;
    const userMessage = UserMessage(
      title: 'Test',
      body: 'Body',
    );

    AudioServiceImpl buildService({Stream<ClassificationResult>? stream}) {
      return AudioServiceImpl(
        bridge: bridge,
        permissionService: permission,
        errorTranslator: translator,
        classificationSource: stream,
      );
    }

    setUp(() {
      permission = _MockPermissionService();
      translator = _MockErrorTranslator();
      bridge = _FakeBridge();

      when(() => translator.translate(any())).thenReturn(userMessage);
    });

    test('returns permission denied when microphone is not granted', () async {
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(
          state: PermissionState.denied,
          shouldShowRationale: true,
        ),
      );

      final service = buildService();
      final result = await service.start(bpm: 120);

      expect(result.success, isFalse);
      expect(result.error, AudioErrorCode.permissionDenied);
      expect(result.userMessage, userMessage);
      expect(service.state, AudioState.idle);
      expect(bridge.initializeCalls, 0);
      verify(() => translator.translate(any())).called(1);
    });

    test('starts successfully when permission granted and engine initializes',
        () async {
      bridge.initializeReturn = true;
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(state: PermissionState.granted),
      );

      final service = buildService();
      final result = await service.start(bpm: 128);

      expect(result.success, isTrue);
      expect(service.state, AudioState.running);
      expect(bridge.initializeCalls, 1);
      verifyNever(() => translator.translate(any()));
    });

    test('rejects invalid BPM values', () async {
      final service = buildService();

      final result = await service.start(bpm: -1);

      expect(result.success, isFalse);
      expect(result.error, AudioErrorCode.invalidBpm);
      expect(result.userMessage, userMessage);
      expect(service.state, AudioState.idle);
      verifyNever(() => permission.requestMicrophone());
    });

    test('setBpm errors when audio is not running', () async {
      final service = buildService();

      final result = await service.setBpm(100);

      expect(result.success, isFalse);
      expect(result.error, AudioErrorCode.notInitialized);
      expect(result.userMessage, userMessage);
      verify(() => translator.translate(any())).called(1);
    });

    test('stop returns success when already idle', () async {
      final service = buildService();

      final result = await service.stop();

      expect(result.success, isTrue);
      expect(service.state, AudioState.idle);
    });

    test('forwards classification stream events and errors', () async {
      final controller = StreamController<ClassificationResult>();
      final service = buildService(stream: controller.stream);
      final events = <ClassificationResult>[];
      final errors = <Object>[];

      final sub = service.classificationStream.listen(
        events.add,
        onError: errors.add,
      );

      final sample = ClassificationResult(
        label: 'kick',
        confidence: 0.92,
        timestamp: DateTime.now(),
      );

      controller.add(sample);
      await Future<void>.delayed(Duration.zero);
      expect(events.single, sample);

      controller.addError(StateError('stream failure'));
      await Future<void>.delayed(Duration.zero);
      expect(errors, isNotEmpty);

      await service.dispose();
      await controller.close();
      await sub.cancel();
    });
  });
}
