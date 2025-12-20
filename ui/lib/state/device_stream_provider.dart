/// Device state streaming provider for real-time KeyFlow updates.
library;

import 'dart:async';

import 'package:core_logic/models/device_state.dart';
import 'package:core_logic/utils/logger.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/device_event_handler.dart';
import 'providers_setup.dart';

part 'device_stream_provider.g.dart';

/// Event type for device state changes.
enum DeviceEventType { connected, disconnected, updated, profileUpdated }

/// Device state change event.
class DeviceStateEvent {
  const DeviceStateEvent({
    required this.type,
    required this.timestamp,
    this.deviceInfo,
    this.profileId,
  });

  final DeviceEventType type;
  final DateTime timestamp;
  final Map<String, dynamic>? deviceInfo;
  final String? profileId;

  @override
  String toString() =>
      'DeviceStateEvent(type: $type, device: $deviceInfo, profile: $profileId)';
}

/// Provider for device state stream.
///
/// Subscribes to DeviceRegistryService events and emits DeviceStateEvent for:
/// - Device connections/disconnections
/// - Device updates
/// - Profile updates
@Riverpod(keepAlive: true)
class DeviceStreamNotifier extends _$DeviceStreamNotifier {
  final _logger = getLogger('device_stream');
  final _eventController = StreamController<DeviceStateEvent>.broadcast();

  StreamSubscription<List<DeviceState>>? _subscription;
  List<DeviceState> _previousDevices = [];

  @override
  Stream<DeviceStateEvent> build() {
    final registry = ref.watch(serviceRegistryProvider);
    _subscribeToDeviceStream(registry.deviceRegistryService.devicesStream);
    ref.onDispose(_dispose);
    return _eventController.stream;
  }

  void _subscribeToDeviceStream(Stream<List<DeviceState>> deviceStream) {
    _logger.info('device_stream.subscribe');

    _subscription = deviceStream.listen(
      _handleDeviceListChange,
      onError: (Object error, StackTrace stackTrace) {
        _logger.error(
          'device_stream.subscription_error',
          message: 'Error in device stream subscription',
          error: error,
          stackTrace: stackTrace,
        );
      },
    );
  }

  void _handleDeviceListChange(List<DeviceState> currentDevices) {
    if (_eventController.isClosed) return;

    try {
      // Detect changes by comparing with previous device list
      final previousKeys = _previousDevices.map((d) => d.identity.toKey()).toSet();
      final currentKeys = currentDevices.map((d) => d.identity.toKey()).toSet();

      // Find new devices (connected)
      final newKeys = currentKeys.difference(previousKeys);
      for (final key in newKeys) {
        final device = currentDevices.firstWhere((d) => d.identity.toKey() == key);
        if (device.connectedAt.isNotEmpty) {
          _emitEvent(DeviceEventType.connected, device);
        }
      }

      // Find removed devices (disconnected)
      final removedKeys = previousKeys.difference(currentKeys);
      for (final key in removedKeys) {
        final device = _previousDevices.firstWhere((d) => d.identity.toKey() == key);
        _emitEvent(DeviceEventType.disconnected, device);
      }

      // Find updated devices
      for (final currentDevice in currentDevices) {
        final key = currentDevice.identity.toKey();
        if (previousKeys.contains(key)) {
          final previousDevice = _previousDevices.firstWhere((d) => d.identity.toKey() == key);

          // Check if profile changed
          if (currentDevice.profileId != previousDevice.profileId) {
            _emitEvent(DeviceEventType.profileUpdated, currentDevice);
          }
          // Check if other properties changed (enabled, remap, etc.)
          else if (_deviceStateChanged(previousDevice, currentDevice)) {
            _emitEvent(DeviceEventType.updated, currentDevice);
          }
        }
      }

      _previousDevices = currentDevices;
    } on Exception catch (e, st) {
      _logger.error(
        'device_stream.change_detection_error',
        message: 'Failed to detect device changes',
        error: e,
        stackTrace: st,
      );
    }
  }

  bool _deviceStateChanged(DeviceState previous, DeviceState current) =>
      previous.deviceEnabled != current.deviceEnabled ||
      previous.remapEnabled != current.remapEnabled ||
      previous.identity.userLabel != current.identity.userLabel ||
      previous.connectedAt != current.connectedAt;

  void _emitEvent(DeviceEventType type, DeviceState device) {
    _logger.debug(
      'device_stream.event',
      context: {
        'type': type.name,
        'device_key': device.identity.toKey(),
        'profile_id': device.profileId,
      },
    );

    final event = DeviceStateEvent(
      type: type,
      timestamp: DateTime.now(),
      deviceInfo: {
        'key': device.identity.toKey(),
        'vendor_id': device.identity.vendorId,
        'product_id': device.identity.productId,
        'serial_number': device.identity.serialNumber,
        'user_label': device.identity.userLabel,
      },
      profileId: device.profileId,
    );

    _eventController.add(event);
  }

  void _dispose() {
    _logger.info('device_stream.dispose');
    _subscription?.cancel();
    _eventController.close();
  }
}

/// Provider for the device event handler service.
///
/// Automatically starts listening to device events and invalidates
/// runtime config when devices connect/disconnect.
@riverpod
DeviceEventHandler deviceEventHandler(Ref ref) {
  final handler = DeviceEventHandler(ref);
  ref.onDispose(handler.dispose);
  return handler;
}
