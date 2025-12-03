/// Performance threshold constants.
///
/// Use these constants for consistent threshold values
/// throughout the application.
library;

/// Threshold constants for performance monitoring and validation.
///
/// These values define warning levels, iteration bounds, and event limits
/// used by the debugger, benchmark, and typing simulator pages.
abstract class ThresholdConstants {
  /// Latency warning threshold in microseconds (20ms).
  ///
  /// Latencies at or above this value are considered high
  /// and displayed in red.
  static const int latencyWarningUs = 20000;

  /// Latency caution threshold in microseconds (10ms).
  ///
  /// Latencies at or above this value (but below warning)
  /// are displayed in orange as a caution indicator.
  static const int latencyCautionUs = 10000;

  /// Warning threshold in nanoseconds (1ms).
  ///
  /// Used by the benchmark page to flag high latency results.
  static const int warningThresholdNs = 1000000;

  /// Minimum number of valid keystrokes required for typing analysis.
  ///
  /// If fewer than this many keystrokes are recorded, the typing
  /// simulation is considered invalid.
  static const int minKeystrokes = 10;

  /// Pause detection threshold in milliseconds (2 seconds).
  ///
  /// Inter-key delays exceeding this value are considered pauses
  /// and excluded from typing speed calculations.
  static const int pauseThresholdMs = 2000;

  /// Maximum number of events to keep in the debugger history.
  ///
  /// Events beyond this limit are discarded to prevent memory growth.
  static const int maxEventsHistory = 300;

  /// Minimum benchmark iterations.
  static const int minIterations = 1000;

  /// Maximum benchmark iterations.
  static const int maxIterations = 100000;
}
