# KeyRx Flutter Facade Guide

## Overview

The KeyRx Facade (`KeyrxFacade`) provides a simplified, unified API over the 19+ underlying services in the Flutter UI layer. Instead of injecting and coordinating 7+ services in each page, developers can inject a single facade that handles common operations, aggregates state, and translates errors.

### Key Benefits

1. **Simplified API**: Single injection point replaces 7+ service dependencies
2. **State Aggregation**: Observe combined engine/device/script state through one stream
3. **Operation Coordination**: Multi-step operations (validate → load → start) handled atomically
4. **Error Translation**: Technical errors automatically converted to user-friendly messages
5. **Easy Testing**: Mock single facade instead of 7+ service mocks

### Architecture

```
┌─────────────────────────────────────┐
│          Flutter Pages              │
│  (EditorPage, DiscoveryPage, etc.)  │
└──────────────┬──────────────────────┘
               │ inject & use
               ▼
┌─────────────────────────────────────┐
│         KeyrxFacade                 │
│  ┌───────────────────────────────┐  │
│  │  Unified API Surface          │  │
│  │  State Aggregation            │  │
│  │  Operation Coordination       │  │
│  │  Error Translation            │  │
│  └───────────────────────────────┘  │
└──────────────┬──────────────────────┘
               │ wraps & coordinates
               ▼
┌─────────────────────────────────────┐
│       ServiceRegistry               │
│  ┌─────────────────────────────┐   │
│  │ EngineService               │   │
│  │ DeviceService               │   │
│  │ ScriptFileService           │   │
│  │ TestService                 │   │
│  │ ValidationService           │   │
│  │ ... 14+ more services       │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

---

## Core Concepts

### Result Type

The facade uses a `Result<T>` type for explicit error handling without exceptions:

```dart
Result<int> divide(int a, int b) {
  if (b == 0) {
    return Result.err(FacadeError.validation('Division by zero'));
  }
  return Result.ok(a ~/ b);
}

// Pattern matching
result.when(
  ok: (value) => print('Result: $value'),
  err: (error) => print('Error: ${error.userMessage}'),
);

// Or use helper methods
final value = result.unwrapOr(0);  // Returns 0 if error
final valueOrNull = result.okOrNull;
```

### FacadeState

Aggregated state from all subsystems:

```dart
class FacadeState {
  final EngineStatus engine;        // uninitialized, ready, running, etc.
  final DeviceStatus device;        // none, available, connected, etc.
  final ValidationStatus validation; // none, valid, invalid, etc.
  final DiscoveryStatus discovery;   // idle, active, completed, etc.

  final String? scriptPath;
  final String? selectedDevicePath;
  final int? validationErrorCount;
  final DateTime timestamp;
}
```

### FacadeError

Structured error with user-friendly messages:

```dart
class FacadeError {
  final String code;           // VALIDATION_ERROR, SERVICE_UNAVAILABLE, etc.
  final String message;        // Technical message for logging
  final String userMessage;    // User-friendly message for display
  final Object? originalError; // Original error for debugging
}

// Factory methods for common error types
FacadeError.validation('Invalid syntax');
FacadeError.serviceUnavailable('EngineService');
FacadeError.operationFailed('startEngine', 'Device not connected');
FacadeError.notFound('/path/to/script.rhai');
```

---

## API Reference

### Getting Started

```dart
// In providers.dart
Provider<KeyrxFacade>(
  create: (context) {
    final registry = context.read<ServiceRegistry>();
    return KeyrxFacade.real(registry);
  },
  dispose: (_, facade) => facade.dispose(),
),

// In your page
class MyPage extends StatefulWidget {
  const MyPage({super.key, required this.facade});

  final KeyrxFacade facade;

  @override
  State<MyPage> createState() => _MyPageState();
}

// Or use Provider
final facade = context.read<KeyrxFacade>();
```

### State Observation

```dart
// Subscribe to aggregated state
StreamSubscription? _stateSubscription;

