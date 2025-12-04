# State Audit - KeyRX Codebase

## Executive Summary

This document catalogs all state types in the KeyRX codebase, identifies their locations, purposes, and analyzes overlaps and duplicates. The audit reveals significant state management complexity with opportunities for consolidation.

**Key Findings:**
- 40+ distinct state-related types across the codebase
- 2 duplicate EngineState definitions (canonical and legacy)
- Overlapping session state between recording and replay
- Mixed state management patterns (mutable refs, mutation API, direct access)
- State scattered across engine, FFI, CLI, and driver modules

## 1. Core Engine State Types

### 1.1 Unified State System (Primary)

#### EngineState
**Location:** `core/src/engine/state/mod.rs:133`
**Purpose:** Canonical unified container for all engine state
**Components:**
- KeyState: Physical key press tracking
- LayerState: Active layer stack management
- ModifierState: Standard and virtual modifier tracking
- PendingState: Tap-hold and combo decision queue
- Version counter for change tracking

**Lifecycle:** Created at engine initialization, mutated via `apply()` and `apply_batch()` methods
**Usage Pattern:** Mutation API with full transactional semantics and rollback
**Status:** ✅ Active - Primary state container

#### KeyState
**Location:** `core/src/engine/state/keys.rs:19`
**Purpose:** Track currently pressed physical keys with timestamps
**Data:** `HashMap<KeyCode, u64>` mapping keys to press timestamps
**Capacity:** Default 256 keys pre-allocated
**Lifecycle:** Owned by EngineState
**Status:** ✅ Active - Component of unified state

#### LayerState
**Location:** `core/src/engine/state/layers.rs:114`
**Purpose:** Track active layer stack with priority ordering
**Data:** Stack of layer IDs, base layer, active status
**Lifecycle:** Owned by EngineState
**Status:** ✅ Active - Component of unified state

#### ModifierState
**Location:** `core/src/engine/state/modifiers.rs:147`
**Purpose:** Track standard OS modifiers, virtual modifiers, and one-shot states
**Components:**
- StandardModifiers: Shift/Ctrl/Alt/Meta bitset (u8)
- VirtualModifiers: 256-bit bitmap for custom modifiers
- OneShotState: Sticky modifier tracking
**Lifecycle:** Owned by EngineState
**Status:** ✅ Active - Component of unified state

#### PendingState
**Location:** `core/src/engine/state/pending.rs:17`
**Purpose:** Wrapper around DecisionQueue for unified state API
**Data:** DecisionQueue with timing config
**Max Capacity:** DecisionQueue::MAX_PENDING
**Lifecycle:** Owned by EngineState
**Status:** ✅ Active - Component of unified state

### 1.2 Supporting State Types

#### StateChange
**Location:** `core/src/engine/state/change.rs:24`
**Purpose:** Event record for state mutations with effects
**Data:** Mutation, version, timestamp, effect list
**Lifecycle:** Returned by `EngineState::apply()` operations
**Status:** ✅ Active - Change tracking

#### StateSnapshot
**Location:** `core/src/engine/state/snapshot.rs:41`
**Purpose:** Serializable point-in-time state capture
**Usage:** Debugging, GUI inspection, test verification
**Status:** ✅ Active - Observation

#### StateHistory
**Location:** `core/src/engine/state/history.rs:73`
**Purpose:** Ring buffer of historical state snapshots
**Configuration:** Configurable depth and interval
**Status:** ✅ Active - Debugging/telemetry

#### PersistedState
**Location:** `core/src/engine/state/persistence.rs:51`
**Purpose:** Serializable subset for save/restore
**Status:** ✅ Active - Persistence

## 2. Legacy/Duplicate State Types

### 2.1 ⚠️ DUPLICATE: EngineStateSnapshot (advanced.rs)

**Location:** `core/src/engine/advanced.rs:61`
**Purpose:** Legacy serializable snapshot format
**Status:** 🔴 DEPRECATED - Marked for removal
**Issue:** Duplicates functionality of `StateSnapshot`
**Migration Path:** Use `EngineState::state_snapshot()` instead of `AdvancedEngine::snapshot()`

**Overlap Analysis:**
```
EngineStateSnapshot (legacy)     StateSnapshot (unified)
├── pressed_keys: Vec<...>  →   ├── keys: KeyState
├── modifiers: ModifierState →   ├── modifiers: ModifierState
├── layers: LayerStack       →   ├── layers: LayerState
├── pending: Vec<...>        →   ├── pending: PendingState
├── timing: TimingConfig     →   (moved to engine config)
└── safe_mode: bool          →   (moved to engine config)
```

### 2.2 KeyStateView Adapter

**Location:** `core/src/engine/advanced.rs:24`
**Purpose:** Read-only view adapter for KeyStateProvider trait
**Status:** ⚠️ COMPATIBILITY SHIM - Temporary during migration
**Issue:** Provides trait compatibility during gradual migration
**Removal Timeline:** After all code uses unified state directly

## 3. Decision/Pending State

### 3.1 Decision Queue System

#### DecisionQueue
**Location:** `core/src/engine/decision/pending.rs` (inferred)
**Purpose:** Queue of pending tap-hold and combo decisions
**Max Capacity:** Const MAX_PENDING
**Lifecycle:** Owned by AdvancedEngine (legacy) or wrapped by PendingState
**Status:** ✅ Active

#### PendingDecisionState
**Location:** `core/src/engine/decision/pending.rs:15`
**Purpose:** Enum representing pending decision variants
**Variants:**
- TapHold { key, pressed_at, tap, hold }
- Combo { keys, started_at, action }
**Status:** ✅ Active

#### DecisionResolution
**Location:** Referenced in code, exact location TBD
**Purpose:** Result of resolving a pending decision
**Variants:** Tap, Hold, Timeout, Cancelled
**Status:** ✅ Active

## 4. Session/Recording State

