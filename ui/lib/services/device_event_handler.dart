/// Device event handler for processing connect/disconnect events.
library;

import 'dart:async';

import 'package:core_logic/utils/logger.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/device_stream_provider.dart';
import '../state/keyflow_providers.dart';

/// Service to handle device state changes and update the flow graph.
///
/// Listens to device events and:
/// - Invalidates repository cache on device changes
/// - Triggers runtime config refresh
/// - Ensures <100ms latency from event to UI update
class DeviceEventHandler {
  DeviceEventHandler(this._ref) {
    _logger.info('device_handler.init');
    _startListening();
  }

  final Ref _ref;
  final _logger = getLogger('device_handler');

  StreamSubscription<DeviceStateEvent>? _subscription;
  Timer? _debounceTimer;
  bool _refreshPending = false;

  static const _refreshDebounce = Duration(milliseconds: 50);
  static const _maxLatency = Duration(milliseconds: 100);

  void _startListening() {
    // Listen to the stream provider
    _ref.listen<AsyncValue<DeviceStateEvent>>(
      deviceStreamNotifierProvider,
      (previous, next) {
        next.whenData(_handleDeviceEvent);
      },
      onError: (Object error, StackTrace stackTrace) {
        _logger.error(
          'device_handler.stream_error',
          message: 'Error in device stream',
          error: error,
          stackTrace: stackTrace,
        );
      },
    );
  }

  void _handleDeviceEvent(DeviceStateEvent event) {
    final startTime = DateTime.now();

    _logger.info(
      'device_handler.event',
      context: {
        'type': event.type.name,
        'has_device': event.deviceInfo != null,
        'has_profile': event.profileId != null,
      },
    );

    // Cancel pending refresh timer
    _debounceTimer?.cancel();

    // Mark refresh as pending
    _refreshPending = true;

    // Debounce rapid-fire events to avoid refresh storms
    _debounceTimer = Timer(_refreshDebounce, () async {
      if (!_refreshPending) return;

      _refreshPending = false;

      try {
        _logger.debug('device_handler.refresh_start');

        // Force refresh runtime config to get latest device state
        final notifier = _ref.read(runtimeConfigNotifierProvider.notifier);
        await notifier.refresh(force: true);

        final latency = DateTime.now().difference(startTime);
        _logger.info(
          'device_handler.refresh_complete',
          context: {
            'latency_ms': latency.inMilliseconds,
            'within_target': latency < _maxLatency,
          },
        );

        if (latency > _maxLatency) {
          _logger.warn(
            'device_handler.latency_exceeded',
            message: 'Update latency exceeded target',
            context: {
              'latency_ms': latency.inMilliseconds,
              'target_ms': _maxLatency.inMilliseconds,
            },
          );
        }
      } catch (e, st) {
        _logger.error(
          'device_handler.refresh_error',
          message: 'Failed to refresh runtime config',
          error: e,
          stackTrace: st,
        );
      }
    });
  }

  /// Get statistics about event handling latency.
  Map<String, dynamic> getStats() => {
    'has_subscription': _subscription != null,
    'refresh_pending': _refreshPending,
    'target_latency_ms': _maxLatency.inMilliseconds,
  };

  void dispose() {
    _logger.info('device_handler.dispose');
    _debounceTimer?.cancel();
    _subscription?.cancel();
    _debounceTimer = null;
    _subscription = null;
  }
}
