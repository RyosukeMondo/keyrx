# Ultra-Speed Dev Experience - Design

## Architecture Overview

This design implements a comprehensive development acceleration system for KeyRx Flutter/Dart development, bringing Dart tooling to parity with the existing excellent Rust infrastructure.

### Design Principles

1. **Parity with Rust**: Dart should have same quality gates as Rust
2. **Fail Fast**: Catch issues in pre-commit, not in CI
3. **Zero Configuration**: Tools should work out of the box
4. **Automation First**: Eliminate manual steps
5. **Measurable**: Track velocity improvements quantitatively

---

## Component 1: Enhanced Pre-Commit Hooks

### Current State
```bash
# .githooks/pre-commit (existing)
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test --lib
- FFI bindings sync check
```

### Design: Add Dart Checks

```bash
# .githooks/pre-commit (enhanced)
#!/bin/bash

echo "🦀 Rust Quality Gates..."
cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test --lib || exit 1

echo "🎯 Dart Quality Gates..."
cd ui

# 1. Format check
dart format --set-exit-if-changed . || exit 1

# 2. Analysis (linting)
flutter analyze --fatal-infos || exit 1

# 3. Run tests with coverage
flutter test --coverage --reporter=compact || exit 1

# 4. Coverage threshold check (80%)
./scripts/check_coverage.sh 80 || exit 1

# 5. Build verification (quick check)
flutter build bundle --debug --verbose || exit 1

echo "✅ All quality gates passed!"
```

### Implementation Details

**Script: `ui/scripts/check_coverage.sh`**
```bash
#!/bin/bash
THRESHOLD=$1
COVERAGE_FILE="coverage/lcov.info"

if [ ! -f "$COVERAGE_FILE" ]; then
  echo "❌ Coverage file not found"
  exit 1
fi

# Use lcov to calculate coverage percentage
COVERAGE=$(lcov --summary "$COVERAGE_FILE" 2>&1 | grep "lines" | awk '{print $2}' | sed 's/%//')

if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
  echo "❌ Coverage $COVERAGE% is below threshold $THRESHOLD%"
  exit 1
fi

echo "✅ Coverage: $COVERAGE% (threshold: $THRESHOLD%)"
```

**Benefits**:
- Catch issues locally before pushing
- Same quality bar as Rust (80% coverage)
- <30s feedback loop (fast tests only)

---

## Component 2: Comprehensive Analysis Options

### Current State
```yaml
# ui/analysis_options.yaml (current)
include: package:flutter_lints/flutter.yaml
# (minimal rules)
```

### Design: Strict Lint Rules

