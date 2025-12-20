/// WebSocket client for real-time event streaming.
library;

import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

/// WebSocket client for receiving real-time events from the backend.
///
/// Connects to the backend WebSocket endpoint and distributes
/// events to subscribers based on event names.
class WebSocketClient {
  /// Create a WebSocket client and connect to [url].
  WebSocketClient(String url)
      : _channel = WebSocketChannel.connect(Uri.parse(url)) {
    _channel.stream.listen(
      _handleMessage,
      onError: _handleError,
      onDone: _handleDone,
    );
  }

  final WebSocketChannel _channel;
  final Map<String, StreamController<dynamic>> _eventStreams = {};
  final StreamController<String> _errorController =
      StreamController<String>.broadcast();
  bool _disposed = false;

  /// Subscribe to a specific event type.
  ///
  /// [event] is the event name (e.g., 'device.updated').
  /// [parser] converts the raw event data to type [T].
  ///
  /// Returns a stream of parsed events of type [T].
  Stream<T> subscribe<T>(String event, T Function(dynamic) parser) {
    if (_disposed) {
      throw StateError('WebSocketClient has been disposed');
    }

    if (!_eventStreams.containsKey(event)) {
      _eventStreams[event] = StreamController<dynamic>.broadcast();
    }

    return _eventStreams[event]!.stream.map(parser).cast<T>();
  }

  /// Stream of WebSocket errors.
  Stream<String> get errors => _errorController.stream;

  /// Handle incoming WebSocket messages.
  void _handleMessage(dynamic message) {
    try {
      final json = jsonDecode(message as String) as Map<String, dynamic>;
      final event = json['event'] as String?;
      final data = json['data'];

      if (event == null) {
        _errorController.add('Received message without event field');
        return;
      }

      if (_eventStreams.containsKey(event)) {
        _eventStreams[event]!.add(data);
      }
    } catch (e) {
      _errorController.add('Failed to parse WebSocket message: $e');
    }
  }

  /// Handle WebSocket errors.
  void _handleError(dynamic error) {
    _errorController.add('WebSocket error: $error');
  }

  /// Handle WebSocket connection close.
  void _handleDone() {
    if (!_disposed) {
      _errorController.add('WebSocket connection closed unexpectedly');
    }
  }

  /// Dispose resources and close the WebSocket connection.
  void dispose() {
    if (_disposed) {
      return;
    }

    _disposed = true;
    _channel.sink.close();

    for (final controller in _eventStreams.values) {
      controller.close();
    }
    _eventStreams.clear();

    _errorController.close();
  }
}
