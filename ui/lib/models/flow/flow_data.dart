enum FlowNodeType { device, layout, keymap }

abstract class FlowNodeData {
  const FlowNodeData();
}

class DeviceNodeData extends FlowNodeData {
  final String name;
  final String vid;
  final String pid;
  final bool isConnected;
  final List<DeviceWiring> wirings;

  const DeviceNodeData({
    required this.name,
    required this.vid,
    required this.pid,
    required this.isConnected,
    this.wirings = const [],
  });
}

class DeviceWiring {
  final String id;
  final String name;
  final bool isActive;

  const DeviceWiring({
    required this.id,
    required this.name,
    required this.isActive,
  });
}

class LayoutNodeData extends FlowNodeData {
  final String name;
  final String type;

  const LayoutNodeData({required this.name, required this.type});
}

class KeymapNodeData extends FlowNodeData {
  final String name;
  final int layers;
  final bool isActive;

  const KeymapNodeData({
    required this.name,
    required this.layers,
    required this.isActive,
  });
}
