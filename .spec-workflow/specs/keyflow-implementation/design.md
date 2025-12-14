# KeyFlow Implementation - Design

## Architecture Overview

KeyFlow uses a layered architecture following Riverpod + Repository pattern, with clear separation between UI, business logic, and data access.

```
┌─────────────────────────────────────────────────────┐
│              UI Layer (Widgets)                     │
│  FlowEditorPage, DeviceNodeWidget, etc.            │
│  - Presentation only                                │
│  - ConsumerWidget for state                        │
└──────────────────┬──────────────────────────────────┘
                   │ watch(provider)
┌──────────────────▼──────────────────────────────────┐
│         State Layer (Riverpod Providers)            │
│  keyFlowProvider, deviceNodesProvider, etc.         │
│  - State management                                 │
│  - Business logic orchestration                     │
└──────────────────┬──────────────────────────────────┘
                   │ inject
┌──────────────────▼──────────────────────────────────┐
│       Repository Layer (Data Access)                │
│  KeyFlowRepository                                  │
│  - FFI calls                                        │
│  - Data transformation                              │
│  - Caching                                          │
└──────────────────┬──────────────────────────────────┘
                   │ FFI
┌──────────────────▼──────────────────────────────────┐
│            FFI Bridge (Rust ↔ Dart)                 │
│  KeyrxBridge (real or mock)                         │
│  - RuntimeService                                   │
│  - DeviceRegistryService                            │
└─────────────────────────────────────────────────────┘
```

---

## Data Model Design

### Core Data Structures

#### FlowGraph (Top-level State)
```dart
@freezed
class FlowGraph with _$FlowGraph {
  const factory FlowGraph({
    required List<FlowNode> nodes,
    required List<FlowEdge> edges,
    required Map<String, Offset> nodePositions,
    DateTime? lastUpdated,
  }) = _FlowGraph;

  factory FlowGraph.fromRuntimeConfig(RuntimeConfig config) {
    // Transform RuntimeConfig into FlowGraph
  }
}
```

#### FlowNode (Generic Node)
```dart
@freezed
class FlowNode with _$FlowNode {
  const factory FlowNode.device({
    required String id,  // device serial
    required DeviceState device,
    required List<String> slotIds,
  }) = DeviceNode;

  const factory FlowNode.hardwareProfile({
    required String id,  // profile ID
    required HardwareProfile profile,
  }) = HardwareProfileNode;

  const factory FlowNode.keymap({
    required String id,  // keymap ID
    required Keymap keymap,
  }) = KeymapNode;
}
```

#### FlowEdge (Connection)
```dart
@freezed
class FlowEdge with _$FlowEdge {
  const factory FlowEdge({
    required String id,
    required String fromNodeId,
    required String toNodeId,
    required String slotId,  // ProfileSlot ID
    required int priority,
    required bool active,
    EdgeStatus? status,  // valid, invalid, warning
  }) = _FlowEdge;
}

enum EdgeStatus { valid, invalid, warning }
```

---

## Repository Layer

### KeyFlowRepository

**File**: `ui/lib/repositories/keyflow_repository.dart`

