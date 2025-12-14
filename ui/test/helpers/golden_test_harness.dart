import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

import 'widget_test_harness.dart';

const defaultGoldenDevices = [
  Device.phone,
  Device.iphone11,
  Device.tabletLandscape,
];

class GoldenTestHarness {
  GoldenTestHarness({
    WidgetTestHarness? widgetHarness,
    ThemeData? theme,
    this.devices = defaultGoldenDevices,
    bool wrapWithScaffold = true,
  }) : widgetHarness = widgetHarness ??
            WidgetTestHarness(
              theme: theme ?? ThemeData.dark(useMaterial3: true),
              wrapWithScaffold: wrapWithScaffold,
            );

  final WidgetTestHarness widgetHarness;
  final List<Device> devices;

  Future<void> pump(
    WidgetTester tester,
    Widget widget, {
    bool? withScaffold,
  }) async {
    await tester.pumpWidgetBuilder(
      widgetHarness.build(widget, withScaffold: withScaffold),
      surfaceSize: devices.first.size,
    );
    await tester.pumpAndSettle();
  }

  Future<void> expectGolden(
    WidgetTester tester,
    String name,
  ) async {
    await screenMatchesGolden(
      tester,
      name,
    );
  }

  Future<void> expectMultiDevice(
    WidgetTester tester, {
    required String name,
    required Widget widget,
    List<Device>? devicesOverride,
    bool? withScaffold,
  }) async {
    final activeDevices = devicesOverride ?? devices;
    await tester.pumpWidgetBuilder(
      widgetHarness.build(widget, withScaffold: withScaffold),
      surfaceSize: activeDevices.first.size,
    );
    await tester.pumpAndSettle();

    await multiScreenGolden(
      tester,
      name,
      devices: activeDevices,
    );
  }
}
