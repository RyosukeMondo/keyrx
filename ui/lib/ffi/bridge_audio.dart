/// Audio capture FFI methods.
///
/// Provides audio capture start/stop, BPM control, and classification stream
/// for the KeyRx bridge.
library;

import 'dart:async';
import 'dart:convert';

import 'bindings.dart';

/// Simple payload representing a classification event emitted by the bridge.
class BridgeClassification {
  const BridgeClassification({
    required this.label,
    required this.confidence,
    required this.timestamp,
  });

  final String label;
  final double confidence;
  final DateTime timestamp;
}

/// Mixin providing audio capture FFI methods.
mixin BridgeAudioMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;
  StreamController<BridgeClassification>? get classificationController;

  /// Subscribe to classification events if the native layer exposes them.
  Stream<BridgeClassification>? get classificationStream =>
      classificationController?.stream;

  /// Start audio capture/processing via the native bridge.
  ///
  /// Returns `true` when the bridge reports success.
  Future<bool> startAudio({required int bpm}) async {
    final start = bindings?.startAudio;
    if (start == null) {
      return false;
    }
    return start(bpm) == 0;
  }

  /// Stop audio capture/processing via the native bridge.
  Future<bool> stopAudio() async {
    final stop = bindings?.stopAudio;
    if (stop == null) {
      return false;
    }
    return stop() == 0;
  }

  /// Update BPM on the native engine.
  Future<bool> setBpm(int bpm) async {
    final setter = bindings?.setBpm;
    if (setter == null) {
      return false;
    }
    return setter(bpm) == 0;
  }
}

/// Extension for classification stream parsing.
extension BridgeAudioClassificationParser on BridgeAudioMixin {
  /// Parse classification payload from JSON bytes.
  static BridgeClassification? parseClassificationPayload(List<int> bytes) {
    try {
      final payload = json.decode(utf8.decode(bytes));
      if (payload is! Map<String, dynamic>) return null;

      final label = payload['label'] as String? ?? 'unknown';
      final confidence = (payload['confidence'] as num?)?.toDouble() ?? 0.0;
      final timestampMs = (payload['timestamp'] as num?)?.toInt() ??
          DateTime.now().millisecondsSinceEpoch;

      return BridgeClassification(
        label: label,
        confidence: confidence,
        timestamp: DateTime.fromMillisecondsSinceEpoch(timestampMs),
      );
    } catch (_) {
      return null;
    }
  }
}
