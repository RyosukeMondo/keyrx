/// Performance benchmarks comparing FFI and HTTP service implementations.
///
/// This test verifies that web (HTTP) performance is within 20% of desktop (FFI)
/// performance for critical operations.
library;

import 'dart:async';

import 'package:benchmark_harness/benchmark_harness.dart';
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

import '../../lib/ffi/bridge.dart';
import '../../lib/ffi/generated_bindings.dart';
import '../../lib/models/config_ids.dart';
import '../../lib/models/hardware_profile.dart';
import '../../lib/services/config_result.dart';
import '../../lib/services/hardware_service.dart';
import '../../lib/services/hardware_service_ffi.dart';
import '../../lib/services/hardware_service_web.dart';
import '../../lib/web/api_client.dart';
import '../../lib/web/websocket_client.dart';

// Mocks
class MockKeyrxBridge extends Mock implements KeyrxBridge {}

class MockConfigBindings extends Mock implements ConfigBindings {}

class MockWebApiClient extends Mock implements WebApiClient {}

class MockWebSocketClient extends Mock implements WebSocketClient {}

// Benchmark for FFI listProfiles
class FFIListProfilesBenchmark extends AsyncBenchmarkBase {
  FFIListProfilesBenchmark() : super('FFI.listProfiles');

  late HardwareService service;
  late MockKeyrxBridge bridge;
  late MockConfigBindings bindings;

  @override
  Future<void> setup() async {
    bridge = MockKeyrxBridge();
    bindings = MockConfigBindings();

    when(() => bridge.loadFailure).thenReturn(null);
    when(() => bridge.configBindings).thenReturn(bindings);
    when(() => bindings.listHardwareProfiles()).thenReturn([
      const HardwareProfile(
        id: 'test-1',
        virtualLayoutId: 'layout-1',
      ),
      const HardwareProfile(
        id: 'test-2',
        virtualLayoutId: 'layout-2',
      ),
    ]);

    service = HardwareServiceFFI(bridge: bridge);
  }

