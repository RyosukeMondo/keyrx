import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/runtime_config.dart';

void main() {
  const device = DeviceInstanceId(
    vendorId: 1200,
    productId: 3400,
    serial: 'xyz',
  );

  test('builds flow graph nodes and edges from runtime config', () {
    final config = _configWithSlots(
      [
        _slot(
          id: 'slot-a',
          hardwareProfileId: 'hp-a',
          keymapId: 'km-a',
          priority: 0,
          active: true,
        ),
        _slot(
          id: 'slot-b',
          hardwareProfileId: 'hp-b',
          keymapId: 'km-b',
          priority: 1,
        ),
      ],
      device: device,
    );

    final graph = FlowGraph.fromRuntimeConfig(config);

    expect(graph.nodes.length, 5);
    expect(
      graph.nodes.whereType<DeviceFlowNode>().single.id,
      equals('1200:3400:xyz'),
    );
    expect(graph.edges.length, 4);
    expect(
      graph.edges.where((edge) => edge.slotId == 'slot-a').every(
            (edge) =>
                edge.active &&
                edge.priority == 0 &&
                edge.status == EdgeStatus.valid,
          ),
      isTrue,
    );
  });

  test('deduplicates shared nodes across slots', () {
    final config = _configWithSlots(
      [
        _slot(
          id: 'slot-a',
          hardwareProfileId: 'hp-shared',
          keymapId: 'km-shared',
          priority: 0,
        ),
        _slot(
          id: 'slot-b',
          hardwareProfileId: 'hp-shared',
          keymapId: 'km-other',
          priority: 1,
        ),
      ],
      device: device,
    );

    final graph = FlowGraph.fromRuntimeConfig(config);

    expect(
      graph.nodes.whereType<HardwareProfileFlowNode>().length,
      1,
    );
    expect(graph.nodes.whereType<KeymapFlowNode>().length, 2);
  });

  test('marks duplicate priorities as warnings', () {
    final config = _configWithSlots(
      [
        _slot(
          id: 'slot-a',
          hardwareProfileId: 'hp-a',
          keymapId: 'km-a',
          priority: 0,
        ),
        _slot(
          id: 'slot-b',
          hardwareProfileId: 'hp-b',
          keymapId: 'km-b',
          priority: 0,
        ),
      ],
      device: device,
    );

    final graph = FlowGraph.fromRuntimeConfig(config);

    expect(
      graph.edges.where((edge) => edge.slotId == 'slot-a').every(
            (edge) => edge.status == EdgeStatus.valid,
          ),
      isTrue,
    );
    expect(
      graph.edges.where((edge) => edge.slotId == 'slot-b').every(
            (edge) => edge.status == EdgeStatus.warning,
          ),
      isTrue,
    );
  });

  test('ignores slots missing identifiers', () {
    final config = _configWithSlots(
      [
        _slot(
          id: 'slot-a',
          hardwareProfileId: '',
          keymapId: 'km-a',
        ),
        _slot(
          id: 'slot-b',
          hardwareProfileId: 'hp-b',
          keymapId: '',
        ),
      ],
      device: device,
    );

    final graph = FlowGraph.fromRuntimeConfig(config);

    expect(graph.nodes.length, 1, reason: 'Only the device node is valid');
    expect(graph.edges, isEmpty);
  });

  test('returns empty graph for empty config', () {
    const config = RuntimeConfig(devices: []);

    final graph = FlowGraph.fromRuntimeConfig(config);

    expect(graph.nodes, isEmpty);
    expect(graph.edges, isEmpty);
  });
}

ProfileSlot _slot({
  required String id,
  required String hardwareProfileId,
  required String keymapId,
  int priority = 0,
  bool active = false,
}) {
  return ProfileSlot(
    id: id,
    hardwareProfileId: hardwareProfileId,
    keymapId: keymapId,
    priority: priority,
    active: active,
  );
}

RuntimeConfig _configWithSlots(
  List<ProfileSlot> slots, {
  required DeviceInstanceId device,
}) {
  return RuntimeConfig(devices: [
    DeviceSlots(
      device: device,
      slots: slots,
    )
  ]);
}
