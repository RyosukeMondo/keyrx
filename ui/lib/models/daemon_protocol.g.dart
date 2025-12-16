// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'daemon_protocol.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$JsonRpcRequestImpl _$$JsonRpcRequestImplFromJson(Map<String, dynamic> json) =>
    _$JsonRpcRequestImpl(
      jsonrpc: json['jsonrpc'] as String,
      method: json['method'] as String,
      params: json['params'] as Map<String, dynamic>?,
      id: RequestId.fromJson(json['id']),
    );

Map<String, dynamic> _$$JsonRpcRequestImplToJson(
  _$JsonRpcRequestImpl instance,
) => <String, dynamic>{
  'jsonrpc': instance.jsonrpc,
  'method': instance.method,
  'params': instance.params,
  'id': instance.id,
};

_$JsonRpcResponseImpl _$$JsonRpcResponseImplFromJson(
  Map<String, dynamic> json,
) => _$JsonRpcResponseImpl(
  jsonrpc: json['jsonrpc'] as String,
  result: json['result'],
  id: RequestId.fromJson(json['id']),
);

Map<String, dynamic> _$$JsonRpcResponseImplToJson(
  _$JsonRpcResponseImpl instance,
) => <String, dynamic>{
  'jsonrpc': instance.jsonrpc,
  'result': instance.result,
  'id': instance.id,
};

_$ErrorObjectImpl _$$ErrorObjectImplFromJson(Map<String, dynamic> json) =>
    _$ErrorObjectImpl(
      code: (json['code'] as num).toInt(),
      message: json['message'] as String,
      data: json['data'],
    );

Map<String, dynamic> _$$ErrorObjectImplToJson(_$ErrorObjectImpl instance) =>
    <String, dynamic>{
      'code': instance.code,
      'message': instance.message,
      'data': instance.data,
    };

_$JsonRpcErrorImpl _$$JsonRpcErrorImplFromJson(Map<String, dynamic> json) =>
    _$JsonRpcErrorImpl(
      jsonrpc: json['jsonrpc'] as String,
      error: ErrorObject.fromJson(json['error'] as Map<String, dynamic>),
      id: RequestId.fromJson(json['id']),
    );

Map<String, dynamic> _$$JsonRpcErrorImplToJson(_$JsonRpcErrorImpl instance) =>
    <String, dynamic>{
      'jsonrpc': instance.jsonrpc,
      'error': instance.error,
      'id': instance.id,
    };

_$JsonRpcNotificationImpl _$$JsonRpcNotificationImplFromJson(
  Map<String, dynamic> json,
) => _$JsonRpcNotificationImpl(
  jsonrpc: json['jsonrpc'] as String,
  method: json['method'] as String,
  params: json['params'] as Map<String, dynamic>,
);

Map<String, dynamic> _$$JsonRpcNotificationImplToJson(
  _$JsonRpcNotificationImpl instance,
) => <String, dynamic>{
  'jsonrpc': instance.jsonrpc,
  'method': instance.method,
  'params': instance.params,
};

_$DaemonDeviceInfoImpl _$$DaemonDeviceInfoImplFromJson(
  Map<String, dynamic> json,
) => _$DaemonDeviceInfoImpl(
  path: json['path'] as String,
  name: json['name'] as String,
  vendorId: json['vendorId'] as String?,
  productId: json['productId'] as String?,
);

Map<String, dynamic> _$$DaemonDeviceInfoImplToJson(
  _$DaemonDeviceInfoImpl instance,
) => <String, dynamic>{
  'path': instance.path,
  'name': instance.name,
  'vendorId': instance.vendorId,
  'productId': instance.productId,
};

_$EngineStatusImpl _$$EngineStatusImplFromJson(Map<String, dynamic> json) =>
    _$EngineStatusImpl(
      status: json['status'] as String,
      configPath: json['configPath'] as String?,
      scriptPath: json['scriptPath'] as String?,
      devices:
          (json['devices'] as List<dynamic>?)
              ?.map((e) => DaemonDeviceInfo.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      uptimeSeconds: (json['uptimeSeconds'] as num?)?.toInt(),
      eventsProcessed: (json['eventsProcessed'] as num?)?.toInt(),
    );

Map<String, dynamic> _$$EngineStatusImplToJson(_$EngineStatusImpl instance) =>
    <String, dynamic>{
      'status': instance.status,
      'configPath': instance.configPath,
      'scriptPath': instance.scriptPath,
      'devices': instance.devices,
      'uptimeSeconds': instance.uptimeSeconds,
      'eventsProcessed': instance.eventsProcessed,
    };

_$InputEventImpl _$$InputEventImplFromJson(Map<String, dynamic> json) =>
    _$InputEventImpl(
      timestampUs: (json['timestampUs'] as num).toInt(),
      devicePath: json['devicePath'] as String,
      deviceName: json['deviceName'] as String,
      eventType: json['eventType'] as String,
      keyCode: (json['keyCode'] as num).toInt(),
      keyName: json['keyName'] as String,
      modifiers:
          (json['modifiers'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
    );

Map<String, dynamic> _$$InputEventImplToJson(_$InputEventImpl instance) =>
    <String, dynamic>{
      'timestampUs': instance.timestampUs,
      'devicePath': instance.devicePath,
      'deviceName': instance.deviceName,
      'eventType': instance.eventType,
      'keyCode': instance.keyCode,
      'keyName': instance.keyName,
      'modifiers': instance.modifiers,
    };

_$OutputEventImpl _$$OutputEventImplFromJson(Map<String, dynamic> json) =>
    _$OutputEventImpl(
      timestampUs: (json['timestampUs'] as num).toInt(),
      actionType: json['actionType'] as String,
      keyCode: (json['keyCode'] as num).toInt(),
      keyName: json['keyName'] as String,
    );

Map<String, dynamic> _$$OutputEventImplToJson(_$OutputEventImpl instance) =>
    <String, dynamic>{
      'timestampUs': instance.timestampUs,
      'actionType': instance.actionType,
      'keyCode': instance.keyCode,
      'keyName': instance.keyName,
    };

_$StateChangeEventImpl _$$StateChangeEventImplFromJson(
  Map<String, dynamic> json,
) => _$StateChangeEventImpl(
  type: json['type'] as String,
  timestampUs: (json['timestampUs'] as num).toInt(),
  message: json['message'] as String,
  details: json['details'] as Map<String, dynamic>?,
);

Map<String, dynamic> _$$StateChangeEventImplToJson(
  _$StateChangeEventImpl instance,
) => <String, dynamic>{
  'type': instance.type,
  'timestampUs': instance.timestampUs,
  'message': instance.message,
  'details': instance.details,
};