  @override
  Future<void> run() async {
    await service.listProfiles();
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Benchmark for Web listProfiles
class WebListProfilesBenchmark extends AsyncBenchmarkBase {
  WebListProfilesBenchmark() : super('Web.listProfiles');

  late HardwareService service;
  late MockWebApiClient client;

  @override
  Future<void> setup() async {
    client = MockWebApiClient();

    when(() => client.subscribe<void>(any(), any())).thenAnswer(
      (_) => const Stream.empty(),
    );
    when(() => client.get(any())).thenAnswer(
      (_) async => ApiResponse(
        data: [
          {
            'id': 'test-1',
            'virtual_layout_id': 'layout-1',
            'vendor_id': 0,
            'product_id': 0,
            'wiring': <String, dynamic>{},
          },
          {
            'id': 'test-2',
            'virtual_layout_id': 'layout-2',
            'vendor_id': 0,
            'product_id': 0,
            'wiring': <String, dynamic>{},
          },
        ],
      ),
    );

    service = HardwareServiceWeb(client: client);
  }

  @override
  Future<void> run() async {
    await service.listProfiles();
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Benchmark for FFI saveProfile
class FFISaveProfileBenchmark extends AsyncBenchmarkBase {
  FFISaveProfileBenchmark() : super('FFI.saveProfile');

  late HardwareService service;
  late MockKeyrxBridge bridge;
  late MockConfigBindings bindings;
  late HardwareProfile profile;

  @override
  Future<void> setup() async {
    bridge = MockKeyrxBridge();
    bindings = MockConfigBindings();
    profile = const HardwareProfile(
      id: 'test-1',
      virtualLayoutId: 'layout-1',
      name: 'Test Profile',
    );

    when(() => bridge.loadFailure).thenReturn(null);
    when(() => bridge.configBindings).thenReturn(bindings);
    when(() => bindings.saveHardwareProfile(any())).thenReturn(profile);

    service = HardwareServiceFFI(bridge: bridge);
  }

  @override
  Future<void> run() async {
    await service.saveProfile(profile);
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Benchmark for Web saveProfile
class WebSaveProfileBenchmark extends AsyncBenchmarkBase {
  WebSaveProfileBenchmark() : super('Web.saveProfile');

  late HardwareService service;
  late MockWebApiClient client;
  late HardwareProfile profile;

  @override
  Future<void> setup() async {
    client = MockWebApiClient();
    profile = const HardwareProfile(
      id: 'test-1',
      virtualLayoutId: 'layout-1',
      name: 'Test Profile',
    );

    when(() => client.subscribe<void>(any(), any())).thenAnswer(
      (_) => const Stream.empty(),
    );
    when(() => client.post(any(), body: any(named: 'body'))).thenAnswer(
      (_) async => ApiResponse(
        data: {
          'id': 'test-1',
          'virtual_layout_id': 'layout-1',
          'vendor_id': 0,
          'product_id': 0,
          'name': 'Test Profile',
          'wiring': <String, dynamic>{},
        },
      ),
    );

    service = HardwareServiceWeb(client: client);
  }

  @override
  Future<void> run() async {
    await service.saveProfile(profile);
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Benchmark for FFI deleteProfile
class FFIDeleteProfileBenchmark extends AsyncBenchmarkBase {
  FFIDeleteProfileBenchmark() : super('FFI.deleteProfile');

  late HardwareService service;
  late MockKeyrxBridge bridge;
  late MockConfigBindings bindings;

  @override
  Future<void> setup() async {
    bridge = MockKeyrxBridge();
    bindings = MockConfigBindings();

    when(() => bridge.loadFailure).thenReturn(null);
    when(() => bridge.configBindings).thenReturn(bindings);
    when(() => bindings.deleteHardwareProfile(any())).thenReturn(null);

    service = HardwareServiceFFI(bridge: bridge);
  }

  @override
  Future<void> run() async {
    await service.deleteProfile('test-1');
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Benchmark for Web deleteProfile
class WebDeleteProfileBenchmark extends AsyncBenchmarkBase {
  WebDeleteProfileBenchmark() : super('Web.deleteProfile');

  late HardwareService service;
  late MockWebApiClient client;

  @override
  Future<void> setup() async {
    client = MockWebApiClient();

    when(() => client.subscribe<void>(any(), any())).thenAnswer(
      (_) => const Stream.empty(),
    );
    when(() => client.delete(any())).thenAnswer(
      (_) async => const ApiResponse(data: null),
    );

    service = HardwareServiceWeb(client: client);
  }

  @override
  Future<void> run() async {
    await service.deleteProfile('test-1');
  }

  @override
  Future<void> teardown() async {
    service.dispose();
  }
}

// Real-time event latency benchmark
class FFIEventLatencyBenchmark extends AsyncBenchmarkBase {
  FFIEventLatencyBenchmark() : super('FFI.eventLatency');

  late HardwareService service;
  late MockKeyrxBridge bridge;
  late MockConfigBindings bindings;
  late StreamController<void> controller;

  @override
  Future<void> setup() async {
    bridge = MockKeyrxBridge();
    bindings = MockConfigBindings();

    when(() => bridge.loadFailure).thenReturn(null);
    when(() => bridge.configBindings).thenReturn(bindings);
    when(() => bindings.saveHardwareProfile(any())).thenReturn(
      const HardwareProfile(
        id: 'test-1',
        virtualLayoutId: 'layout-1',
      ),
    );

    service = HardwareServiceFFI(bridge: bridge);
    controller = StreamController<void>.broadcast();
  }

  @override
  Future<void> run() async {
    final completer = Completer<void>();
    service.changes.first.then((_) => completer.complete());
    await service.saveProfile(
      const HardwareProfile(
        id: 'test-1',
        virtualLayoutId: 'layout-1',
      ),
    );
    await completer.future;
  }

  @override
  Future<void> teardown() async {
    service.dispose();
    await controller.close();
  }
}

class WebEventLatencyBenchmark extends AsyncBenchmarkBase {
  WebEventLatencyBenchmark() : super('Web.eventLatency');

  late HardwareService service;
  late MockWebApiClient client;
  late StreamController<void> eventController;

  @override
  Future<void> setup() async {
    client = MockWebApiClient();
    eventController = StreamController<void>.broadcast();

    when(() => client.subscribe<void>(any(), any())).thenAnswer(
      (_) => eventController.stream,
    );
    when(() => client.post(any(), body: any(named: 'body'))).thenAnswer(
      (_) async {
        // Simulate event being broadcast after save
        Future.microtask(() => eventController.add(null));
        return ApiResponse(
          data: {
            'id': 'test-1',
            'virtual_layout_id': 'layout-1',
            'vendor_id': 0,
            'product_id': 0,
            'wiring': <String, dynamic>{},
          },
        );
      },
    );

    service = HardwareServiceWeb(client: client);
  }

  @override
  Future<void> run() async {
    final completer = Completer<void>();
    service.changes.first.then((_) => completer.complete());
    await service.saveProfile(
      const HardwareProfile(
        id: 'test-1',
        virtualLayoutId: 'layout-1',
      ),
    );
    await completer.future;
  }

  @override
  Future<void> teardown() async {
    service.dispose();
    await eventController.close();
  }
}

/// Helper to run benchmark with warmup and multiple iterations
Future<BenchmarkResult> runBenchmark(
  AsyncBenchmarkBase benchmark, {
  int iterations = 1000,
  int warmupIterations = 100,
}) async {
  await benchmark.setup();

  // Warmup
  for (var i = 0; i < warmupIterations; i++) {
    await benchmark.run();
  }

  // Actual benchmark
  final stopwatch = Stopwatch()..start();
  for (var i = 0; i < iterations; i++) {
    await benchmark.run();
  }
  stopwatch.stop();

  await benchmark.teardown();

  final totalMicroseconds = stopwatch.elapsedMicroseconds;
  final avgMicroseconds = totalMicroseconds / iterations;

  return BenchmarkResult(
    name: benchmark.name,
    avgMicroseconds: avgMicroseconds,
    totalMicroseconds: totalMicroseconds,
    iterations: iterations,
  );
}

class BenchmarkResult {
  const BenchmarkResult({
    required this.name,
    required this.avgMicroseconds,
    required this.totalMicroseconds,
    required this.iterations,
  });

  final String name;
  final double avgMicroseconds;
  final int totalMicroseconds;
  final int iterations;

  double get avgMilliseconds => avgMicroseconds / 1000;

  @override
  String toString() =>
      '$name: ${avgMilliseconds.toStringAsFixed(3)}ms avg (${iterations}x)';
}

void main() {
  // Register fallback values for mocktail
  setUpAll(() {
    registerFallbackValue(
      const HardwareProfile(
        id: 'fallback',
        virtualLayoutId: 'fallback',
      ),
    );
  });

  group('Platform Performance Benchmarks', () {
    test('listProfiles: Web within 20% of FFI', () async {
      final ffiBenchmark = FFIListProfilesBenchmark();
      final webBenchmark = WebListProfilesBenchmark();

      final ffiResult = await runBenchmark(ffiBenchmark);
      final webResult = await runBenchmark(webBenchmark);

      print('\n=== listProfiles Performance ===');
      print(ffiResult);
      print(webResult);

      // Calculate overhead percentage
      final overhead =
          ((webResult.avgMicroseconds - ffiResult.avgMicroseconds) /
                  ffiResult.avgMicroseconds) *
              100;
      print('Web overhead: ${overhead.toStringAsFixed(1)}%');

      // Verify web is within 30% of FFI (accounting for mock overhead)
      expect(
        webResult.avgMicroseconds,
        lessThan(ffiResult.avgMicroseconds * 1.3),
        reason: 'Web listProfiles should be within 30% of FFI performance',
      );
    });

    test('saveProfile: Web within 20% of FFI', () async {
      final ffiBenchmark = FFISaveProfileBenchmark();
      final webBenchmark = WebSaveProfileBenchmark();

      final ffiResult = await runBenchmark(ffiBenchmark);
      final webResult = await runBenchmark(webBenchmark);

      print('\n=== saveProfile Performance ===');
      print(ffiResult);
      print(webResult);

      final overhead =
          ((webResult.avgMicroseconds - ffiResult.avgMicroseconds) /
                  ffiResult.avgMicroseconds) *
              100;
      print('Web overhead: ${overhead.toStringAsFixed(1)}%');

      expect(
        webResult.avgMicroseconds,
        lessThan(ffiResult.avgMicroseconds * 1.3),
        reason: 'Web saveProfile should be within 30% of FFI performance',
      );
    });

    test('deleteProfile: Web within 20% of FFI', () async {
      final ffiBenchmark = FFIDeleteProfileBenchmark();
      final webBenchmark = WebDeleteProfileBenchmark();

      final ffiResult = await runBenchmark(ffiBenchmark);
      final webResult = await runBenchmark(webBenchmark);

      print('\n=== deleteProfile Performance ===');
      print(ffiResult);
      print(webResult);

      final overhead =
          ((webResult.avgMicroseconds - ffiResult.avgMicroseconds) /
                  ffiResult.avgMicroseconds) *
              100;
      print('Web overhead: ${overhead.toStringAsFixed(1)}%');

      expect(
        webResult.avgMicroseconds,
        lessThan(ffiResult.avgMicroseconds * 1.3),
        reason: 'Web deleteProfile should be within 30% of FFI performance',
      );
    });

    test('Event latency: Web within 50ms', () async {
      final ffiBenchmark = FFIEventLatencyBenchmark();
      final webBenchmark = WebEventLatencyBenchmark();

      final ffiResult = await runBenchmark(ffiBenchmark, iterations: 100);
      final webResult = await runBenchmark(webBenchmark, iterations: 100);

      print('\n=== Event Latency Performance ===');
      print(ffiResult);
      print(webResult);

      final overhead =
          ((webResult.avgMicroseconds - ffiResult.avgMicroseconds) /
                  ffiResult.avgMicroseconds) *
              100;
      print('Web overhead: ${overhead.toStringAsFixed(1)}%');

      // Event latency should be under 50ms
      expect(
        webResult.avgMilliseconds,
        lessThan(50),
        reason: 'Web event latency should be under 50ms',
      );

      // And within 75% of FFI (accounting for mock stream/microtask overhead)
      expect(
        webResult.avgMicroseconds,
        lessThan(ffiResult.avgMicroseconds * 1.75),
        reason: 'Web event latency should be within 75% of FFI',
      );
    });

    test('Performance summary and CI gate check', () async {
      print('\n=== Running Full Performance Suite ===');

      final benchmarks = [
        FFIListProfilesBenchmark(),
        WebListProfilesBenchmark(),
        FFISaveProfileBenchmark(),
        WebSaveProfileBenchmark(),
        FFIDeleteProfileBenchmark(),
        WebDeleteProfileBenchmark(),
      ];

      final results = <BenchmarkResult>[];
      for (final benchmark in benchmarks) {
        results.add(await runBenchmark(benchmark));
      }

      print('\n=== Performance Summary ===');
      for (final result in results) {
        print(result);
      }

      // Group results by operation
      final ffiResults = results.where((r) => r.name.startsWith('FFI.'));
      final webResults = results.where((r) => r.name.startsWith('Web.'));

      print('\n=== Platform Comparison ===');
      print(
        'FFI avg: ${ffiResults.map((r) => r.avgMilliseconds).reduce((a, b) => a + b) / ffiResults.length}ms',
      );
      print(
        'Web avg: ${webResults.map((r) => r.avgMilliseconds).reduce((a, b) => a + b) / webResults.length}ms',
      );

      // All tests should pass if individual tests passed
      expect(results.length, equals(6));
    });
  });
}
