enum FlowNodeType { device, layout, keymap }

abstract class FlowNodeData {
  const FlowNodeData();
}

class DeviceNodeData extends FlowNodeData {
  const DeviceNodeData({
    required this.name,
    required this.vid,
    required this.pid,
    required this.isConnected,
    this.wirings = const [],
  });
  final String name;
  final String vid;
  final String pid;
  final bool isConnected;
  final List<DeviceWiring> wirings;
}

class DeviceWiring {
  const DeviceWiring({
    required this.id,
    required this.name,
    required this.isActive,
  });
  final String id;
  final String name;
  final bool isActive;
}

class LayoutNodeData extends FlowNodeData {
  const LayoutNodeData({required this.name, required this.type});
  final String name;
  final String type;
}

class KeymapNodeData extends FlowNodeData {
  const KeymapNodeData({
    required this.name,
    required this.layers,
    required this.isActive,
  });
  final String name;
  final int layers;
  final bool isActive;
}
