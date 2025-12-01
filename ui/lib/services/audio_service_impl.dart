import 'dart:async';
import 'dart:developer';

import '../ffi/bridge.dart';
import 'audio_service.dart';
import 'error_translator.dart';
import 'permission_service.dart';

/// Real AudioService implementation that wraps the FFI bridge.
class AudioServiceImpl implements AudioService {
  AudioServiceImpl({
    required KeyrxBridge bridge,
    required PermissionService permissionService,
    required ErrorTranslator errorTranslator,
    Stream<ClassificationResult>? classificationSource,
  }) : _bridge = bridge,
       _permissionService = permissionService,
       _errorTranslator = errorTranslator {
    _controller = StreamController<ClassificationResult>.broadcast(
      onListen: _handleStreamListen,
      onCancel: _handleStreamCancel,
    );

    final stream =
        classificationSource ?? _bridge.classificationStream?.map(_mapEvent);
    if (stream != null) {
      _attachClassificationSource(stream);
    }
  }

  final KeyrxBridge _bridge;
  final PermissionService _permissionService;
  final ErrorTranslator _errorTranslator;

  late final StreamController<ClassificationResult> _controller;
  StreamSubscription<ClassificationResult>? _classificationSubscription;
  AudioState _state = AudioState.idle;
  bool _initialized = false;

  @override
  AudioState get state => _state;

  @override
  Stream<ClassificationResult> get classificationStream => _controller.stream;

  Future<bool> _ensureInitialized() async {
    if (_initialized) return true;
    _initialized = _bridge.initialize();
    if (!_initialized) {
      _trace('audio.init.failed', {});
    }
    return _initialized;
  }

  @override
  Future<AudioOperationResult> start({required int bpm}) async {
    if (bpm <= 0) {
      return _errorResult(
        AudioErrorCode.invalidBpm,
        ArgumentError.value(bpm, 'bpm', 'BPM must be positive'),
      );
    }

    if (_state == AudioState.running || _state == AudioState.starting) {
      return const AudioOperationResult(success: true);
    }

    _state = AudioState.starting;
    final stopwatch = Stopwatch()..start();

    try {
      final permission = await _permissionService.requestMicrophone();
      if (!permission.isGranted) {
        _state = AudioState.idle;
        return _errorResult(
          AudioErrorCode.permissionDenied,
          PermissionDeniedError(
            permission.state,
            shouldShowRationale: permission.shouldShowRationale,
          ),
        );
      }

      final initOk = await _ensureInitialized();
      if (!initOk) {
        _state = AudioState.idle;
        return _errorResult(
          AudioErrorCode.notInitialized,
          StateError('Engine failed to initialize'),
        );
      }

      final started = await _bridge.startAudio(bpm: bpm);
      if (!started) {
        _state = AudioState.idle;
        return _errorResult(
          AudioErrorCode.startFailed,
          StateError('Engine rejected start'),
        );
      }

      _trace('audio.start', {'bpm': bpm});
      _state = AudioState.running;
      return const AudioOperationResult(success: true);
    } catch (e, st) {
      _state = AudioState.idle;
      log('audio.start failed', error: e, stackTrace: st);
      return _errorResult(AudioErrorCode.startFailed, e);
    } finally {
      stopwatch.stop();
      _trace('audio.start.complete', {
        'state': _state.name,
        'ms': stopwatch.elapsedMilliseconds,
      });
    }
  }

  @override
  Future<AudioOperationResult> stop() async {
    if (_state == AudioState.idle) {
      return const AudioOperationResult(success: true);
    }

    _state = AudioState.stopping;
    final stopwatch = Stopwatch()..start();

    try {
      _trace('audio.stop', {});
      final stopped = await _bridge.stopAudio();
      if (!stopped) {
        _state = AudioState.idle;
        return _errorResult(
          AudioErrorCode.stopFailed,
          StateError('Engine rejected stop'),
        );
      }

      _state = AudioState.idle;
      return const AudioOperationResult(success: true);
    } catch (e, st) {
      _state = AudioState.idle;
      log('audio.stop failed', error: e, stackTrace: st);
      return _errorResult(AudioErrorCode.stopFailed, e);
    } finally {
      stopwatch.stop();
      _trace('audio.stop.complete', {
        'state': _state.name,
        'ms': stopwatch.elapsedMilliseconds,
      });
    }
  }

  @override
  Future<AudioOperationResult> setBpm(int bpm) async {
    if (bpm <= 0) {
      return _errorResult(
        AudioErrorCode.invalidBpm,
        ArgumentError.value(bpm, 'bpm', 'BPM must be positive'),
      );
    }

    if (_state != AudioState.running) {
      return _errorResult(
        AudioErrorCode.notInitialized,
        StateError('Cannot set BPM while audio is not running'),
      );
    }

    try {
      _trace('audio.setBpm', {'bpm': bpm});
      final updated = await _bridge.setBpm(bpm);
      if (!updated) {
        return _errorResult(
          AudioErrorCode.invalidBpm,
          StateError('Engine rejected BPM update'),
        );
      }
      return const AudioOperationResult(success: true);
    } catch (e, st) {
      log('audio.setBpm failed', error: e, stackTrace: st);
      return _errorResult(AudioErrorCode.invalidBpm, e);
    }
  }

  @override
  Future<void> dispose() async {
    await _classificationSubscription?.cancel();
    await _controller.close();
    _trace('audio.dispose', {});
  }

  void _attachClassificationSource(Stream<ClassificationResult> source) {
    _classificationSubscription = source.listen(
      (event) {
        _controller.add(event);
      },
      onError: (error, stackTrace) {
        log(
          'classification stream error',
          error: error,
          stackTrace: stackTrace,
        );
        _controller.addError(error, stackTrace);
        _trace('audio.stream.error', {'error': error.toString()});
      },
      onDone: () => _trace('audio.stream.done', {}),
      cancelOnError: false,
    );
  }

  ClassificationResult _mapEvent(BridgeClassification event) {
    return ClassificationResult(
      label: event.label,
      confidence: event.confidence,
      timestamp: event.timestamp,
    );
  }

  AudioOperationResult _errorResult(AudioErrorCode code, Object error) {
    return AudioOperationResult(
      success: false,
      error: code,
      userMessage: _errorTranslator.translate(
        error is AudioFailure ? error : AudioFailure(code, cause: error),
      ),
    );
  }

  void _handleStreamListen() {
    _trace('audio.stream.listen', {
      'subscribers': _controller.hasListener ? 1 : 0,
    });
  }

  void _handleStreamCancel() {
    _trace('audio.stream.cancel', {'closed': _controller.isClosed});
  }

  void _trace(String event, Map<String, Object?> payload) {
    log(event, name: 'audio_service', error: payload.isEmpty ? null : payload);
  }
}