@override
void initState() {
  super.initState();
  _stateSubscription = widget.facade.stateStream.listen((state) {
    setState(() {
      // Update UI based on state
      if (state.engine == EngineStatus.running) {
        _statusText = 'Engine Running';
      }
      if (state.validation == ValidationStatus.invalid) {
        _showValidationErrors(state.validationErrorCount ?? 0);
      }
    });
  });
}

@override
void dispose() {
  _stateSubscription?.cancel();
  super.dispose();
}

// Or get current state without subscribing
final currentState = widget.facade.currentState;
if (currentState.isEngineRunning) {
  // Do something
}
```

### Engine Operations

#### Start Engine

```dart
Future<void> _startEngine() async {
  final result = await widget.facade.startEngine('/path/to/script.rhai');

  result.when(
    ok: (_) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Engine started')),
      );
    },
    err: (error) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error.userMessage)),
      );
    },
  );
}
```

The `startEngine` method performs a coordinated operation:
1. Validates the script syntax
2. Loads the script into the engine
3. Initializes the engine
4. Updates state to reflect running engine

If any step fails, the operation is rolled back automatically.

#### Stop Engine

```dart
Future<void> _stopEngine() async {
  final result = await widget.facade.stopEngine();

  if (result.isErr) {
    // Handle error
    final error = result.errOrNull!;
    print('Failed to stop: ${error.userMessage}');
  }
}
```

The `stopEngine` method:
1. Stops any active recording
2. Shuts down the engine
3. Cleans up resources
4. Updates state to reflect stopped engine

#### Get Engine Status

```dart
final statusResult = await widget.facade.getEngineStatus();
statusResult.when(
  ok: (status) => print('Engine status: $status'),
  err: (error) => print('Failed to get status: ${error.message}'),
);
```

Note: For most cases, prefer observing `stateStream` which includes engine status along with other subsystem states.

### Script Operations

#### Validate Script

```dart
Future<void> _validateScript(String path) async {
  final result = await widget.facade.validateScript(path);

  result.when(
    ok: (validationResult) {
      if (validationResult.isValid) {
        print('Script is valid');
      } else {
        print('Validation errors:');
        for (final error in validationResult.errors) {
          print('  ${error.format()}');
        }
      }
    },
    err: (error) {
      print('Validation failed: ${error.userMessage}');
    },
  );
}
```

#### Load and Save Scripts

```dart
// Load script content
final loadResult = await widget.facade.loadScriptContent('/path/to/script.rhai');
final content = loadResult.unwrapOr('// Default content');

// Save script content
final saveResult = await widget.facade.saveScript(
  '/path/to/script.rhai',
  scriptContent,
);

saveResult.when(
  ok: (_) => print('Saved successfully'),
  err: (error) => showError(error.userMessage),
);
```

### Device Operations

#### List Devices

```dart
Future<void> _loadDevices() async {
  final result = await widget.facade.listDevices();

  result.when(
    ok: (devices) {
      setState(() {
        _availableDevices = devices;
      });
    },
    err: (error) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(error.userMessage)),
      );
    },
  );
}
```

#### Select Device

```dart
Future<void> _selectDevice(String devicePath) async {
  final result = await widget.facade.selectDevice(devicePath);

  if (result.isOk) {
    print('Device selected');
  }
}
```

#### Device Discovery

```dart
Future<void> _startDiscovery() async {
  final result = await widget.facade.startDiscovery(
    device: selectedDevice,
    rows: 6,
    colsPerRow: [15, 14, 13, 12, 12, 6],
  );

  if (result.isOk) {
    // Monitor state stream for discovery progress
    widget.facade.stateStream.listen((state) {
      if (state.discovery == DiscoveryStatus.completed) {
        print('Discovery completed: ${state.discoveredDeviceCount} keys');
      }
    });
  }
}

