import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'package:shelf/shelf.dart';
import 'package:shelf_web_socket/shelf_web_socket.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/services/services.dart';

/// WebSocket handler for real-time event broadcasting to web clients.
///
/// Subscribes to core_logic service streams and FFI events, then broadcasts
/// them to all connected WebSocket clients.
class WebSocketHandler {
  WebSocketHandler({
    required KeyrxBridge bridge,
    required DeviceRegistryService deviceRegistryService,
    required HardwareService hardwareService,
    required LayoutService layoutService,
  })  : _bridge = bridge,
        _deviceRegistryService = deviceRegistryService,
        _hardwareService = hardwareService,
        _layoutService = layoutService {
    _initialize();
  }

  final KeyrxBridge _bridge;
  final DeviceRegistryService _deviceRegistryService;
  final HardwareService _hardwareService;
  final LayoutService _layoutService;

  final Set<WebSocketChannel> _clients = {};
  final List<StreamSubscription<dynamic>> _subscriptions = [];

  /// Initialize subscriptions to service streams and FFI events
  void _initialize() {
    // Subscribe to device registry stream
    _subscriptions.add(
      _deviceRegistryService.devicesStream.listen((devices) {
        _broadcast({
          'type': 'devices_updated',
          'data': devices.map((d) => d.toJson()).toList(),
          'timestamp': DateTime.now().toIso8601String(),
        });
      }),
    );

    // Subscribe to hardware service changes
    _subscriptions.add(
      _hardwareService.changes.listen((_) {
        _broadcast({
          'type': 'hardware_changed',
          'timestamp': DateTime.now().toIso8601String(),
        });
      }),
    );

    // Subscribe to layout service changes
    _subscriptions.add(
      _layoutService.changes.listen((_) {
        _broadcast({
          'type': 'layout_changed',
          'timestamp': DateTime.now().toIso8601String(),
        });
      }),
    );

    // Register FFI event callbacks
    _registerEventCallbacks();
  }

  /// Register callbacks for all relevant FFI events
  void _registerEventCallbacks() {
    // Device events
    _bridge.registerEventCallback(
      EventType.deviceConnected,
      (eventData) => _broadcastEvent('device_connected', eventData),
    );
    _bridge.registerEventCallback(
      EventType.deviceDisconnected,
      (eventData) => _broadcastEvent('device_disconnected', eventData),
    );
    _bridge.registerEventCallback(
      EventType.deviceUpdated,
      (eventData) => _broadcastEvent('device_updated', eventData),
    );

    // Profile events
    _bridge.registerEventCallback(
      EventType.profileUpdated,
      (eventData) => _broadcastEvent('profile_updated', eventData),
    );

    // Engine state events
    _bridge.registerEventCallback(
      EventType.engineState,
      (eventData) => _broadcastEvent('engine_state', eventData),
    );

    // Test events
    _bridge.registerEventCallback(
      EventType.testProgress,
      (eventData) => _broadcastEvent('test_progress', eventData),
    );
    _bridge.registerEventCallback(
      EventType.testResult,
      (eventData) => _broadcastEvent('test_result', eventData),
    );

    // Discovery events
    _bridge.registerEventCallback(
      EventType.discoveryProgress,
      (eventData) => _broadcastEvent('discovery_progress', eventData),
    );
    _bridge.registerEventCallback(
      EventType.discoverySummary,
      (eventData) => _broadcastEvent('discovery_summary', eventData),
    );

    // Validation events
    _bridge.registerEventCallback(
      EventType.validationProgress,
      (eventData) => _broadcastEvent('validation_progress', eventData),
    );
    _bridge.registerEventCallback(
      EventType.validationResult,
      (eventData) => _broadcastEvent('validation_result', eventData),
    );

    // Recording events
    _bridge.registerEventCallback(
      EventType.recordingStarted,
      (eventData) => _broadcastEvent('recording_started', eventData),
    );
    _bridge.registerEventCallback(
      EventType.recordingStopped,
      (eventData) => _broadcastEvent('recording_stopped', eventData),
    );

    // Diagnostics events
    _bridge.registerEventCallback(
      EventType.diagnosticsLog,
      (eventData) => _broadcastEvent('diagnostics_log', eventData),
    );
    _bridge.registerEventCallback(
      EventType.diagnosticsMetric,
      (eventData) => _broadcastEvent('diagnostics_metric', eventData),
    );

    // Analysis events
    _bridge.registerEventCallback(
      EventType.analysisProgress,
      (eventData) => _broadcastEvent('analysis_progress', eventData),
    );
    _bridge.registerEventCallback(
      EventType.analysisResult,
      (eventData) => _broadcastEvent('analysis_result', eventData),
    );

    // Raw input/output events (optional - might be high volume)
    // Uncomment if needed for debugging
    // _bridge.registerEventCallback(
    //   EventType.rawInput,
    //   (eventData) => _broadcastEvent('raw_input', eventData),
    // );
    // _bridge.registerEventCallback(
    //   EventType.rawOutput,
    //   (eventData) => _broadcastEvent('raw_output', eventData),
    // );
  }

