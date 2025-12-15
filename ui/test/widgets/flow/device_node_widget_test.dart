import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/flow_data.dart';
import 'package:keyrx_ui/widgets/flow/device_node_widget.dart';

void main() {
  group('DeviceNodeWidget', () {
    group('connected device', () {
      testWidgets('displays device info correctly', (tester) async {
        const data = DeviceNodeData(
          name: 'Keychron Q1',
          vid: '05AC',
          pid: '024F',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Verify device name is displayed
        expect(find.text('Keychron Q1'), findsOneWidget);

        // Verify VID:PID is displayed
        expect(find.text('05AC:024F'), findsOneWidget);

        // Verify connected status
        expect(find.text('● Connected'), findsOneWidget);

        // Verify "VIRTUAL DEVICE" label
        expect(find.text('VIRTUAL DEVICE'), findsOneWidget);
      });

      testWidgets('displays connected state with proper styling', (
        tester,
      ) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(DeviceNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify connected background color
        expect(decoration.color, const Color(0xFF1E1E1E));

        // Verify border is present for connected device
        expect(decoration.border, isNotNull);

        // Verify status badge shows green color for connected
        final statusText = tester.widget<Text>(find.text('● Connected'));
        expect(statusText.style?.color, const Color(0xFF2ECC71));
      });

      testWidgets('displays active wirings with priority order', (
        tester,
      ) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: true,
          wirings: [
            DeviceWiring(id: '1', name: 'ANSI-104 → Default', isActive: true),
            DeviceWiring(id: '2', name: 'ISO-105 → Gaming', isActive: false),
            DeviceWiring(id: '3', name: 'Ergo-60 → Vim', isActive: false),
          ],
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Verify header label
        expect(find.text('ACTIVE PROFILES (PRIORITY ORDER)'), findsOneWidget);

        // Verify all wirings are displayed
        expect(find.text('ANSI-104 → Default'), findsOneWidget);
        expect(find.text('ISO-105 → Gaming'), findsOneWidget);
        expect(find.text('Ergo-60 → Vim'), findsOneWidget);

        // Verify priority numbers
        expect(find.text('#1'), findsOneWidget);
        expect(find.text('#2'), findsOneWidget);
        expect(find.text('#3'), findsOneWidget);
      });

      testWidgets('distinguishes active from inactive wirings', (tester) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: true,
          wirings: [
            DeviceWiring(id: '1', name: 'Active Profile', isActive: true),
            DeviceWiring(id: '2', name: 'Inactive Profile', isActive: false),
          ],
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Find active wiring text
        final activeText = tester.widget<Text>(find.text('Active Profile'));
        expect(activeText.style?.color, const Color(0xFF1AB394));

        // Find inactive wiring text
        final inactiveText = tester.widget<Text>(find.text('Inactive Profile'));
        expect(inactiveText.style?.color, Colors.grey);

        // Verify connection handle color
        // Active should be teal, inactive should be grey
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(DeviceNodeWidget),
            matching: find.byType(Container),
          ),
        );

        // Find containers with circle shape (connection handles)
        final circleContainers = containers.where((c) {
          final decoration = c.decoration;
          return decoration is BoxDecoration &&
              decoration.shape == BoxShape.circle;
        }).toList();

        expect(circleContainers.length, 2);

        // First should be active (teal)
        final activeDecor = circleContainers[0].decoration as BoxDecoration;
        expect(activeDecor.color, const Color(0xFF1AB394));

        // Second should be inactive (grey)
        final inactiveDecor = circleContainers[1].decoration as BoxDecoration;
        expect(inactiveDecor.color, Colors.grey);
      });

      testWidgets('does not show wiring section when no wirings', (
        tester,
      ) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Verify header is not shown when no wirings
        expect(find.text('ACTIVE PROFILES (PRIORITY ORDER)'), findsNothing);
      });
    });

    group('disconnected device', () {
      testWidgets('displays offline status correctly', (tester) async {
        const data = DeviceNodeData(
          name: 'Offline Device',
          vid: 'ABCD',
          pid: 'EF01',
          isConnected: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Verify device name
        expect(find.text('Offline Device'), findsOneWidget);

        // Verify VID:PID
        expect(find.text('ABCD:EF01'), findsOneWidget);

        // Verify offline status
        expect(find.text('○ Offline'), findsOneWidget);
      });

      testWidgets('displays disconnected state with proper styling', (
        tester,
      ) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(DeviceNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify disconnected background color
        expect(decoration.color, const Color(0xFF1A1A1A));

        // Verify status badge shows grey color for offline
        final statusText = tester.widget<Text>(find.text('○ Offline'));
        expect(statusText.style?.color, const Color(0xFFBDC3C7));
      });

      testWidgets('displays greyed keyboard icon when offline', (tester) async {
        const data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Find the keyboard emoji text
        final emojiText = tester.widget<Text>(find.text('⌨️'));
        expect(emojiText.style?.color, Colors.grey);
      });
    });

    group('edge cases', () {
      testWidgets('handles long device names gracefully', (tester) async {
        const data = DeviceNodeData(
          name: 'Very Long Device Name That Might Overflow',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Widget should render without overflow errors
        expect(tester.takeException(), isNull);

        // Device name should still be visible
        expect(
          find.text('Very Long Device Name That Might Overflow'),
          findsOneWidget,
        );
      });

      testWidgets('handles many wirings', (tester) async {
        final wirings = List.generate(
          10,
          (i) => DeviceWiring(id: '$i', name: 'Wiring $i', isActive: i == 0),
        );

        final data = DeviceNodeData(
          name: 'Test Device',
          vid: '1234',
          pid: '5678',
          isConnected: true,
          wirings: wirings,
        );

        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(child: DeviceNodeWidget(data: data)),
            ),
          ),
        );

        // Widget should render without errors
        expect(tester.takeException(), isNull);

        // All wirings should be present
        for (var i = 0; i < 10; i++) {
          expect(find.text('Wiring $i'), findsOneWidget);
          expect(find.text('#${i + 1}'), findsOneWidget);
        }
      });

      testWidgets('has fixed width of 240 pixels', (tester) async {
        const data = DeviceNodeData(
          name: 'Test',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        final widget = tester.widget<DeviceNodeWidget>(
          find.byType(DeviceNodeWidget),
        );

        // Verify the widget renders without errors
        expect(widget, isNotNull);

        // Verify the widget has a reasonable size by checking the rendered box
        final size = tester.getSize(find.byType(DeviceNodeWidget));
        expect(size.width, 240);
      });
    });

    group('visual styling', () {
      testWidgets('has proper shadows and borders', (tester) async {
        const data = DeviceNodeData(
          name: 'Test',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(DeviceNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify border radius
        expect(decoration.borderRadius, BorderRadius.circular(8));

        // Verify shadow is present
        expect(decoration.boxShadow, isNotNull);
        expect(decoration.boxShadow!.length, 1);
        expect(decoration.boxShadow![0].blurRadius, 6);
      });

      testWidgets('displays keyboard icon in container with proper styling', (
        tester,
      ) async {
        const data = DeviceNodeData(
          name: 'Test',
          vid: '1234',
          pid: '5678',
          isConnected: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: DeviceNodeWidget(data: data)),
          ),
        );

        // Find the keyboard emoji text widget
        final emojiText = tester.widget<Text>(find.text('⌨️'));
        expect(emojiText, isNotNull);

        // Find the containers to verify icon container styling exists
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(DeviceNodeWidget),
            matching: find.byType(Container),
          ),
        );

        expect(containers.length, greaterThan(0));
      });
    });
  });
}
