# Design Document: advanced-remapping-engine

## Overview

The Advanced Remapping Engine implements a 4-layer architecture for timing-based keyboard customization. The core innovation is treating pending timing decisions as first-class citizens, enabling tap-hold, combos, and other behaviors while maintaining sub-millisecond latency.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Event Sourcing**: All decisions are derived from InputEvent stream
- **No Global State**: Engine state is encapsulated in structs
- **Dependency Injection**: TimingConfig is injectable, clock is mockable
- **< 1ms latency**: No blocking operations, no heap allocations in hot path

### Project Structure (structure.md)
- New module: `core/src/engine/state/` - state management
- New module: `core/src/engine/decision/` - timing decisions
- New module: `core/src/engine/action/` - action execution
- Extend: `core/src/engine/types.rs` - new types
- Extend: `core/src/scripting/runtime.rs` - new Rhai functions

## Code Reuse Analysis

### Existing Components to Leverage
- **`InputEvent`**: Already has `timestamp_us`, `is_repeat`, `is_synthetic`
- **`RemapAction`**: Extend with new action types
- **`Engine<I, S, St>`**: Add decision processing to event loop
- **`StateStore` trait**: Extend for layer/modifier management
- **`RhaiRuntime`**: Add new function registrations

### Integration Points
- **`process_event()`**: Insert decision checking before script lookup
- **`MockState`**: Extend for testing layer/modifier state
- **`InMemoryState`**: Production implementation of new state

## Architecture

### 4-Layer Engine Model

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: COMPOSED BEHAVIORS                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ TapHoldBehavior │ ComboBehavior │ OneShotBehavior        │   │
│  │ (resolves pending decisions into actions)                │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: ACTION PRIMITIVES                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ emit_key() │ block() │ modifier_set() │ layer_push()     │   │
│  │ (translate decisions to OutputAction)                    │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: DECISION PRIMITIVES                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ PendingDecision { key, started_at, deadline, config }    │   │
│  │ DecisionQueue (check timeouts, resolve on events)        │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: STATE MANAGEMENT                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ KeyStateTracker │ ModifierState │ LayerStack │ Timers    │   │
│  │ (track what's pressed, active, pending)                  │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 0: EVENT METADATA (from platform-drivers)                 │
│  InputEvent { key, pressed, timestamp_us, is_repeat, ... }      │
└─────────────────────────────────────────────────────────────────┘
```

### Event Processing Flow

```
InputEvent
    │
    ▼
┌───────────────────────┐
│ 1. Skip synthetic?    │──yes──▶ PassThrough
└───────────┬───────────┘
            │no
            ▼
┌───────────────────────┐
│ 2. Update KeyState    │ (track pressed keys)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ 3. Check Safe Mode    │──active──▶ PassThrough
└───────────┬───────────┘
            │inactive
            ▼
┌───────────────────────┐
│ 4. Check Combos       │──match──▶ Execute combo action
└───────────┬───────────┘
            │no match
            ▼
┌───────────────────────┐
│ 5. Check Pending      │──resolves──▶ Execute resolved action
│    Decisions          │
└───────────┬───────────┘
            │no resolution
            ▼
┌───────────────────────┐
│ 6. Create Pending?    │──tap-hold key──▶ Queue decision, Block
└───────────┬───────────┘
            │normal key
            ▼
┌───────────────────────┐
│ 7. Layer Lookup       │ (top to bottom)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ 8. Apply Modifiers    │ (one-shot consumption)
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ 9. Execute Action     │──▶ OutputAction
└───────────────────────┘
```

## Components and Interfaces

### Component 1: KeyStateTracker

**Purpose:** Track which physical keys are currently held
**File:** `core/src/engine/state/key_state.rs`

```rust
/// Tracks physical key press state with timestamps.
pub struct KeyStateTracker {
    /// Currently pressed keys with their press time
    pressed: HashMap<KeyCode, u64>,  // key -> timestamp_us
}

impl KeyStateTracker {
    pub fn new() -> Self;

    /// Record a key press. Returns false if already pressed (repeat).
    pub fn key_down(&mut self, key: KeyCode, timestamp_us: u64) -> bool;

    /// Record a key release. Returns hold duration in microseconds.
    pub fn key_up(&mut self, key: KeyCode, timestamp_us: u64) -> Option<u64>;

    /// Check if a key is currently pressed.
    pub fn is_pressed(&self, key: KeyCode) -> bool;

    /// Get press timestamp for a key.
    pub fn press_time(&self, key: KeyCode) -> Option<u64>;

    /// Get all currently pressed keys.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode>;

    /// Get count of pressed keys.
    pub fn pressed_count(&self) -> usize;
}
```

**Dependencies:** `KeyCode`
**Reuses:** None (new component)

### Component 2: ModifierState

**Purpose:** Track standard and virtual modifier state
**File:** `core/src/engine/state/modifiers.rs`

```rust
/// Bitmap for 256 virtual modifiers (0-255).
pub struct VirtualModifiers {
    bits: [u64; 4],  // 256 bits
}

/// Complete modifier state.
pub struct ModifierState {
    /// Standard OS modifiers
    pub standard: StandardModifiers,
    /// Custom virtual modifiers (Mod_Thumb, etc.)
    pub virtual_mods: VirtualModifiers,
    /// One-shot modifier states
    pub one_shot: OneShotState,
}

#[derive(Clone, Copy)]
pub struct StandardModifiers {
    pub left_shift: bool,
    pub right_shift: bool,
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_alt: bool,
    pub right_alt: bool,
    pub left_meta: bool,
    pub right_meta: bool,
}

/// One-shot modifier tracking.
pub struct OneShotState {
    /// Modifiers that are armed (will apply to next key)
    armed: VirtualModifiers,
    /// Modifiers that are locked (persist until disabled)
    locked: VirtualModifiers,
}

impl ModifierState {
    pub fn new() -> Self;
    pub fn activate(&mut self, modifier_id: u8);
    pub fn deactivate(&mut self, modifier_id: u8);
    pub fn is_active(&self, modifier_id: u8) -> bool;
    pub fn arm_one_shot(&mut self, modifier_id: u8);
    pub fn consume_one_shot(&mut self) -> VirtualModifiers;
}
```

**Dependencies:** None
**Reuses:** Pattern from `MockState`

### Component 3: LayerStack

**Purpose:** Manage keyboard layers with priority
**File:** `core/src/engine/state/layers.rs`

```rust
pub type LayerId = u16;

/// A keyboard layer with mappings.
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    /// Key mappings for this layer
    pub mappings: HashMap<KeyCode, LayerAction>,
    /// If true, unmapped keys fall through to lower layers
    pub transparent: bool,
}