### 4.1 ⚠️ OVERLAPPING: RecordingState vs ReplayState

#### RecordingState
**Location:** `core/src/ffi/domains/recording.rs:19`
**Purpose:** FFI domain state for recording sessions
**Data:** Session metadata, recording status
**Lifecycle:** FFI domain-owned
**Status:** ✅ Active

#### ReplayState (enum)
**Location:** `core/src/engine/replay.rs:47`
**Purpose:** Replay lifecycle tracking
**Variants:** Idle, Playing, Paused, Completed
**Lifecycle:** Owned by ReplaySession
**Status:** ✅ Active

**Common Pattern Identified:**
Both recording and replay track:
- Session metadata (start time, duration)
- Playback/recording state (active, paused, stopped)
- Event buffering

**Consolidation Opportunity:**
Extract common `SessionState` base:
```rust
pub struct SessionState {
    start_time: Option<Instant>,
    metadata: SessionMetadata,
    state: SessionLifecycle, // Idle/Active/Paused/Complete
}
```

Then compose into:
- RecordingSession { base: SessionState, writer: EventWriter }
- ReplaySession { base: SessionState, reader: EventReader }

## 5. FFI Domain States

### 5.1 FFI-Specific States

#### DeviceState
**Location:** `core/src/ffi/domains/device.rs:23`
**Purpose:** FFI domain for device configuration
**Ownership:** FFI domain system
**Status:** ✅ Active - FFI layer

#### DiscoverySessionState
**Location:** `core/src/ffi/domains/discovery.rs:24`
**Purpose:** Device discovery session tracking
**Ownership:** FFI domain system
**Status:** ✅ Active - FFI layer

#### StateEvent
**Location:** `core/src/ffi/domains/engine.rs:78`
**Purpose:** FFI event wrapper for state changes
**Ownership:** FFI domain system
**Status:** ✅ Active - FFI layer

## 6. Test/Mock State Types

#### MockState
**Location:** `core/src/mocks/mock_state.rs:21`
**Purpose:** Test double for engine state
**Usage:** Unit tests, integration tests
**Status:** ✅ Active - Testing only

#### StateChange (mock enum)
**Location:** `core/src/mocks/mock_state.rs:9`
**Purpose:** Mock state change events
**Status:** ✅ Active - Testing only

#### TestDomainState / TestState / FuzzDomainState
**Location:** Various test files
**Purpose:** Test-specific state containers
**Status:** ✅ Active - Testing only

## 7. Driver-Specific State

#### ModifierStateTracker
**Location:** `core/src/drivers/linux/reader.rs:59`
**Purpose:** Linux-specific modifier key tracking
**Ownership:** Linux driver
**Status:** ✅ Active - Platform-specific

#### ThreadLocalState
**Location:** `core/src/drivers/windows/safety/thread_local.rs:87`
**Purpose:** Windows thread-local state management
**Ownership:** Windows driver
**Status:** ✅ Active - Platform-specific

## 8. CLI State

#### StateCommand
**Location:** `core/src/cli/commands/state.rs:15`
**Purpose:** CLI command for state inspection
**Status:** ✅ Active - CLI layer

#### StateView
**Location:** `core/src/cli/commands/state.rs:25`
**Purpose:** Formatted state display for CLI
**Status:** ✅ Active - CLI layer

## 9. Overlap and Duplication Analysis

### 9.1 Critical Duplicates (Must Fix)

| Type | Location 1 | Location 2 | Status |
|------|-----------|-----------|--------|
| EngineState (concept) | state/mod.rs:133 (unified) | advanced.rs uses compat layer | ⚠️ Migration in progress |
| EngineStateSnapshot | advanced.rs:61 | state/snapshot.rs:41 | 🔴 Remove legacy |

### 9.2 Overlapping Patterns (Should Consolidate)

| Pattern | Location 1 | Location 2 | Recommendation |
|---------|-----------|-----------|----------------|
| Session State | RecordingState | ReplaySession fields | Extract SessionState base |
| Modifier Tracking | ModifierState (unified) | ModifierStateTracker (Linux) | Keep separate (platform vs unified) |
| State Snapshots | StateSnapshot | EngineStateSnapshot | Already marked deprecated ✓ |

### 9.3 State Ownership Patterns

Three distinct patterns identified:

1. **Unified Ownership** (Modern)
   - Single EngineState owns all components
   - Mutations via transactional API
   - Example: EngineState contains KeyState, LayerState, ModifierState

2. **Split Ownership** (Legacy)
   - AdvancedEngine owns state + compat layers separately
   - Direct mutable access bypasses mutation API
   - Example: `layers_compat: LayerStack` alongside `state: UnifiedEngineState`

3. **Domain Ownership** (FFI)
   - Each FFI domain owns its state
   - Isolated lifecycle per domain
   - Example: DeviceState, RecordingState, DiscoverySessionState

## 10. State Access Patterns

### 10.1 Query-Only Access
**Files:** All readers of state
**Pattern:** `&EngineState` → `state.is_key_pressed()`, `state.active_layers()`
**Safety:** ✅ Safe - immutable borrows

### 10.2 Mutation API (Recommended)
**Files:** Engine core, state management
**Pattern:** `&mut EngineState` → `state.apply(mutation)`, `state.apply_batch(mutations)`
**Safety:** ✅ Safe - validated transitions, rollback on error
**Benefits:** Version tracking, effects, invariant checking

### 10.3 Direct Mutation (Legacy/Dangerous)
**Files:** advanced.rs, compat code
**Pattern:** `state.keys_mut()`, `state.modifiers_mut()` → direct field access
**Safety:** ⚠️ BYPASS - No version increment, no validation
**Usage:** Only during migration period, marked as unsafe in docs

## 11. State Transition Validation

### 11.1 Current Invariants

Checked in `EngineState::validate_invariants()`:
1. Base layer always active
2. No duplicate layers in stack
3. Version counter never decreases