```yaml
# ui/analysis_options.yaml (enhanced)
include: package:flutter_lints/flutter.yaml

analyzer:
  exclude:
    - "**/*.g.dart"
    - "**/*.freezed.dart"
    - "**/generated_bindings.dart"

  strong-mode:
    implicit-casts: false
    implicit-dynamic: false

  errors:
    # Treat all lints as errors in CI
    missing_required_param: error
    missing_return: error
    todo: warning
    deprecated_member_use: warning

  language:
    strict-casts: true
    strict-inference: true
    strict-raw-types: true

linter:
  rules:
    # Error Prevention
    - avoid_empty_else
    - avoid_print
    - avoid_relative_lib_imports
    - avoid_returning_null_for_future
    - avoid_slow_async_io
    - avoid_types_as_parameter_names
    - cancel_subscriptions
    - close_sinks
    - no_adjacent_strings_in_list
    - test_types_in_equals
    - throw_in_finally
    - unnecessary_statements
    - unrelated_type_equality_checks
    - valid_regexps

    # Style
    - always_declare_return_types
    - always_put_required_named_parameters_first
    - annotate_overrides
    - avoid_bool_literals_in_conditional_expressions
    - avoid_catches_without_on_clauses
    - avoid_catching_errors
    - avoid_field_initializers_in_const_classes
    - avoid_function_literals_in_foreach_calls
    - avoid_init_to_null
    - avoid_null_checks_in_equality_operators
    - avoid_positional_boolean_parameters
    - avoid_private_typedef_functions
    - avoid_redundant_argument_values
    - avoid_renaming_method_parameters
    - avoid_return_types_on_setters
    - avoid_returning_null
    - avoid_returning_null_for_void
    - avoid_setters_without_getters
    - avoid_shadowing_type_parameters
    - avoid_single_cascade_in_expression_statements
    - avoid_unnecessary_containers
    - avoid_unused_constructor_parameters
    - avoid_void_async
    - await_only_futures
    - camel_case_extensions
    - camel_case_types
    - cascade_invocations
    - constant_identifier_names
    - curly_braces_in_flow_control_structures
    - directives_ordering
    - empty_catches
    - empty_constructor_bodies
    - file_names
    - flutter_style_todos
    - hash_and_equals
    - implementation_imports
    - iterable_contains_unrelated_type
    - join_return_with_assignment
    - leading_newlines_in_multiline_strings
    - library_names
    - library_prefixes
    - lines_longer_than_80_chars
    - list_remove_unrelated_type
    - literal_only_boolean_expressions
    - missing_whitespace_between_adjacent_strings
    - no_duplicate_case_values
    - no_logic_in_create_state
    - no_runtimeType_toString
    - non_constant_identifier_names
    - null_closures
    - omit_local_variable_types
    - one_member_abstracts
    - only_throw_errors
    - overridden_fields
    - package_api_docs
    - package_names
    - package_prefixed_library_names
    - parameter_assignments
    - prefer_adjacent_string_concatenation
    - prefer_asserts_in_initializer_lists
    - prefer_collection_literals
    - prefer_conditional_assignment
    - prefer_const_constructors
    - prefer_const_constructors_in_immutables
    - prefer_const_declarations
    - prefer_const_literals_to_create_immutables
    - prefer_constructors_over_static_methods
    - prefer_contains
    - prefer_equal_for_default_values
    - prefer_expression_function_bodies
    - prefer_final_fields
    - prefer_final_in_for_each
    - prefer_final_locals
    - prefer_for_elements_to_map_fromIterable
    - prefer_foreach
    - prefer_function_declarations_over_variables
    - prefer_generic_function_type_aliases
    - prefer_if_elements_to_conditional_expressions
    - prefer_if_null_operators
    - prefer_initializing_formals
    - prefer_inlined_adds
    - prefer_int_literals
    - prefer_interpolation_to_compose_strings
    - prefer_is_empty
    - prefer_is_not_empty
    - prefer_is_not_operator
    - prefer_iterable_whereType
    - prefer_mixin
    - prefer_null_aware_operators
    - prefer_relative_imports
    - prefer_single_quotes
    - prefer_spread_collections
    - prefer_typing_uninitialized_variables
    - prefer_void_to_null
    - provide_deprecation_message
    - recursive_getters
    - sized_box_for_whitespace
    - slash_for_doc_comments
    - sort_child_properties_last
    - sort_constructors_first
    - sort_pub_dependencies
    - sort_unnamed_constructors_first
    - type_annotate_public_apis
    - type_init_formals
    - unawaited_futures
    - unnecessary_await_in_return
    - unnecessary_brace_in_string_interps
    - unnecessary_const
    - unnecessary_getters_setters
    - unnecessary_lambdas
    - unnecessary_new
    - unnecessary_null_aware_assignments
    - unnecessary_null_in_if_null_operators
    - unnecessary_overrides
    - unnecessary_parenthesis
    - unnecessary_raw_strings
    - unnecessary_string_escapes
    - unnecessary_string_interpolations
    - unnecessary_this
    - unrelated_type_equality_checks
    - unsafe_html
    - use_full_hex_values_for_flutter_colors
    - use_function_type_syntax_for_parameters
    - use_key_in_widget_constructors
    - use_late_for_private_fields_and_variables
    - use_named_constants
    - use_raw_strings
    - use_rethrow_when_possible
    - use_setters_to_change_state
    - use_string_buffers
    - use_to_and_as_if_applicable
    - void_checks

    # Metrics (align with CLAUDE.md)
    # Note: Dart doesn't have built-in file/function size lints
    # These are enforced in CI via custom scripts
```

**Custom Metrics Script: `ui/scripts/check_metrics.sh`**
```bash
#!/bin/bash
# Check file size (max 500 lines excluding comments)
# Check function size (max 50 lines)

find lib -name "*.dart" ! -name "*.g.dart" ! -name "*.freezed.dart" -exec \
  dart run scripts/check_file_metrics.dart {} \;
```

---

## Component 3: Mock FFI Bridge

### Problem
UI development blocked on Rust compilation (30-60s rebuilds)

### Design: Stub Bridge Implementation

