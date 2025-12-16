import 'dart:async';

import 'package:golden_toolkit/golden_toolkit.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  final config = GoldenToolkitConfiguration(
    enableRealShadows: true,
    defaultDevices: const [
      Device.phone,
      Device.iphone11,
      Device.tabletLandscape,
    ],
    deviceFileNameFactory: (name, device) => '${name}_${device.name}'
        .toLowerCase()
        .replaceAll(RegExp('[^a-z0-9]+'), '_')
        .replaceAll(RegExp('_+'), '_')
        .replaceAll(RegExp(r'^_|_$'), ''),
  );

  return GoldenToolkit.runWithConfiguration(() async {
    await loadAppFonts();
    await testMain();
  }, config: config);
}