```dart
class KeyFlowRepository {
  final KeyrxBridge _bridge;
  final StructuredLogger _log;

  // Cache for expensive operations
  RuntimeConfig? _cachedConfig;
  DateTime? _cacheTime;

  KeyFlowRepository({
    required KeyrxBridge bridge,
    StructuredLogger? log,
  })  : _bridge = bridge,
        _log = log ?? getLogger('KeyFlowRepository');

  /// Fetch current runtime configuration from Rust
  Future<RuntimeConfig> getRuntimeConfig({bool forceRefresh = false}) async {
    if (!forceRefresh && _cachedConfig != null && _isCacheValid()) {
      _log.debug('runtime_config_cache_hit');
      return _cachedConfig!;
    }

    _log.info('fetching_runtime_config', {'force_refresh': forceRefresh});

    try {
      final stopwatch = Stopwatch()..start();
      final config = await _bridge.getRuntimeConfig();

      _cachedConfig = config;
      _cacheTime = DateTime.now();

      _log.info('runtime_config_fetched', {
        'device_count': config.devices.length,
        'duration_ms': stopwatch.elapsedMilliseconds,
      });

      return config;
    } catch (e, stackTrace) {
      _log.error('runtime_config_fetch_failed',
        error: e,
        stack: stackTrace,
      );
      rethrow;
    }
  }

  /// Update a profile slot (priority, active status)
  Future<void> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot updatedSlot,
  }) async {
    _log.info('updating_slot', {
      'device': '${device.vendorId}:${device.productId}',
      'slot_id': updatedSlot.id,
      'active': updatedSlot.active,
      'priority': updatedSlot.priority,
    });

    try {
      // Update via FFI
      await _bridge.updateDeviceSlot(device, updatedSlot);

      // Invalidate cache
      _cachedConfig = null;

      _log.info('slot_updated');
    } catch (e, stackTrace) {
      _log.error('slot_update_failed',
        error: e,
        stack: stackTrace,
      );
      rethrow;
    }
  }

  /// Add a new profile slot to a device
  Future<void> addSlot({
    required DeviceInstanceId device,
    required String hardwareProfileId,
    required String keymapId,
  }) async {
    _log.info('adding_slot', {
      'device': '${device.vendorId}:${device.productId}',
      'profile': hardwareProfileId,
      'keymap': keymapId,
    });

    try {
      // Generate new slot ID
      final slotId = 'slot-${DateTime.now().millisecondsSinceEpoch}';

      // Get current config to determine next priority
      final config = await getRuntimeConfig(forceRefresh: true);
      final deviceSlots = config.devices
          .firstWhere((d) => d.device == device, orElse: () => throw Exception('Device not found'))
          .slots;

      final nextPriority = deviceSlots.isEmpty
          ? 0
          : deviceSlots.map((s) => s.priority).reduce((a, b) => a > b ? a : b) + 1;

      final newSlot = ProfileSlot(
        id: slotId,
        hardwareProfileId: hardwareProfileId,
        keymapId: keymapId,
        active: false,  // New slots start inactive
        priority: nextPriority,
      );

      await _bridge.addDeviceSlot(device, newSlot);

      _cachedConfig = null;

      _log.info('slot_added', {'slot_id': slotId});
    } catch (e, stackTrace) {
      _log.error('slot_add_failed',
        error: e,
        stack: stackTrace,
      );
      rethrow;
    }
  }

  /// Remove a profile slot from a device
  Future<void> removeSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) async {
    _log.info('removing_slot', {
      'device': '${device.vendorId}:${device.productId}',
      'slot_id': slotId,
    });

    try {
      await _bridge.removeDeviceSlot(device, slotId);
      _cachedConfig = null;
      _log.info('slot_removed');
    } catch (e, stackTrace) {
      _log.error('slot_remove_failed',
        error: e,
        stack: stackTrace,
      );
      rethrow;
    }
  }

  /// Stream of runtime config updates
  Stream<RuntimeConfig> watchRuntimeConfig() {
    return _bridge.stateUpdates.asyncMap((_) async {
      // On any state update, refetch config
      return getRuntimeConfig(forceRefresh: true);
    });
  }

  bool _isCacheValid() {
    if (_cacheTime == null) return false;
    return DateTime.now().difference(_cacheTime!) < const Duration(seconds: 5);
  }
}
```

---

## State Management (Riverpod)

### Provider Definitions

**File**: `ui/lib/state/keyflow_providers.dart`

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'keyflow_providers.g.dart';

/// Repository provider
@riverpod
KeyFlowRepository keyFlowRepository(KeyFlowRepositoryRef ref) {
  final bridge = ref.watch(bridgeProvider);
  return KeyFlowRepository(bridge: bridge);
}

/// Runtime config provider (async)
@riverpod
Future<RuntimeConfig> runtimeConfig(RuntimeConfigRef ref) async {
  final repository = ref.watch(keyFlowRepositoryProvider);
  return repository.getRuntimeConfig();
}