**File: `ui/lib/ffi/mock_bridge.dart`**
```dart
/// Mock FFI bridge for UI development without Rust compilation
class MockKeyrxBridge implements KeyrxBridge {
  // Configurable delays to simulate FFI latency
  final Duration ffiLatency;

  // In-memory state for testing
  final Map<String, dynamic> _state = {};

  MockKeyrxBridge({this.ffiLatency = const Duration(milliseconds: 10)});

  @override
  Future<List<DeviceState>> getDevices() async {
    await Future.delayed(ffiLatency);
    return [
      DeviceState(
        identity: DeviceIdentity(
          vendorId: 0x1234,
          productId: 0x5678,
          serialNumber: 'MOCK-001',
          userLabel: 'Mock Keyboard',
        ),
        remapEnabled: true,
        profileId: 'mock-profile',
        connectedAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      ),
    ];
  }

  @override
  Future<RuntimeConfig> getRuntimeConfig() async {
    await Future.delayed(ffiLatency);
    return RuntimeConfig(
      devices: [
        DeviceSlots(
          device: DeviceInstanceId(
            vendorId: 0x1234,
            productId: 0x5678,
            serial: 'MOCK-001',
          ),
          slots: [
            ProfileSlot(
              id: 'slot-1',
              hardwareProfileId: 'mock-profile',
              keymapId: 'mock-keymap',
              active: true,
              priority: 0,
            ),
          ],
        ),
      ],
    );
  }

  // ... implement all other bridge methods with mock data
}
```

**Usage in Development**:
```dart
// ui/lib/state/providers.dart
Provider<KeyrxBridge>(
  create: (_) {
    const bool useMockBridge = bool.fromEnvironment('MOCK_FFI', defaultValue: false);
    if (useMockBridge) {
      return MockKeyrxBridge();
    }
    return KeyrxBridge.real();
  },
);
```

**Run with mock**:
```bash
flutter run --dart-define=MOCK_FFI=true
# Now hot reload works for all UI changes!
```

---

## Component 4: Automated FFI Binding Regeneration

### Current Workflow (Manual)
1. Change Rust code
2. Run `just codegen` manually
3. Rebuild Flutter app
4. Test

### Design: Watch Mode Automation

**Script: `scripts/watch_ffi.sh`**
```bash
#!/bin/bash
# Watch Rust FFI files and auto-regenerate bindings

RUST_FFI_DIR="core/src/ffi"
DART_BINDINGS="ui/lib/ffi/generated_bindings.dart"

echo "👀 Watching FFI files for changes..."
echo "Rust: $RUST_FFI_DIR"
echo "Dart: $DART_BINDINGS"

# Use watchexec (install: cargo install watchexec-cli)
watchexec \
  --watch "$RUST_FFI_DIR" \
  --exts rs \
  --debounce 500ms \
  --clear \
  -- bash -c "
    echo '🔄 FFI changed, regenerating bindings...'
    just codegen || exit 1
    echo '✅ Bindings updated!'
  "
```

**Usage**:
```bash
# Terminal 1: Watch FFI and auto-regenerate
./scripts/watch_ffi.sh

# Terminal 2: Flutter with hot reload
cd ui && flutter run

# Now: Edit Rust FFI → Auto-regenerate → Hot reload Flutter!
```

---

## Component 5: State Management Standardization

### Decision: Use Riverpod

**Rationale**:
- Compile-time safety (no runtime Provider errors)
- Built-in testing support
- Better performance (granular rebuilds)
- Active maintenance
- Code generation support

### Architecture

```
┌─────────────────────────────────────────┐
│         UI Layer (Widgets)              │
│  - Pages, Widgets                       │
│  - Use ConsumerWidget/Consumer          │
└────────────┬────────────────────────────┘
             │ watch(provider)
┌────────────▼────────────────────────────┐
│      Provider Layer                     │
│  - StateNotifierProvider<T>             │
│  - FutureProvider<T>                    │
│  - StreamProvider<T>                    │
└────────────┬────────────────────────────┘
             │ inject
┌────────────▼────────────────────────────┐
│    Service/Repository Layer             │
│  - KeyFlowRepository                    │
│  - DeviceService                        │
│  - RuntimeService                       │
└────────────┬────────────────────────────┘
             │ FFI calls
┌────────────▼────────────────────────────┐
│         FFI Bridge                      │
│  - KeyrxBridge (real or mock)           │
└─────────────────────────────────────────┘
```

