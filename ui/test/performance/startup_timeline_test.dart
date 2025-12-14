import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/pages/developer/profiler_page.dart';
import 'package:keyrx_ui/utils/performance_timeline.dart';

import '../helpers/widget_test_harness.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final harness = WidgetTestHarness();

  testWidgets('captures timeline span for profiler page load', (tester) async {
    final span = PerformanceTimelineTracer.start(
      'test.profiler_page_load',
      context: {'mode': 'test'},
    );

    await harness.pumpPage(tester, const ProfilerPage());
    await tester.pump(const Duration(milliseconds: 16));

    final duration = span.finish();
    expect(duration.inMilliseconds, lessThan(500));
  });
}
