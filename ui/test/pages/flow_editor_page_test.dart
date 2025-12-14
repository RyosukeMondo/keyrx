import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/flow/node_position.dart';
import 'package:keyrx_ui/pages/flow_editor_page.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/node_positions_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _FakeNodePositions extends NodePositions {
  @override
  Future<Map<String, NodePosition>> build() async => const {};
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  testWidgets('shows loading skeleton while providers load', (tester) async {
    final completer = Completer<FlowGraph>();

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          flowGraphProvider.overrideWith((ref) => completer.future),
          nodePositionsProvider.overrideWith(_FakeNodePositions.new),
        ],
        child: const MaterialApp(home: FlowEditorPage()),
      ),
    );

    expect(find.byKey(const Key('flow-loading-skeleton')), findsOneWidget);
  });

  testWidgets('renders error state and retries loading graph', (tester) async {
    var callCount = 0;
    const deviceId = DeviceInstanceId(vendorId: 1, productId: 2, serial: 'abc');
    const graph = FlowGraph(
      nodes: [
        FlowNode.device(
          id: 'device-1',
          deviceId: deviceId,
          label: 'Demo Device',
        ),
        FlowNode.hardwareProfile(id: 'hp-1'),
        FlowNode.keymap(id: 'km-1'),
      ],
      edges: [
        FlowEdge(
          from: 'device-1',
          to: 'hp-1',
          slotId: 'slot-1',
          priority: 1,
          active: true,
        ),
        FlowEdge(
          from: 'hp-1',
          to: 'km-1',
          slotId: 'slot-1',
          priority: 1,
          active: true,
        ),
      ],
    );

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          flowGraphProvider.overrideWith((ref) async {
            callCount++;
            if (callCount == 1) throw Exception('load failed');
            return graph;
          }),
          nodePositionsProvider.overrideWith(_FakeNodePositions.new),
        ],
        child: const MaterialApp(home: FlowEditorPage()),
      ),
    );

    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(find.text('Something went wrong'), findsOneWidget);
    await tester.tap(find.text('Retry'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 200));

    expect(callCount, 2);
    expect(find.text('Demo Device'), findsOneWidget);
  });

  testWidgets('shows empty state when no nodes are available', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          flowGraphProvider.overrideWith((ref) async => const FlowGraph()),
          nodePositionsProvider.overrideWith(_FakeNodePositions.new),
        ],
        child: const MaterialApp(home: FlowEditorPage()),
      ),
    );

    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(find.text('No devices connected'), findsOneWidget);
    expect(
      find.text('Connect a device to populate your flow.'),
      findsOneWidget,
    );
  });
}
