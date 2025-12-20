import 'dart:convert';
import 'dart:ui';

import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/flow/node_position.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/storage_keys.dart';
import 'keyflow_providers.dart';

part 'node_positions_provider.g.dart';

@Riverpod(keepAlive: true)
class NodePositions extends _$NodePositions {
  @override
  Future<Map<String, NodePosition>> build() async {
    final graph = await ref.watch(flowGraphProvider.future);
    final manualPositions = await _readStoredManual();
    final layout = _autoLayout(graph);
    return {...layout, ...manualPositions};
  }

  /// Persist a single node position. Marks as manual by default.
  Future<void> setPosition(
    String nodeId,
    Offset position, {
    bool manual = true,
  }) async {
    final manualPositions = _manualOnly(state.value ?? await future);
    final next = NodePosition(
      x: position.dx,
      y: position.dy,
      isManual: manual,
      updatedAt: DateTime.now(),
    );
    if (manualPositions[nodeId] == next) return;

    manualPositions[nodeId] = next;
    await _updateState(manualPositions);
  }

  /// Persist multiple node positions efficiently, ignoring unchanged entries.
  Future<void> setPositions(
    Map<String, Offset> positions, {
    bool manual = true,
  }) async {
    final manualPositions = _manualOnly(state.value ?? await future);

    var changed = false;
    final timestamp = DateTime.now();
    positions.forEach((id, offset) {
      final existing = manualPositions[id];
      final updated = NodePosition(
        x: offset.dx,
        y: offset.dy,
        isManual: manual,
        updatedAt: timestamp,
      );
      if (existing != updated) {
        manualPositions[id] = updated;
        changed = true;
      }
    });

    if (changed) {
      await _updateState(manualPositions);
    }
  }

  /// Remove a stored manual position so it falls back to auto layout.
  Future<void> removePosition(String nodeId) async {
    final manualPositions = _manualOnly(state.value ?? await future)
      ..remove(nodeId);
    await _updateState(manualPositions);
  }

  /// Reset all manual positions and recompute the default layout.
  Future<void> resetToDefaultLayout() async {
    await _updateState({});
  }

  /// Move a node by a delta offset, constrained within canvas bounds.
  Future<void> moveNode(String nodeId, Offset delta, Size canvasBounds) async {
    final positions = state.valueOrNull ?? {};
    final current = positions[nodeId];

    if (current == null) return;

    // Use x/y directly if offset helper is missing, but assuming offset helper exists or constructing it
    final currentOffset = Offset(current.x, current.y);
    final newOffset = currentOffset + delta;

    // Keep node within canvas bounds (with some margin)
    final constrainedOffset = Offset(
      newOffset.dx.clamp(0.0, canvasBounds.width - 200),
      newOffset.dy.clamp(0.0, canvasBounds.height - 200),
    );

    // Call setPosition (which is async) but we can fire and forget or await
    await setPosition(nodeId, constrainedOffset);
  }

  Map<String, NodePosition> _manualOnly(Map<String, NodePosition> source) =>
      Map<String, NodePosition>.fromEntries(
        source.entries.where((entry) => entry.value.isManual),
      );

  Future<void> _updateState(Map<String, NodePosition> manualPositions) async {
    final graph = await ref.read(flowGraphProvider.future);
    final layout = _autoLayout(graph);
    final merged = {...layout, ...manualPositions};
    state = AsyncValue.data(merged);
    await _persist(manualPositions);
  }

  Future<Map<String, NodePosition>> _readStoredManual() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(StorageKeys.keyFlowNodePositions);
    if (raw == null || raw.isEmpty) return {};

    try {
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      return decoded.map(
        (key, value) => MapEntry(
          key,
          NodePosition.fromJson(Map<String, dynamic>.from(value as Map)),
        ),
      );
    } on Object catch (_) {
      await prefs.remove(StorageKeys.keyFlowNodePositions);
      return {};
    }
  }

  Future<void> _persist(Map<String, NodePosition> manualPositions) async {
    final prefs = await SharedPreferences.getInstance();
    if (manualPositions.isEmpty) {
      await prefs.remove(StorageKeys.keyFlowNodePositions);
      return;
    }
    final serialized = jsonEncode(
      manualPositions.map((key, value) => MapEntry(key, value.toJson())),
    );
    await prefs.setString(StorageKeys.keyFlowNodePositions, serialized);
  }
}

Map<String, NodePosition> _autoLayout(FlowGraph graph) {
  const startY = 80.0;
  const verticalGap = 180.0;
  const deviceX = 80.0;
  const profileX = 460.0;
  const keymapX = 820.0;

  final devices =
      graph.nodes
          .where(
            (node) => node.map(
              device: (_) => true,
              hardwareProfile: (_) => false,
              keymap: (_) => false,
            ),
          )
          .toList()
        ..sort((a, b) => a.nodeId.compareTo(b.nodeId));
  final profiles =
      graph.nodes
          .where(
            (node) => node.map(
              device: (_) => false,
              hardwareProfile: (_) => true,
              keymap: (_) => false,
            ),
          )
          .toList()
        ..sort((a, b) => a.nodeId.compareTo(b.nodeId));
  final keymaps =
      graph.nodes
          .where(
            (node) => node.map(
              device: (_) => false,
              hardwareProfile: (_) => false,
              keymap: (_) => true,
            ),
          )
          .toList()
        ..sort((a, b) => a.nodeId.compareTo(b.nodeId));

  final positions = <String, NodePosition>{};

  void layoutColumn(List<FlowNode> nodes, double x) {
    var y = startY;
    for (final node in nodes) {
      positions[node.nodeId] = NodePosition(x: x, y: y);
      y += verticalGap;
    }
  }

  layoutColumn(devices, deviceX);
  layoutColumn(profiles, profileX);
  layoutColumn(keymaps, keymapX);

  return positions;
}