/// Flow graph provider (derived from runtime config)
@riverpod
Future<FlowGraph> flowGraph(FlowGraphRef ref) async {
  final config = await ref.watch(runtimeConfigProvider.future);

  // Transform RuntimeConfig into FlowGraph
  final nodes = <FlowNode>[];
  final edges = <FlowEdge>[];

  // Create device nodes
  for (final deviceSlots in config.devices) {
    final deviceId = deviceSlots.device.serial;
    nodes.add(FlowNode.device(
      id: deviceId,
      device: DeviceState(/* ... */),  // Fetch from device registry
      slotIds: deviceSlots.slots.map((s) => s.id).toList(),
    ));

    // Create profile and keymap nodes + edges
    for (final slot in deviceSlots.slots) {
      // Hardware profile node
      if (!nodes.any((n) => n.id == slot.hardwareProfileId)) {
        nodes.add(FlowNode.hardwareProfile(
          id: slot.hardwareProfileId,
          profile: await _fetchProfile(ref, slot.hardwareProfileId),
        ));
      }

      // Keymap node
      if (!nodes.any((n) => n.id == slot.keymapId)) {
        nodes.add(FlowNode.keymap(
          id: slot.keymapId,
          keymap: await _fetchKeymap(ref, slot.keymapId),
        ));
      }

      // Device → Profile edge
      edges.add(FlowEdge(
        id: '${deviceId}_${slot.hardwareProfileId}_${slot.id}',
        fromNodeId: deviceId,
        toNodeId: slot.hardwareProfileId,
        slotId: slot.id,
        priority: slot.priority,
        active: slot.active,
        status: EdgeStatus.valid,
      ));

      // Profile → Keymap edge
      edges.add(FlowEdge(
        id: '${slot.hardwareProfileId}_${slot.keymapId}_${slot.id}',
        fromNodeId: slot.hardwareProfileId,
        toNodeId: slot.keymapId,
        slotId: slot.id,
        priority: slot.priority,
        active: slot.active,
        status: EdgeStatus.valid,
      ));
    }
  }

  // Load node positions from local storage
  final positions = await _loadNodePositions(nodes.map((n) => n.id).toList());

  return FlowGraph(
    nodes: nodes,
    edges: edges,
    nodePositions: positions,
    lastUpdated: DateTime.now(),
  );
}

/// Node positions provider (persistent)
@riverpod
class NodePositions extends _$NodePositions {
  @override
  Future<Map<String, Offset>> build() async {
    // Load from SharedPreferences
    final prefs = await SharedPreferences.getInstance();
    final json = prefs.getString('keyflow_node_positions');
    if (json == null) return {};

    final data = jsonDecode(json) as Map<String, dynamic>;
    return data.map((key, value) => MapEntry(
      key,
      Offset(value['x'] as double, value['y'] as double),
    ));
  }

  Future<void> updatePosition(String nodeId, Offset position) async {
    final positions = await future;
    positions[nodeId] = position;

    // Persist to SharedPreferences
    final prefs = await SharedPreferences.getInstance();
    final json = jsonEncode(
      positions.map((key, value) => MapEntry(
        key,
        {'x': value.dx, 'y': value.dy},
      )),
    );
    await prefs.setString('keyflow_node_positions', json);

    ref.invalidateSelf();
  }
}

/// Actions provider (mutations)
@riverpod
class KeyFlowActions extends _$KeyFlowActions {
  @override
  void build() {}

