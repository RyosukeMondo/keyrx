# Design Document

## Overview

This design addresses critical code quality issues across both Rust core and Flutter UI. The refactoring follows a systematic approach: first eliminate testability blockers, then consolidate state, then reduce complexity, and finally eliminate duplication.

## Steering Document Alignment

### Technical Standards (CLAUDE.md)

- **Max 500 lines/file**: Split 4 oversized Rust files, 3 oversized Flutter pages
- **Max 50 lines/function**: Refactor `process_event_traced()` (98 lines)
- **SOLID/DI mandatory**: Add trait-based injection to AdvancedEngine
- **SSOT**: Consolidate duplicate state in Flutter AppState
- **No testability blockers**: Remove globals, enable parallel tests

## Architecture

### Refactoring Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Phase 1: Testability                                  │
│  Remove globals → Enable DI → Enable parallel tests                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Phase 2: SSOT                                         │
│  Consolidate state → Single MappingRepository → Unified EngineState         │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Phase 3: Complexity                                   │
│  Split functions → Split files → Extract services                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Phase 4: DRY                                          │
│  Extract helpers → Create mixins → Unify patterns                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Component 1: Runtime Context (Rust)

**Purpose:** Replace global `RUNTIME_SLOT` with injectable context

**Current (Bad):**
```rust
// Global static with unsafe
static RUNTIME_SLOT: RuntimeSlot = RuntimeSlot { ... };
```

**New Design:**
```rust
// core/src/scripting/context.rs (new)
pub struct RuntimeContext {
    engine: Engine,
    pending_ops: Vec<PendingOp>,
    registry: Registry,
}

impl RuntimeContext {
    pub fn new() -> Self { ... }
    pub fn with_registry(registry: Registry) -> Self { ... }
}

// Passed to functions that need it
pub fn process_script(ctx: &mut RuntimeContext, script: &str) -> Result<()>
```

**Files:**
- `core/src/scripting/context.rs` (new)
- `core/src/scripting/runtime.rs` (modify - remove global)
- `core/src/scripting/mod.rs` (modify - export context)

### Component 2: Bypass State Injection (Rust)

**Purpose:** Make bypass mode testable without global state

**Current (Bad):**
```rust
static BYPASS_MODE: AtomicBool = AtomicBool::new(false);
```

**New Design:**
```rust
// core/src/drivers/bypass.rs (new)
pub struct BypassController {
    active: AtomicBool,
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl BypassController {
    pub fn new() -> Self { ... }
    pub fn activate(&self) { ... }
    pub fn is_active(&self) -> bool { ... }
}

// Injected into engine
pub struct Engine<B: BypassControl> {
    bypass: B,
    // ...
}
```

**Files:**
- `core/src/drivers/bypass.rs` (new)
- `core/src/drivers/emergency_exit.rs` (modify)
- `core/src/engine/mod.rs` (modify - accept bypass)

### Component 3: Engine Dependency Traits (Rust)

**Purpose:** Enable injection of all engine dependencies

**New Design:**
```rust
// core/src/traits/state.rs (new)
pub trait KeyStateProvider {
    fn is_pressed(&self, key: KeyCode) -> bool;
    fn press(&mut self, key: KeyCode);
    fn release(&mut self, key: KeyCode);
}

pub trait ModifierProvider {
    fn is_active(&self, modifier: Modifier) -> bool;
    fn activate(&mut self, modifier: Modifier);
    fn deactivate(&mut self, modifier: Modifier);
}

pub trait LayerProvider {
    fn active_layer(&self) -> &str;
    fn push(&mut self, layer: &str);
    fn pop(&mut self);
}

// Engine accepts trait objects
pub struct AdvancedEngine<S, K, M, L>
where
    S: ScriptRuntime,
    K: KeyStateProvider,
    M: ModifierProvider,
    L: LayerProvider,
{
    script: S,
    key_state: K,
    modifiers: M,
    layers: L,
    // ...
}
```

**Files:**
- `core/src/traits/state.rs` (new)
- `core/src/traits/mod.rs` (modify)
- `core/src/engine/advanced.rs` (modify)

### Component 4: Split process_event_traced (Rust)

**Purpose:** Reduce 98-line function to ≤50 lines each

**Current Structure:**
```rust
fn process_event_traced() {
    // 1. Input validation (10 lines)
    // 2. Safe mode check (10 lines)
    // 3. Key state update (10 lines)
    // 4. Combo handling (15 lines)
    // 5. Decision resolution (15 lines)
    // 6. Layer lookup (15 lines)
    // 7. Output composition (15 lines)
    // 8. Tracing (8 lines)
}
```

**New Design:**
```rust
// core/src/engine/processing.rs (new)
fn process_event_traced(&mut self, event: KeyEvent) -> ProcessResult {
    self.validate_and_check_safe_mode(&event)?;
    self.update_key_state(&event);

    let decision = self.resolve_decision(&event);
    let outputs = self.apply_decision(decision);

    self.trace_event(&event, &outputs);
    Ok(outputs)
}

fn validate_and_check_safe_mode(&self, event: &KeyEvent) -> Result<()> { ... }
fn update_key_state(&mut self, event: &KeyEvent) { ... }
fn resolve_decision(&mut self, event: &KeyEvent) -> Decision { ... }
fn apply_decision(&mut self, decision: Decision) -> Vec<Output> { ... }
fn trace_event(&self, event: &KeyEvent, outputs: &[Output]) { ... }
```

