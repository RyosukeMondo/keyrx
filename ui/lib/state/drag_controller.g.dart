// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'drag_controller.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$dragControllerHash() => r'f394d122e3d89e0bd09ae6879d36d3a3f8585ce3';

/// Controller for managing drag state across the flow canvas.
/// Tracks which node is being dragged and its current position.
///
/// Copied from [DragController].
@ProviderFor(DragController)
final dragControllerProvider =
    AutoDisposeNotifierProvider<DragController, DragState?>.internal(
      DragController.new,
      name: r'dragControllerProvider',
      debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
          ? null
          : _$dragControllerHash,
      dependencies: null,
      allTransitiveDependencies: null,
    );

typedef _$DragController = AutoDisposeNotifier<DragState?>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