/// Action within a layer (superset of RemapAction).
pub enum LayerAction {
    /// Simple remap
    Remap(KeyCode),
    /// Block the key
    Block,
    /// Tap-hold behavior
    TapHold { tap: KeyCode, hold: HoldAction },
    /// Layer control
    LayerPush(LayerId),
    LayerPop,
    LayerToggle(LayerId),
    /// Modifier control
    ModifierActivate(u8),
    ModifierDeactivate(u8),
    ModifierOneShot(u8),
    /// Pass through (explicit)
    Pass,
}

pub enum HoldAction {
    Key(KeyCode),
    Modifier(u8),
    Layer(LayerId),
}

/// Stack of active layers.
pub struct LayerStack {
    /// All defined layers
    layers: HashMap<LayerId, Layer>,
    /// Active layer IDs in priority order (last = highest)
    stack: Vec<LayerId>,
    /// Base layer ID (always at bottom)
    base: LayerId,
}

impl LayerStack {
    pub fn new() -> Self;
    pub fn define_layer(&mut self, layer: Layer);
    pub fn push(&mut self, layer_id: LayerId);
    pub fn pop(&mut self) -> Option<LayerId>;
    pub fn toggle(&mut self, layer_id: LayerId);
    pub fn is_active(&self, layer_id: LayerId) -> bool;

    /// Look up action for key, checking layers top to bottom.
    pub fn lookup(&self, key: KeyCode) -> Option<&LayerAction>;

    /// Get active layer names for debugging.
    pub fn active_layers(&self) -> Vec<&str>;
}
```

**Dependencies:** `KeyCode`, `LayerAction`
**Reuses:** Pattern from `RemapRegistry`

### Component 4: TimingConfig

**Purpose:** Configurable timing parameters
**File:** `core/src/engine/decision/timing.rs`

```rust
/// Timing configuration for decision-making.
#[derive(Clone, Debug)]
pub struct TimingConfig {
    /// Duration (ms) to distinguish tap from hold. Default: 200
    pub tap_timeout_ms: u32,

    /// Window (ms) for detecting simultaneous keypresses. Default: 50
    pub combo_timeout_ms: u32,

    /// Delay (ms) before considering a hold. Default: 0
    pub hold_delay_ms: u32,

