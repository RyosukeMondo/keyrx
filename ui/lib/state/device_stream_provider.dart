/// Device state streaming provider for real-time KeyFlow updates.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:core_logic/ffi/bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/bridge_stub.dart';
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
/// Subscribes to FFI bridge events and emits DeviceStateEvent for:
/// - Device connections/disconnections
/// - Device updates
/// - Profile updates
@Riverpod(keepAlive: true)
class DeviceStreamNotifier extends _$DeviceStreamNotifier {
  final _logger = getLogger('device_stream');
  final _eventController = StreamController<DeviceStateEvent>.broadcast();

  KeyrxBridge? _bridge;
  bool _subscribed = false;

  @override
  Stream<DeviceStateEvent> build() {
    _bridge = ref.watch(bridgeProvider);
    _subscribeToEvents();
    ref.onDispose(_dispose);
    return _eventController.stream;
  }

  void _subscribeToEvents() {
    if (_subscribed || _bridge == null) return;

    _logger.info(
      'device_stream.subscribe',
      context: {'bridge_available': _bridge != null},
    );

    final eventTypes = {
      EventType.deviceConnected: DeviceEventType.connected,
      EventType.deviceDisconnected: DeviceEventType.disconnected,
      EventType.deviceUpdated: DeviceEventType.updated,
      EventType.profileUpdated: DeviceEventType.profileUpdated,
    };

    for (final entry in eventTypes.entries) {
      final success = _bridge!.registerEventCallback(
        entry.key,
        (payload) => _handleEvent(payload, entry.value),
      );
      _logger.debug(
        'device_stream.register',
        context: {'event_type': entry.key.name, 'success': success},
      );
    }

    _subscribed = true;
  }

  void _handleEvent(Uint8List payload, DeviceEventType eventType) {
    if (_eventController.isClosed) return;

    try {
      final jsonString = utf8.decode(payload);
      final data = jsonDecode(jsonString) as Map<String, dynamic>;

      _logger.debug(
        'device_stream.event',
        context: {'type': eventType.name, 'payload_keys': data.keys.toList()},
      );

      final event = DeviceStateEvent(
        type: eventType,
        timestamp: DateTime.now(),
        deviceInfo: data['device'] as Map<String, dynamic>?,
        profileId: data['profile_id'] as String?,
      );

      _eventController.add(event);
    } on Exception catch (e, st) {
      _logger.error(
        'device_stream.parse_error',
        message: 'Failed to parse event payload',
        error: e,
        stackTrace: st,
      );
    }
  }

  void _dispose() {
    _logger.info('device_stream.dispose');

    if (_bridge != null && _subscribed) {
      for (final eventType in [
        EventType.deviceConnected,
        EventType.deviceDisconnected,
        EventType.deviceUpdated,
        EventType.profileUpdated,
      ]) {
        _bridge!.unregisterEventCallback(eventType);
      }
    }

    _eventController.close();
    _subscribed = false;
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
