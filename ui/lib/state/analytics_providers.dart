/// Riverpod providers for analytics and feature flags.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/feature_flags.dart';
import '../services/keyflow_analytics.dart';

part 'analytics_providers.g.dart';

/// Provider for SharedPreferences instance.
@Riverpod(keepAlive: true)
Future<SharedPreferences> sharedPreferences(Ref ref) async {
  return await SharedPreferences.getInstance();
}

/// Provider for FeatureFlagService.
@Riverpod(keepAlive: true)
class FeatureFlagServiceNotifier extends _$FeatureFlagServiceNotifier {
  @override
  Future<FeatureFlagService> build() async {
    final prefs = await ref.watch(sharedPreferencesProvider.future);
    final service = FeatureFlagService(preferences: prefs);
    await service.initialize();
    return service;
  }

  /// Check if a feature flag is enabled.
  Future<bool> isEnabled(FeatureFlag flag) async {
    final service = await future;
    return service.isEnabled(flag);
  }

  /// Set a feature flag value.
  Future<void> setFlag(FeatureFlag flag, bool value) async {
    final service = await future;
    await service.setFlag(flag, value);
    // Invalidate to trigger rebuild
    ref.invalidateSelf();
  }

  /// Reset a feature flag to default.
  Future<void> resetFlag(FeatureFlag flag) async {
    final service = await future;
    await service.resetFlag(flag);
    ref.invalidateSelf();
  }
}

/// Provider for checking if KeyFlow is enabled.
@riverpod
Future<bool> keyFlowEnabled(Ref ref) async {
  final service = await ref.watch(featureFlagServiceNotifierProvider.future);
  return service.isEnabled(FeatureFlag.keyFlowEnabled);
}

/// Provider for checking if KeyFlow analytics is enabled.
@riverpod
Future<bool> keyFlowAnalyticsEnabled(Ref ref) async {
  final service = await ref.watch(featureFlagServiceNotifierProvider.future);
  return service.isEnabled(FeatureFlag.keyFlowAnalytics);
}

/// Provider for KeyFlowAnalytics service.
@Riverpod(keepAlive: true)
KeyFlowAnalytics keyFlowAnalytics(Ref ref) {
  final analytics = KeyFlowAnalytics();
  ref.onDispose(analytics.dispose);
  return analytics;
}

/// Provider for tracking analytics (only if enabled).
@riverpod
class AnalyticsTracker extends _$AnalyticsTracker {
  @override
  Future<bool> build() async {
    return await ref.watch(keyFlowAnalyticsEnabledProvider.future);
  }

  KeyFlowAnalytics get _analytics => ref.read(keyFlowAnalyticsProvider);

  /// Track page view.
  void trackPageView({int? deviceCount, int? slotCount, int? nodeCount}) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackPageView(
          deviceCount: deviceCount,
          slotCount: slotCount,
          nodeCount: nodeCount,
        );
      }
    });
  }

  /// Track slot added.
  void trackSlotAdded({
    required String deviceId,
    required String profileId,
    required String keymapId,
    int? priority,
    bool? autoAssigned,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackSlotAdded(
          deviceId: deviceId,
          profileId: profileId,
          keymapId: keymapId,
          priority: priority,
          autoAssigned: autoAssigned,
        );
      }
    });
  }

  /// Track slot deleted.
  void trackSlotDeleted({
    required String deviceId,
    required String slotId,
    int? priority,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackSlotDeleted(
          deviceId: deviceId,
          slotId: slotId,
          priority: priority,
        );
      }
    });
  }

  /// Track slot reordered.
  void trackSlotReordered({
    required String deviceId,
    required String slotId,
    required int oldPriority,
    required int newPriority,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackSlotReordered(
          deviceId: deviceId,
          slotId: slotId,
          oldPriority: oldPriority,
          newPriority: newPriority,
        );
      }
    });
  }

  /// Track node dragged.
  void trackNodeDragged({
    required String nodeId,
    required String nodeType,
    required double deltaX,
    required double deltaY,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackNodeDragged(
          nodeId: nodeId,
          nodeType: nodeType,
          deltaX: deltaX,
          deltaY: deltaY,
        );
      }
    });
  }

  /// Track error.
  void trackError({
    required String errorType,
    required String errorMessage,
    String? operation,
    Map<String, Object?>? additionalContext,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackError(
          errorType: errorType,
          errorMessage: errorMessage,
          operation: operation,
          additionalContext: additionalContext,
        );
      }
    });
  }

  /// Track edge interaction.
  void trackEdgeInteraction({
    required String action,
    required String edgeId,
    bool? isActive,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackEdgeInteraction(
          action: action,
          edgeId: edgeId,
          isActive: isActive,
        );
      }
    });
  }

  /// Track dialog opened.
  void trackDialogOpened({required String dialogType}) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackDialogOpened(dialogType: dialogType);
      }
    });
  }

  /// Track keyboard shortcut.
  void trackKeyboardShortcut({
    required String shortcut,
    required String action,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackKeyboardShortcut(shortcut: shortcut, action: action);
      }
    });
  }

  /// Track canvas manipulation.
  void trackCanvasManipulation({
    required String action,
    double? scale,
    double? deltaX,
    double? deltaY,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackCanvasManipulation(
          action: action,
          scale: scale,
          deltaX: deltaX,
          deltaY: deltaY,
        );
      }
    });
  }

  /// Track auto-save.
  void trackAutoSave({
    required String operation,
    required bool success,
    int? retryCount,
    String? errorMessage,
  }) {
    state.whenData((enabled) {
      if (enabled) {
        _analytics.trackAutoSave(
          operation: operation,
          success: success,
          retryCount: retryCount,
          errorMessage: errorMessage,
        );
      }
    });
  }
}
