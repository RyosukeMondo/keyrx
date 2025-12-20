/// Feature flags configuration for controlling feature rollout.
library;

import 'package:core_logic/utils/logger.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Feature flag identifiers.
enum FeatureFlag {
  /// Enable KeyFlow visualization page
  keyFlowEnabled,

  /// Enable KeyFlow analytics tracking
  keyFlowAnalytics,

  /// Enable KeyFlow onboarding dialog
  keyFlowOnboarding,

  /// Enable KeyFlow feature tour
  keyFlowFeatureTour,
}

/// Service for managing feature flags.
///
/// Feature flags can be:
/// - Set at compile time via const defaults
/// - Overridden at runtime via SharedPreferences
/// - Queried by any component to check feature availability
///
/// This enables gradual feature rollout and A/B testing.
class FeatureFlagService {
  FeatureFlagService({StructuredLogger? logger, SharedPreferences? preferences})
    : _logger = logger ?? getLogger('feature_flags'),
      _preferences = preferences;

  final StructuredLogger _logger;
  SharedPreferences? _preferences;
  bool _initialized = false;

  /// Default flag values (compile-time defaults).
  static const _defaults = <FeatureFlag, bool>{
    FeatureFlag.keyFlowEnabled: true,
    FeatureFlag.keyFlowAnalytics: true,
    FeatureFlag.keyFlowOnboarding: true,
    FeatureFlag.keyFlowFeatureTour: true,
  };

  /// Runtime overrides loaded from SharedPreferences.
  final Map<FeatureFlag, bool> _overrides = {};

  /// Initialize the feature flag service.
  ///
  /// Must be called before using any flags.
  Future<void> initialize() async {
    if (_initialized) return;

    try {
      _preferences ??= await SharedPreferences.getInstance();
      _loadOverrides();
      _initialized = true;

      _logger.info('feature_flags.init', context: {'flags': _getAllFlags()});
    } on Exception catch (e, st) {
      _logger.error(
        'feature_flags.init_failed',
        message: 'Failed to initialize feature flags',
        error: e,
        stackTrace: st,
      );
      _initialized = true; // Continue with defaults
    }
  }

  /// Load flag overrides from SharedPreferences.
  void _loadOverrides() {
    if (_preferences == null) return;

    for (final flag in FeatureFlag.values) {
      final key = _getKey(flag);
      if (_preferences!.containsKey(key)) {
        _overrides[flag] = _preferences!.getBool(key) ?? _defaults[flag]!;
      }
    }

    _logger.debug(
      'feature_flags.overrides_loaded',
      context: {'count': _overrides.length},
    );
  }

  /// Check if a feature flag is enabled.
  ///
  /// Returns the runtime override if present, otherwise the default value.
  bool isEnabled(FeatureFlag flag) {
    if (!_initialized) {
      _logger.warn(
        'feature_flags.not_initialized',
        message: 'Feature flags not initialized, using defaults',
      );
      return _defaults[flag] ?? false;
    }

    return _overrides[flag] ?? _defaults[flag] ?? false;
  }

  /// Set a feature flag value at runtime.
  ///
  /// The value is persisted to SharedPreferences.
  Future<void> setFlag(FeatureFlag flag, bool value) async {
    if (!_initialized) {
      await initialize();
    }

    try {
      final key = _getKey(flag);
      await _preferences?.setBool(key, value);
      _overrides[flag] = value;

      _logger.info(
        'feature_flags.flag_set',
        context: {'flag': flag.name, 'value': value},
      );
    } on Exception catch (e, st) {
      _logger.error(
        'feature_flags.set_failed',
        message: 'Failed to set feature flag',
        error: e,
        stackTrace: st,
        context: {'flag': flag.name, 'value': value},
      );
    }
  }

  /// Reset a feature flag to its default value.
  Future<void> resetFlag(FeatureFlag flag) async {
    if (!_initialized) {
      await initialize();
    }

    try {
      final key = _getKey(flag);
      await _preferences?.remove(key);
      _overrides.remove(flag);

      _logger.info('feature_flags.flag_reset', context: {'flag': flag.name});
    } on Exception catch (e, st) {
      _logger.error(
        'feature_flags.reset_failed',
        message: 'Failed to reset feature flag',
        error: e,
        stackTrace: st,
        context: {'flag': flag.name},
      );
    }
  }

  /// Reset all feature flags to defaults.
  Future<void> resetAllFlags() async {
    if (!_initialized) {
      await initialize();
    }

    try {
      for (final flag in FeatureFlag.values) {
        final key = _getKey(flag);
        await _preferences?.remove(key);
      }
      _overrides.clear();

      _logger.info('feature_flags.all_reset');
    } on Exception catch (e, st) {
      _logger.error(
        'feature_flags.reset_all_failed',
        message: 'Failed to reset all feature flags',
        error: e,
        stackTrace: st,
      );
    }
  }

  /// Get all feature flags with their current values.
  Map<String, bool> _getAllFlags() {
    final result = <String, bool>{};
    for (final flag in FeatureFlag.values) {
      result[flag.name] = isEnabled(flag);
    }
    return result;
  }

  /// Get SharedPreferences key for a flag.
  String _getKey(FeatureFlag flag) => 'feature_flag_${flag.name}';

  /// Get default value for a flag.
  bool getDefault(FeatureFlag flag) => _defaults[flag] ?? false;

  /// Check if a flag has a runtime override.
  bool hasOverride(FeatureFlag flag) => _overrides.containsKey(flag);
}
