import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_device_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_keymap_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_layout_node.dart';

import '../../helpers/golden_test_harness.dart';
import '../../helpers/widget_test_harness.dart';

void main() {
  final darkHarness = GoldenTestHarness();
  final lightHarness = GoldenTestHarness(
    widgetHarness: WidgetTestHarness(
      theme: ThemeData.light(useMaterial3: true),
    ),
  );

  Widget nodeShell(Widget child, {Color? backgroundColor}) => Scaffold(
    backgroundColor: backgroundColor ?? Colors.black,
    body: Center(child: child),
  );

  group('DeviceNode Goldens', () {
    const mockDeviceId = DeviceInstanceId(
      vendorId: 0x3434,
      productId: 0x0111,
      serial: 'KB12345678',
    );

    testGoldens('renders device node - connected active', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-1',
                      deviceId: mockDeviceId,
                      label: 'Keychron Q1',
                    )
                    as DeviceFlowNode,
            activeSlotCount: 2,
            totalSlotCount: 3,
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_connected_active');
    });

    testGoldens('renders device node - connected inactive', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-2',
                      deviceId: mockDeviceId,
                      label: 'HHKB Pro',
                    )
                    as DeviceFlowNode,
            totalSlotCount: 2,
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_connected_inactive');
    });

    testGoldens('renders device node - disconnected', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-3',
                      deviceId: mockDeviceId,
                      isConnected: false,
                      label: 'Moonlander',
                    )
                    as DeviceFlowNode,
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_disconnected');
    });

    testGoldens('renders device node - error state', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-4',
                      deviceId: mockDeviceId,
                      label: 'ErgoDox EZ',
                    )
                    as DeviceFlowNode,
            activeSlotCount: 1,
            totalSlotCount: 2,
            hasError: true,
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_error');
    });

    testGoldens('renders device node - with add slot button', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          EnhancedDeviceNode(
            node:
                const FlowNode.device(
                      id: 'device-5',
                      deviceId: mockDeviceId,
                      label: 'Keychron K8',
                    )
                    as DeviceFlowNode,
            activeSlotCount: 1,
            totalSlotCount: 1,
            onAddSlot: () {},
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_with_add_button');
    });

    testGoldens('renders device node - no serial', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-6',
                      deviceId: DeviceInstanceId(
                        vendorId: 0x3434,
                        productId: 0x0111,
                      ),
                      label: 'Generic Keyboard',
                    )
                    as DeviceFlowNode,
            activeSlotCount: 1,
            totalSlotCount: 1,
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'device_node_no_serial');
    });

    testGoldens('renders device node - light theme', (tester) async {
      await lightHarness.pump(
        tester,
        nodeShell(
          const EnhancedDeviceNode(
            node:
                FlowNode.device(
                      id: 'device-7',
                      deviceId: mockDeviceId,
                      label: 'Keychron Q1',
                    )
                    as DeviceFlowNode,
            activeSlotCount: 2,
            totalSlotCount: 3,
          ),
          backgroundColor: Colors.white,
        ),
      );

      await lightHarness.expectGolden(tester, 'device_node_light_theme');
    });
  });

  group('LayoutNode Goldens', () {
    testGoldens('renders layout node - valid', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedLayoutNode(
            node: HardwareProfileFlowNode(id: 'ergo-60', label: 'Ergo-60'),
            layoutType: 'Split Ergonomic',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'layout_node_valid');
    });

    testGoldens('renders layout node - invalid', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedLayoutNode(
            node: HardwareProfileFlowNode(
              id: 'missing-profile',
              label: 'Missing Profile',
            ),
            isValid: false,
            errorMessage: 'Hardware profile not found',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'layout_node_invalid');
    });

    testGoldens('renders layout node - standard layout', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedLayoutNode(
            node: HardwareProfileFlowNode(id: 'ansi-60', label: 'ANSI 60%'),
            layoutType: 'Standard ANSI',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'layout_node_standard');
    });

    testGoldens('renders layout node - long label', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedLayoutNode(
            node: HardwareProfileFlowNode(
              id: 'long-name',
              label: 'Very Long Hardware Profile Name That Should Truncate',
            ),
            layoutType: 'Custom Layout',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'layout_node_long_label');
    });

    testGoldens('renders layout node - light theme', (tester) async {
      await lightHarness.pump(
        tester,
        nodeShell(
          const EnhancedLayoutNode(
            node: HardwareProfileFlowNode(id: 'ergo-60', label: 'Ergo-60'),
            layoutType: 'Split Ergonomic',
          ),
          backgroundColor: Colors.white,
        ),
      );

      await lightHarness.expectGolden(tester, 'layout_node_light_theme');
    });
  });

  group('KeymapNode Goldens', () {
    testGoldens('renders keymap node - valid', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(id: 'default', label: 'Default', layers: 3),
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_valid');
    });

    testGoldens('renders keymap node - compilation failed', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(id: 'gaming', label: 'Gaming', layers: 2),
            isValid: false,
            compilationFailed: true,
            errorMessage: 'Failed to compile keymap: syntax error',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_compilation_failed');
    });

    testGoldens('renders keymap node - invalid', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(id: 'missing', label: 'Missing Keymap'),
            isValid: false,
            errorMessage: 'Keymap not found',
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_invalid');
    });

    testGoldens('renders keymap node - single layer', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(id: 'simple', label: 'Simple', layers: 1),
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_single_layer');
    });

    testGoldens('renders keymap node - many layers', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(
              id: 'complex',
              label: 'Complex Layout',
              layers: 8,
            ),
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_many_layers');
    });

    testGoldens('renders keymap node - long label', (tester) async {
      await darkHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(
              id: 'long-name',
              label: 'Very Long Keymap Name That Should Truncate Properly',
              layers: 4,
            ),
          ),
        ),
      );

      await darkHarness.expectGolden(tester, 'keymap_node_long_label');
    });

    testGoldens('renders keymap node - light theme', (tester) async {
      await lightHarness.pump(
        tester,
        nodeShell(
          const EnhancedKeymapNode(
            node: KeymapFlowNode(id: 'default', label: 'Default', layers: 3),
          ),
          backgroundColor: Colors.white,
        ),
      );

      await lightHarness.expectGolden(tester, 'keymap_node_light_theme');
    });
  });
}
