/// Event type codes for unified event callback registration.
///
/// These codes must match the EventType enum in core/src/ffi/events.rs
library;

/// Event types that can be registered for callbacks.
enum EventType {
  discoveryProgress(0),
  discoveryDuplicate(1),
  discoverySummary(2),
  engineState(3),
  validationProgress(4),
  validationResult(5),
  deviceConnected(6),
  deviceDisconnected(7),
  testProgress(8),
  testResult(9),
  analysisProgress(10),
  analysisResult(11),
  diagnosticsLog(12),
  diagnosticsMetric(13),
  recordingStarted(14),
  recordingStopped(15),
  rawInput(16),
  rawOutput(17);

  const EventType(this.code);

  /// The integer code for this event type.
  final int code;
}
