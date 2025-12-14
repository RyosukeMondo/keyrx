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
String _$runtimeConfigHash() => r'0ef28adbb533d8ba8f07e99a7144a6761b13bdf5';

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
String _$keyFlowActionsHash() => r'a6e8e0db1afa49cb89a695ef34b2e8e521cad85e';

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
String _$nodePositionsHash() => r'0df06ebf5c444c946c5bb23edf15df5041768d8c';

/// See also [NodePositions].
@ProviderFor(NodePositions)
final nodePositionsProvider =
    AsyncNotifierProvider<NodePositions, Map<String, FlowPosition>>.internal(
      NodePositions.new,
      name: r'nodePositionsProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$nodePositionsHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$NodePositions = AsyncNotifier<Map<String, FlowPosition>>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