  Future<void> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot updatedSlot,
  }) async {
    final repository = ref.read(keyFlowRepositoryProvider);
    await repository.updateSlot(device: device, updatedSlot: updatedSlot);

    // Invalidate runtime config to trigger refetch
    ref.invalidate(runtimeConfigProvider);
  }

  Future<void> addSlot({
    required DeviceInstanceId device,
    required String hardwareProfileId,
    required String keymapId,
  }) async {
    final repository = ref.read(keyFlowRepositoryProvider);
    await repository.addSlot(
      device: device,
      hardwareProfileId: hardwareProfileId,
      keymapId: keymapId,
    );

    ref.invalidate(runtimeConfigProvider);
  }

  Future<void> removeSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) async {
    final repository = ref.read(keyFlowRepositoryProvider);
    await repository.removeSlot(device: device, slotId: slotId);

    ref.invalidate(runtimeConfigProvider);
  }

  Future<void> toggleSlotActive({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) async {
    final updatedSlot = slot.copyWith(active: !slot.active);
    await updateSlot(device: device, updatedSlot: updatedSlot);
  }

  Future<void> reorderSlot({
    required DeviceInstanceId device,
    required ProfileSlot slot,
    required int newPriority,
  }) async {
    final updatedSlot = slot.copyWith(priority: newPriority);
    await updateSlot(device: device, updatedSlot: updatedSlot);
  }
}
```

---

## UI Layer Design

### FlowEditorPage

**File**: `ui/lib/pages/flow_editor_page.dart` (refactored)

```dart
class FlowEditorPage extends ConsumerStatefulWidget {
  const FlowEditorPage({super.key});

  @override
  ConsumerState<FlowEditorPage> createState() => _FlowEditorPageState();
}

class _FlowEditorPageState extends ConsumerState<FlowEditorPage> {
  final TransformationController _transformController = TransformationController();

