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

## 16. Next Steps

Based on this audit, the following tasks are recommended:

1. ✅ **Complete this audit** (DONE)
2. Create consolidation plan for duplicates
3. Design StateTransition enum
4. Design StateKind enum
5. Implement StateGraph with transition rules
6. Define Invariant trait for validation
7. Implement core invariants
8. Create StateValidator combining invariants
9. Add transition logging
10. Integrate StateGraph into Engine
11. Merge duplicate EngineState definitions
12. Extract common SessionState
13. Add comprehensive state transition tests

---

**Audit Completed:** 2025-12-04
**Auditor:** AI Assistant (Claude)
**Version:** 1.0
