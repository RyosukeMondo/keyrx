// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'analytics_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$sharedPreferencesHash() => r'd22b545aefe95500327f9dce52c645d746349271';

/// Provider for SharedPreferences instance.
///
/// Copied from [sharedPreferences].
@ProviderFor(sharedPreferences)
final sharedPreferencesProvider = FutureProvider<SharedPreferences>.internal(
  sharedPreferences,
  name: r'sharedPreferencesProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$sharedPreferencesHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef SharedPreferencesRef = FutureProviderRef<SharedPreferences>;
String _$keyFlowEnabledHash() => r'de048c7b119a60575227a9e2f117d935770f19aa';

/// Provider for checking if KeyFlow is enabled.
///
/// Copied from [keyFlowEnabled].
@ProviderFor(keyFlowEnabled)
final keyFlowEnabledProvider = AutoDisposeFutureProvider<bool>.internal(
  keyFlowEnabled,
  name: r'keyFlowEnabledProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$keyFlowEnabledHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef KeyFlowEnabledRef = AutoDisposeFutureProviderRef<bool>;
String _$keyFlowAnalyticsEnabledHash() =>
    r'0267444e4e26c8755843c96daed7ea8354f7d4ab';

/// Provider for checking if KeyFlow analytics is enabled.
///
/// Copied from [keyFlowAnalyticsEnabled].
@ProviderFor(keyFlowAnalyticsEnabled)
final keyFlowAnalyticsEnabledProvider =
    AutoDisposeFutureProvider<bool>.internal(
      keyFlowAnalyticsEnabled,
      name: r'keyFlowAnalyticsEnabledProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$keyFlowAnalyticsEnabledHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef KeyFlowAnalyticsEnabledRef = AutoDisposeFutureProviderRef<bool>;
String _$keyFlowAnalyticsHash() => r'3210f2b91c38e19679545ea55daf03a3f1dafe66';

/// Provider for KeyFlowAnalytics service.
///
/// Copied from [keyFlowAnalytics].
@ProviderFor(keyFlowAnalytics)
final keyFlowAnalyticsProvider = Provider<KeyFlowAnalytics>.internal(
  keyFlowAnalytics,
  name: r'keyFlowAnalyticsProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$keyFlowAnalyticsHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef KeyFlowAnalyticsRef = ProviderRef<KeyFlowAnalytics>;
String _$featureFlagServiceNotifierHash() =>
    r'2483585b37d24786d8371744a8bca13a68bca9a9';

/// Provider for FeatureFlagService.
///
/// Copied from [FeatureFlagServiceNotifier].
@ProviderFor(FeatureFlagServiceNotifier)
final featureFlagServiceNotifierProvider =
    AsyncNotifierProvider<
      FeatureFlagServiceNotifier,
      FeatureFlagService
    >.internal(
      FeatureFlagServiceNotifier.new,
      name: r'featureFlagServiceNotifierProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$featureFlagServiceNotifierHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$FeatureFlagServiceNotifier = AsyncNotifier<FeatureFlagService>;
String _$analyticsTrackerHash() => r'b345e4e36714aa72a11df15cc99d60a1f33b1c15';

/// Provider for tracking analytics (only if enabled).
///
/// Copied from [AnalyticsTracker].
@ProviderFor(AnalyticsTracker)
final analyticsTrackerProvider =
    AutoDisposeAsyncNotifierProvider<AnalyticsTracker, bool>.internal(
      AnalyticsTracker.new,
      name: r'analyticsTrackerProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$analyticsTrackerHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$AnalyticsTracker = AutoDisposeAsyncNotifier<bool>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