  @override
  void dispose() {
    _transformController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final flowGraphAsync = ref.watch(flowGraphProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('KeyFlow Configuration'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => ref.invalidate(runtimeConfigProvider),
            tooltip: 'Refresh',
          ),
          IconButton(
            icon: const Icon(Icons.zoom_out_map),
            onPressed: _resetZoom,
            tooltip: 'Reset Zoom',
          ),
        ],
      ),
      body: flowGraphAsync.when(
        data: (flowGraph) => _buildCanvas(flowGraph),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => ErrorView(
          error: error,
          onRetry: () => ref.invalidate(runtimeConfigProvider),
        ),
      ),
    );
  }

  Widget _buildCanvas(FlowGraph flowGraph) {
    return InteractiveViewer(
      transformationController: _transformController,
      minScale: 0.1,
      maxScale: 4.0,
      constrained: false,
      child: SizedBox(
        width: 2000,  // Large canvas
        height: 2000,
        child: Stack(
          children: [
            // Background grid
            CustomPaint(
              painter: GridPainter(
                gridSize: 20,
                color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
              ),
              child: Container(),
            ),

            // Edges layer
            CustomPaint(
              painter: EdgesPainter(
                edges: flowGraph.edges,
                nodePositions: flowGraph.nodePositions,
              ),
              child: Container(),
            ),

            // Nodes layer
            ...flowGraph.nodes.map((node) {
              final position = flowGraph.nodePositions[node.id] ?? Offset.zero;
              return Positioned(
                left: position.dx,
                top: position.dy,
                child: _buildNode(node),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildNode(FlowNode node) {
    return Draggable(
      data: node.id,
      feedback: Material(
        elevation: 8,
        child: _buildNodeWidget(node),
      ),
      childWhenDragging: Opacity(
        opacity: 0.5,
        child: _buildNodeWidget(node),
      ),
      onDragEnd: (details) => _handleNodeDragEnd(node.id, details),
      child: _buildNodeWidget(node),
    );
  }

  Widget _buildNodeWidget(FlowNode node) {
    return node.when(
      device: (id, device, slotIds) => DeviceNodeWidget(
        device: device,
        slotIds: slotIds,
        onAddSlot: () => _showAddSlotDialog(device.identity),
      ),
      hardwareProfile: (id, profile) => HardwareProfileNodeWidget(
        profile: profile,
      ),
      keymap: (id, keymap) => KeymapNodeWidget(
        keymap: keymap,
      ),
    );
  }

  void _handleNodeDragEnd(String nodeId, DraggableDetails details) {
    final positions = ref.read(nodePositionsProvider.notifier);
    positions.updatePosition(nodeId, details.offset);
  }

  void _showAddSlotDialog(DeviceIdentity device) {
    // Show dialog to select hardware profile + keymap
    // On confirm, call keyFlowActionsProvider.addSlot()
  }

  void _resetZoom() {
    _transformController.value = Matrix4.identity();
  }
}
```

---

## Edge Rendering

### EdgesPainter

**File**: `ui/lib/widgets/flow/edges_painter.dart` (enhanced)

```dart
class EdgesPainter extends CustomPainter {
  final List<FlowEdge> edges;
  final Map<String, Offset> nodePositions;
  final String? hoveredEdgeId;

  EdgesPainter({
    required this.edges,
    required this.nodePositions,
    this.hoveredEdgeId,
  });

  @override
  void paint(Canvas canvas, Size size) {
    for (final edge in edges) {
      final fromPos = nodePositions[edge.fromNodeId];
      final toPos = nodePositions[edge.toNodeId];

      if (fromPos == null || toPos == null) continue;

      final paint = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = edge.id == hoveredEdgeId ? 3.0 : 2.0
        ..color = _getEdgeColor(edge);

      // Dashed line for inactive slots
      if (!edge.active) {
        paint.style = PaintingStyle.stroke;
        _drawDashedLine(canvas, fromPos, toPos, paint);
      } else {
        // Bezier curve for active slots
        final path = Path()
          ..moveTo(fromPos.dx + 100, fromPos.dy + 50)  // Right edge of node
          ..cubicTo(
            fromPos.dx + 200, fromPos.dy + 50,  // Control point 1
            toPos.dx - 200, toPos.dy + 50,      // Control point 2
            toPos.dx, toPos.dy + 50,             // End point (left edge of node)
          );
        canvas.drawPath(path, paint);
      }

      // Draw priority badge
      _drawPriorityBadge(canvas, fromPos, toPos, edge.priority);
    }
  }

  Color _getEdgeColor(FlowEdge edge) {
    if (edge.status == EdgeStatus.invalid) return AppColors.error;
    if (edge.status == EdgeStatus.warning) return AppColors.warning;
    return edge.active ? AppColors.primary : AppColors.onSurfaceLight;
  }

  void _drawDashedLine(Canvas canvas, Offset from, Offset to, Paint paint) {
    const dashWidth = 5.0;
    const dashSpace = 3.0;

    final path = Path()..moveTo(from.dx + 100, from.dy + 50);
    final totalDistance = (to - from).distance;
    var distance = 0.0;

    while (distance < totalDistance) {
      final dashEnd = distance + dashWidth;
      final progress = distance / totalDistance;
      final nextProgress = dashEnd / totalDistance;

      final start = Offset.lerp(from, to, progress)!;
      final end = Offset.lerp(from, to, nextProgress)!;

      path.moveTo(start.dx + 100, start.dy + 50);
      path.lineTo(end.dx + 100, end.dy + 50);

      distance += dashWidth + dashSpace;
    }

    canvas.drawPath(path, paint);
  }

  void _drawPriorityBadge(Canvas canvas, Offset from, Offset to, int priority) {
    final midpoint = Offset.lerp(from, to, 0.5)!;

    // Badge background
    final badgePaint = Paint()..color = AppColors.accent;
    canvas.drawCircle(midpoint.translate(100, 50), 12, badgePaint);

    // Priority number
    final textPainter = TextPainter(
      text: TextSpan(
        text: priority.toString(),
        style: const TextStyle(color: Colors.white, fontSize: 10, fontWeight: FontWeight.bold),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(
      canvas,
      midpoint.translate(100 - textPainter.width / 2, 50 - textPainter.height / 2),
    );
  }

  @override
  bool shouldRepaint(EdgesPainter oldDelegate) =>
      edges != oldDelegate.edges ||
      nodePositions != oldDelegate.nodePositions ||
      hoveredEdgeId != oldDelegate.hoveredEdgeId;
}
```

---

## Node Widgets

### DeviceNodeWidget (Enhanced)

**File**: `ui/lib/widgets/flow/device_node_widget.dart`

```dart
class DeviceNodeWidget extends ConsumerWidget {
  final DeviceState device;
  final List<String> slotIds;
  final VoidCallback? onAddSlot;

  const DeviceNodeWidget({
    super.key,
    required this.device,
    required this.slotIds,
    this.onAddSlot,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Card(
      elevation: 4,
      child: Container(
        width: 250,
        padding: EdgeInsets.all(Spacing.space4),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Row(
              children: [
                Icon(
                  Icons.keyboard,
                  color: _getStatusColor(),
                ),
                SizedBox(width: Spacing.space2),
                Expanded(
                  child: Text(
                    device.identity.userLabel,
                    style: theme.textTheme.titleMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            SizedBox(height: Spacing.space2),

            // Device info
            Text(
              'VID:${device.identity.vendorId.toRadixString(16).padLeft(4, '0')} '
              'PID:${device.identity.productId.toRadixString(16).padLeft(4, '0')}',
              style: theme.textTheme.bodySmall,
            ),
            SizedBox(height: Spacing.space1),
            Text(
              device.identity.serialNumber,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.outline,
              ),
            ),

            // Slots count
            SizedBox(height: Spacing.space3),
            Row(
              children: [
                Icon(Icons.layers, size: 16, color: theme.colorScheme.primary),
                SizedBox(width: Spacing.space1),
                Text(
                  '${slotIds.length} slot${slotIds.length != 1 ? "s" : ""}',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),

            // Add slot button
            if (onAddSlot != null) ...[
              SizedBox(height: Spacing.space3),
              SizedBox(
                width: double.infinity,
                child: OutlinedButton.icon(
                  onPressed: onAddSlot,
                  icon: const Icon(Icons.add, size: 16),
                  label: const Text('Add Slot'),
                  style: OutlinedButton.styleFrom(
                    padding: EdgeInsets.symmetric(vertical: Spacing.space2),
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Color _getStatusColor() {
    if (!device.remapEnabled) return AppColors.deviceDisabled;
    // Check if any slot is active by checking if profileId is set
    if (device.profileId.isEmpty) return AppColors.deviceConnected;
    return AppColors.deviceActive;
  }
}
```

---

## Testing Strategy

### Unit Tests

**Repository Tests** (`test/repositories/keyflow_repository_test.dart`):
```dart
void main() {
  late KeyFlowRepository repository;
  late MockKeyrxBridge mockBridge;

  setUp(() {
    mockBridge = MockKeyrxBridge();
    repository = KeyFlowRepository(bridge: mockBridge);
  });

  group('getRuntimeConfig', () {
    test('fetches from bridge and caches', () async {
      final mockConfig = RuntimeConfig(devices: []);
      when(() => mockBridge.getRuntimeConfig()).thenAnswer((_) async => mockConfig);

      final result1 = await repository.getRuntimeConfig();
      final result2 = await repository.getRuntimeConfig();

      expect(result1, mockConfig);
      expect(result2, mockConfig);
      verify(() => mockBridge.getRuntimeConfig()).called(1);  // Cached!
    });

    test('force refresh bypasses cache', () async {
      final mockConfig = RuntimeConfig(devices: []);
      when(() => mockBridge.getRuntimeConfig()).thenAnswer((_) async => mockConfig);

      await repository.getRuntimeConfig();
      await repository.getRuntimeConfig(forceRefresh: true);

      verify(() => mockBridge.getRuntimeConfig()).called(2);
    });
  });
}
```

### Widget Tests

**FlowEditorPage Tests** (`test/pages/flow_editor_page_test.dart`):
```dart
void main() {
  late WidgetTestHarness harness;

  setUp(() {
    harness = WidgetTestHarness();
  });

  testWidgets('displays device nodes from runtime config', (tester) async {
    harness.mockBridge.setRuntimeConfig(RuntimeConfig(
      devices: [
        DeviceSlots(device: mockDeviceId, slots: [mockSlot]),
      ],
    ));

    await harness.pumpPage(tester, const FlowEditorPage());
    await tester.pumpAndSettle();

    expect(find.byType(DeviceNodeWidget), findsOneWidget);
  });

  testWidgets('updates positions on drag', (tester) async {
    await harness.pumpPage(tester, const FlowEditorPage());
    await tester.pumpAndSettle();

    await tester.drag(find.byType(DeviceNodeWidget), const Offset(100, 100));
    await tester.pumpAndSettle();

    // Verify position saved to SharedPreferences
    final prefs = await SharedPreferences.getInstance();
    expect(prefs.getString('keyflow_node_positions'), isNotNull);
  });
}
```

### Golden Tests

**Visual Regression** (`test/widgets/flow/device_node_widget_golden_test.dart`):
```dart
@Tags(['golden'])
void main() {
  testGoldens('DeviceNodeWidget renders correctly', (tester) async {
    final builder = GoldenBuilder.grid(columns: 2, widthToHeightRatio: 1)
      ..addScenario('Active Device', DeviceNodeWidget(
        device: mockActiveDevice,
        slotIds: ['slot-1', 'slot-2'],
      ))
      ..addScenario('Inactive Device', DeviceNodeWidget(
        device: mockInactiveDevice,
        slotIds: [],
      ));

    await tester.pumpWidgetBuilder(builder.build());
    await screenMatchesGolden(tester, 'device_node_widget');
  });
}
```

---

## Performance Optimization

### 1. CustomPainter for Edges
- **Why**: Avoids widget rebuilds for edges
- **Benefit**: 60fps even with 100+ edges

### 2. Repository Caching
- **Why**: Avoid redundant FFI calls
- **Benefit**: Sub-100ms updates

### 3. Const Constructors
- **Why**: Widgets don't rebuild unnecessarily
- **Benefit**: Reduced frame time

### 4. Riverpod Granular Invalidation
- **Why**: Only affected providers rebuild
- **Benefit**: Minimal state updates

---

## Error Handling

### FFI Errors
```dart
try {
  await repository.addSlot(...);
} on FfiException catch (e) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(
      content: Text('Failed to add slot: ${e.userMessage}'),
      backgroundColor: AppColors.error,
      action: SnackBarAction(
        label: 'Retry',
        onPressed: () => _retryAddSlot(),
      ),
    ),
  );
}
```

### Validation Errors
```dart
if (edge.status == EdgeStatus.invalid) {
  return Tooltip(
    message: 'Hardware profile "${edge.toNodeId}" not found',
    child: Icon(Icons.error, color: AppColors.error),
  );
}
```

---

## Migration from Prototype

### Step 1: Add Riverpod
- Install dependencies
- Wrap app in `ProviderScope`
- Create `keyFlowRepository` provider

### Step 2: Create Repository
- Implement `KeyFlowRepository`
- Write unit tests
- Mock FFI bridge for testing

### Step 3: Refactor FlowEditorPage
- Replace hardcoded data with `ref.watch(flowGraphProvider)`
- Add loading/error states
- Test with mock data

### Step 4: Add Interactivity
- Implement drag handlers
- Wire up `keyFlowActionsProvider`
- Add persistence

### Step 5: Polish
- Add animations
- Improve error messages
- Golden tests

---

## Deployment Checklist

- [ ] All unit tests passing (80%+ coverage)
- [ ] All widget tests passing
- [ ] All golden tests passing
- [ ] Performance benchmarks met (60fps)
- [ ] Accessibility audit passed (WCAG 2.1 AA)
- [ ] Error handling comprehensive
- [ ] Logging instrumented
- [ ] Documentation updated
- [ ] Feature flag configured
- [ ] Metrics dashboard ready

---

## References

- Riverpod Architecture: https://riverpod.dev/docs/concepts/providers
- CustomPainter Performance: https://docs.flutter.dev/perf/rendering-performance
- Accessibility Testing: https://docs.flutter.dev/accessibility-and-localization/accessibility