**Files:**
- `core/src/engine/processing.rs` (new)
- `core/src/engine/advanced.rs` (modify - delegate to processing)

### Component 5: Split Oversized Rust Files

**Purpose:** Reduce files to ≤500 lines

| Current File | Lines | Split Into |
|--------------|-------|------------|
| `run.rs` (713) | 713 | `run.rs` (200), `run_builder.rs` (200), `run_recorder.rs` (150), `run_tracer.rs` (150) |
| `discover.rs` (712) | 712 | `discover.rs` (200), `discover_session.rs` (250), `discover_validation.rs` (200) |
| `runtime.rs` (683) | 683 | `runtime.rs` (250), `pending_ops.rs` (200), `registry_sync.rs` (200) |
| `exports.rs` (635) | 635 | `exports.rs` (200), `exports_device.rs` (150), `exports_session.rs` (150), `exports_engine.rs` (150) |

### Component 6: Flutter Service Injection Pattern

**Purpose:** Enable constructor-based dependency injection

**Current (Bad):**
```dart
class EditorPage extends StatefulWidget {
  @override
  _EditorPageState createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  EngineService? _engine;  // Nullable!

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _engine = Provider.of<ServiceRegistry>(context).engineService;
    });
  }
}
```

**New Design:**
```dart
// ui/lib/pages/editor_page.dart
class EditorPage extends StatefulWidget {
  final EngineService engine;  // Required!
  final MappingRepository mappings;

  const EditorPage({
    required this.engine,
    required this.mappings,
    super.key,
  });

  @override
  _EditorPageState createState() => _EditorPageState();
}

// In main.dart or router
EditorPage(
  engine: registry.engineService,
  mappings: registry.mappingRepository,
)
```

**Files:**
- `ui/lib/pages/editor_page.dart` (modify)
- `ui/lib/pages/console.dart` (modify)
- `ui/lib/pages/debugger_page.dart` (modify)
- `ui/lib/pages/training_screen.dart` (modify)
- `ui/lib/main.dart` (modify - pass dependencies)

### Component 7: MappingRepository (Flutter)

**Purpose:** Single source of truth for key mappings

**New Design:**
```dart
// ui/lib/repositories/mapping_repository.dart (new)
class MappingRepository extends ChangeNotifier {
  final Map<String, KeyMapping> _mappings = {};
  final List<ComboMapping> _combos = [];
  final List<TapHoldConfig> _tapHolds = [];

  // Single source of truth
  UnmodifiableMapView<String, KeyMapping> get mappings =>
      UnmodifiableMapView(_mappings);

  void addMapping(String key, KeyMapping mapping) {
    _mappings[key] = mapping;
    notifyListeners();
  }

  void removeMapping(String key) {
    _mappings.remove(key);
    notifyListeners();
  }

  String generateScript() {
    return ScriptGenerator.build(_mappings, _combos, _tapHolds);
  }
}
```

**Files:**
- `ui/lib/repositories/mapping_repository.dart` (new)
- `ui/lib/repositories/mod.dart` (new - barrel export)
- `ui/lib/services/service_registry.dart` (modify - add repository)

### Component 8: Consolidate Layer State (Flutter)

**Purpose:** Remove duplicate layer state from EditorPage

**Current (Bad):**
```dart
// editor_page.dart
List<LayerInfo> _layers = [LayerInfo(name: 'base', ...)];

// app_state.dart
List<LayerInfo> _layers = [];
```

**New Design:**
```dart
// app_state.dart - ONLY source
class AppState extends ChangeNotifier {
  List<LayerInfo> _layers = [LayerInfo(name: 'base', active: true, priority: 0)];

  UnmodifiableListView<LayerInfo> get layers => UnmodifiableListView(_layers);

  void addLayer(LayerInfo layer) { ... }
  void removeLayer(String name) { ... }
  void setLayerActive(String name, bool active) { ... }
}

// editor_page.dart - reads from AppState
class _EditorPageState extends State<EditorPage> {
  // NO local _layers field!

  @override
  Widget build(BuildContext context) {
    final appState = context.watch<AppState>();
    final layers = appState.layers;  // Read from single source
    // ...
  }
}
```

**Files:**
- `ui/lib/state/app_state.dart` (modify)
- `ui/lib/pages/editor_page.dart` (modify - remove local layers)
- `ui/lib/pages/visual_editor_page.dart` (modify - use AppState)

### Component 9: Extract Business Logic from Pages (Flutter)

**Purpose:** Move validation/parsing to dedicated services

