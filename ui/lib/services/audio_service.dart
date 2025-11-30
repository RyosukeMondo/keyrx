/// Contracts for audio interactions exposed to the Flutter UI.
///
/// Implementations wrap FFI and platform concerns while remaining
/// replaceable in tests.
library;

import 'error_translator.dart';

/// Lifecycle states exposed by the audio engine.
enum AudioState { idle, starting, running, stopping }

/// Structured error categories surfaced to the UI.
enum AudioErrorCode {
  notInitialized,
  startFailed,
  stopFailed,
  invalidBpm,
  streamUnavailable,
  permissionDenied,
}

/// Result model for audio operations.
class AudioOperationResult {
  const AudioOperationResult({
    required this.success,
    this.error,
    this.userMessage,
  });

  final bool success;
  final AudioErrorCode? error;
  final UserMessage? userMessage;

  bool get hasError => !success || error != null;
}

/// Classification payload emitted by the engine.
class ClassificationResult {
  const ClassificationResult({
    required this.label,
    required this.confidence,
    required this.timestamp,
  });

  final String label;
  final double confidence;
  final DateTime timestamp;
}

/// Audio service abstraction to start/stop audio processing
/// and consume classification results.
abstract class AudioService {
  /// Current engine state.
  AudioState get state;

  /// Stream of classification results from the engine.
  Stream<ClassificationResult> get classificationStream;

  /// Initialize and start audio capture/processing at the given BPM.
  Future<AudioOperationResult> start({required int bpm});

  /// Stop audio capture/processing.
  Future<AudioOperationResult> stop();

  /// Adjust BPM while running; may return an error if invalid.
  Future<AudioOperationResult> setBpm(int bpm);

  /// Dispose any underlying resources or subscriptions.
  Future<void> dispose();
}