### 11.2 Missing Invariants (To Be Added)

Identified but not yet enforced:
- No orphaned modifiers (modifier active without triggering key pressed)
- Layer stack never empty (base layer minimum)
- Pending queue size bounds respected
- Key timestamps monotonically increasing per key

## 12. Consolidation Recommendations

### Phase 1: Immediate (Breaking)
1. **Remove EngineStateSnapshot** from advanced.rs
   - All callers use `state_snapshot()` instead
   - Update FFI layer to use new format

2. **Remove KeyStateView adapter** after migration complete
   - Direct use of unified state everywhere
   - Remove KeyStateProvider trait if no other users

### Phase 2: Extract Common Patterns
3. **Extract SessionState base type**
   - Common: start_time, metadata, lifecycle enum
   - Compose into RecordingSession and ReplaySession
   - Reduces duplication, clarifies ownership

### Phase 3: Unify Access Patterns
4. **Deprecate direct `_mut()` accessors**
   - Force all mutations through `apply()` API
   - Add helper mutations for common operations
   - Remove compat layer from AdvancedEngine

## 13. State Machine Design (Future Work)

### Proposed StateGraph Structure

```rust
pub enum StateTransition {
    KeyPress,
    KeyRelease,
    LayerPush,
    LayerPop,
    ModifierActivate,
    ModifierDeactivate,
    // ... all valid transitions
}

pub enum StateKind {
    Empty,          // No keys pressed, base layer only
    Typing,         // Normal key input
    Pending,        // Awaiting tap-hold/combo resolution
    LayerActive,    // Non-base layer active
    ModifierHeld,   // Modifiers active
    // ... categorization
}

pub struct StateGraph {
    rules: HashMap<(StateKind, StateTransition), StateKind>,
}
```

**Benefits:**
- Explicit valid transitions
- Rejecting invalid state changes at compile time
- Clear state machine visualization
- Easier testing of edge cases

## 14. Summary Statistics

| Category | Count | Notes |
|----------|-------|-------|
| Core Engine State Types | 10 | EngineState + components |
| Legacy/Deprecated | 2 | EngineStateSnapshot, KeyStateView |
| Decision/Pending | 3 | Queue + state enum + resolution |
| Session/Recording | 3 | Recording + Replay states |
| FFI Domain States | 3 | Device, Discovery, StateEvent |
| Test/Mock States | 5+ | Various test doubles |
| Driver-Specific | 2 | Linux, Windows platform states |
| CLI States | 2 | Command + View |
| **Total Identified** | **30+** | Excluding test-only types |

## 15. State Ownership Map

```
EngineState (Canonical)
├── KeyState
├── LayerState
├── ModifierState
└── PendingState
    └── DecisionQueue

AdvancedEngine (In Migration)
├── state: UnifiedEngineState    ✅ Modern
├── layers_compat: LayerStack    ⚠️ Legacy compat
├── pending: DecisionQueue       ⚠️ Should use PendingState
└── blocked_releases: HashSet    ⚠️ Not in unified state yet

FFI Domains (Isolated)
├── DeviceState
├── RecordingState
└── DiscoverySessionState

Session Management (Separate)
├── ReplaySession { events, state: ReplayState, ... }
└── (Recording handled via FFI domain)

Drivers (Platform-Specific)
├── ModifierStateTracker (Linux)
└── ThreadLocalState (Windows)
```

## 16. State Ownership and Lifecycle Documentation

This section provides detailed ownership mappings and lifecycle documentation for all state types, clarifying boundaries and responsibilities.

### 16.1 Core Engine State Ownership

#### EngineState (Canonical Owner)
**Owner:** `Engine` struct in `core/src/engine/mod.rs`
**Lifecycle:**
1. **Creation:** Initialized at engine startup via `Engine::new()` or `EngineState::default()`
2. **Active:** Lives for entire engine lifetime, mutated via `apply()` and `apply_batch()`
3. **Destruction:** Dropped when engine is destroyed

**Owned Components:**
- `KeyState` - Physical key press tracking
- `LayerState` - Active layer stack
- `ModifierState` - Standard and virtual modifiers
- `PendingState` - Decision queue wrapper

**Access Patterns:**
- Read-only: `&self` methods (`is_key_pressed()`, `active_layers()`, etc.)
- Mutation: `&mut self` via `apply(mutation)` only
- Unsafe direct mutation (legacy): `keys_mut()`, `modifiers_mut()` - to be removed

**Boundaries:**
- ✅ Owns all core input state
- ❌ Does NOT own: FFI state, session state, driver state
- ❌ Does NOT manage: Device configuration, recording/replay

---

#### KeyState
**Owner:** `EngineState` (field `keys: KeyState`)
**Lifecycle:**
1. **Creation:** Created with `KeyState::with_capacity(256)` at engine init
2. **Active:** Updated on every key press/release event
3. **Mutation:** Only via `EngineState::apply(KeyMutation::*)`
4. **Destruction:** Dropped with parent EngineState

**Responsibilities:**
- Track currently pressed keys with timestamps
- Provide `is_pressed(keycode)` queries
- Maintain press timestamp for each key

**Boundaries:**
- ✅ Owns key press timestamps
- ❌ Does NOT own: Key definitions, keymaps, bindings

**Invariants:**
- All timestamps must be monotonically increasing per key
- Maximum 256 keys tracked (pre-allocated capacity)

---

#### LayerState
**Owner:** `EngineState` (field `layers: LayerState`)
**Lifecycle:**
1. **Creation:** Created with base layer at index 0
2. **Active:** Modified on layer push/pop/switch events
3. **Mutation:** Only via `EngineState::apply(LayerMutation::*)`
4. **Destruction:** Dropped with parent EngineState

**Responsibilities:**
- Maintain ordered stack of active layers
- Track base layer (always present)
- Provide highest priority layer for keymap resolution

