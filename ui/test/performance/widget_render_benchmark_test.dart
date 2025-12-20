import 'package:core_logic/models/models.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/pages/migration_prompt_page.dart';
import 'package:keyrx_ui/pages/trade_off_chart.dart';
import 'package:keyrx_ui/pages/typing_simulator.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const renderThresholdMs = 350;
  const pageThresholdMs = 300;

  testWidgets('TradeOffChart renders under ${renderThresholdMs}ms', (
    tester,
  ) async {
    final stats = TypingStatistics(mean: 180, stdDev: 35);
    final widget = TradeOffChart(
      statistics: stats,
      currentTimeout: stats.calculateRecommendedTimeout(),
    );

    await tester.binding.setSurfaceSize(const Size(1280, 800));
    addTearDown(() async {
      await tester.binding.setSurfaceSize(null);
    });

    final stopwatch = Stopwatch()..start();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: Center(child: widget)),
        theme: ThemeData.dark(useMaterial3: true),
      ),
    );
    await tester.pumpAndSettle();

    stopwatch.stop();
    expect(stopwatch.elapsedMilliseconds, lessThan(renderThresholdMs));
  });

  testWidgets('MigrationPromptPage builds under ${pageThresholdMs}ms', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1280, 900));
    addTearDown(() async {
      await tester.binding.setSurfaceSize(null);
    });

    final stopwatch = Stopwatch()..start();

    await tester.pumpWidget(
      MaterialApp(
        home: MigrationPromptPage(
          onMigrate: () async {
            await Future<void>.delayed(const Duration(milliseconds: 15));
            return MigrationReport(
              totalCount: 1,
              migratedCount: 1,
              failedCount: 0,
            );
          },
          onSkip: () {},
        ),
        theme: ThemeData.dark(useMaterial3: true),
      ),
    );
    await tester.pumpAndSettle();

    stopwatch.stop();
    expect(stopwatch.elapsedMilliseconds, lessThan(pageThresholdMs));
  });
}
