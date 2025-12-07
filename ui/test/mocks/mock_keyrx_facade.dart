/// Mock implementation of KeyrxFacade for testing.
///
/// Provides sensible default stubbing for all facade operations to simplify
/// widget testing. Default behaviors can be overridden using mocktail's when() API.
library;

import 'package:keyrx_ui/ffi/bridge_discovery.dart';
import 'package:keyrx_ui/services/facade/facade_state.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/facade/result.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/test_service.dart';
import 'package:mocktail/mocktail.dart';
import 'package:rxdart/rxdart.dart';

/// Mock implementation of [KeyrxFacade] using mocktail.
///
/// Provides default stubs for common operations that can be overridden in tests:
///
/// Example usage:
/// ```dart
/// final mockFacade = MockKeyrxFacade.withDefaults();
///
/// // Override specific behaviors
/// when(() => mockFacade.startEngine(any()))
///     .thenAnswer((_) async => Result.err(FacadeError.validation('Invalid script')));
///
/// // Use in widget tests
/// testWidgets('shows error when engine fails to start', (tester) async {
///   await tester.pumpWidget(MyApp(facade: mockFacade));
///   // ... test logic
/// });
/// ```
class MockKeyrxFacade extends Mock implements KeyrxFacade {
  MockKeyrxFacade._();

  /// Create a mock facade with sensible default stubs.
  ///
  /// Default behaviors:
  /// - All operations return Result.ok(...)
  /// - State stream emits initial state
  /// - currentState returns initial state
  /// - All async operations complete successfully
  factory MockKeyrxFacade.withDefaults() {
    final mock = MockKeyrxFacade._();
    _setupDefaultStubs(mock);
    return mock;
  }

  /// Create a mock facade without any default stubs.
  ///
  /// Use this when you want complete control over mock behavior.
  /// You must stub all methods before using the mock.
  factory MockKeyrxFacade.noDefaults() {
    return MockKeyrxFacade._();
  }

  static void _setupDefaultStubs(MockKeyrxFacade mock) {
    // State stream and current state
    final stateSubject = BehaviorSubject<FacadeState>.seeded(
      FacadeState.initial(),
    );
    when(() => mock.stateStream).thenAnswer((_) => stateSubject.stream);
    when(() => mock.currentState).thenReturn(FacadeState.initial());

    // Engine operations
    when(
      () => mock.startEngine(any()),
    ).thenAnswer((_) async => const Result.ok(null));
    when(
      () => mock.stopEngine(),
    ).thenAnswer((_) async => const Result.ok(null));
    when(
      () => mock.getEngineStatus(),
    ).thenAnswer((_) async => const Result.ok(EngineStatus.uninitialized));

    // Script operations
    when(() => mock.validateScript(any())).thenAnswer(
      (_) async => const Result.ok(
        ScriptValidationResult(isValid: true, errors: [], warnings: []),
      ),
    );
    when(
      () => mock.loadScriptContent(any()),
    ).thenAnswer((_) async => const Result.ok('// Empty script'));
    when(
      () => mock.saveScript(any(), any()),
    ).thenAnswer((_) async => const Result.ok(null));

    // Device operations
    when(() => mock.listDevices()).thenAnswer((_) async => const Result.ok([]));
    when(
      () => mock.selectDevice(any()),
    ).thenAnswer((_) async => const Result.ok(null));

    // Test operations
    when(() => mock.discoverTests(any())).thenAnswer(
      (_) async => const Result.ok(TestDiscoveryServiceResult(tests: [])),
    );
    when(() => mock.runTests(any(), filter: any(named: 'filter'))).thenAnswer(
      (_) async => const Result.ok(
        TestRunServiceResult(
          total: 0,
          passed: 0,
          failed: 0,
          durationMs: 0.0,
          results: [],
        ),
      ),
    );
    when(
      () => mock.cancelTests(),
    ).thenAnswer((_) async => const Result.ok(null));

    // Lifecycle
    when(() => mock.dispose()).thenAnswer((_) async {
      await stateSubject.close();
    });

    // Services (return null as it shouldn't be used in most tests)
    when(() => mock.services).thenReturn(_MockServiceRegistry());
  }
}

/// Minimal mock of ServiceRegistry for facade tests.
class _MockServiceRegistry extends Mock implements ServiceRegistry {}

/// Register mocktail fallback values for KeyrxFacade types.
///
/// Call this in your test file's setUpAll() before using MockKeyrxFacade:
///
/// ```dart
/// setUpAll(() {
///   registerKeyrxFacadeFallbackValues();
/// });
/// ```
void registerKeyrxFacadeFallbackValues() {
  registerFallbackValue(
    const KeyboardDevice(
      path: '/dev/input/event0',
      name: 'Test Keyboard',
      vendorId: 0x1234,
      productId: 0x5678,
      hasProfile: false,
    ),
  );
  registerFallbackValue(<int>[]);
}