Future<void> _cancelDiscovery() async {
  await widget.facade.cancelDiscovery();
}
```

### Test Operations

#### Discover Tests

```dart
Future<void> _discoverTests(String scriptPath) async {
  final result = await widget.facade.discoverTests(scriptPath);

  result.when(
    ok: (discovery) {
      print('Found ${discovery.tests.length} tests');
      for (final test in discovery.tests) {
        print('  - ${test.name}');
      }
    },
    err: (error) {
      print('Discovery failed: ${error.userMessage}');
    },
  );
}
```

#### Run Tests

```dart
Future<void> _runTests(String scriptPath) async {
  final result = await widget.facade.runTests(
    scriptPath,
    filter: 'test_key_*',  // Optional filter
  );

  result.when(
    ok: (testResults) {
      print('Tests passed: ${testResults.passed}');
      print('Tests failed: ${testResults.failed}');

      for (final failure in testResults.failures) {
        print('  ${failure.testName}: ${failure.message}');
      }
    },
    err: (error) {
      print('Test execution failed: ${error.userMessage}');
    },
  );
}

// Cancel running tests
Future<void> _cancelTests() async {
  await widget.facade.cancelTests();
}
```

### Advanced: Direct Service Access

For rare operations not exposed through the facade:

```dart
// Access underlying services through escape hatch
final services = widget.facade.services;

// Example: Use a service-specific method
final keyRegistry = await services.engineService.fetchKeyRegistry();
final rawDeviceInfo = await services.deviceService.getRawDeviceInfo(deviceId);
```

This should be used sparingly. If you find yourself frequently needing direct service access, consider whether the facade API should be extended.

---

## Migration Guide

### Before: Direct Service Injection

```dart
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    required this.engineService,
    required this.scriptService,
    required this.validationService,
    required this.deviceService,
    required this.errorTranslator,
  });

  final EngineService engineService;
  final ScriptFileService scriptService;
  final ValidationService validationService;
  final DeviceService deviceService;
  final ErrorTranslator errorTranslator;
}

// In the page
Future<void> _startEngine() async {
  try {
    // Manual validation
    final validationResult = await widget.validationService.validate(scriptPath);
    if (!validationResult.isValid) {
      _showError('Validation failed');
      return;
    }

    // Manual loading
    final content = await widget.scriptService.loadScript(scriptPath);
    await widget.engineService.loadScript(content);

    // Manual start
    await widget.engineService.start();

    // Manual state update
    setState(() {
      _engineRunning = true;
    });
  } catch (e) {
    // Manual error translation
    final userMsg = widget.errorTranslator.translate(e);
    _showError(userMsg.body);
  }
}
```

### After: Facade Pattern

```dart
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    required this.facade,
  });

  final KeyrxFacade facade;
}