    /// If true, emit tap immediately and correct if becomes hold. Default: false
    pub eager_tap: bool,

    /// If true, consider as hold if another key pressed during hold. Default: true
    pub permissive_hold: bool,

    /// If true, release tap even if interrupted. Default: false
    pub retro_tap: bool,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            tap_timeout_ms: 200,
            combo_timeout_ms: 50,
            hold_delay_ms: 0,
            eager_tap: false,
            permissive_hold: true,
            retro_tap: false,
        }
    }
}
```

**Dependencies:** None
**Reuses:** None (matches tech.md spec exactly)

### Component 5: PendingDecision & DecisionQueue

**Purpose:** Track and resolve timing-based decisions
**File:** `core/src/engine/decision/pending.rs`

```rust
/// A pending timing decision waiting for resolution.
#[derive(Debug, Clone)]
pub enum PendingDecision {
    TapHold {
        key: KeyCode,
        pressed_at: u64,
        deadline: u64,
        tap_action: KeyCode,
        hold_action: HoldAction,
        /// Has another key been pressed? (for permissive_hold)
        interrupted: bool,
    },
    Combo {
        keys: SmallVec<[KeyCode; 4]>,
        started_at: u64,
        deadline: u64,
        action: LayerAction,
        /// Keys pressed so far
        matched: SmallVec<[KeyCode; 4]>,
    },
}

/// Resolution of a pending decision.
pub enum DecisionResolution {
    /// Decision resolved as tap
    Tap(KeyCode),
    /// Decision resolved as hold
    Hold(HoldAction),
    /// Combo triggered
    ComboTriggered(LayerAction),
    /// Combo timed out, release original keys
    ComboTimeout(SmallVec<[KeyCode; 4]>),
    /// Not yet resolved
    Pending,
}

/// Queue of pending decisions.
pub struct DecisionQueue {
    pending: Vec<PendingDecision>,
    config: TimingConfig,
}

impl DecisionQueue {
    pub fn new(config: TimingConfig) -> Self;

    /// Add a new tap-hold pending decision.
    pub fn add_tap_hold(
        &mut self,
        key: KeyCode,
        pressed_at: u64,
        tap: KeyCode,
        hold: HoldAction,
    );

    /// Check for resolution based on new event.
    pub fn check_event(&mut self, event: &InputEvent) -> Vec<DecisionResolution>;

    /// Check for timeout-based resolutions.
    pub fn check_timeouts(&mut self, now_us: u64) -> Vec<DecisionResolution>;

    /// Mark a pending decision as interrupted.
    pub fn mark_interrupted(&mut self, by_key: KeyCode);

    /// Get all pending decisions for debugging.
    pub fn pending(&self) -> &[PendingDecision];

    /// Clear all pending decisions.
    pub fn clear(&mut self);
}
```

**Dependencies:** `KeyCode`, `TimingConfig`, `HoldAction`
**Reuses:** None (core new functionality)

### Component 6: ComboRegistry

**Purpose:** Store and match combo definitions
**File:** `core/src/engine/decision/combos.rs`

```rust
/// A combo definition.
pub struct ComboDef {
    /// Keys that must be pressed simultaneously
    pub keys: SmallVec<[KeyCode; 4]>,
    /// Action to execute when combo triggers
    pub action: LayerAction,
}

/// Registry of defined combos.
pub struct ComboRegistry {
    combos: Vec<ComboDef>,
}

impl ComboRegistry {
    pub fn new() -> Self;

    /// Register a new combo.
    pub fn register(&mut self, keys: &[KeyCode], action: LayerAction);

    /// Find combos that could match given pressed keys.
    pub fn find_potential(&self, pressed: &[KeyCode]) -> Vec<&ComboDef>;

    /// Check if pressed keys exactly match any combo.
    pub fn find_exact(&self, pressed: &[KeyCode]) -> Option<&ComboDef>;
}
```

**Dependencies:** `KeyCode`, `LayerAction`

### Component 7: AdvancedEngine (Extended Engine)

**Purpose:** Orchestrate all components
**File:** `core/src/engine/advanced.rs`

```rust
/// Extended engine with timing-based decisions.
pub struct AdvancedEngine<I, S>
where
    I: InputSource,
    S: ScriptRuntime,
{
    input: I,
    script: S,

    // State (Layer 1)
    key_state: KeyStateTracker,
    modifiers: ModifierState,
    layers: LayerStack,

    // Decisions (Layer 2)
    pending: DecisionQueue,
    combos: ComboRegistry,

    // Config
    timing: TimingConfig,
    safe_mode: bool,
    running: bool,
}

