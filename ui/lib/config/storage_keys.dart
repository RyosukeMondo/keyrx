/// Centralized SharedPreferences key constants.
///
/// All storage keys used for persistent application state
/// are defined here to ensure consistency and prevent typos.
library;

/// Storage key constants for SharedPreferences.
///
/// This abstract class prevents instantiation and provides
/// a namespace for all persistent storage keys.
abstract class StorageKeys {
  /// Key for storing developer mode enabled state.
  ///
  /// Stores a boolean indicating whether developer mode is active.
  /// Used by [AppState] to persist the developer mode toggle.
  static const String developerModeKey = 'developer_mode';

  /// Key for storing migration completion state.
  ///
  /// Stores a boolean indicating whether V1 to V2 migration has been completed.
  /// Used to determine if migration prompt should be shown on startup.
  static const String migrationCompletedKey = 'migration_v1_to_v2_completed';
}
