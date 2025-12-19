// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'flow_graph.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_FlowPosition _$FlowPositionFromJson(Map<String, dynamic> json) =>
    _FlowPosition(
      x: (json['x'] as num).toDouble(),
      y: (json['y'] as num).toDouble(),
    );

Map<String, dynamic> _$FlowPositionToJson(_FlowPosition instance) =>
    <String, dynamic>{'x': instance.x, 'y': instance.y};

_FlowEdge _$FlowEdgeFromJson(Map<String, dynamic> json) => _FlowEdge(
  from: json['from'] as String,
  to: json['to'] as String,
  slotId: json['slotId'] as String,
  priority: (json['priority'] as num).toInt(),
  active: json['active'] as bool? ?? false,
  status:
      $enumDecodeNullable(_$EdgeStatusEnumMap, json['status']) ??
      EdgeStatus.valid,
);

Map<String, dynamic> _$FlowEdgeToJson(_FlowEdge instance) => <String, dynamic>{
  'from': instance.from,
  'to': instance.to,
  'slotId': instance.slotId,
  'priority': instance.priority,
  'active': instance.active,
  'status': _$EdgeStatusEnumMap[instance.status]!,
};

const _$EdgeStatusEnumMap = {
  EdgeStatus.valid: 'valid',
  EdgeStatus.invalid: 'invalid',
  EdgeStatus.warning: 'warning',
};

DeviceFlowNode _$DeviceFlowNodeFromJson(Map<String, dynamic> json) =>
    DeviceFlowNode(
      id: json['id'] as String,
      deviceId: DeviceInstanceId.fromJson(
        json['deviceId'] as Map<String, dynamic>,
      ),
      isConnected: json['isConnected'] as bool? ?? true,
      label: json['label'] as String?,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$DeviceFlowNodeToJson(DeviceFlowNode instance) =>
    <String, dynamic>{
      'id': instance.id,
      'deviceId': instance.deviceId,
      'isConnected': instance.isConnected,
      'label': instance.label,
      'type': instance.$type,
    };

HardwareProfileFlowNode _$HardwareProfileFlowNodeFromJson(
  Map<String, dynamic> json,
) => HardwareProfileFlowNode(
  id: json['id'] as String,
  label: json['label'] as String?,
  $type: json['type'] as String?,
);

Map<String, dynamic> _$HardwareProfileFlowNodeToJson(
  HardwareProfileFlowNode instance,
) => <String, dynamic>{
  'id': instance.id,
  'label': instance.label,
  'type': instance.$type,
};

KeymapFlowNode _$KeymapFlowNodeFromJson(Map<String, dynamic> json) =>
    KeymapFlowNode(
      id: json['id'] as String,
      label: json['label'] as String?,
      layers: (json['layers'] as num?)?.toInt(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$KeymapFlowNodeToJson(KeymapFlowNode instance) =>
    <String, dynamic>{
      'id': instance.id,
      'label': instance.label,
      'layers': instance.layers,
      'type': instance.$type,
    };

_FlowGraph _$FlowGraphFromJson(Map<String, dynamic> json) => _FlowGraph(
  nodes:
      (json['nodes'] as List<dynamic>?)
          ?.map((e) => FlowNode.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <FlowNode>[],
  edges:
      (json['edges'] as List<dynamic>?)
          ?.map((e) => FlowEdge.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <FlowEdge>[],
  positions:
      (json['positions'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, FlowPosition.fromJson(e as Map<String, dynamic>)),
      ) ??
      const <String, FlowPosition>{},
);

Map<String, dynamic> _$FlowGraphToJson(_FlowGraph instance) =>
    <String, dynamic>{
      'nodes': instance.nodes,
      'edges': instance.edges,
      'positions': instance.positions,
    };
