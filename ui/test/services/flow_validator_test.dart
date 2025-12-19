import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/flow_validator.dart';

void main() {
  group('FlowValidator', () {
    late FlowValidator validator;

    setUp(() {
      validator = const FlowValidator();
    });

    test('validates empty graph successfully', () {
      const graph = FlowGraph();
      final errors = validator.validate(graph);

      expect(errors, isEmpty);
      expect(validator.isValid(graph), isTrue);
    });

    test('detects disconnected device', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
            isConnected: false,
          ),
        ],
      );

      final errors = validator.validate(graph);

      expect(errors, hasLength(1));
      expect(errors.first.message, 'Device disconnected');
      expect(errors.first.severity, ValidationSeverity.info);
    });

    test('detects device with no slots', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
        ],
      );

      final errors = validator.validate(graph);

      expect(errors, hasLength(1));
      expect(errors.first.message, 'No slots configured');
      expect(errors.first.severity, ValidationSeverity.info);
    });

    test('detects priority conflict', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
          FlowNode.hardwareProfile(id: 'profile-2'),
          FlowNode.keymap(id: 'keymap-1'),
          FlowNode.keymap(id: 'keymap-2'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
          ),
          FlowEdge(
            from: 'profile-1',
            to: 'keymap-1',
            slotId: 'slot-1',
            priority: 1,
          ),
          FlowEdge(
            from: 'device-1',
            to: 'profile-2',
            slotId: 'slot-2',
            priority: 1, // Same priority as slot-1!
          ),
          FlowEdge(
            from: 'profile-2',
            to: 'keymap-2',
            slotId: 'slot-2',
            priority: 1,
          ),
        ],
      );

      final errors = validator.validate(graph);

      final priorityErrors = errors.where(
        (e) => e.message == 'Priority conflict on device',
      );
      expect(priorityErrors, isNotEmpty);
      expect(priorityErrors.first.severity, ValidationSeverity.warning);
    });

    test('detects invalid edge status', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
          FlowNode.keymap(id: 'keymap-1'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
            status: EdgeStatus.invalid,
          ),
        ],
      );

      final errors = validator.validate(graph);

      final invalidEdgeErrors = errors.where(
        (e) => e.message == 'Invalid slot configuration',
      );
      expect(invalidEdgeErrors, isNotEmpty);
      expect(invalidEdgeErrors.first.severity, ValidationSeverity.error);
    });

    test('detects missing source node', () {
      const graph = FlowGraph(
        nodes: [FlowNode.hardwareProfile(id: 'profile-1')],
        edges: [
          FlowEdge(
            from: 'device-999', // This device doesn't exist
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
          ),
        ],
      );

      final errors = validator.validate(graph);

      final missingNodeErrors = errors.where(
        (e) => e.message.contains('Missing source node'),
      );
      expect(missingNodeErrors, isNotEmpty);
      expect(missingNodeErrors.first.severity, ValidationSeverity.error);
    });

    test('detects missing target node', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-999', // This profile doesn't exist
            slotId: 'slot-1',
            priority: 1,
          ),
        ],
      );

      final errors = validator.validate(graph);

      final missingNodeErrors = errors.where(
        (e) => e.message.contains('Missing target node'),
      );
      expect(missingNodeErrors, isNotEmpty);
      expect(missingNodeErrors.first.severity, ValidationSeverity.error);
    });

    test('detects unused hardware profile', () {
      const graph = FlowGraph(
        nodes: [FlowNode.hardwareProfile(id: 'profile-orphan')],
      );

      final errors = validator.validate(graph);

      final orphanErrors = errors.where(
        (e) => e.message.contains('Unused hardware profile'),
      );
      expect(orphanErrors, isNotEmpty);
      expect(orphanErrors.first.severity, ValidationSeverity.info);
    });

    test('detects unused keymap', () {
      const graph = FlowGraph(nodes: [FlowNode.keymap(id: 'keymap-orphan')]);

      final errors = validator.validate(graph);

      final orphanErrors = errors.where(
        (e) => e.message.contains('Unused keymap'),
      );
      expect(orphanErrors, isNotEmpty);
      expect(orphanErrors.first.severity, ValidationSeverity.info);
    });

    test('getNodeErrors filters errors for specific node', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
            isConnected: false,
          ),
          FlowNode.device(
            id: 'device-2',
            deviceId: DeviceInstanceId(vendorId: 2, productId: 2),
          ),
        ],
      );

      final device1Errors = validator.getNodeErrors(graph, 'device-1');
      final device2Errors = validator.getNodeErrors(graph, 'device-2');

      expect(device1Errors, hasLength(1));
      expect(device1Errors.first.message, 'Device disconnected');
      expect(device2Errors, hasLength(1));
      expect(device2Errors.first.message, 'No slots configured');
    });

    test('getEdgeErrors filters errors for specific edge', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
            status: EdgeStatus.invalid,
          ),
        ],
      );

      final edgeErrors = validator.getEdgeErrors(graph, 'slot-1');

      expect(edgeErrors, isNotEmpty);
      expect(edgeErrors.first.edgeId, 'slot-1');
    });

    test('getErrorCounts returns counts by severity', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
            isConnected: false, // info
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
            status: EdgeStatus.invalid, // error
          ),
        ],
      );

      final counts = validator.getErrorCounts(graph);

      expect(counts[ValidationSeverity.error], greaterThan(0));
      expect(counts[ValidationSeverity.info], greaterThan(0));
    });

    test('isValid returns true for graph with only warnings/info', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
            isConnected: false, // info level
          ),
        ],
      );

      expect(validator.isValid(graph), isTrue);
    });

    test('isValid returns false for graph with errors', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
            status: EdgeStatus.invalid, // error!
          ),
        ],
      );

      expect(validator.isValid(graph), isFalse);
    });

    test('validates complex graph with multiple issues', () {
      const graph = FlowGraph(
        nodes: [
          FlowNode.device(
            id: 'device-1',
            deviceId: DeviceInstanceId(vendorId: 1, productId: 1),
          ),
          FlowNode.device(
            id: 'device-2',
            deviceId: DeviceInstanceId(vendorId: 2, productId: 2),
            isConnected: false, // info: disconnected
          ),
          FlowNode.hardwareProfile(id: 'profile-1'),
          FlowNode.hardwareProfile(id: 'profile-orphan'), // info: unused
          FlowNode.keymap(id: 'keymap-1'),
        ],
        edges: [
          FlowEdge(
            from: 'device-1',
            to: 'profile-1',
            slotId: 'slot-1',
            priority: 1,
            status: EdgeStatus.invalid, // error
          ),
          FlowEdge(
            from: 'profile-1',
            to: 'keymap-1',
            slotId: 'slot-1',
            priority: 1,
          ),
        ],
      );

      final errors = validator.validate(graph);

      // Should have multiple errors
      expect(errors.length, greaterThan(2));

      // Should have at least one error, warning, and info
      final errorCount = errors
          .where((e) => e.severity == ValidationSeverity.error)
          .length;
      final infoCount = errors
          .where((e) => e.severity == ValidationSeverity.info)
          .length;

      expect(errorCount, greaterThan(0));
      expect(infoCount, greaterThan(0));
    });
  });
}
