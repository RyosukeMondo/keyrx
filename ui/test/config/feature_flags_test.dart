import 'package:core_logic/utils/logger.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/config/feature_flags.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  group('FeatureFlagService', () {
    late FeatureFlagService service;
    late List<String> logOutput;

    setUp(() async {
      SharedPreferences.setMockInitialValues({});
      logOutput = [];
      final logger = getLogger(
        'test_feature_flags',
        sink: (line) => logOutput.add(line),
      );
      service = FeatureFlagService(logger: logger);
      await service.initialize();
    });

    tearDown(() async {
      await LoggingConfigurator.resetForTesting();
    });

    test('initializes with default values', () {
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);
      expect(service.isEnabled(FeatureFlag.keyFlowAnalytics), isTrue);
      expect(service.isEnabled(FeatureFlag.keyFlowOnboarding), isTrue);
      expect(service.isEnabled(FeatureFlag.keyFlowFeatureTour), isTrue);
    });

    test('returns default when not initialized', () {
      final uninitializedService = FeatureFlagService();
      expect(
        uninitializedService.isEnabled(FeatureFlag.keyFlowEnabled),
        isTrue,
      );
    });

    test('sets and persists flag value', () async {
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);

      // Create new service instance to verify persistence
      final prefs = await SharedPreferences.getInstance();
      final newService = FeatureFlagService(preferences: prefs);
      await newService.initialize();
      expect(newService.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);
    });

    test('resets flag to default', () async {
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);

      await service.resetFlag(FeatureFlag.keyFlowEnabled);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);
    });

    test('resets all flags to defaults', () async {
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      await service.setFlag(FeatureFlag.keyFlowAnalytics, false);
      await service.setFlag(FeatureFlag.keyFlowOnboarding, false);

      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);
      expect(service.isEnabled(FeatureFlag.keyFlowAnalytics), isFalse);
      expect(service.isEnabled(FeatureFlag.keyFlowOnboarding), isFalse);

      await service.resetAllFlags();

      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);
      expect(service.isEnabled(FeatureFlag.keyFlowAnalytics), isTrue);
      expect(service.isEnabled(FeatureFlag.keyFlowOnboarding), isTrue);
    });

    test('loads overrides from SharedPreferences', () async {
      SharedPreferences.setMockInitialValues({
        'feature_flag_keyFlowEnabled': false,
        'feature_flag_keyFlowAnalytics': false,
      });

      final prefs = await SharedPreferences.getInstance();
      final newService = FeatureFlagService(preferences: prefs);
      await newService.initialize();

      expect(newService.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);
      expect(newService.isEnabled(FeatureFlag.keyFlowAnalytics), isFalse);
      expect(newService.isEnabled(FeatureFlag.keyFlowOnboarding), isTrue);
    });

    test('getDefault returns default value', () {
      expect(service.getDefault(FeatureFlag.keyFlowEnabled), isTrue);
      expect(service.getDefault(FeatureFlag.keyFlowAnalytics), isTrue);
    });

    test('hasOverride checks for runtime override', () async {
      expect(service.hasOverride(FeatureFlag.keyFlowEnabled), isFalse);

      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(service.hasOverride(FeatureFlag.keyFlowEnabled), isTrue);

      await service.resetFlag(FeatureFlag.keyFlowEnabled);
      expect(service.hasOverride(FeatureFlag.keyFlowEnabled), isFalse);
    });

    test('handles multiple flag changes', () async {
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);

      await service.setFlag(FeatureFlag.keyFlowEnabled, true);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);

      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isFalse);
    });

    test('logs flag changes', () async {
      logOutput.clear();
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);

      await Future.delayed(const Duration(milliseconds: 10));

      expect(
        logOutput.any((line) => line.contains('feature_flags.flag_set')),
        isTrue,
      );
    });

    test('logs flag resets', () async {
      await service.setFlag(FeatureFlag.keyFlowEnabled, false);
      logOutput.clear();

      await service.resetFlag(FeatureFlag.keyFlowEnabled);
      await Future.delayed(const Duration(milliseconds: 10));

      expect(
        logOutput.any((line) => line.contains('feature_flags.flag_reset')),
        isTrue,
      );
    });

    test('handles SharedPreferences errors gracefully', () async {
      // Service should still work with defaults even if SharedPreferences fails
      final serviceWithoutPrefs = FeatureFlagService();
      await serviceWithoutPrefs.initialize();

      expect(serviceWithoutPrefs.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);
    });

    test('multiple initializations are safe', () async {
      await service.initialize();
      await service.initialize();
      await service.initialize();

      expect(service.isEnabled(FeatureFlag.keyFlowEnabled), isTrue);
    });

    test('all feature flags have defaults', () {
      for (final flag in FeatureFlag.values) {
        expect(service.getDefault(flag), isNotNull);
      }
    });

    test('setting flag before initialization works', () async {
      final uninitializedService = FeatureFlagService();
      await uninitializedService.setFlag(FeatureFlag.keyFlowEnabled, false);
      expect(
        uninitializedService.isEnabled(FeatureFlag.keyFlowEnabled),
        isFalse,
      );
    });
  });
}
