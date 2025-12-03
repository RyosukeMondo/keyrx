import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:keyrx_ui/ffi/bindings.dart';
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

  // Mixin interface implementations - these are required by the mixins.
  @override
  KeyrxBindings? get bindings => null;

  @override
  StreamController<BridgeClassification>? get classificationController => null;

  @override
  StreamController<BridgeState>? get stateController => null;

  @override
  bool get initialized => _initialized;

  @override
  set initialized(bool value) => _initialized = value;

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

  @override
  Future<void> dispose() async {}

  @override
  Future<String?> eval(String command) async => 'ok';

  @override
  Object? get loadFailure => null;

  @override
  Stream<BridgeState>? get stateStream => null;

  @override
  KeyRegistryResult listKeys() => const KeyRegistryResult(entries: []);

  @override
  bool isBypassActive() => false;

  @override
  void setBypass(bool active) {}

  @override
  DeviceListResult listDevices() => const DeviceListResult(devices: []);

  @override
  int selectDevice(String path) => 0;

  @override
  ScriptValidationResult checkScript(String path) =>
      const ScriptValidationResult(valid: true, errors: []);

  @override
  TestDiscoveryResult discoverTests(String path) =>
      const TestDiscoveryResult(tests: []);

  @override
  TestRunResult runTests(String path, {String? filter}) => const TestRunResult(
        total: 0,
        passed: 0,
        failed: 0,
        durationMs: 0,
        results: [],
      );

  @override
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) =>
      const SimulationResult(mappings: [], activeLayers: [], pending: []);

  @override
  SessionListResult listSessions(String dirPath) =>
      const SessionListResult(sessions: []);

  @override
  SessionAnalysisResult analyzeSession(String path) =>
      SessionAnalysisResult.error('not implemented');

  @override
  ReplayResult replaySession(String path, {bool verify = false}) =>
      ReplayResult.error('not implemented');

  @override
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) =>
      BenchmarkResult.error('not implemented');

  @override
  DoctorResult runDoctor() => DoctorResult.error('not implemented');

  @override
  DiscoveryStartResult startDiscovery(
    String deviceId,
    int rows,
    List<int> colsPerRow,
  ) =>
      DiscoveryStartResult.error('not implemented');

  @override
  RecordingStartResult startRecording(String path) =>
      RecordingStartResult.error('not implemented');

  @override
  RecordingStopResult stopRecording() =>
      RecordingStopResult.error('not implemented');
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

    test('maps bridge classification events to ClassificationResult', () async {
      final bridgeStream = StreamController<BridgeClassification>();
      bridge = _FakeBridge(classificationStream: bridgeStream.stream);
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(state: PermissionState.granted),
      );

      final service = buildService();
      final events = <ClassificationResult>[];
      final sub = service.classificationStream.listen(events.add);

      final event = BridgeClassification(
        label: 'snare',
        confidence: 0.87,
        timestamp: DateTime(2025, 1, 1),
      );

      bridgeStream.add(event);
      await Future<void>.delayed(Duration.zero);

      expect(events.single.label, 'snare');
      expect(events.single.confidence, 0.87);
      expect(events.single.timestamp, DateTime(2025, 1, 1));

      await sub.cancel();
      await bridgeStream.close();
    });

    test('surfaces start failures when bridge rejects start', () async {
      bridge.initializeReturn = true;
      bridge.startReturn = false;
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(state: PermissionState.granted),
      );

      final service = buildService();
      final result = await service.start(bpm: 120);

      expect(result.success, isFalse);
      expect(result.error, AudioErrorCode.startFailed);
      expect(service.state, AudioState.idle);
      expect(bridge.initializeCalls, 1);
      expect(bridge.startCalls, 1);
      verify(() => translator.translate(any())).called(1);
    });

    test('uses bridge setBpm when running', () async {
      bridge.initializeReturn = true;
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(state: PermissionState.granted),
      );

      final service = buildService();
      await service.start(bpm: 120);
      final result = await service.setBpm(140);

      expect(result.success, isTrue);
      expect(bridge.setBpmCalls, 1);
      expect(service.state, AudioState.running);
    });

    test('surfaces stop errors when bridge rejects stop', () async {
      bridge.initializeReturn = true;
      bridge.stopReturn = false;
      when(() => permission.requestMicrophone()).thenAnswer(
        (_) async => const PermissionResult(state: PermissionState.granted),
      );

      final service = buildService();
      await service.start(bpm: 120);
      clearInteractions(translator);

      final result = await service.stop();

      expect(result.success, isFalse);
      expect(result.error, AudioErrorCode.stopFailed);
      expect(service.state, AudioState.idle);
      verify(() => translator.translate(any())).called(1);
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
