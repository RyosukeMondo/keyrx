/// JSON-RPC 2.0 protocol types for daemon communication.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

part 'daemon_protocol.freezed.dart';
part 'daemon_protocol.g.dart';

/// JSON-RPC request ID (can be number or string).
///
/// Note: This uses custom JSON serialization to support union types.
class RequestId {
  factory RequestId.fromJson(dynamic json) {
    if (json is int) {
      return RequestId.number(json);
    } else if (json is String) {
      return RequestId.string(json);
    }
    throw ArgumentError('Invalid RequestId: $json');
  }
  const RequestId.number(this.value) : _isNumber = true;
  const RequestId.string(this.value) : _isNumber = false;

  final dynamic value;
  final bool _isNumber;

  bool get isNumber => _isNumber;
  bool get isString => !_isNumber;

  int get asNumber => value as int;
  String get asString => value as String;

  dynamic toJson() => value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is RequestId &&
          value == other.value &&
          _isNumber == other._isNumber;

  @override
  int get hashCode => Object.hash(value, _isNumber);

  @override
  String toString() =>
      _isNumber ? 'RequestId.number($value)' : 'RequestId.string($value)';
}

/// JSON-RPC 2.0 request.
@freezed
class JsonRpcRequest with _$JsonRpcRequest {
  const factory JsonRpcRequest({
    required String jsonrpc,
    required String method,
    required RequestId id,
    Map<String, dynamic>? params,
  }) = _JsonRpcRequest;

  factory JsonRpcRequest.fromJson(Map<String, dynamic> json) =>
      _$JsonRpcRequestFromJson(json);
}

/// JSON-RPC 2.0 response (success).
@freezed
class JsonRpcResponse with _$JsonRpcResponse {
  const factory JsonRpcResponse({
    required String jsonrpc,
    required dynamic result,
    required RequestId id,
  }) = _JsonRpcResponse;

  factory JsonRpcResponse.fromJson(Map<String, dynamic> json) =>
      _$JsonRpcResponseFromJson(json);
}

/// JSON-RPC 2.0 error object.
@freezed
class ErrorObject with _$ErrorObject {
  const factory ErrorObject({
    required int code,
    required String message,
    dynamic data,
  }) = _ErrorObject;

  factory ErrorObject.fromJson(Map<String, dynamic> json) =>
      _$ErrorObjectFromJson(json);
}

/// JSON-RPC 2.0 error response.
@freezed
class JsonRpcError with _$JsonRpcError {
  const factory JsonRpcError({
    required String jsonrpc,
    required ErrorObject error,
    required RequestId id,
  }) = _JsonRpcError;

  factory JsonRpcError.fromJson(Map<String, dynamic> json) =>
      _$JsonRpcErrorFromJson(json);
}

/// JSON-RPC 2.0 notification (no ID).
@freezed
class JsonRpcNotification with _$JsonRpcNotification {
  const factory JsonRpcNotification({
    required String jsonrpc,
    required String method,
    required Map<String, dynamic> params,
  }) = _JsonRpcNotification;

  factory JsonRpcNotification.fromJson(Map<String, dynamic> json) =>
      _$JsonRpcNotificationFromJson(json);
}

/// Incoming message (response, error, or notification).
@freezed
class IncomingMessage with _$IncomingMessage {
  const factory IncomingMessage.response(JsonRpcResponse response) =
      _IncomingResponse;
  const factory IncomingMessage.error(JsonRpcError error) = _IncomingError;
  const factory IncomingMessage.notification(JsonRpcNotification notification) =
      _IncomingNotification;

  factory IncomingMessage.fromJson(Map<String, dynamic> json) {
    // Check if it's a notification (no 'id' field)
    if (!json.containsKey('id')) {
      return IncomingMessage.notification(JsonRpcNotification.fromJson(json));
    }

    // Check if it's an error (has 'error' field)
    if (json.containsKey('error')) {
      return IncomingMessage.error(JsonRpcError.fromJson(json));
    }

    // Otherwise it's a response
    return IncomingMessage.response(JsonRpcResponse.fromJson(json));
  }
}

