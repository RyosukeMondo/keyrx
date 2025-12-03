/// UI dimension and spacing constants.
///
/// This file centralizes all padding, margin, elevation, scale, and other
/// dimension-related constants used throughout the app.
library;

/// Centralized UI dimension constants for consistent spacing and sizing.
///
/// Use these constants to ensure consistent visual styling across the app.
/// All dimension values are in logical pixels (doubles) for Flutter widgets.
abstract final class UiConstants {
  // ─────────────────────────────────────────────────────────────────────────
  // Padding and Spacing
  // ─────────────────────────────────────────────────────────────────────────

  /// Default padding for containers and surfaces (16.0px).
  ///
  /// Used as the standard internal spacing for cards, dialogs, and sections.
  static const double defaultPadding = 16.0;

  /// Small padding for tighter spacing (8.0px).
  ///
  /// Used for compact layouts and secondary spacing between related elements.
  static const double smallPadding = 8.0;

  /// Tiny padding for minimal spacing (4.0px).
  ///
  /// Used for very compact elements like chip contents or dense lists.
  static const double tinyPadding = 4.0;

  /// Default margin between surfaces and containers (12.0px).
  ///
  /// Used for separation between major UI sections.
  static const double defaultMargin = 12.0;

  // ─────────────────────────────────────────────────────────────────────────
  // Elevation and Shadows
  // ─────────────────────────────────────────────────────────────────────────

  /// Default elevation for surfaces (6.0).
  ///
  /// Provides subtle depth without being too prominent.
  static const double defaultElevation = 6.0;

  /// Elevated state for interactive elements (4.0).
  ///
  /// Used for drag feedback and hover states.
  static const double dragFeedbackElevation = 4.0;

  // ─────────────────────────────────────────────────────────────────────────
  // Border Radius
  // ─────────────────────────────────────────────────────────────────────────

  /// Default border radius for rounded corners (4.0px).
  ///
  /// Used for standard button and input field corners.
  static const double defaultBorderRadius = 4.0;

  /// Key border radius for keyboard keys (6.0px).
  ///
  /// Slightly larger radius for keyboard key styling.
  static const double keyBorderRadius = 6.0;

  /// Surface border radius for containers (12.0px).
  ///
  /// Larger radius for cards and surface containers.
  static const double surfaceBorderRadius = 12.0;

  // ─────────────────────────────────────────────────────────────────────────
  // Scale Bounds
  // ─────────────────────────────────────────────────────────────────────────

  /// Minimum scale factor for keyboard rendering (0.5).
  ///
  /// Prevents the visual keyboard from becoming too small to be usable.
  static const double minKeyboardScale = 0.5;

  /// Maximum scale factor for keyboard rendering (1.0).
  ///
  /// Full size is the maximum to prevent pixelation.
  static const double maxKeyboardScale = 1.0;

  // ─────────────────────────────────────────────────────────────────────────
  // Icon Sizes
  // ─────────────────────────────────────────────────────────────────────────

  /// Default icon size for standard icons (24.0px).
  ///
  /// Standard Material Design icon size.
  static const double defaultIconSize = 24.0;

  /// Small icon size for compact areas (16.0px).
  ///
  /// Used in dense list tiles and secondary actions.
  static const double smallIconSize = 16.0;

  // ─────────────────────────────────────────────────────────────────────────
  // Border Widths
  // ─────────────────────────────────────────────────────────────────────────

  /// Standard border width (1.0px).
  ///
  /// Used for container borders and dividers.
  static const double defaultBorderWidth = 1.0;

  /// Thick border width for emphasis (2.0px).
  ///
  /// Used for selected states and drag targets.
  static const double thickBorderWidth = 2.0;
}