**Boundaries:**
- ✅ Owns layer activation stack
- ❌ Does NOT own: Layer definitions, keymaps, layer configurations

**Invariants:**
- Layer stack NEVER empty (base layer minimum)
- No duplicate layers in stack
- Base layer always at bottom of stack

---

#### ModifierState
**Owner:** `EngineState` (field `modifiers: ModifierState`)
**Lifecycle:**
1. **Creation:** Created with zero modifiers active
2. **Active:** Updated on modifier key events and one-shot triggers
3. **Mutation:** Only via `EngineState::apply(ModifierMutation::*)`
4. **Destruction:** Dropped with parent EngineState

**Responsibilities:**
- Track standard OS modifiers (Shift/Ctrl/Alt/Meta)
- Track 256 virtual modifiers
- Manage one-shot modifier state
- Provide combined modifier query

**Boundaries:**
- ✅ Owns modifier activation state
- ❌ Does NOT own: Modifier definitions, keymap modifier logic

**Invariants:**
- No orphaned modifiers (modifiers active without triggering key)
- One-shot state cleared after next key press

---

#### PendingState
**Owner:** `EngineState` (field `pending: PendingState`)
**Lifecycle:**
1. **Creation:** Wraps DecisionQueue at engine init
2. **Active:** Decisions added on tap-hold/combo detection, resolved on timeout/disambiguating events
3. **Mutation:** Only via `EngineState::apply(PendingMutation::*)`
4. **Destruction:** Dropped with parent EngineState

**Responsibilities:**
- Queue pending tap-hold decisions
- Queue pending combo detections
- Resolve decisions on timeout or disambiguating events

**Boundaries:**
- ✅ Owns pending decision queue
- ❌ Does NOT own: Tap-hold config, combo definitions

**Invariants:**
- Queue size never exceeds `DecisionQueue::MAX_PENDING`
- Decisions resolved in FIFO order
- No duplicate decisions for same key

---

### 16.2 Supporting State Ownership

#### StateChange
**Owner:** Returned by mutation operations, short-lived
**Lifecycle:**
1. **Creation:** Created by `EngineState::apply()` on successful mutation
2. **Active:** Exists only during event processing
3. **Consumption:** Consumed by effect executor or logged
4. **Destruction:** Dropped after effects processed

**Responsibilities:**
- Record mutation that occurred
- Track version increment
- List effects to execute
- Provide timestamp

**Boundaries:**
- ✅ Immutable record of change
- ❌ Does NOT mutate state

---

#### StateSnapshot
**Owner:** Caller of `EngineState::state_snapshot()`
**Lifecycle:**
1. **Creation:** Created on-demand via snapshot API
2. **Active:** Exists as immutable copy of state at point in time
3. **Usage:** Serialization, debugging, GUI display, testing
4. **Destruction:** Dropped when no longer needed

**Responsibilities:**
- Provide serializable state view
- Enable state comparison
- Support debugging workflows

**Boundaries:**
- ✅ Immutable point-in-time copy
- ❌ Does NOT track changes or mutations

---

#### StateHistory
**Owner:** Optional component, owned by `Engine` or debug tooling
**Lifecycle:**
1. **Creation:** Created with configurable depth on debug builds
2. **Active:** Accumulates snapshots on configurable interval
3. **Query:** Provides historical state lookup
4. **Destruction:** Dropped with owning component

**Responsibilities:**
- Ring buffer of historical snapshots
- Time-based or event-based sampling
- Bounded memory usage

**Boundaries:**
- ✅ Owns historical snapshot ring buffer
- ❌ Does NOT own live state

**Invariants:**
- Buffer depth never exceeds configured maximum
- Oldest entries evicted first

---

### 16.3 Legacy State Ownership (Migration Path)

#### AdvancedEngine State Management
**Owner:** `AdvancedEngine` struct (legacy)
**Current State (In Migration):**
```rust
pub struct AdvancedEngine {
    state: UnifiedEngineState,           // ✅ Modern unified state
    layers_compat: LayerStack,           // ⚠️ Legacy compat layer
    pending: DecisionQueue,              // ⚠️ Should use PendingState
    blocked_releases: HashSet<KeyCode>, // ⚠️ Not in unified state
}
```

**Migration Lifecycle:**
1. **Phase 1 (Current):** Dual state management - unified + compat layers
2. **Phase 2 (In Progress):** Gradually migrate all access to unified state
3. **Phase 3 (Target):** Remove compat layers entirely

**Boundaries After Migration:**
- ✅ All state in `UnifiedEngineState`
- ❌ No separate `layers_compat` or `pending` fields
- ❌ `blocked_releases` moved into unified state or removed

---

### 16.4 FFI Domain State Ownership

FFI domains follow isolated ownership pattern - each domain owns its state independently.

#### DeviceState
**Owner:** Device FFI Domain (`core/src/ffi/domains/device.rs`)
**Lifecycle:**
1. **Creation:** Created when device domain initialized
2. **Active:** Updated via FFI calls from GUI/CLI
3. **Destruction:** Dropped when domain destroyed or process exits

**Responsibilities:**
- Store device configuration
- Track device enumeration
- Manage device selection state

**Boundaries:**
- ✅ Isolated from engine state
- ❌ Does NOT interact with EngineState directly

---

#### RecordingState
**Owner:** Recording FFI Domain (`core/src/ffi/domains/recording.rs`)
**Lifecycle:**
1. **Creation:** Created when recording session starts
2. **Active:** Updated during recording (events, metadata)
3. **Destruction:** Persisted to file and dropped on session end

**Responsibilities:**
- Track recording session metadata
- Buffer events for serialization
- Manage recording lifecycle (idle/active/paused)

**Boundaries:**
- ✅ Isolated FFI domain state
- ❌ Does NOT read engine state directly (receives events via observer)

---

