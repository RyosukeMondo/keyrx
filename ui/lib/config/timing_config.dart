/// Animation and debounce timing constants.
///
/// This file centralizes all timing-related constants for UI animations,
/// debounce intervals, and transition durations used throughout the app.
library;

/// Centralized timing constants for animations and debounce.
///
/// Use these constants to ensure consistent animation timing across the app.
/// All values are in milliseconds unless otherwise noted.
abstract final class TimingConfig {
  /// Standard animation duration for UI transitions (150ms).
  ///
  /// Used for most animated containers, scale transitions, and state changes.
  /// This provides a quick, responsive feel without being jarring.
  static const int animationDurationMs = 150;

  /// Pulse animation duration for visual feedback (300ms).
  ///
  /// Used for pulse effects that highlight state changes in the debugger
  /// and other areas requiring attention.
  static const int pulseAnimationMs = 300;

  /// Debounce interval for input validation (500ms).
  ///
  /// Used to delay validation until user stops typing, preventing
  /// excessive validation calls during rapid input.
  static const int debounceMs = 500;

  /// Key animation duration for keyboard visuals (100ms).
  ///
  /// Shorter duration for responsive key press/release animations
  /// on the visual keyboard.
  static const int keyAnimationMs = 100;

  /// Typing simulator time limit in seconds (30s).
  ///
  /// Duration allowed for the typing speed test simulation.
  static const int typingTimeLimitSec = 30;

  /// Training screen animation duration (400ms).
  ///
  /// Slightly longer animation for training screen transitions
  /// to provide a smoother, more deliberate feel.
  static const int trainingAnimationMs = 400;

  /// Tooltip delay before showing (150ms).
  ///
  /// Used for keyboard key tooltips to prevent flickering
  /// during rapid mouse movements.
  static const int tooltipDelayMs = 150;
}