// In the page
Future<void> _startEngine() async {
  final result = await widget.facade.startEngine(scriptPath);

  result.when(
    ok: (_) {
      // State automatically updated through stateStream
      _showSuccess('Engine started');
    },
    err: (error) {
      // Error already translated
      _showError(error.userMessage);
    },
  );
}
```

### Migration Steps

1. **Add facade to page constructor**:
   ```dart
   class MyPage extends StatefulWidget {
     const MyPage({
       super.key,
       required this.facade,
       // Remove individual services
       // required this.engineService,
       // required this.scriptService,
       // ...
     });

     final KeyrxFacade facade;
   ```

2. **Subscribe to state stream**:
   ```dart
   @override
   void initState() {
     super.initState();
     _stateSubscription = widget.facade.stateStream.listen((state) {
       setState(() {
         _engineStatus = state.engine;
         _deviceStatus = state.device;
         // ... update local state
       });
     });
   }
   ```

3. **Replace service calls with facade methods**:
   ```dart
   // Before
   await widget.engineService.start();

   // After
   await widget.facade.startEngine(scriptPath);
   ```

4. **Handle Results instead of try-catch**:
   ```dart
   // Before
   try {
     await widget.engineService.start();
   } catch (e) {
     _handleError(e);
   }

   // After
   final result = await widget.facade.startEngine(scriptPath);
   result.when(
     ok: (_) => _onSuccess(),
     err: (error) => _handleError(error),
   );
   ```

5. **Update providers** (already done in `providers.dart`):
   ```dart
   Provider<KeyrxFacade>(
     create: (context) => KeyrxFacade.real(context.read<ServiceRegistry>()),
     dispose: (_, facade) => facade.dispose(),
   ),
   ```

---

## Testing Guide

### Widget Tests with MockKeyrxFacade

Before the facade, testing required mocking 7+ services:

```dart
// OLD: Complex test setup
late MockEngineService mockEngine;
late MockScriptFileService mockScript;
late MockValidationService mockValidation;
late MockDeviceService mockDevice;
late MockErrorTranslator mockTranslator;

setUp(() {
  mockEngine = MockEngineService();
  mockScript = MockScriptFileService();
  mockValidation = MockValidationService();
  mockDevice = MockDeviceService();
  mockTranslator = MockErrorTranslator();

  when(mockEngine.start()).thenAnswer((_) async => {});
  when(mockScript.loadScript(any)).thenAnswer((_) async => 'content');
  when(mockValidation.validate(any)).thenAnswer((_) async => ValidationResult.valid());
  // ... many more stubs
});
```

With the facade, you only mock one thing:

```dart
// NEW: Simple test setup
late MockKeyrxFacade mockFacade;

setUp(() {
  mockFacade = MockKeyrxFacade();

  // Stub only what you need
  when(mockFacade.startEngine(any))
    .thenAnswer((_) async => Result.ok(null));

  when(mockFacade.stateStream)
    .thenAnswer((_) => Stream.value(FacadeState.initial()));
});
```

### Creating a Mock Facade

```dart
import 'package:mockito/mockito.dart';
import 'package:keyrx/services/facade/keyrx_facade.dart';

class MockKeyrxFacade extends Mock implements KeyrxFacade {}
```

Generate mocks with mockito:

```dart
// In your test file
import 'package:mockito/annotations.dart';

@GenerateMocks([KeyrxFacade])
import 'my_test.mocks.dart';
```

### Example Widget Test

```dart
testWidgets('EditorPage starts engine on button press', (tester) async {
  // Arrange
  final mockFacade = MockKeyrxFacade();
  when(mockFacade.startEngine(any))
    .thenAnswer((_) async => Result.ok(null));
  when(mockFacade.stateStream)
    .thenAnswer((_) => Stream.value(FacadeState.initial()));
  when(mockFacade.currentState)
    .thenReturn(FacadeState.initial());

  // Act
  await tester.pumpWidget(
    MaterialApp(
      home: EditorPage(facade: mockFacade),
    ),
  );

  await tester.tap(find.byKey(Key('start_engine_button')));
  await tester.pump();

  // Assert
  verify(mockFacade.startEngine(any)).called(1);
});
```

### Testing State Transitions

```dart
testWidgets('UI updates when engine state changes', (tester) async {
  final stateController = StreamController<FacadeState>();
  final mockFacade = MockKeyrxFacade();

  when(mockFacade.stateStream).thenAnswer((_) => stateController.stream);
  when(mockFacade.currentState).thenReturn(FacadeState.initial());

  await tester.pumpWidget(
    MaterialApp(home: MyPage(facade: mockFacade)),
  );

  // Emit state change
  stateController.add(
    FacadeState.initial().withEngineStatus(EngineStatus.running),
  );
  await tester.pump();

  // Verify UI updated
  expect(find.text('Running'), findsOneWidget);
});
```

### Integration Tests

For integration tests with real services but mock FFI:

```dart
testWidgets('Full engine lifecycle integration', (tester) async {
  final mockBridge = MockBridge();
  final registry = ServiceRegistry.real(bridge: mockBridge);
  final facade = KeyrxFacade.real(registry);

  // Test with real facade and services, mock bridge
  await tester.pumpWidget(
    Provider<KeyrxFacade>.value(
      value: facade,
      child: MaterialApp(home: EditorPage()),
    ),
  );

  // Perform operations...
});
```

---

## Performance Considerations

### Overhead

The facade adds minimal overhead:
- Method call overhead: < 1ms
- State aggregation: Lazy evaluation (only computed when observed)
- Memory overhead: < 1MB for facade + state management

### State Stream Debouncing

State updates are debounced by 100ms to avoid excessive emissions during rapid changes:

```dart
// Multiple rapid state changes
engineService.updateState();   // T+0ms
deviceService.updateState();   // T+10ms
validationService.updateState(); // T+20ms

// Facade emits once at T+120ms with combined state
facade.stateStream.listen((state) {
  // Receives single aggregated state update
});
```

### Best Practices

1. **Subscribe once**: Don't create multiple subscriptions to `stateStream`
   ```dart
   // Good: Single subscription
   _subscription = facade.stateStream.listen(_handleState);

   // Bad: Multiple subscriptions
   facade.stateStream.listen(_handleEngine);
   facade.stateStream.listen(_handleDevice);
   ```

2. **Dispose properly**: Always cancel subscriptions
   ```dart
   @override
   void dispose() {
     _stateSubscription?.cancel();
     super.dispose();
   }
   ```

3. **Use currentState for one-off checks**:
   ```dart
   // Good: Check state once
   if (facade.currentState.isEngineRunning) {
     // Do something
   }

   // Bad: Subscribe just for one check
   facade.stateStream.first.then((state) {
     if (state.isEngineRunning) { /* ... */ }
   });
   ```

---

## Common Patterns

### Loading Pattern

```dart
Future<void> _loadScript() async {
  setState(() => _isLoading = true);

  final result = await widget.facade.loadScriptContent(_scriptPath);

  setState(() {
    _isLoading = false;
    _content = result.unwrapOr('');
  });
}
```

### Error Display Pattern

```dart
void _handleResult<T>(Result<T> result) {
  result.when(
    ok: (value) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Success')),
      );
    },
    err: (error) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(error.userMessage),
          backgroundColor: Colors.red,
          action: SnackBarAction(
            label: 'Details',
            onPressed: () => _showErrorDetails(error),
          ),
        ),
      );
    },
  );
}
```

### State-Dependent UI Pattern

```dart
@override
Widget build(BuildContext context) {
  return StreamBuilder<FacadeState>(
    stream: widget.facade.stateStream,
    initialData: widget.facade.currentState,
    builder: (context, snapshot) {
      final state = snapshot.data ?? FacadeState.initial();

      return Column(
        children: [
          _buildEngineStatus(state.engine),
          _buildDeviceStatus(state.device),

          ElevatedButton(
            onPressed: state.canStartEngine ? _startEngine : null,
            child: const Text('Start Engine'),
          ),
        ],
      );
    },
  );
}
```

### Chaining Operations Pattern

```dart
Future<void> _setupAndStart() async {
  // Load script
  final loadResult = await widget.facade.loadScriptContent(scriptPath);
  if (loadResult.isErr) {
    _showError(loadResult.errOrNull!.userMessage);
    return;
  }

  // Validate
  final validateResult = await widget.facade.validateScript(scriptPath);
  if (validateResult.isErr) {
    _showError(validateResult.errOrNull!.userMessage);
    return;
  }

  final validation = validateResult.unwrap();
  if (!validation.isValid) {
    _showValidationErrors(validation.errors);
    return;
  }

  // Start engine
  final startResult = await widget.facade.startEngine(scriptPath);
  _handleResult(startResult);
}

