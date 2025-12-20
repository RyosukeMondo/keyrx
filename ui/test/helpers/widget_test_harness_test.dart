import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/state/providers_setup.dart';

import 'widget_test_harness.dart';

void main() {
  group('WidgetTestHarness', () {
    testWidgets('builds default shell with ProviderScope + Scaffold', (
      WidgetTester tester,
    ) async {
      final harness = WidgetTestHarness();

      await harness.pumpPage(tester, const Text('Hello Harness'));

      tester
        ..expectOne(find.byType(Scaffold))
        ..expectOne(find.text('Hello Harness'));
    });

    testWidgets('supports Riverpod reads and tester extensions', (
      WidgetTester tester,
    ) async {
      final harness = WidgetTestHarness();
      final counter = ValueNotifier<int>(0);

      await harness.pumpWidget(
        tester,
        Consumer(
          builder: (context, ref, _) {
            final registry = ref.read(serviceRegistryProvider);
            final bridge = registry.bridge;
            return Column(
              children: [
                Text('Bridge: ${bridge.version}'),
                ElevatedButton(
                  onPressed: () => counter.value++,
                  child: const Text('Increment'),
                ),
                ValueListenableBuilder<int>(
                  valueListenable: counter,
                  builder: (_, value, __) => Text('Tapped: $value'),
                ),
              ],
            );
          },
        ),
      );

      tester.expectOne(find.textContaining('Bridge:'));
      await tester.tapAndSettle(find.text('Increment'));
      tester.expectOne(find.text('Tapped: 1'));
    });
  });
}