#### DiscoverySessionState
**Owner:** Discovery FFI Domain (`core/src/ffi/domains/discovery.rs`)
**Lifecycle:**
1. **Creation:** Created when device discovery initiated
2. **Active:** Updated as devices discovered
3. **Destruction:** Dropped when discovery session ends

**Responsibilities:**
- Track discovered devices
- Manage discovery progress
- Handle discovery timeouts

**Boundaries:**
- ✅ Isolated FFI domain state
- ❌ Does NOT access engine state

---

### 16.5 Session State Ownership

#### ReplaySession State
**Owner:** `ReplaySession` struct (`core/src/engine/replay.rs`)
**Lifecycle:**
1. **Creation:** Created from serialized event log
2. **Active:** Transitions through Idle → Playing → Paused → Completed
3. **Destruction:** Dropped after replay completes or is cancelled

**Responsibilities:**
- Track replay position in event stream
- Manage replay lifecycle (idle/playing/paused/completed)
- Time-based event playback

**Boundaries:**
- ✅ Owns replay session state
- ❌ Does NOT own the events (borrows from event log)

**State Enum:**
```rust
pub enum ReplayState {
    Idle,       // Not started
    Playing,    // Actively replaying
    Paused,     // Paused mid-replay
    Completed,  // Finished replay
}
```

---

### 16.6 Driver State Ownership

Platform-specific drivers own state needed for platform integration.

#### ModifierStateTracker (Linux)
**Owner:** Linux driver (`core/src/drivers/linux/reader.rs`)
**Lifecycle:**
1. **Creation:** Created when Linux driver initializes
2. **Active:** Tracks system modifier state from evdev
3. **Destruction:** Dropped when driver shuts down

**Responsibilities:**
- Track Linux kernel modifier state
- Synchronize with evdev modifier events
- Provide modifier state for key event context

**Boundaries:**
- ✅ Platform-specific tracking
- ❌ Does NOT replace EngineState's ModifierState (separate concern)

---

#### ThreadLocalState (Windows)
**Owner:** Windows driver thread-local storage
**Lifecycle:**
1. **Creation:** Created per-thread when Windows driver thread starts
2. **Active:** Stores thread-local hook context
3. **Destruction:** Dropped when thread exits

**Responsibilities:**
- Store Windows hook context per-thread
- Avoid global mutable state in hooks
- Thread-safe state isolation

**Boundaries:**
- ✅ Thread-isolated state
- ❌ Does NOT share state across threads

---

### 16.7 CLI State Ownership

#### StateCommand & StateView
**Owner:** CLI command handler
**Lifecycle:**
1. **Creation:** Created when `keyrx state` command invoked
2. **Active:** Queries engine state and formats output
3. **Destruction:** Dropped after command completes

**Responsibilities:**
- Format state for human-readable CLI output
- Query engine state snapshot
- Display state in various formats (JSON, pretty-print)

**Boundaries:**
- ✅ Owns formatting logic only
- ❌ Does NOT own engine state (queries via snapshot)

---

### 16.8 Test State Ownership

Test doubles follow standard test patterns with isolated ownership.

#### MockState
**Owner:** Test code
**Lifecycle:** Created in test setup, destroyed at test teardown
**Responsibilities:** Provide controllable state for testing
**Boundaries:** Test-only, never used in production

---

### 16.9 State Ownership Boundary Rules

**Rule 1: Single Owner Principle**
- Every state has exactly one owner
- Owner responsible for lifecycle (creation, mutation, destruction)
- No shared mutable state across boundaries

**Rule 2: Ownership Transfer**
- State ownership can transfer via move semantics
- Example: `StateSnapshot` ownership transfers to caller

**Rule 3: Cross-Boundary Access**
- FFI domains ← observe → EngineState (via events, not direct access)
- CLI ← queries → EngineState (via immutable snapshot)
- Drivers ← integrate → EngineState (via mutation API)

**Rule 4: Isolation Boundaries**
- EngineState isolated from FFI domains
- FFI domains isolated from each other
- Drivers isolated from FFI domains

**Rule 5: Migration Boundaries**
- Legacy code temporarily violates boundaries during migration
- All violations documented with ⚠️ markers
- Clear removal timeline for compat layers

---

## 17. State Consolidation Plan

This section provides a detailed roadmap for consolidating duplicate and overlapping state types, with clear migration paths and minimal breaking changes.

### 17.1 Consolidation Overview

**Goals:**
- Eliminate duplicate state definitions
- Extract common patterns into shared types
- Minimize breaking changes through gradual migration
- Maintain backward compatibility during transition where needed
- Clear removal timeline for all deprecated code

**Guiding Principles:**
1. **No backward compatibility required** - Clean breaks preferred over compatibility shims
2. **Fail fast** - Validate at boundaries, reject invalid states immediately
3. **Single source of truth** - One canonical definition per concept
4. **Clear ownership** - Every state has exactly one owner
5. **Testability** - All changes must be testable

---

### 17.2 Phase 1: Remove Critical Duplicates (IMMEDIATE - Breaking)

#### 17.2.1 Remove EngineStateSnapshot from advanced.rs

**Status:** 🔴 Critical - Immediate action required
**Impact:** Breaking change - requires FFI and API updates
**Timeline:** Complete in single atomic commit

**Current State:**
- Two snapshot types exist: `EngineStateSnapshot` (legacy) and `StateSnapshot` (unified)
- Legacy format used by `AdvancedEngine::snapshot()`
- New format used by `EngineState::state_snapshot()`

**Migration Steps:**

1. **Identify all callers** of `AdvancedEngine::snapshot()`
   - Files: FFI layer, test code, CLI commands
   - Search pattern: `snapshot\(\)` in context of `AdvancedEngine`