impl<I, S> AdvancedEngine<I, S> {
    pub fn new(input: I, script: S, timing: TimingConfig) -> Self;

    /// Process a single event through all layers.
    pub fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction>;

    /// Check for timeout-based resolutions (call periodically).
    pub fn tick(&mut self, now_us: u64) -> Vec<OutputAction>;

    // State access for GUI/debugging
    pub fn key_state(&self) -> &KeyStateTracker;
    pub fn modifiers(&self) -> &ModifierState;
    pub fn layers(&self) -> &LayerStack;
    pub fn pending(&self) -> &[PendingDecision];
    pub fn timing_config(&self) -> &TimingConfig;
}
```

## Data Models

### TimingConfig (serializable)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingConfig {
    pub tap_timeout_ms: u32,
    pub combo_timeout_ms: u32,
    pub hold_delay_ms: u32,
    pub eager_tap: bool,
    pub permissive_hold: bool,
    pub retro_tap: bool,
}
```

### EngineState (for GUI/FFI)

```rust
#[derive(Debug, Serialize)]
pub struct EngineState {
    pub pending_decisions: Vec<PendingDecisionInfo>,
    pub active_layers: Vec<String>,
    pub active_modifiers: Vec<u8>,
    pub pressed_keys: Vec<KeyCode>,
    pub safe_mode: bool,
    pub timing_config: TimingConfig,
}

#[derive(Debug, Serialize)]
pub struct PendingDecisionInfo {
    pub key: KeyCode,
    pub decision_type: String,  // "tap_hold", "combo"
    pub elapsed_ms: u32,
    pub deadline_ms: u32,
}
```

## Error Handling

### Error Scenarios

1. **Layer not found**
   - **Handling:** Return error from `layer_push()`, log warning
   - **User Impact:** Script error with layer name

2. **Combo conflict (overlapping key sets)**
   - **Handling:** Log warning, first-registered wins
   - **User Impact:** Warning in debug output

3. **Timer overflow**
   - **Handling:** Use wrapping arithmetic, resolve all pending as timeout
   - **User Impact:** None (transparent recovery)

4. **Too many pending decisions (DoS protection)**
   - **Handling:** Cap at 32, oldest dropped
   - **User Impact:** Warning logged, oldest decisions resolved

## Testing Strategy

### Unit Testing

**KeyStateTracker:**
- Press/release tracking
- Duplicate press handling
- Hold duration calculation

**ModifierState:**
- Virtual modifier activate/deactivate
- One-shot arm/consume/lock cycle
- Standard modifier tracking

**LayerStack:**
- Push/pop/toggle
- Transparent layer fallthrough
- Lookup priority order

**DecisionQueue:**
- Tap resolution (release before timeout)
- Hold resolution (timeout expires)
- Interrupt handling (permissive_hold)
- Eager tap + correction

**ComboRegistry:**
- Exact match
- Partial match
- Order-independent matching

### Integration Testing

```rust
#[tokio::test]
async fn tap_hold_tap_scenario() {
    let mut engine = test_engine();
    engine.register_tap_hold(KeyCode::CapsLock, KeyCode::Escape, HoldAction::Modifier(CTRL_ID));

    // Press and quick release = tap
    let actions = engine.process_event(key_down(KeyCode::CapsLock, 0));
    assert!(actions.is_empty()); // Pending

    let actions = engine.process_event(key_up(KeyCode::CapsLock, 100_000)); // 100ms
    assert_eq!(actions, vec![
        OutputAction::KeyDown(KeyCode::Escape),
        OutputAction::KeyUp(KeyCode::Escape),
    ]);
}

#[tokio::test]
async fn tap_hold_hold_scenario() {
    let mut engine = test_engine();
    engine.register_tap_hold(KeyCode::CapsLock, KeyCode::Escape, HoldAction::Modifier(CTRL_ID));

    // Press and hold past timeout = hold
    let _ = engine.process_event(key_down(KeyCode::CapsLock, 0));
    let actions = engine.tick(250_000); // 250ms > 200ms timeout

    assert!(engine.modifiers().is_active(CTRL_ID));
}
```

### Benchmark Testing

```rust
#[bench]
fn bench_process_event_with_pending(b: &mut Bencher) {
    let mut engine = test_engine();
    engine.add_pending_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));

    b.iter(|| {
        engine.process_event(key_down(KeyCode::X, 100_000))
    });
}
// Target: < 1 microsecond per event
```