### Example: KeyFlow State

**File: `ui/lib/state/keyflow_provider.dart`**
```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import '../models/flow/flow_data.dart';
import '../services/keyflow_repository.dart';

part 'keyflow_provider.g.dart';

@riverpod
class KeyFlowState extends _$KeyFlowState {
  @override
  FutureOr<RuntimeConfig> build() async {
    // Fetch initial state from repository
    final repository = ref.watch(keyFlowRepositoryProvider);
    return repository.getRuntimeConfig();
  }

  Future<void> updateSlot(String deviceId, ProfileSlot slot) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() async {
      final repository = ref.read(keyFlowRepositoryProvider);
      await repository.updateSlot(deviceId, slot);
      return repository.getRuntimeConfig();
    });
  }
}

@riverpod
KeyFlowRepository keyFlowRepository(KeyFlowRepositoryRef ref) {
  final bridge = ref.watch(bridgeProvider);
  return KeyFlowRepository(bridge: bridge);
}
```

**Code Generation**:
```bash
# Auto-generate providers
dart run build_runner watch --delete-conflicting-outputs

# Or in pre-commit
dart run build_runner build --delete-conflicting-outputs
```

---

## Component 6: Widget Test Scaffolding

### Template: Widget Test

**File: `ui/test/helpers/widget_test_harness.dart`**
```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mockito/annotations.dart';
import '../mocks/mock_keyrx_bridge.dart';

/// Test harness for widget tests with common setup
class WidgetTestHarness {
  final List<Override> overrides;
  final MockKeyrxBridge mockBridge;

  WidgetTestHarness({
    List<Override>? overrides,
    MockKeyrxBridge? mockBridge,
  })  : mockBridge = mockBridge ?? MockKeyrxBridge(),
        overrides = overrides ?? [];

  /// Pump a widget with providers
  Future<void> pumpWidget(
    WidgetTester tester,
    Widget widget,
  ) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          bridgeProvider.overrideWithValue(mockBridge),
          ...overrides,
        ],
        child: MaterialApp(
          home: widget,
        ),
      ),
    );
  }

  /// Pump with Material app scaffold
  Future<void> pumpPage(
    WidgetTester tester,
    Widget page,
  ) async {
    await pumpWidget(
      tester,
      Scaffold(body: page),
    );
  }
}

/// Extension for common assertions
extension WidgetTesterX on WidgetTester {
  /// Find by key
  Finder byKey(String key) => find.byKey(Key(key));

  /// Tap and settle
  Future<void> tapAndSettle(Finder finder) async {
    await tap(finder);
    await pumpAndSettle();
  }

  /// Verify text exists
  void expectText(String text) {
    expect(find.text(text), findsOneWidget);
  }
}
```

**Usage Example**:
```dart
// ui/test/pages/flow_editor_page_test.dart
void main() {
  group('FlowEditorPage', () {
    late WidgetTestHarness harness;

    setUp(() {
      harness = WidgetTestHarness();
    });

    testWidgets('displays device nodes', (tester) async {
      // Setup mock data
      harness.mockBridge.setMockDevices([
        DeviceState(/* ... */),
      ]);

      // Pump widget
      await harness.pumpPage(tester, FlowEditorPage());

      // Assertions
      tester.expectText('Mock Keyboard');
      expect(find.byType(DeviceNodeWidget), findsOneWidget);
    });

    testWidgets('updates slot on drag', (tester) async {
      await harness.pumpPage(tester, FlowEditorPage());

      // Drag node
      await tester.drag(
        find.byKey('device-node-0'),
        const Offset(100, 100),
      );
      await tester.pumpAndSettle();

      // Verify state updated
      verify(harness.mockBridge.updateSlot(any, any)).called(1);
    });
  });
}
```

---

## Component 7: Golden Test Infrastructure

### Setup

**File: `ui/test/helpers/golden_test_harness.dart`**
```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

/// Golden test configuration
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  return GoldenToolkit.runWithConfiguration(
    () async {
      await loadAppFonts();
      await testMain();
    },
    config: GoldenToolkitConfiguration(
      defaultDevices: [
        Device.phone,
        Device.tabletPortrait,
        Device.desktop,
      ],
    ),
  );
}
```