2. **Update FFI layer** (`core/src/ffi/domains/engine.rs`)
   ```rust
   // BEFORE
   pub fn engine_snapshot(handle: EngineHandle) -> EngineStateSnapshot {
       handle.engine.snapshot()
   }

   // AFTER
   pub fn engine_snapshot(handle: EngineHandle) -> StateSnapshot {
       handle.engine.state().state_snapshot()
   }
   ```

3. **Update tests** to use new snapshot format
   - Replace `engine.snapshot()` with `engine.state().state_snapshot()`
   - Update test assertions for new field names

4. **Remove legacy type**
   ```rust
   // DELETE from core/src/engine/advanced.rs:61
   pub struct EngineStateSnapshot { ... }

   // DELETE method
   impl AdvancedEngine {
       pub fn snapshot(&self) -> EngineStateSnapshot { ... }
   }
   ```

5. **Update documentation**
   - Remove references to legacy snapshot format
   - Update FFI documentation

**Validation:**
- [ ] All tests pass with new snapshot format
- [ ] FFI layer compiles with new type
- [ ] No references to `EngineStateSnapshot` remain in codebase

**Rollback Plan:** None - breaking change is intentional

---

#### 17.2.2 Remove KeyStateView Adapter

**Status:** ⚠️ Depends on migration completion
**Impact:** Breaking change - removes compatibility shim
**Timeline:** After all code uses unified state directly

**Current State:**
- `KeyStateView` provides `KeyStateProvider` trait compatibility
- Used during gradual migration to unified state
- Adapter wraps unified `KeyState` in legacy trait

**Migration Steps:**

1. **Search for KeyStateProvider trait usage**
   - Find all implementations and consumers
   - Identify code still using trait-based access

2. **Convert trait users to direct state access**
   ```rust
   // BEFORE
   fn check_key<K: KeyStateProvider>(provider: &K, key: KeyCode) -> bool {
       provider.is_key_pressed(key)
   }

   // AFTER
   fn check_key(state: &EngineState, key: KeyCode) -> bool {
       state.is_key_pressed(key)
   }
   ```

3. **Remove KeyStateProvider trait** if no other implementations exist
   - Check for external implementations (drivers, tests)
   - If trait still needed for drivers, keep but remove KeyStateView

4. **Remove KeyStateView adapter**
   ```rust
   // DELETE from core/src/engine/advanced.rs:24
   pub struct KeyStateView { ... }
   ```

**Validation:**
- [ ] No references to `KeyStateView` in production code
- [ ] All key state access uses unified state directly
- [ ] Tests pass without adapter

---

### 17.3 Phase 2: Extract Common Patterns

#### 17.3.1 Extract SessionState Base Type

**Status:** 🟡 Medium priority - reduces duplication
**Impact:** Non-breaking - internal refactoring
**Timeline:** After Phase 1 complete

**Problem:**
`RecordingState` and `ReplaySession` share common session management patterns:
- Session metadata (start time, duration, name)
- Lifecycle tracking (idle/active/paused/completed)
- Event buffering

**Solution:**
Create shared `SessionState` base type with composition pattern.

**Design:**

```rust
// New file: core/src/engine/session/state.rs

/// Session lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionLifecycle {
    Idle,       // Not started
    Active,     // Running
    Paused,     // Paused mid-session
    Completed,  // Finished
    Failed,     // Error occurred
}

/// Session metadata shared by recording and replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub name: Option<String>,
    pub created_at: SystemTime,
    pub duration: Option<Duration>,
    pub event_count: usize,
}

/// Base session state shared by recording and replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub lifecycle: SessionLifecycle,
    pub metadata: SessionMetadata,
    pub started_at: Option<Instant>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            lifecycle: SessionLifecycle::Idle,
            metadata: SessionMetadata {
                name: None,
                created_at: SystemTime::now(),
                duration: None,
                event_count: 0,
            },
            started_at: None,
        }
    }

    pub fn start(&mut self) {
        self.lifecycle = SessionLifecycle::Active;
        self.started_at = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        if self.lifecycle == SessionLifecycle::Active {
            self.lifecycle = SessionLifecycle::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.lifecycle == SessionLifecycle::Paused {
            self.lifecycle = SessionLifecycle::Active;
        }
    }

    pub fn complete(&mut self) {
        self.lifecycle = SessionLifecycle::Completed;
        if let Some(started) = self.started_at {
            self.metadata.duration = Some(started.elapsed());
        }
    }

    pub fn fail(&mut self) {
        self.lifecycle = SessionLifecycle::Failed;
    }

    pub fn is_active(&self) -> bool {
        self.lifecycle == SessionLifecycle::Active
    }
}
```

**Migration Steps:**

1. **Create new SessionState type** as shown above
   - File: `core/src/engine/session/state.rs`
   - Add module: `core/src/engine/session/mod.rs`

2. **Refactor RecordingState** to compose SessionState
   ```rust
   // core/src/ffi/domains/recording.rs

   // BEFORE
   pub struct RecordingState {
       status: RecordingStatus,  // Idle/Recording/Paused
       start_time: Option<Instant>,
       events: Vec<RecordedEvent>,
       // ... other fields
   }

   // AFTER
   pub struct RecordingState {
       session: SessionState,
       writer: EventWriter,
       buffer: Vec<RecordedEvent>,
   }

   impl RecordingState {
       pub fn start_recording(&mut self) {
           self.session.start();
       }

       pub fn is_recording(&self) -> bool {
           self.session.is_active()
       }
   }
   ```

3. **Refactor ReplaySession** to compose SessionState
   ```rust
   // core/src/engine/replay.rs

   // BEFORE
   pub struct ReplaySession {
       state: ReplayState,  // Enum with Idle/Playing/Paused/Completed
       events: Vec<Event>,
       position: usize,
       start_time: Option<Instant>,
       // ... other fields
   }

   // AFTER
   pub struct ReplaySession {
       session: SessionState,
       reader: EventReader,
       position: usize,
   }

   impl ReplaySession {
       pub fn play(&mut self) {
           self.session.start();
       }

       pub fn is_playing(&self) -> bool {
           self.session.is_active()
       }
   }
   ```

