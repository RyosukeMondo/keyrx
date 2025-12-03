/// File path constants.
///
/// Use these constants for consistent path strings
/// throughout the application.
library;

/// Path constants for script files and directories.
///
/// Centralizes all path-related strings to ensure consistency
/// and make path changes easy to manage.
abstract final class PathConstants {
  /// Default path for the generated script file.
  ///
  /// Used when saving a script from the visual editor without
  /// specifying a custom path.
  static const String defaultScriptPath = 'scripts/generated.rhai';

  /// Default file name for new configurations.
  ///
  /// Used as the initial filename suggestion in the visual editor.
  static const String defaultConfigFileName = 'config.rhai';

  /// Scripts directory path.
  ///
  /// The standard directory where script files are stored.
  static const String scriptsDir = 'scripts/';

  /// Example/hint path for script dialogs.
  ///
  /// Shown in text field hints to guide users on path format.
  static const String scriptPathHint = 'scripts/my-config.rhai';

  /// Temporary path for script validation.
  ///
  /// Used when validating scripts before saving to the final location.
  static const String tempValidationPath = '/tmp/keyrx_validation.rhai';
}
