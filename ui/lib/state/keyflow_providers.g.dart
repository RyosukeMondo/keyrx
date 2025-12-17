// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'keyflow_providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$keyFlowRepositoryHash() => r'655fe15ae7fc5241aa3850e56ba45125ff7c39c2';

/// See also [keyFlowRepository].
@ProviderFor(keyFlowRepository)
final keyFlowRepositoryProvider = Provider<KeyFlowRepository>.internal(
  keyFlowRepository,
  name: r'keyFlowRepositoryProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$keyFlowRepositoryHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef KeyFlowRepositoryRef = ProviderRef<KeyFlowRepository>;
String _$runtimeConfigHash() => r'2fe5217bad1e1fd3448c41f5123454ffcb27d510';

/// See also [runtimeConfig].
@ProviderFor(runtimeConfig)
final runtimeConfigProvider = AutoDisposeFutureProvider<RuntimeConfig>.internal(
  runtimeConfig,
  name: r'runtimeConfigProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$runtimeConfigHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef RuntimeConfigRef = AutoDisposeFutureProviderRef<RuntimeConfig>;
String _$flowGraphHash() => r'dcd4a05069a47787cb7912d759a2b0153614a5c3';

/// See also [flowGraph].
@ProviderFor(flowGraph)
final flowGraphProvider = AutoDisposeFutureProvider<FlowGraph>.internal(
  flowGraph,
  name: r'flowGraphProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$flowGraphHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef FlowGraphRef = AutoDisposeFutureProviderRef<FlowGraph>;
String _$deviceStatesHash() => r'a826829e82fbf1fc41c9c85d30701ba0d03c81c7';

/// Provider for device registry states (device enabled/disabled toggles).
///
/// Copied from [deviceStates].
@ProviderFor(deviceStates)
final deviceStatesProvider =
    AutoDisposeStreamProvider<List<DeviceState>>.internal(
      deviceStates,
      name: r'deviceStatesProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$deviceStatesHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef DeviceStatesRef = AutoDisposeStreamProviderRef<List<DeviceState>>;
String _$keyFlowActionsHash() => r'554164098f039b1852574c00311f10c4bb603837';

/// See also [keyFlowActions].
@ProviderFor(keyFlowActions)
final keyFlowActionsProvider = AutoDisposeProvider<KeyFlowActions>.internal(
  keyFlowActions,
  name: r'keyFlowActionsProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$keyFlowActionsHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef KeyFlowActionsRef = AutoDisposeProviderRef<KeyFlowActions>;
String _$availableHardwareProfilesHash() =>
    r'845a4a0a9eca045e1ca87ea6a80ea892b91f4950';

/// Provider for available hardware profiles.
///
/// Copied from [availableHardwareProfiles].
@ProviderFor(availableHardwareProfiles)
final availableHardwareProfilesProvider =
    AutoDisposeFutureProvider<List<HardwareProfile>>.internal(
      availableHardwareProfiles,
      name: r'availableHardwareProfilesProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$availableHardwareProfilesHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef AvailableHardwareProfilesRef =
    AutoDisposeFutureProviderRef<List<HardwareProfile>>;
String _$availableKeymapsHash() => r'b497c50f90c2c16118975cfe892e9db11ac33f8a';

/// Provider for available keymaps.
///
/// Copied from [availableKeymaps].
@ProviderFor(availableKeymaps)
final availableKeymapsProvider =
    AutoDisposeFutureProvider<List<Keymap>>.internal(
      availableKeymaps,
      name: r'availableKeymapsProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$availableKeymapsHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
typedef AvailableKeymapsRef = AutoDisposeFutureProviderRef<List<Keymap>>;
String _$runtimeConfigNotifierHash() =>
    r'cf899419869ce2db7fce829acd2aea4eb80ad4f2';

/// See also [RuntimeConfigNotifier].
@ProviderFor(RuntimeConfigNotifier)
final runtimeConfigNotifierProvider =
    AsyncNotifierProvider<RuntimeConfigNotifier, RuntimeConfig>.internal(
      RuntimeConfigNotifier.new,
      name: r'runtimeConfigNotifierProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$runtimeConfigNotifierHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$RuntimeConfigNotifier = AsyncNotifier<RuntimeConfig>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