// Or use andThen for cleaner chaining
Future<void> _setupAndStartChained() async {
  final result = await widget.facade
    .loadScriptContent(scriptPath)
    .then((r) => r.andThen((_) => widget.facade.validateScript(scriptPath)))
    .then((r) => r.andThen((_) => widget.facade.startEngine(scriptPath)));

  _handleResult(result);
}
```

---

## Troubleshooting

### "Service not available" errors

**Problem**: Getting `SERVICE_UNAVAILABLE` errors when calling facade methods.

**Solution**: Ensure `ServiceRegistry` is properly initialized before creating the facade:

```dart
// In main.dart
final registry = ServiceRegistry.real();
await registry.initialize();  // Wait for initialization

final facade = KeyrxFacade.real(registry);
```

### State stream not updating

**Problem**: Subscribing to `stateStream` but not receiving updates.

**Solution**: Check that:
1. You're subscribed before state changes occur
2. Your subscription isn't being cancelled prematurely
3. The widget is mounted when setState is called

```dart
StreamSubscription? _subscription;

@override
void initState() {
  super.initState();
  _subscription = widget.facade.stateStream.listen((state) {
    if (mounted) {  // Check mounted before setState
      setState(() {
        // Update state
      });
    }
  });
}
```

### Memory leaks

**Problem**: App memory grows over time.

**Solution**: Always dispose subscriptions and the facade:

```dart
@override
void dispose() {
  _stateSubscription?.cancel();
  // Don't dispose facade here - it's managed by Provider
  super.dispose();
}
```

### Test timeout errors

**Problem**: Widget tests timing out when using facade.

**Solution**: Ensure mock streams complete properly:

```dart
// Bad: Stream never completes
when(mockFacade.stateStream).thenAnswer(
  (_) => Stream.periodic(Duration(seconds: 1), (i) => FacadeState.initial()),
);

