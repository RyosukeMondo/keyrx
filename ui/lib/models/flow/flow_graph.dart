library;

import 'package:freezed_annotation/freezed_annotation.dart';

import '../config_ids.dart';
import '../hardware_profile.dart';
import '../runtime_config.dart';

part 'flow_graph.freezed.dart';
part 'flow_graph.g.dart';

@freezed
class FlowPosition with _$FlowPosition {
  const factory FlowPosition({required double x, required double y}) =
      _FlowPosition;

  factory FlowPosition.fromJson(Map<String, dynamic> json) =>
      _$FlowPositionFromJson(json);
}

@freezed
class FlowEdge with _$FlowEdge {
  const factory FlowEdge({
    required String from,
    required String to,
    required String slotId,
    required int priority,
    @Default(false) bool active,
    @Default(EdgeStatus.valid) EdgeStatus status,
  }) = _FlowEdge;

  factory FlowEdge.fromJson(Map<String, dynamic> json) =>
      _$FlowEdgeFromJson(json);
}

@Freezed(unionKey: 'type')
class FlowNode with _$FlowNode {
  const factory FlowNode.device({
    required String id,
    required DeviceInstanceId deviceId,
    @Default(true) bool isConnected,
    String? label,
  }) = DeviceFlowNode;

  const factory FlowNode.hardwareProfile({
    required HardwareProfileId id,
    String? label,
  }) = HardwareProfileFlowNode;

  const factory FlowNode.keymap({
    required KeymapId id,
    String? label,
    int? layers,
  }) = KeymapFlowNode;

  factory FlowNode.fromJson(Map<String, dynamic> json) =>
      _$FlowNodeFromJson(json);
}

@JsonEnum()
enum EdgeStatus { valid, invalid, warning }

@freezed
class FlowGraph with _$FlowGraph {
  const factory FlowGraph({
    @Default(<FlowNode>[]) List<FlowNode> nodes,
    @Default(<FlowEdge>[]) List<FlowEdge> edges,
    @Default(<String, FlowPosition>{}) Map<String, FlowPosition> positions,
  }) = _FlowGraph;

  const FlowGraph._();

  factory FlowGraph.fromJson(Map<String, dynamic> json) =>
      _$FlowGraphFromJson(json);

  factory FlowGraph.fromRuntimeConfig(RuntimeConfig config) {
    final nodeMap = <String, FlowNode>{};
    final edges = <FlowEdge>[];
    final priorityCounts = <String, Map<int, int>>{};

    for (final deviceSlots in config.devices) {
      final deviceKey = _deviceKey(deviceSlots.device);
      nodeMap.putIfAbsent(
        deviceKey,
        () => FlowNode.device(
          id: deviceKey,
          deviceId: deviceSlots.device,
          label: _deviceLabel(deviceSlots.device),
        ),
      );

      final devicePriorityCounts = priorityCounts.putIfAbsent(
        deviceKey,
        () => {},
      );

      for (final slot in deviceSlots.slots) {
        if (slot.hardwareProfileId.isEmpty || slot.keymapId.isEmpty) {
          continue;
        }

        final count = (devicePriorityCounts[slot.priority] ?? 0) + 1;
        devicePriorityCounts[slot.priority] = count;
        final status = count > 1 ? EdgeStatus.warning : EdgeStatus.valid;

        nodeMap.putIfAbsent(
          slot.hardwareProfileId,
          () => FlowNode.hardwareProfile(
            id: slot.hardwareProfileId,
            label: slot.hardwareProfileId,
          ),
        );
        nodeMap.putIfAbsent(
          slot.keymapId,
          () => FlowNode.keymap(id: slot.keymapId, label: slot.keymapId),
        );

        edges.add(
          FlowEdge(
            from: deviceKey,
            to: slot.hardwareProfileId,
            slotId: slot.id,
            priority: slot.priority,
            active: slot.active,
            status: status,
          ),
        );
        edges.add(
          FlowEdge(
            from: slot.hardwareProfileId,
            to: slot.keymapId,
            slotId: slot.id,
            priority: slot.priority,
            active: slot.active,
            status: status,
          ),
        );
      }
    }

    final nodes = nodeMap.values.toList()
      ..sort((a, b) => a.nodeId.compareTo(b.nodeId));
    final sortedEdges = List<FlowEdge>.from(edges)..sort(_edgeSorter);

    return FlowGraph(nodes: nodes, edges: sortedEdges);
  }
}

int _edgeSorter(FlowEdge a, FlowEdge b) {
  final slotCompare = a.slotId.compareTo(b.slotId);
  if (slotCompare != 0) return slotCompare;

  final priorityCompare = a.priority.compareTo(b.priority);
  if (priorityCompare != 0) return priorityCompare;

  final fromCompare = a.from.compareTo(b.from);
  if (fromCompare != 0) return fromCompare;

  return a.to.compareTo(b.to);
}

String _deviceKey(DeviceInstanceId id) {
  final serial = id.serial?.isNotEmpty == true ? id.serial : 'unknown';
  return '${id.vendorId}:${id.productId}:$serial';
}

String _deviceLabel(DeviceInstanceId id) {
  final serialSuffix = id.serial?.isNotEmpty == true ? ' (${id.serial})' : '';
  return 'Device ${id.vendorId}:${id.productId}$serialSuffix';
}

extension FlowNodeId on FlowNode {
  String get nodeId => when(
    device: (id, _, __, ___) => id,
    hardwareProfile: (id, __) => id,
    keymap: (id, __, ___) => id,
  );
}