  /// Broadcast FFI event to all connected clients
  void _broadcastEvent(String eventType, Uint8List eventDataBytes) {
    try {
      // Decode the JSON payload
      final eventData = jsonDecode(utf8.decode(eventDataBytes)) as Map<String, dynamic>;
      _broadcast({
        'type': eventType,
        'data': eventData,
        'timestamp': DateTime.now().toIso8601String(),
      });
    } catch (e) {
      // If JSON decoding fails, send raw string
      _broadcast({
        'type': eventType,
        'data': {'raw': utf8.decode(eventDataBytes)},
        'timestamp': DateTime.now().toIso8601String(),
      });
    }
  }

  /// Broadcast message to all connected WebSocket clients
  void _broadcast(Map<String, dynamic> message) {
    if (_clients.isEmpty) {
      return;
    }

    final json = jsonEncode(message);
    final disconnectedClients = <WebSocketChannel>[];

    for (final client in _clients) {
      try {
        client.sink.add(json);
      } catch (e) {
        // Client disconnected or error occurred
        disconnectedClients.add(client);
      }
    }

    // Remove disconnected clients
    for (final client in disconnectedClients) {
      _removeClient(client);
    }
  }

  /// Create shelf handler for WebSocket connections
  Handler get handler {
    return webSocketHandler((WebSocketChannel channel) {
      _addClient(channel);

      // Listen for messages from client (for potential filtering/subscriptions)
      channel.stream.listen(
        (message) {
          // Future enhancement: handle subscription filters from clients
          // For now, we broadcast all events to all clients
        },
        onError: (_) => _removeClient(channel),
        onDone: () => _removeClient(channel),
        cancelOnError: true,
      );
    });
  }

  /// Add a new client connection
  void _addClient(WebSocketChannel channel) {
    _clients.add(channel);
    // ignore: avoid_print
    print('WebSocket client connected (total: ${_clients.length})');

    // Send welcome message
    try {
      channel.sink.add(jsonEncode({
        'type': 'connected',
        'message': 'Connected to KeyRx event stream',
        'timestamp': DateTime.now().toIso8601String(),
      }));
    } catch (e) {
      _removeClient(channel);
    }
  }

  /// Remove a client connection
  void _removeClient(WebSocketChannel channel) {
    if (_clients.remove(channel)) {
      try {
        channel.sink.close();
      } catch (_) {
        // Ignore close errors
      }
      // ignore: avoid_print
      print('WebSocket client disconnected (total: ${_clients.length})');
    }
  }

  /// Dispose resources and close all connections
  Future<void> dispose() async {
    // Cancel all subscriptions
    for (final subscription in _subscriptions) {
      await subscription.cancel();
    }
    _subscriptions.clear();

    // Close all client connections
    for (final client in _clients) {
      try {
        await client.sink.close();
      } catch (_) {
        // Ignore close errors
      }
    }
    _clients.clear();
  }

  /// Get current number of connected clients
  int get clientCount => _clients.length;
}
