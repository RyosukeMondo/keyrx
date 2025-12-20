import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/metrics_service.dart';
import 'package:keyrx_ui/widgets/settings/settings.dart';
import 'package:mockito/mockito.dart';

class MockMetricsService extends Mock implements MetricsService {}

void main() {
  group('ThresholdSettingsDialog', () {
    testWidgets('should render', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (BuildContext context) => ElevatedButton(
                onPressed: () {
                  showDialog<void>(
                    context: context,
                    builder: (BuildContext context) => ThresholdSettingsDialog(
                      initialThresholds: MetricsThresholds.defaults(),
                      onSave: (thresholds) {},
                    ),
                  );
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();

      expect(find.byType(ThresholdSettingsDialog), findsOneWidget);
    });
  });
}
