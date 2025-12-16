/// Service for communicating with the KeyRx daemon via IPC.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../models/daemon_protocol.dart';

/// Exception thrown when daemon communication fails.
class DaemonException implements Exception {
  const DaemonException(this.message, {this.errorCode, this.details});

  final String message;
  final int? errorCode;
  final dynamic details;

  @override
  String toString() {
    if (errorCode != null) {
      return 'DaemonException (code $errorCode): $message';
    }
    return 'DaemonException: $message';
  }
}

/// Connection state for the daemon.
enum DaemonConnectionState { disconnected, connecting, connected, error }

/// Service for communicating with the KeyRx daemon.
class DaemonService {
  DaemonService({String? socketPath})
    : _socketPath = socketPath ?? _defaultSocketPath();

  final String _socketPath;
  Socket? _socket;
  int _requestId = 0;
  DaemonConnectionState _state = DaemonConnectionState.disconnected;

  final Map<int, Completer<dynamic>> _pendingRequests = {};
  final StreamController<DaemonEvent> _eventController =
      StreamController<DaemonEvent>.broadcast();
  final StreamController<DaemonConnectionState> _stateController =
      StreamController<DaemonConnectionState>.broadcast();

  StreamSubscription<Uint8List>? _socketSubscription;
  String _receiveBuffer = '';

  /// Stream of daemon events (input, output, state changes).
  Stream<DaemonEvent> get events => _eventController.stream;

  /// Stream of connection state changes.
  Stream<DaemonConnectionState> get connectionState => _stateController.stream;

  /// Current connection state.
  DaemonConnectionState get state => _state;

  /// Whether the service is connected to the daemon.
  bool get isConnected => _state == DaemonConnectionState.connected;

  /// Get default socket path based on platform.
  static String _defaultSocketPath() {
    if (Platform.isLinux || Platform.isMacOS) {
      final runtimeDir = Platform.environment['XDG_RUNTIME_DIR'];
      if (runtimeDir != null && runtimeDir.isNotEmpty) {
        return '$runtimeDir/keyrx/daemon.sock';
      }
      return '/tmp/keyrx/daemon.sock';
    } else if (Platform.isWindows) {
      // TODO: Named pipe support
      throw UnsupportedError('Windows not yet supported');
    } else {
      throw UnsupportedError('Platform not supported');
    }
  }

  /// Connect to the daemon.
  Future<void> connect() async {
    if (_state == DaemonConnectionState.connected) {
      return; // Already connected
    }

    if (_state == DaemonConnectionState.connecting) {
      throw const DaemonException('Connection already in progress');
    }

    _updateState(DaemonConnectionState.connecting);

    try {
      debugPrint('DaemonService: Connecting to $_socketPath');

      // Connect to Unix socket
      _socket = await Socket.connect(
        InternetAddress(_socketPath, type: InternetAddressType.unix),
        0,
        timeout: const Duration(seconds: 5),
      );

      debugPrint('DaemonService: Connected');

      // Listen for incoming messages
      _socketSubscription = _socket!.listen(
        _handleData,
        onError: _handleError,
        onDone: _handleDisconnect,
        cancelOnError: false,
      );

      _updateState(DaemonConnectionState.connected);
    } catch (e) {
      debugPrint('DaemonService: Connection failed: $e');
      _updateState(DaemonConnectionState.error);
      throw DaemonException('Failed to connect to daemon: $e');
    }
  }

  /// Disconnect from the daemon.
  Future<void> disconnect() async {
    debugPrint('DaemonService: Disconnecting');

    await _socketSubscription?.cancel();
    _socketSubscription = null;

    await _socket?.close();
    _socket = null;

    _receiveBuffer = '';

    // Complete all pending requests with error
    for (final completer in _pendingRequests.values) {
      if (!completer.isCompleted) {
        completer.completeError(
          const DaemonException('Disconnected from daemon'),
        );
      }
    }
    _pendingRequests.clear();

    _updateState(DaemonConnectionState.disconnected);
  }