// Good: Stream provides values immediately
when(mockFacade.stateStream).thenAnswer(
  (_) => Stream.value(FacadeState.initial()),
);
```

---

## FAQ

### When should I use the facade vs. direct service access?

Use the facade for:
- Common operations (start/stop engine, load scripts, list devices)
- Operations that need coordination between multiple services
- Any UI interaction that needs error translation

Use direct service access for:
- Rare, service-specific operations not exposed through facade
- Operations that need fine-grained control over individual services
- Performance-critical paths where facade overhead matters (rare)

### Can I mix facade and direct service usage?

Yes! The facade coexists with `ServiceRegistry`. You can access underlying services through `facade.services`:

```dart
// Use facade for common operations
await facade.startEngine(scriptPath);

// Use direct service for advanced operations
final rawData = await facade.services.engineService.getRawInternalState();
```

### How do I add a new operation to the facade?

1. Add the method to `KeyrxFacade` abstract class
2. Implement it in `KeyrxFacadeImpl`
3. Update `MockKeyrxFacade` if needed
4. Add tests
5. Update this documentation

### Does the facade replace ServiceRegistry?

No. The facade wraps `ServiceRegistry` and coordinates its services. `ServiceRegistry` remains the foundation. The facade is an additional layer for convenience.

### What about performance?

The facade adds < 1ms overhead per call. For typical UI operations (button clicks, page loads), this is negligible. If you have a tight loop calling facade methods thousands of times per second, consider direct service access.

### How do I handle complex workflows?

Use the Result type's `andThen` method for chaining:

```dart
final result = await facade.validateScript(path)
  .then((r) => r.andThen((_) => facade.loadScriptContent(path)))
  .then((r) => r.andThen((_) => facade.startEngine(path)));

result.when(
  ok: (_) => print('Workflow complete'),
  err: (e) => print('Failed at: ${e.message}'),
);
```

---

## Related Documentation

- [Architecture Overview](./ARCHITECTURE.md) - Overall system architecture
- [FFI Architecture](./ffi-architecture.md) - Rust-Dart FFI bridge design
- [Service Registry](../ui/lib/services/service_registry.dart) - Service dependency management

---

## Changelog

### 2025-12-04 - Initial Release

- Created KeyrxFacade with unified API over 19+ services
- Implemented Result type for explicit error handling
- Added FacadeState for aggregated state observation
- Migrated 5 pages to use facade (EditorPage, DiscoveryPage, DebuggerPage, TradeOffVisualizerPage, KeyrxTrainingScreen)
- Achieved 90% test coverage
- Reduced test setup code by 60% through MockKeyrxFacade
