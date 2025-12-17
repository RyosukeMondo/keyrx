// ignore_for_file: cascade_invocations

/// Service for communicating with the KeyRx daemon via IPC.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';

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
enum DaemonConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  error,
}

class _QueuedRequest {
  _QueuedRequest({
    required this.id,
    required this.request,
    required this.completer,
  });
  final int id;
  final JsonRpcRequest request;
  final Completer<dynamic> completer;
}

/// Service for communicating with the KeyRx daemon.
class DaemonService {
  DaemonService({String? socketPath})
    : _socketPath = socketPath ?? _defaultSocketPath(),
      _log = Logger('DaemonService');

  final String _socketPath;
  final Logger _log;
  Socket? _socket;
  int _requestId = 0;
  DaemonConnectionState _state = DaemonConnectionState.disconnected;
  bool _explicitDisconnect = false;
  int _reconnectAttempts = 0;

  final Map<int, Completer<dynamic>> _pendingRequests = {};
  final List<_QueuedRequest> _requestQueue = [];
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
      // TODO(rmondo): Implement named pipe support
      throw UnsupportedError('Windows not yet supported');
    } else {
      throw UnsupportedError('Platform not supported');
    }
  }

  /// Connect to the daemon.
  Future<void> connect() async {
    _explicitDisconnect = false;

    if (state == DaemonConnectionState.connected ||
        state == DaemonConnectionState.connecting) {
      _log.info('Connection attempt ignored, already connected or connecting.');
      return;
    }

    await _connect();
  }

  Future<void> _connect() async {
    if (state == DaemonConnectionState.connected) return;

    _updateState(
      _reconnectAttempts == 0
          ? DaemonConnectionState.connecting
          : DaemonConnectionState.reconnecting,
    );

    try {
      _log.info('Connecting to daemon at $_socketPath...');
      _socket = await Socket.connect(
        InternetAddress(_socketPath, type: InternetAddressType.unix),
        0,
        timeout: const Duration(seconds: 5),
      );
      _log.info('Successfully connected to daemon.');

      _socketSubscription = _socket!.listen(
        _handleData,
        onError: _handleError,
        onDone: _handleDisconnect,
        cancelOnError: true,
      );

      _updateState(DaemonConnectionState.connected);
      _reconnectAttempts = 0;
      unawaited(_sendQueuedRequests());
    } on SocketException catch (e) {
      _log.warning('Failed to connect to daemon', e);
      _clearSocket();
      _handleDisconnect();
    }
  }

  /// Disconnect from the daemon.
  Future<void> disconnect() async {
    _log.info('Disconnecting from daemon...');
    _explicitDisconnect = true;
    _reconnectAttempts = 0; // Stop any reconnection attempts

    _clearSocket();
    _receiveBuffer = '';

    // Complete all pending requests with an error
    for (final completer in _pendingRequests.values) {
      if (!completer.isCompleted) {
        completer.completeError(
          const DaemonException('Disconnected from daemon'),
        );
      }
    }
    _pendingRequests.clear();

    // Fail any queued requests
    for (final queuedRequest in _requestQueue) {
      if (!queuedRequest.completer.isCompleted) {
        queuedRequest.completer.completeError(
          const DaemonException('Disconnected from daemon'),
        );
      }
    }
    _requestQueue.clear();

    _updateState(DaemonConnectionState.disconnected);
  }

  /// Send a JSON-RPC request and wait for response.
  Future<T> sendRequest<T>(
    String method, {
    required T Function(dynamic) parseResult,
    Map<String, dynamic>? params,
  }) async {
    final id = ++_requestId;
    final request = JsonRpcRequest(
      jsonrpc: '2.0',
      method: method,
      params: params,
      id: RequestId.number(id),
    );

    final completer = Completer<dynamic>();
    final queuedRequest = _QueuedRequest(
      id: id,
      request: request,
      completer: completer,
    );

    if (!isConnected) {
      _log.fine('Queueing request: $method');
      _requestQueue.add(queuedRequest);
    } else {
      _sendRequest(queuedRequest);
    }

    final result = await completer.future;
    return parseResult(result);
  }

  void _sendRequest(_QueuedRequest queuedRequest) {
    final id = queuedRequest.id;
    final request = queuedRequest.request;
    final completer = queuedRequest.completer;

    _pendingRequests[id] = completer;

    try {
      final requestJson = jsonEncode(request.toJson());
      _log.finer('→ $requestJson');
      _socket!.write('$requestJson\n');
      _socket!.flush();

      // Handle response timeout
      completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () {
          if (_pendingRequests.containsKey(id)) {
            _pendingRequests.remove(id);
            if (!completer.isCompleted) {
              completer.completeError(
                const DaemonException('Request timed out'),
              );
            }
          }
        },
      );
    } on SocketException catch (e) {
      _pendingRequests.remove(id);
      if (!completer.isCompleted) {
        completer.completeError(
          DaemonException('Daemon communication error: ${e.message}'),
        );
      }
    }
  }

  @visibleForTesting
  Future<void> sendRequestRaw(Map<String, dynamic> request) async {
    if (!isConnected) {
      throw const DaemonException('Not connected to daemon');
    }

    final requestJson = jsonEncode(request);
    _log.finer('→ $requestJson');
    _socket!.write('$requestJson\n');
    await _socket!.flush();
  }

  Future<void> _sendQueuedRequests() async {
    if (_requestQueue.isEmpty) return;

    _log.info('Sending ${_requestQueue.length} queued requests...');
    final requests = List<_QueuedRequest>.from(_requestQueue);
    _requestQueue.clear();

    requests.forEach(_sendRequest);
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
  Future<void> reloadConfig({String? configPath, String? scriptPath}) async {
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

      _log.finer('← $line');

      try {
        final json = jsonDecode(line) as Map<String, dynamic>;
        final message = IncomingMessage.fromJson(json);

        message.when(
          response: _handleResponse,
          error: _handleErrorResponse,
          notification: _handleNotification,
        );
      } on FormatException catch (e) {
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
    } on Object catch (e) {
      _log.warning('Failed to parse notification: ${notification.method}', e);
    }
  }

  /// Handle socket error.
  void _handleError(dynamic error) {
    _log.warning('Socket error', error);
    _clearSocket();
    _handleDisconnect();
  }

  /// Handle socket disconnect.
  void _handleDisconnect() {
    _log.info('Daemon disconnected.');
    _clearSocket();

    if (state == DaemonConnectionState.disconnected) {
      return;
    }

    if (_explicitDisconnect) {
      _log.info('Explicit disconnect, not reconnecting.');
      _updateState(DaemonConnectionState.disconnected);
    } else {
      _log.info('Unexpected disconnect, attempting to reconnect...');
      _reconnect();
    }
  }

  void _clearSocket() {
    _socketSubscription?.cancel();
    _socketSubscription = null;
    _socket?.destroy();
    _socket = null;
  }

  Future<void> _reconnect() async {
    if (_explicitDisconnect || state == DaemonConnectionState.connecting) {
      return;
    }
    _updateState(DaemonConnectionState.reconnecting);

    _reconnectAttempts++;
    final delay = Duration(
      seconds: min(
        pow(2, _reconnectAttempts).toInt(),
        60, // Max delay
      ),
    );

    _log.info('Reconnecting in $delay...');
    await Future<void>.delayed(delay);

    if (!_explicitDisconnect) {
      await _connect();
    }
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