// Method names
class DaemonMethods {
  static const engineStart = 'engine.start';
  static const engineStop = 'engine.stop';
  static const engineReload = 'engine.reload';
  static const engineStatus = 'engine.status';
  static const eventsSubscribe = 'events.subscribe';
  static const eventsUnsubscribe = 'events.unsubscribe';
}

// Notification method names
class DaemonNotifications {
  static const eventInput = 'event.input';
  static const eventOutput = 'event.output';
  static const eventStateChange = 'event.state_change';
}

/// Device info in responses.
@freezed
class DaemonDeviceInfo with _$DaemonDeviceInfo {
  const factory DaemonDeviceInfo({
    required String path,
    required String name,
    String? vendorId,
    String? productId,
  }) = _DaemonDeviceInfo;

  factory DaemonDeviceInfo.fromJson(Map<String, dynamic> json) =>
      _$DaemonDeviceInfoFromJson(json);
}

/// Engine status response.
@freezed
class EngineStatus with _$EngineStatus {
  const factory EngineStatus({
    required String status, // "running", "stopped", "error"
    String? configPath,
    String? scriptPath,
    @Default([]) List<DaemonDeviceInfo> devices,
    int? uptimeSeconds,
    int? eventsProcessed,
  }) = _EngineStatus;

  factory EngineStatus.fromJson(Map<String, dynamic> json) =>
      _$EngineStatusFromJson(json);
}

/// Input event notification.
@freezed
class InputEvent with _$InputEvent {
  const factory InputEvent({
    required int timestampUs,
    required String devicePath,
    required String deviceName,
    required String eventType, // "key_press", "key_release"
    required int keyCode,
    required String keyName,
    @Default([]) List<String> modifiers,
  }) = _InputEvent;

  factory InputEvent.fromJson(Map<String, dynamic> json) =>
      _$InputEventFromJson(json);
}

/// Output event notification.
@freezed
class OutputEvent with _$OutputEvent {
  const factory OutputEvent({
    required int timestampUs,
    required String actionType, // "key_press", "key_release", "tap"
    required int keyCode,
    required String keyName,
  }) = _OutputEvent;

  factory OutputEvent.fromJson(Map<String, dynamic> json) =>
      _$OutputEventFromJson(json);
}

/// State change notification.
@freezed
class StateChangeEvent with _$StateChangeEvent {
  const factory StateChangeEvent({
    required String type, // "config_reloaded", "script_reloaded", etc.
    required int timestampUs,
    required String message,
    Map<String, dynamic>? details,
  }) = _StateChangeEvent;

  factory StateChangeEvent.fromJson(Map<String, dynamic> json) =>
      _$StateChangeEventFromJson(json);
}

/// Unified daemon event.
@freezed
class DaemonEvent with _$DaemonEvent {
  const factory DaemonEvent.input(InputEvent event) = _DaemonInputEvent;
  const factory DaemonEvent.output(OutputEvent event) = _DaemonOutputEvent;
  const factory DaemonEvent.stateChange(StateChangeEvent event) =
      _DaemonStateChangeEvent;

  factory DaemonEvent.fromNotification(JsonRpcNotification notification) {
    final params = notification.params;

    switch (notification.method) {
      case DaemonNotifications.eventInput:
        return DaemonEvent.input(InputEvent.fromJson(params));
      case DaemonNotifications.eventOutput:
        return DaemonEvent.output(OutputEvent.fromJson(params));
      case DaemonNotifications.eventStateChange:
        return DaemonEvent.stateChange(StateChangeEvent.fromJson(params));
      default:
        throw ArgumentError(
          'Unknown notification method: ${notification.method}',
        );
    }
  }
}

/// Error codes.
class ErrorCodes {
  // Standard JSON-RPC errors
  static const parseError = -32700;
  static const invalidRequest = -32600;
  static const methodNotFound = -32601;
  static const invalidParams = -32602;
  static const internalError = -32603;

  // Custom errors
  static const engineNotRunning = -32000;
  static const deviceNotFound = -32001;
  static const configLoadFailed = -32002;
  static const scriptCompileFailed = -32003;
  static const permissionDenied = -32004;
}