**Example Golden Test**:
```dart
// ui/test/widgets/flow/device_node_widget_golden_test.dart
@Tags(['golden'])
void main() {
  group('DeviceNodeWidget Golden Tests', () {
    testGoldens('renders correctly', (tester) async {
      final builder = GoldenBuilder.grid(
        columns: 2,
        widthToHeightRatio: 1,
      )
        ..addScenario('Active Device', DeviceNodeWidget(
          data: DeviceNodeData(/* ... */),
        ))
        ..addScenario('Inactive Device', DeviceNodeWidget(
          data: DeviceNodeData(/* ... */),
        ));

      await tester.pumpWidgetBuilder(builder.build());
      await screenMatchesGolden(tester, 'device_node_widget');
    });
  });
}
```

**Run Golden Tests**:
```bash
# Update goldens
flutter test --update-goldens --tags golden

# Verify goldens
flutter test --tags golden
```

---

## Component 8: Structured Logging (Dart)

### Design: Match Rust JSON Logging

**File: `ui/lib/utils/logger.dart`**
```dart
import 'dart:convert';
import 'package:logging/logging.dart';

/// Structured logger matching Rust's JSON format
class StructuredLogger {
  final Logger _logger;
  final String service = 'keyrx-ui';

  StructuredLogger(String name) : _logger = Logger(name);

  void _log(Level level, String event, Map<String, dynamic> context) {
    final entry = {
      'timestamp': DateTime.now().toIso8601String(),
      'level': level.name.toUpperCase(),
      'service': service,
      'logger': _logger.name,
      'event': event,
      ...context,
    };

    _logger.log(level, jsonEncode(entry));
  }

  void debug(String event, [Map<String, dynamic>? context]) =>
      _log(Level.FINE, event, context ?? {});

  void info(String event, [Map<String, dynamic>? context]) =>
      _log(Level.INFO, event, context ?? {});

  void warn(String event, [Map<String, dynamic>? context]) =>
      _log(Level.WARNING, event, context ?? {});

  void error(String event, {Object? error, StackTrace? stack, Map<String, dynamic>? context}) {
    _log(Level.SEVERE, event, {
      ...?context,
      if (error != null) 'error': error.toString(),
      if (stack != null) 'stack_trace': stack.toString(),
    });
  }
}

/// Global logger factory
StructuredLogger getLogger(String name) => StructuredLogger(name);
```

**Usage**:
```dart
final log = getLogger('KeyFlowRepository');

log.info('runtime_config_fetched', {
  'device_count': config.devices.length,
  'duration_ms': stopwatch.elapsedMilliseconds,
});

log.error('ffi_call_failed',
  error: e,
  stack: stackTrace,
  context: {'operation': 'getRuntimeConfig'},
);
```

---

## Component 9: Performance Profiling Workflow

### Tools Integration

1. **Flutter DevTools**: Already available, document usage
2. **Timeline Tracing**: For FFI call profiling
3. **Memory Profiling**: For leak detection

**Guide: `docs/PROFILING.md`**
```markdown
# Performance Profiling Guide

## Flutter DevTools

### Setup
```bash
# Open DevTools
flutter pub global activate devtools
flutter pub global run devtools
```

### Profile Widget Build Performance
1. Run app in profile mode: `flutter run --profile`
2. Open DevTools → Performance tab
3. Record timeline during interaction
4. Look for long frames (>16ms)

### Profile Memory
1. DevTools → Memory tab
2. Take heap snapshot before/after operation
3. Look for retained objects

## FFI Call Profiling

### Add Tracing
```dart
// Wrap FFI calls with Timeline
await Timeline.timeSync('ffi.getRuntimeConfig', () async {
  return bridge.getRuntimeConfig();
});
```

### View Trace
- DevTools → Timeline tab
- Filter by "ffi." prefix
- Identify slow calls

## Automated Performance Tests

```dart
// test/performance/keyflow_perf_test.dart
void main() {
  testWidgets('FlowEditorPage renders in <16ms', (tester) async {
    await tester.pumpWidget(/* ... */);

    final binding = tester.binding;
    final renderTime = await binding.measureTimeAsync(() async {
      await tester.pumpWidget(FlowEditorPage());
      await tester.pumpAndSettle();
    });

    expect(renderTime.inMilliseconds, lessThan(16)); // 60fps
  });
}
```
```

---

## Component 10: Justfile Recipes for Dart Workflows

### Enhanced Justfile