**New Services:**
```dart
// ui/lib/services/mapping_validator.dart (new)
class MappingValidator {
  final KeyRegistry keyRegistry;

  MappingValidator(this.keyRegistry);

  ValidationResult validate(String fromKey, KeyMapping mapping) {
    if (!keyRegistry.isKnownKey(fromKey)) {
      return ValidationResult.error('Unknown key: $fromKey');
    }
    // ... more validation
    return ValidationResult.success();
  }
}

// ui/lib/services/console_parser.dart (new)
class ConsoleParser {
  ConsoleEntryType classify(String text) {
    final lower = text.toLowerCase();
    if (lower.startsWith('error:')) return ConsoleEntryType.error;
    if (lower.startsWith('ok:')) return ConsoleEntryType.success;
    return ConsoleEntryType.output;
  }

  bool needsInitButton(String text) {
    return text.toLowerCase().contains('not initialized');
  }
}

// ui/lib/services/script_file_service.dart (new)
class ScriptFileService {
  Future<void> saveScript(String path, String content) async {
    final file = File(path);
    await file.parent.create(recursive: true);
    await file.writeAsString(content);
  }

  Future<String> loadScript(String path) async {
    return File(path).readAsString();
  }
}
```

**Files:**
- `ui/lib/services/mapping_validator.dart` (new)
- `ui/lib/services/console_parser.dart` (new)
- `ui/lib/services/script_file_service.dart` (new)

### Component 10: Stream Subscription Mixin (Flutter)

**Purpose:** Extract repeated stream subscription pattern

**New Design:**
```dart
// ui/lib/mixins/stream_subscriber.dart (new)
mixin StreamSubscriber<T extends StatefulWidget> on State<T> {
  final List<StreamSubscription> _subscriptions = [];

  void subscribe<S>(Stream<S> stream, void Function(S) onData, {
    void Function(Object)? onError,
  }) {
    final sub = stream.listen(
      (data) {
        if (!mounted) return;
        onData(data);
      },
      onError: onError ?? (e) => debugPrint('Stream error: $e'),
    );
    _subscriptions.add(sub);
  }

  @override
  void dispose() {
    for (final sub in _subscriptions) {
      sub.cancel();
    }
    super.dispose();
  }
}

// Usage in pages:
class _DebuggerPageState extends State<DebuggerPage>
    with StreamSubscriber {

  @override
  void initState() {
    super.initState();
    subscribe(widget.engine.stateStream, _handleSnapshot);
  }

  void _handleSnapshot(EngineSnapshot snapshot) {
    setState(() => _recent.insert(0, snapshot));
  }
}
```

**Files:**
- `ui/lib/mixins/stream_subscriber.dart` (new)
- `ui/lib/mixins/mod.dart` (new)

### Component 11: Unify Layer Action Handlers (Rust)

**Purpose:** Remove 95% duplicate code

**Current (Bad):**
```rust
// Two nearly identical functions
pub fn handle_layer_action(..., event, action) -> LayerActionResult { ... }
pub fn execute_layer_action(..., action) -> Vec<OutputAction> { ... }
```

**New Design:**
```rust
// core/src/engine/layer_actions.rs (new)
pub fn apply_layer_action(
    action: &LayerAction,
    modifiers: &mut ModifierState,
    layers: &mut LayerStack,
    event: Option<&KeyEvent>,  // Optional context
) -> LayerActionResult {
    match action {
        LayerAction::Push(name) => { ... }
        LayerAction::Pop => { ... }
        LayerAction::Toggle(name) => { ... }
        // ... unified handling
    }
}

// Wrappers for backward compatibility
pub fn handle_layer_action(...) -> LayerActionResult {
    apply_layer_action(action, modifiers, layers, Some(event))
}

pub fn execute_layer_action(...) -> Vec<OutputAction> {
    apply_layer_action(action, modifiers, layers, None).into_outputs()
}
```

**Files:**
- `core/src/engine/layer_actions.rs` (new)
- `core/src/engine/decision_engine.rs` (modify - use unified function)

## Data Models

No new data models required - this refactoring reorganizes existing code.

## Error Handling

- All refactored functions maintain existing error handling behavior
- New extracted functions propagate errors via `Result<T, E>`
- Flutter services use typed exceptions

## Testing Strategy

### Unit Testing
- Each extracted Rust function gets dedicated unit tests
- Each new Flutter service gets widget tests with mocks
- Dependency injection enables isolated testing

### Integration Testing
- Existing integration tests must continue passing
- New parallel test execution validates removal of globals

### Regression Testing
- All CLI commands tested before/after
- All UI flows tested before/after
- Benchmark comparison for performance regression

## Implementation Sequence

1. **Rust Testability** (Tasks 1-4) - Remove globals, add DI
2. **Rust Complexity** (Tasks 5-8) - Split functions, split files
3. **Rust DRY** (Tasks 9-10) - Unify duplicates
4. **Flutter Testability** (Tasks 11-14) - Constructor injection
5. **Flutter SSOT** (Tasks 15-17) - Consolidate state
6. **Flutter Complexity** (Tasks 18-21) - Extract services
7. **Flutter DRY** (Tasks 22-23) - Create mixins
8. **Verification** (Tasks 24-25) - Tests, coverage