4. **Update callers** to use new API
   - Search for `.state` field access on `ReplaySession`
   - Replace with `.session.lifecycle`

5. **Remove old enums**
   - Remove `ReplayState` enum if fully replaced
   - Remove `RecordingStatus` enum if fully replaced

**Validation:**
- [ ] All recording operations use SessionState
- [ ] All replay operations use SessionState
- [ ] Tests pass with new composition
- [ ] No duplicate lifecycle tracking

**Benefits:**
- Single source of truth for session lifecycle
- Consistent API across recording and replay
- Easier to add new session types (e.g., test sessions)
- Reduces code duplication by ~100 lines

---

### 17.4 Phase 3: Unify Access Patterns

#### 17.4.1 Deprecate Direct `_mut()` Accessors

**Status:** 🟡 Post-migration cleanup
**Impact:** Breaking change - enforces mutation API
**Timeline:** After legacy code fully migrated

**Problem:**
Direct mutable access bypasses:
- Version tracking
- Invariant validation
- Effect generation
- Transition logging

**Current Unsafe Patterns:**
```rust
// UNSAFE: Bypasses mutation API
engine.state.keys_mut().insert(keycode, timestamp);
engine.state.modifiers_mut().set_shift(true);
engine.state.layers_mut().push(layer_id);
```

**Solution:**
Force all mutations through `apply()` API.

**Migration Steps:**

1. **Identify all `_mut()` accessor usage**
   ```bash
   # Search pattern
   rg "keys_mut|modifiers_mut|layers_mut|pending_mut"
   ```

2. **Create helper mutations** for common operations
   ```rust
   // Add to core/src/engine/state/mutation.rs

   impl Mutation {
       /// Convenience: Press key with current timestamp
       pub fn press_key(key: KeyCode) -> Self {
           Mutation::Key(KeyMutation::Press {
               key,
               timestamp: Instant::now(),
           })
       }

       /// Convenience: Set modifier state
       pub fn set_modifier(modifier: Modifier, active: bool) -> Self {
           if active {
               Mutation::Modifier(ModifierMutation::Activate(modifier))
           } else {
               Mutation::Modifier(ModifierMutation::Deactivate(modifier))
           }
       }

       /// Convenience: Push layer
       pub fn push_layer(layer: LayerId) -> Self {
           Mutation::Layer(LayerMutation::Push(layer))
       }
   }
   ```

3. **Convert direct mutations** to use helper methods
   ```rust
   // BEFORE
   state.keys_mut().insert(keycode, timestamp);

   // AFTER
   state.apply(Mutation::press_key(keycode))?;
   ```

4. **Remove `_mut()` accessors** from public API
   ```rust
   // core/src/engine/state/mod.rs

   impl EngineState {
       // DELETE these methods
       // pub fn keys_mut(&mut self) -> &mut KeyState { ... }
       // pub fn modifiers_mut(&mut self) -> &mut ModifierState { ... }
       // pub fn layers_mut(&mut self) -> &mut LayerState { ... }

       // KEEP read-only accessors
       pub fn keys(&self) -> &KeyState { &self.keys }
       pub fn modifiers(&self) -> &ModifierState { &self.modifiers }
       pub fn layers(&self) -> &LayerState { &self.layers }
   }
   ```

5. **Remove compat layers** from AdvancedEngine
   ```rust
   // core/src/engine/advanced.rs

   pub struct AdvancedEngine {
       state: UnifiedEngineState,  // ✅ Keep
       // DELETE compat fields
       // layers_compat: LayerStack,
       // pending: DecisionQueue,
       // blocked_releases: HashSet<KeyCode>,
   }
   ```

**Validation:**
- [ ] No `_mut()` accessors remain in public API
- [ ] All mutations go through `apply()` or `apply_batch()`
- [ ] Version counter increments on all mutations
- [ ] Invariants checked on all mutations
- [ ] All tests pass with enforced mutation API

**Benefits:**
- Guaranteed invariant checking
- Complete transition history
- Easier debugging (all mutations logged)
- Consistent API surface

---

### 17.5 Phase 4: Move State into Unified State (Advanced)

#### 17.5.1 Move blocked_releases into EngineState

**Status:** 🟢 Nice-to-have
**Impact:** Breaking change to AdvancedEngine internals
**Timeline:** After Phase 3

**Problem:**
`AdvancedEngine.blocked_releases: HashSet<KeyCode>` is state that should be part of unified state.

**Solution:**
Add `BlockedReleasesState` component to `EngineState`.

**Design:**
```rust
// New file: core/src/engine/state/blocked.rs

/// Keys with blocked release events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockedReleasesState {
    blocked: HashSet<KeyCode>,
}

impl BlockedReleasesState {
    pub fn block(&mut self, key: KeyCode) {
        self.blocked.insert(key);
    }

    pub fn unblock(&mut self, key: KeyCode) {
        self.blocked.remove(&key);
    }

    pub fn is_blocked(&self, key: KeyCode) -> bool {
        self.blocked.contains(&key)
    }

    pub fn clear(&mut self) {
        self.blocked.clear();
    }
}
```

**Migration Steps:**

1. **Add BlockedReleasesState to EngineState**
   ```rust
   // core/src/engine/state/mod.rs

   pub struct EngineState {
       keys: KeyState,
       layers: LayerState,
       modifiers: ModifierState,
       pending: PendingState,
       blocked: BlockedReleasesState,  // NEW
       version: u64,
   }
   ```

2. **Add mutations for blocked releases**
   ```rust
   // core/src/engine/state/mutation.rs

   pub enum BlockedMutation {
       Block(KeyCode),
       Unblock(KeyCode),
       ClearAll,
   }

   pub enum Mutation {
       Key(KeyMutation),
       Layer(LayerMutation),
       Modifier(ModifierMutation),
       Pending(PendingMutation),
       Blocked(BlockedMutation),  // NEW
   }
   ```

