import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

import '../../helpers/golden_test_harness.dart';
import '../../helpers/widget_test_harness.dart';

// Mock flow editor page components for golden tests
class _MockFlowEditorPage extends StatelessWidget {
  const _MockFlowEditorPage({this.state = _PageState.populated});

  final _PageState state;

  @override
  Widget build(BuildContext context) => Scaffold(
    backgroundColor: const Color(0xFF121212),
    body: switch (state) {
      _PageState.loading => const _FlowLoadingSkeleton(),
      _PageState.error => _FlowErrorView(
        error: Exception('Failed to load runtime configuration'),
        onRetry: () {},
      ),
      _PageState.empty => const _FlowEmptyState(),
      _PageState.populated => _PopulatedFlowView(),
    },
  );
}

enum _PageState { loading, error, empty, populated }

// Loading skeleton that mimics the actual implementation
class _FlowLoadingSkeleton extends StatelessWidget {
  const _FlowLoadingSkeleton();

  @override
  Widget build(BuildContext context) => Container(
    color: const Color(0xFF121212),
    padding: const EdgeInsets.all(32),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _shimmerBar(width: 220),
        const SizedBox(height: 20),
        Expanded(
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              _shimmerCard(width: 240, height: 180),
              _shimmerCard(width: 200, height: 140),
              _shimmerCard(width: 180, height: 120),
            ],
          ),
        ),
      ],
    ),
  );

  Widget _shimmerBar({required double width}) =>
      _shimmerBox(width: width, height: 16, radius: 6);

  Widget _shimmerCard({required double width, required double height}) =>
      _shimmerBox(width: width, height: height, radius: 10);

  Widget _shimmerBox({
    required double width,
    required double height,
    required double radius,
  }) => Container(
    width: width,
    height: height,
    decoration: BoxDecoration(
      color: const Color(0xFF1E1E1E),
      borderRadius: BorderRadius.circular(radius),
    ),
  );
}

// Error view
class _FlowErrorView extends StatelessWidget {
  const _FlowErrorView({required this.error, required this.onRetry});

  final Object? error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.error_outline, color: Colors.redAccent, size: 48),
        const SizedBox(height: 12),
        const Text(
          'Something went wrong',
          style: TextStyle(color: Colors.white, fontSize: 18),
        ),
        const SizedBox(height: 8),
        Text(
          '$error',
          style: const TextStyle(color: Colors.grey, fontSize: 12),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        ElevatedButton(onPressed: onRetry, child: const Text('Retry')),
      ],
    ),
  );
}

// Empty state
class _FlowEmptyState extends StatelessWidget {
  const _FlowEmptyState();

  @override
  Widget build(BuildContext context) => const Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.usb_off_rounded, color: Colors.grey, size: 52),
        SizedBox(height: 12),
        Text(
          'No devices connected',
          style: TextStyle(color: Colors.white, fontSize: 18),
        ),
        SizedBox(height: 6),
        Text(
          'Connect a device to populate your flow.',
          style: TextStyle(color: Colors.grey, fontSize: 13),
        ),
      ],
    ),
  );
}

// Populated view with sample data
class _PopulatedFlowView extends StatelessWidget {
  @override
  Widget build(BuildContext context) => Stack(
    children: [
      // Grid background
      Positioned.fill(child: CustomPaint(painter: _GridPainter())),
      // Sample flow visualization
      const Center(
        child: Padding(
          padding: EdgeInsets.all(40),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              // Device node placeholder
              _MockNode(
                color: Color(0xFF2ECC71),
                width: 240,
                height: 180,
                label: 'Device',
              ),
              // Profile node placeholder
              _MockNode(
                color: Color(0xFF3498DB),
                width: 200,
                height: 140,
                label: 'Profile',
              ),
              // Keymap node placeholder
              _MockNode(
                color: Color(0xFF2ECC71),
                width: 180,
                height: 120,
                label: 'Keymap',
              ),
            ],
          ),
        ),
      ),
      // Save indicator
      const Positioned(bottom: 16, right: 16, child: _SaveIndicator()),
    ],
  );
}

