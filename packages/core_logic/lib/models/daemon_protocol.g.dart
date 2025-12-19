// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'daemon_protocol.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_JsonRpcRequest _$JsonRpcRequestFromJson(Map<String, dynamic> json) =>
    _JsonRpcRequest(
      jsonrpc: json['jsonrpc'] as String,
      method: json['method'] as String,
      id: RequestId.fromJson(json['id']),
      params: json['params'] as Map<String, dynamic>?,
    );

Map<String, dynamic> _$JsonRpcRequestToJson(_JsonRpcRequest instance) =>
    <String, dynamic>{
      'jsonrpc': instance.jsonrpc,
      'method': instance.method,
      'id': instance.id,
      'params': instance.params,
    };

_JsonRpcResponse _$JsonRpcResponseFromJson(Map<String, dynamic> json) =>
    _JsonRpcResponse(
      jsonrpc: json['jsonrpc'] as String,
      result: json['result'],
      id: RequestId.fromJson(json['id']),
    );

Map<String, dynamic> _$JsonRpcResponseToJson(_JsonRpcResponse instance) =>
    <String, dynamic>{
      'jsonrpc': instance.jsonrpc,
      'result': instance.result,
      'id': instance.id,
    };

_ErrorObject _$ErrorObjectFromJson(Map<String, dynamic> json) => _ErrorObject(
  code: (json['code'] as num).toInt(),
  message: json['message'] as String,
  data: json['data'],
);

Map<String, dynamic> _$ErrorObjectToJson(_ErrorObject instance) =>
    <String, dynamic>{
      'code': instance.code,
      'message': instance.message,
      'data': instance.data,
    };

_JsonRpcError _$JsonRpcErrorFromJson(Map<String, dynamic> json) =>
    _JsonRpcError(
      jsonrpc: json['jsonrpc'] as String,
      error: ErrorObject.fromJson(json['error'] as Map<String, dynamic>),
      id: RequestId.fromJson(json['id']),
    );

Map<String, dynamic> _$JsonRpcErrorToJson(_JsonRpcError instance) =>
    <String, dynamic>{
      'jsonrpc': instance.jsonrpc,
      'error': instance.error,
      'id': instance.id,
    };

_JsonRpcNotification _$JsonRpcNotificationFromJson(Map<String, dynamic> json) =>
    _JsonRpcNotification(
      jsonrpc: json['jsonrpc'] as String,
      method: json['method'] as String,
      params: json['params'] as Map<String, dynamic>,
    );

Map<String, dynamic> _$JsonRpcNotificationToJson(
  _JsonRpcNotification instance,
) => <String, dynamic>{
  'jsonrpc': instance.jsonrpc,
  'method': instance.method,
  'params': instance.params,
};

_DaemonDeviceInfo _$DaemonDeviceInfoFromJson(Map<String, dynamic> json) =>
    _DaemonDeviceInfo(
      path: json['path'] as String,
      name: json['name'] as String,
      vendorId: json['vendor_id'] as String?,
      productId: json['product_id'] as String?,
    );

Map<String, dynamic> _$DaemonDeviceInfoToJson(_DaemonDeviceInfo instance) =>
    <String, dynamic>{
      'path': instance.path,
      'name': instance.name,
      'vendor_id': instance.vendorId,
      'product_id': instance.productId,
    };

_EngineStatus _$EngineStatusFromJson(Map<String, dynamic> json) =>
    _EngineStatus(
      status: json['status'] as String,
      configPath: json['config_path'] as String?,
      scriptPath: json['script_path'] as String?,
      devices:
          (json['devices'] as List<dynamic>?)
              ?.map((e) => DaemonDeviceInfo.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      uptimeSeconds: (json['uptime_seconds'] as num?)?.toInt(),
      eventsProcessed: (json['events_processed'] as num?)?.toInt(),
    );

Map<String, dynamic> _$EngineStatusToJson(_EngineStatus instance) =>
    <String, dynamic>{
      'status': instance.status,
      'config_path': instance.configPath,
      'script_path': instance.scriptPath,
      'devices': instance.devices,
      'uptime_seconds': instance.uptimeSeconds,
      'events_processed': instance.eventsProcessed,
    };

_InputEvent _$InputEventFromJson(Map<String, dynamic> json) => _InputEvent(
  timestampUs: (json['timestamp_us'] as num).toInt(),
  devicePath: json['device_path'] as String,
  deviceName: json['device_name'] as String,
  eventType: json['event_type'] as String,
  keyCode: (json['key_code'] as num).toInt(),
  keyName: json['key_name'] as String,
  modifiers:
      (json['modifiers'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const [],
);

Map<String, dynamic> _$InputEventToJson(_InputEvent instance) =>
    <String, dynamic>{
      'timestamp_us': instance.timestampUs,
      'device_path': instance.devicePath,
      'device_name': instance.deviceName,
      'event_type': instance.eventType,
      'key_code': instance.keyCode,
      'key_name': instance.keyName,
      'modifiers': instance.modifiers,
    };

_OutputEvent _$OutputEventFromJson(Map<String, dynamic> json) => _OutputEvent(
  timestampUs: (json['timestamp_us'] as num).toInt(),
  actionType: json['action_type'] as String,
  keyCode: (json['key_code'] as num).toInt(),
  keyName: json['key_name'] as String,
);

Map<String, dynamic> _$OutputEventToJson(_OutputEvent instance) =>
    <String, dynamic>{
      'timestamp_us': instance.timestampUs,
      'action_type': instance.actionType,
      'key_code': instance.keyCode,
      'key_name': instance.keyName,
    };

_DecisionEvent _$DecisionEventFromJson(Map<String, dynamic> json) =>
    _DecisionEvent(
      timestampUs: (json['timestamp_us'] as num).toInt(),
      keyCode: (json['key_code'] as num).toInt(),
      keyName: json['key_name'] as String,
      decisionType: json['decision_type'] as String,
      outputs:
          (json['outputs'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
    );

Map<String, dynamic> _$DecisionEventToJson(_DecisionEvent instance) =>
    <String, dynamic>{
      'timestamp_us': instance.timestampUs,
      'key_code': instance.keyCode,
      'key_name': instance.keyName,
      'decision_type': instance.decisionType,
      'outputs': instance.outputs,
    };

_StateChangeEvent _$StateChangeEventFromJson(Map<String, dynamic> json) =>
    _StateChangeEvent(
      type: json['type'] as String,
      timestampUs: (json['timestamp_us'] as num).toInt(),
      message: json['message'] as String,
      details: json['details'] as Map<String, dynamic>?,
    );

Map<String, dynamic> _$StateChangeEventToJson(_StateChangeEvent instance) =>
    <String, dynamic>{
      'type': instance.type,
      'timestamp_us': instance.timestampUs,
      'message': instance.message,
      'details': instance.details,
    };

_DiscoveryCandidate _$DiscoveryCandidateFromJson(Map<String, dynamic> json) =>
    _DiscoveryCandidate(
      deviceId: json['deviceId'] as String,
      vendorId: (json['vendorId'] as num).toInt(),
      productId: (json['productId'] as num).toInt(),
      name: json['name'] as String,
    );

Map<String, dynamic> _$DiscoveryCandidateToJson(_DiscoveryCandidate instance) =>
    <String, dynamic>{
      'deviceId': instance.deviceId,
      'vendorId': instance.vendorId,
      'productId': instance.productId,
      'name': instance.name,
    };
