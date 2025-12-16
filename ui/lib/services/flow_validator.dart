/// Flow graph validation service with clear error messages.
library;

import '../models/flow/flow_graph.dart';

/// Validation error for a specific node or edge in the flow graph.
class ValidationError {
  const ValidationError({
    required this.nodeId,
    required this.message,
    required this.suggestion,
    this.edgeId,
    this.severity = ValidationSeverity.error,
  });

  /// ID of the node with the error
  final String nodeId;

  /// ID of the edge with the error (if applicable)
  final String? edgeId;

  /// User-friendly error message
  final String message;

  /// Actionable suggestion to fix the error
  final String suggestion;

  /// Severity level
  final ValidationSeverity severity;

  @override
  String toString() => '$message: $suggestion';

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is ValidationError &&
          runtimeType == other.runtimeType &&
          nodeId == other.nodeId &&
          edgeId == other.edgeId &&
          message == other.message;

  @override
  int get hashCode => Object.hash(nodeId, edgeId, message);
}

/// Severity level for validation errors.
enum ValidationSeverity {
  /// Error that must be fixed
  error,

  /// Warning that should be reviewed
  warning,

  /// Informational message
  info,
}

/// Service for validating flow graphs and providing actionable feedback.
class FlowValidator {
  const FlowValidator();

  /// Validates entire flow graph and returns list of errors.
  ///
  /// Checks for:
  /// - Missing/invalid hardware profiles
  /// - Missing/invalid keymaps
  /// - Orphaned slots (disconnected edges)
  /// - Priority conflicts (duplicate priorities on same device)
  /// - Disconnected devices
  List<ValidationError> validate(FlowGraph graph) {
    final errors = <ValidationError>[];

    // Track which nodes are referenced by edges
    final referencedProfiles = <String>{};
    final referencedKeymaps = <String>{};

    // Track priority usage per device
    final devicePriorities = <String, Map<int, int>>{};

    // Validate edges and collect referenced nodes
    for (final edge in graph.edges) {
      // Track which profiles and keymaps are actually used
      final fromNode = graph.nodes
          .where((n) => n.nodeId == edge.from)
          .firstOrNull;
      final toNode = graph.nodes.where((n) => n.nodeId == edge.to).firstOrNull;

      // Check if edge endpoints exist
      if (fromNode == null) {
        errors.add(
          ValidationError(
            nodeId: edge.from,
            edgeId: edge.slotId,
            message: 'Missing source node "${edge.from}"',
            suggestion:
                'This edge references a node that does not exist. '
                'Remove the slot or fix the configuration.',
          ),
        );
      }

      if (toNode == null) {
        errors.add(
          ValidationError(
            nodeId: edge.to,
            edgeId: edge.slotId,
            message: 'Missing target node "${edge.to}"',
            suggestion:
                'This edge references a node that does not exist. '
                'Remove the slot or fix the configuration.',
          ),
        );
      }

      // Track referenced profiles and keymaps
      toNode?.when(
        device: (_, __, ___, ____) {}, // devices don't need tracking
        hardwareProfile: (id, _) => referencedProfiles.add(id),
        keymap: (id, _, __) => referencedKeymaps.add(id),
      );

      // Track priority conflicts per device
      fromNode?.when(
        device: (id, _, __, ___) {
          final priorities = devicePriorities.putIfAbsent(id, () => {});
          final count = (priorities[edge.priority] ?? 0) + 1;
          priorities[edge.priority] = count;

          if (count > 1) {
            errors.add(
              ValidationError(
                nodeId: id,
                edgeId: edge.slotId,
                message: 'Priority conflict on device',
                suggestion:
                    'Multiple slots on this device have priority ${edge.priority}. '
                    'Adjust priorities to ensure unique ordering.',
                severity: ValidationSeverity.warning,
              ),
            );
          }
        },
        hardwareProfile: (_, __) {},
        keymap: (_, __, ___) {},
      );

      // Validate edge status
      if (edge.status == EdgeStatus.invalid) {
        errors.add(
          ValidationError(
            nodeId: edge.from,
            edgeId: edge.slotId,
            message: 'Invalid slot configuration',
            suggestion:
                'This slot has validation errors. '
                'Check that the hardware profile and keymap are correctly configured.',
          ),
        );
      }
    }

    // Check for unreferenced profiles and keymaps (orphaned nodes)
    for (final node in graph.nodes) {
      node.when(
        device: (id, deviceId, isConnected, _) {
          // Check if device is disconnected
          if (!isConnected) {
            errors.add(
              ValidationError(
                nodeId: id,
                message: 'Device disconnected',
                suggestion: 'Connect the device to activate its slots.',
                severity: ValidationSeverity.info,
              ),
            );
          }

          // Check if device has no slots
          final hasSlots = graph.edges.any((e) => e.from == id);
          if (isConnected && !hasSlots) {
            errors.add(
              ValidationError(
                nodeId: id,
                message: 'No slots configured',
                suggestion:
                    'Add a slot to assign a hardware profile and keymap to this device.',
                severity: ValidationSeverity.info,
              ),
            );
          }
        },
        hardwareProfile: (id, _) {
          if (!referencedProfiles.contains(id)) {
            errors.add(
              ValidationError(
                nodeId: id,
                message: 'Unused hardware profile "$id"',
                suggestion:
                    'This profile is not connected to any device. '
                    'Create a slot to use it, or it may have been removed.',
                severity: ValidationSeverity.info,
              ),
            );
          }
        },
        keymap: (id, _, __) {
          if (!referencedKeymaps.contains(id)) {
            errors.add(
              ValidationError(
                nodeId: id,
                message: 'Unused keymap "$id"',
                suggestion:
                    'This keymap is not connected to any device. '
                    'Create a slot to use it, or it may have been removed.',
                severity: ValidationSeverity.info,
              ),
            );
          }
        },
      );
    }

    return errors;
  }

  /// Gets all errors for a specific node.
  List<ValidationError> getNodeErrors(FlowGraph graph, String nodeId) =>
      validate(graph).where((e) => e.nodeId == nodeId).toList();

  /// Gets all errors for a specific edge.
  List<ValidationError> getEdgeErrors(FlowGraph graph, String edgeId) =>
      validate(graph).where((e) => e.edgeId == edgeId).toList();

  /// Checks if the graph is valid (no errors, only warnings/info allowed).
  bool isValid(FlowGraph graph) => validate(
    graph,
  ).where((e) => e.severity == ValidationSeverity.error).isEmpty;

  /// Gets count of errors by severity.
  Map<ValidationSeverity, int> getErrorCounts(FlowGraph graph) {
    final errors = validate(graph);
    return {
      ValidationSeverity.error: errors
          .where((e) => e.severity == ValidationSeverity.error)
          .length,
      ValidationSeverity.warning: errors
          .where((e) => e.severity == ValidationSeverity.warning)
          .length,
      ValidationSeverity.info: errors
          .where((e) => e.severity == ValidationSeverity.info)
          .length,
    };
  }
}