```makefile
# Add to justfile

# Flutter/Dart workflows
ui-setup:
  cd ui && flutter pub get
  cd ui && dart run build_runner build

ui-test:
  cd ui && flutter test --coverage

ui-test-watch:
  cd ui && flutter test --watch

ui-coverage:
  cd ui && flutter test --coverage
  cd ui && genhtml coverage/lcov.info -o coverage/html
  @echo "Coverage report: ui/coverage/html/index.html"

ui-analyze:
  cd ui && flutter analyze --fatal-infos

ui-format:
  cd ui && dart format --set-exit-if-changed .

ui-metrics:
  cd ui && dart run scripts/check_file_metrics.dart

ui-golden-update:
  cd ui && flutter test --update-goldens --tags golden

ui-golden-verify:
  cd ui && flutter test --tags golden

ui-quality: ui-format ui-analyze ui-test ui-metrics
  @echo "✅ All Dart quality gates passed"

# Development workflows
dev-mock:
  cd ui && flutter run --dart-define=MOCK_FFI=true

dev-watch:
  ./scripts/watch_ffi.sh

# Full quality check (Rust + Dart)
quality: check ui-quality
  @echo "✅ All quality gates passed (Rust + Dart)"
```

---

## Implementation Phases

### Phase 1: Foundation (Priority: Critical)
1. ✅ Enhanced `analysis_options.yaml`
2. ✅ Pre-commit hooks for Dart
3. ✅ Coverage tracking (80% threshold)
4. ✅ Mock FFI bridge
5. ✅ Justfile recipes

**Deliverable**: Dart quality parity with Rust

### Phase 2: State & Testing (Priority: High)
6. ✅ Riverpod integration
7. ✅ Widget test harness
8. ✅ Golden test infrastructure
9. ✅ Structured logging (Dart)

**Deliverable**: Comprehensive testing infrastructure

### Phase 3: Optimization (Priority: Medium)
10. ✅ FFI watch mode automation
11. ✅ Performance profiling guide
12. ✅ File/function metrics checking
13. ✅ Code generation workflow

**Deliverable**: Sub-5s FFI rebuild cycle

### Phase 4: Documentation (Priority: Medium)
14. ✅ Create `design.md` steering doc
15. ✅ Update `tech.md` with Dart sections
16. ✅ Update `structure.md` to match actual layout
17. ✅ Create profiling guide

**Deliverable**: Complete developer documentation

---

## Success Metrics

### Velocity Improvements (Measurable)
| Metric | Before | Target | Measurement |
|--------|--------|--------|-------------|
| FFI change feedback | 60s | <10s | Time from Rust edit to Flutter reload |
| UI-only change | 3-5s | <1s | Hot reload time |
| New feature impl | 4-6h | <2h | Time to add new page + tests |
| FFI debugging | 2h | <30min | Time to diagnose FFI boundary issue |
| Test feedback | Manual | <30s | Pre-commit test suite runtime |

### Quality Metrics (Maintained)
- ✅ Dart coverage ≥80% (matching Rust)
- ✅ Zero lint warnings in CI
- ✅ Zero type errors
- ✅ Max 500 lines/file (excluding comments)
- ✅ Max 50 lines/function

### Developer Experience
- ✅ Single command setup (`just setup`)
- ✅ Clear error messages
- ✅ Comprehensive docs
- ✅ Fast feedback loops (<30s for most operations)

---

## Risk Mitigation

### Risk 1: Pre-commit Hooks Too Slow
**Mitigation**:
- Run only fast unit tests in pre-commit
- Run full test suite + golden tests in CI only
- Use parallel test execution (`flutter test --concurrency=4`)

### Risk 2: Coverage Threshold Blocks Commits
**Mitigation**:
- Allow `--no-verify` for WIP commits
- Show coverage diff (not absolute) in PR reviews
- Incrementally increase threshold (start at 60%, target 80%)

### Risk 3: Riverpod Migration Overhead
**Mitigation**:
- Migrate incrementally (new code uses Riverpod)
- Keep existing Provider code working
- Create migration guide with examples

### Risk 4: FFI Watch Mode Resource Usage
**Mitigation**:
- Make watch mode opt-in (not default)
- Add debouncing (500ms) to avoid excessive regeneration
- Provide kill switch if system becomes unresponsive

---

## References

- Flutter DevTools: https://docs.flutter.dev/tools/devtools
- Riverpod: https://riverpod.dev
- Golden Toolkit: https://pub.dev/packages/golden_toolkit
- CLAUDE.md: File size (500), function size (50), coverage (80%)