class _GridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFF333333).withValues(alpha: 0.4)
      ..strokeWidth = 1;

    const gridSize = 20.0;

    for (double x = 0; x < size.width; x += gridSize) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y < size.height; y += gridSize) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

class _MockNode extends StatelessWidget {
  const _MockNode({
    required this.color,
    required this.width,
    required this.height,
    required this.label,
  });

  final Color color;
  final double width;
  final double height;
  final String label;

  @override
  Widget build(BuildContext context) => Material(
    elevation: 2,
    borderRadius: BorderRadius.circular(12),
    child: Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color, width: 2),
      ),
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.2),
              borderRadius: BorderRadius.circular(8),
            ),
          ),
          const SizedBox(height: 8),
          Text(
            label,
            style: const TextStyle(
              color: Colors.white,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    ),
  );
}

class _SaveIndicator extends StatelessWidget {
  const _SaveIndicator();

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
    decoration: BoxDecoration(
      color: Colors.black.withValues(alpha: 0.6),
      borderRadius: BorderRadius.circular(20),
      border: Border.all(color: const Color(0xFF2ECC71).withValues(alpha: 0.5)),
    ),
    child: const Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.check_circle, color: Color(0xFF2ECC71), size: 16),
        SizedBox(width: 6),
        Text(
          'Saved',
          style: TextStyle(
            color: Color(0xFF2ECC71),
            fontSize: 12,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    ),
  );
}

void main() {
  final darkHarness = GoldenTestHarness(
    devices: [Device.phone, Device.tabletLandscape],
  );
  final lightHarness = GoldenTestHarness(
    widgetHarness: WidgetTestHarness(
      theme: ThemeData.light(useMaterial3: true),
      wrapWithScaffold: false,
    ),
    devices: [Device.phone, Device.tabletLandscape],
  );

  group('FlowEditorPage Goldens', () {
    testGoldens('renders loading state', (tester) async {
      await darkHarness.pump(
        tester,
        const _MockFlowEditorPage(state: _PageState.loading),
      );

      await darkHarness.expectGolden(tester, 'flow_page_loading');
    });

    testGoldens('renders error state', (tester) async {
      await darkHarness.pump(
        tester,
        const _MockFlowEditorPage(state: _PageState.error),
      );

      await darkHarness.expectGolden(tester, 'flow_page_error');
    });

    testGoldens('renders empty state - no devices', (tester) async {
      await darkHarness.pump(
        tester,
        const _MockFlowEditorPage(state: _PageState.empty),
      );

      await darkHarness.expectGolden(tester, 'flow_page_empty');
    });

    testGoldens('renders populated flow', (tester) async {
      await darkHarness.pump(tester, const _MockFlowEditorPage());

      await darkHarness.expectGolden(tester, 'flow_page_populated');
    });

    testGoldens('renders populated flow - light theme', (tester) async {
      await lightHarness.pump(tester, const _MockFlowEditorPage());

      await lightHarness.expectGolden(tester, 'flow_page_populated_light');
    });
  });

  group('FlowEditorPage Multi-Device Goldens', () {
    testGoldens('renders on multiple devices - loading', (tester) async {
      await darkHarness.expectMultiDevice(
        tester,
        name: 'flow_page_loading_multi_device',
        widget: const _MockFlowEditorPage(state: _PageState.loading),
      );
    });

    testGoldens('renders on multiple devices - empty', (tester) async {
      await darkHarness.expectMultiDevice(
        tester,
        name: 'flow_page_empty_multi_device',
        widget: const _MockFlowEditorPage(state: _PageState.empty),
      );
    });

    testGoldens('renders on multiple devices - populated', (tester) async {
      await darkHarness.expectMultiDevice(
        tester,
        name: 'flow_page_populated_multi_device',
        widget: const _MockFlowEditorPage(),
      );
    });
  });
}