  /// Send a JSON-RPC request and wait for response.
  Future<T> sendRequest<T>(
    String method, {
    required T Function(dynamic) parseResult,
    Map<String, dynamic>? params,
  }) async {
    if (!isConnected) {
      throw const DaemonException('Not connected to daemon');
    }

    final id = ++_requestId;
    final request = JsonRpcRequest(
      jsonrpc: '2.0',
      method: method,
      params: params,
      id: RequestId.number(id),
    );

    final completer = Completer<dynamic>();
    _pendingRequests[id] = completer;

    try {
      // Send request
      final requestJson = jsonEncode(request.toJson());
      debugPrint('DaemonService → $requestJson');
      _socket!.write('$requestJson\n');
      await _socket!.flush();

      // Wait for response (with timeout)
      final result = await completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () {
          _pendingRequests.remove(id);
          throw const DaemonException('Request timed out');
        },
      );

      return parseResult(result);
    } catch (e) {
      _pendingRequests.remove(id);
      rethrow;
    }
  }

  /// Start the engine.
  Future<List<DaemonDeviceInfo>> startEngine({
    String? configPath,
    String? scriptPath,
    bool useAllDevices = true,
  }) async {
    final result = await sendRequest<Map<String, dynamic>>(
      DaemonMethods.engineStart,
      params: {
        if (configPath != null) 'config_path': configPath,
        if (scriptPath != null) 'script_path': scriptPath,
        'use_all_devices': useAllDevices,
      },
      parseResult: (result) => result as Map<String, dynamic>,
    );

    final deviceList = result['devices'] as List<dynamic>? ?? [];
    return deviceList
        .map((d) => DaemonDeviceInfo.fromJson(d as Map<String, dynamic>))
        .toList();
  }

  /// Stop the engine.
  Future<void> stopEngine() async {
    await sendRequest<void>(DaemonMethods.engineStop, parseResult: (_) {});
  }

  /// Reload config and/or script.
  Future<void> reloadEngine({String? configPath, String? scriptPath}) async {
    await sendRequest<void>(
      DaemonMethods.engineReload,
      params: {
        if (configPath != null) 'config_path': configPath,
        if (scriptPath != null) 'script_path': scriptPath,
      },
      parseResult: (_) {},
    );
  }

  /// Get current engine status.
  Future<EngineStatus> getStatus() async {
    final result = await sendRequest<Map<String, dynamic>>(
      DaemonMethods.engineStatus,
      parseResult: (result) => result as Map<String, dynamic>,
    );

    return EngineStatus.fromJson(result);
  }

  /// Subscribe to events.
  Future<void> subscribeEvents({
    List<String> eventTypes = const ['input', 'output', 'state_change'],
  }) async {
    await sendRequest<void>(
      DaemonMethods.eventsSubscribe,
      params: {'event_types': eventTypes},
      parseResult: (_) {},
    );
  }

  /// Unsubscribe from events.
  Future<void> unsubscribeEvents() async {
    await sendRequest<void>(
      DaemonMethods.eventsUnsubscribe,
      parseResult: (_) {},
    );
  }

  /// Handle incoming data from socket.
  void _handleData(Uint8List data) {
    // Append to buffer
    _receiveBuffer += utf8.decode(data);

    // Process complete lines
    while (_receiveBuffer.contains('\n')) {
      final lineEnd = _receiveBuffer.indexOf('\n');
      final line = _receiveBuffer.substring(0, lineEnd).trim();
      _receiveBuffer = _receiveBuffer.substring(lineEnd + 1);

      if (line.isEmpty) continue;

      debugPrint('DaemonService ← $line');

      try {
        final json = jsonDecode(line) as Map<String, dynamic>;
        final message = IncomingMessage.fromJson(json);

        message.when(
          response: _handleResponse,
          error: _handleErrorResponse,
          notification: _handleNotification,
        );
      } catch (e) {
        debugPrint('DaemonService: Failed to parse message: $e');
      }
    }
  }

  /// Handle successful response.
  void _handleResponse(JsonRpcResponse response) {
    final id = response.id;
    if (id.isNumber) {
      final completer = _pendingRequests.remove(id.asNumber);
      if (completer != null && !completer.isCompleted) {
        completer.complete(response.result);
      }
    }
  }

  /// Handle error response.
  void _handleErrorResponse(JsonRpcError errorResponse) {
    final id = errorResponse.id;
    if (id.isNumber) {
      final completer = _pendingRequests.remove(id.asNumber);
      if (completer != null && !completer.isCompleted) {
        completer.completeError(
          DaemonException(
            errorResponse.error.message,
            errorCode: errorResponse.error.code,
            details: errorResponse.error.data,
          ),
        );
      }
    }
  }

  /// Handle notification (event).
  void _handleNotification(JsonRpcNotification notification) {
    try {
      final event = DaemonEvent.fromNotification(notification);
      _eventController.add(event);
    } catch (e) {
      debugPrint('DaemonService: Unknown notification: ${notification.method}');
    }
  }

  /// Handle socket error.
  void _handleError(dynamic error) {
    debugPrint('DaemonService: Socket error: $error');
    _updateState(DaemonConnectionState.error);
  }

  /// Handle socket disconnect.
  void _handleDisconnect() {
    debugPrint('DaemonService: Socket disconnected');
    disconnect();
  }

  /// Update connection state.
  void _updateState(DaemonConnectionState newState) {
    if (_state != newState) {
      _state = newState;
      _stateController.add(newState);
    }
  }

  /// Dispose resources.
  Future<void> dispose() async {
    await disconnect();
    await _eventController.close();
    await _stateController.close();
  }
}