3. **Migrate AdvancedEngine usage**
   ```rust
   // BEFORE
   self.blocked_releases.insert(keycode);

   // AFTER
   self.state.apply(Mutation::Blocked(BlockedMutation::Block(keycode)))?;
   ```

4. **Remove field from AdvancedEngine**
   ```rust
   pub struct AdvancedEngine {
       state: UnifiedEngineState,
       // DELETE: blocked_releases: HashSet<KeyCode>,
   }
   ```

**Validation:**
- [ ] Blocked releases tracked in unified state
- [ ] Mutations go through apply API
- [ ] Tests pass

---

### 17.6 Phase 5: Driver State Isolation (No Changes)

**Decision:** Keep driver state separate from engine state.

**Rationale:**
- `ModifierStateTracker` (Linux) tracks kernel-level modifier state
- `ThreadLocalState` (Windows) is thread-local for hooks
- These are platform integration concerns, not engine state
- Mixing platform state with engine state violates separation of concerns

**Action:** No changes required - current separation is correct.

---

### 17.7 Phase 6: FFI Domain State Isolation (No Changes)

**Decision:** Keep FFI domain state isolated from engine state.

**Rationale:**
- FFI domains follow isolated ownership pattern
- Each domain manages its own lifecycle
- Domain state does not interact with engine state directly
- Current architecture follows Domain-Driven Design principles

**Action:** No changes required - current separation is correct.

---

### 17.8 Migration Safety Guidelines

**Before Making Changes:**
1. Read all affected files
2. Run existing tests to establish baseline
3. Search for all usages of types being changed
4. Document breaking changes

**During Migration:**
1. Make one atomic change at a time
2. Commit after each logical change completes
3. Run tests after each commit
4. Use clear commit messages describing what changed

**After Changes:**
1. Verify all tests pass
2. Check code metrics (file/function size limits)
3. Run full build to catch compilation errors
4. Update documentation

**Rollback Strategy:**
- Each phase is independently revertable
- Atomic commits allow granular rollback
- No partial migrations - complete phase or revert

---

### 17.9 Breaking Change Policy

Per project guidelines: **No backward compatibility required unless explicitly requested.**

**What This Means:**
- We will break existing APIs freely if it improves the codebase
- No deprecation warnings or transitional shims (except temporary during migration)
- Users must update code when upgrading
- Clear migration documentation provided

**Communication:**
- Document all breaking changes in commit messages
- Update CHANGELOG with breaking changes
- Provide migration examples in docs

---

### 17.10 Success Metrics

**Code Quality:**
- [ ] Zero duplicate state definitions
- [ ] All files < 500 lines
- [ ] All functions < 50 lines
- [ ] 80%+ test coverage maintained

**Architecture:**
- [ ] Single canonical definition per state concept
- [ ] All mutations through validated API
- [ ] Clear ownership boundaries
- [ ] No cross-boundary state access

**Performance:**
- [ ] No performance regression in state operations
- [ ] Mutation API overhead < 1% vs direct access
- [ ] Memory usage unchanged or reduced

---

### 17.11 Timeline Summary

| Phase | Description | Impact | Dependencies |
|-------|-------------|--------|--------------|
| 1.1 | Remove EngineStateSnapshot | Breaking | None |
| 1.2 | Remove KeyStateView | Breaking | Phase 1.1 |
| 2.1 | Extract SessionState | Non-breaking | Phase 1 complete |
| 3.1 | Deprecate `_mut()` | Breaking | Phase 2 complete |
| 4.1 | Move blocked_releases | Breaking | Phase 3 complete |

**Estimated Effort:**
- Phase 1: 1 commit (2-3 hours)
- Phase 2: 1 commit (2-3 hours)
- Phase 3: 2 commits (4-6 hours)
- Phase 4: 1 commit (1-2 hours)
- **Total:** ~12-16 hours of development time

---

### 17.12 Open Questions

1. **Should DecisionQueue be completely wrapped by PendingState?**
   - Current: PendingState wraps DecisionQueue
   - Alternative: Merge DecisionQueue logic into PendingState
   - Recommendation: Keep separate - DecisionQueue is complex and tested

2. **Should StateHistory be part of EngineState or separate?**
   - Current: Optional separate component
   - Alternative: Built into EngineState with feature flag
   - Recommendation: Keep separate - not all users need history

3. **Should we add a feature flag to disable state validation in release builds?**
   - Current: Always validate
   - Alternative: Debug-only validation
   - Recommendation: Always validate - fail fast is better than corrupt state

---

## 18. Next Steps

Based on this audit and consolidation plan, the following tasks are recommended:

1. ✅ **Complete this audit** (DONE)
2. ✅ **Document state ownership and lifecycle** (DONE)
3. ✅ **Create consolidation plan for duplicates** (DONE)
4. **Execute Phase 1.1:** Remove EngineStateSnapshot
5. **Execute Phase 1.2:** Remove KeyStateView
6. **Execute Phase 2.1:** Extract SessionState
7. Design StateTransition enum
8. Design StateKind enum
9. Implement StateGraph with transition rules
10. Define Invariant trait for validation
11. Implement core invariants
12. Create StateValidator combining invariants
13. Add transition logging
14. Integrate StateGraph into Engine
15. **Execute Phase 3.1:** Deprecate `_mut()` accessors
16. **Execute Phase 4.1:** Move blocked_releases into unified state
17. Add comprehensive state transition tests

---

**Audit Completed:** 2025-12-04
**State Ownership Documentation Completed:** 2025-12-04
**Consolidation Plan Completed:** 2025-12-04
**Auditor:** AI Assistant (Claude)
**Version:** 1.2
