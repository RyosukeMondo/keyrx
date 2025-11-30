# Requirements Document: advanced-remapping-engine

## Introduction

Phase 2.5 "The Brain" - the advanced remapping engine that transforms KeyRx from a simple key swapper into a QMK/KMonad-class keyboard customization powerhouse. This spec implements the timing-based decision system, virtual modifiers, layer stack, and composed behaviors that enable tap-hold, combos, one-shot modifiers, and more.

**Core differentiator**: All timing parameters are configurable and exposed to users, with the future GUI visualizing trade-offs in real-time.

## Alignment with Product Vision

From `product.md`:
> "Unlike traditional remappers that simply map key A to key B, KeyRx treats input as a programmable stream of events, enabling complex behaviors, layer systems, and context-aware remapping."

From `tech.md`:
> "Input latency: < 1ms processing overhead (hard requirement)"

This spec implements the 4-layer engine architecture defined in `tech.md`:
- Layer 1: State Management
- Layer 2: Decision Primitives
- Layer 3: Action Primitives
- Layer 4: Composed Behaviors

## Requirements

### REQ-1: Key State Tracking

**User Story:** As a power user, I want the engine to track which physical keys are currently held, so that timing-based decisions can be made accurately.

#### Acceptance Criteria

1. WHEN a key is pressed THEN the engine SHALL record the key and timestamp in the key state tracker
2. WHEN a key is released THEN the engine SHALL remove the key from the tracker and calculate hold duration
3. WHEN querying key state THEN the engine SHALL return all currently held keys with their press timestamps
4. IF the key state tracker receives duplicate key-down events THEN it SHALL ignore them (is_repeat handling)

### REQ-2: Timer System

**User Story:** As a power user, I want the engine to support timing-based decisions, so that tap-hold and other time-sensitive behaviors work accurately.

#### Acceptance Criteria

1. WHEN a timing decision is needed THEN the engine SHALL use microsecond-precision timestamps from InputEvent
2. WHEN creating a pending decision THEN the engine SHALL associate it with a deadline (press_time + timeout)
3. WHEN the deadline expires THEN the engine SHALL resolve the pending decision as "hold"
4. IF another key is pressed during a pending decision THEN the engine SHALL optionally resolve early (permissive_hold)

### REQ-3: Virtual Modifier System

**User Story:** As a power user, I want to define custom modifiers like "Mod_Thumb" that exist only in KeyRx, so that I can create behaviors beyond what the OS supports.

#### Acceptance Criteria

1. WHEN defining a virtual modifier THEN the engine SHALL support up to 255 custom modifier IDs
2. WHEN activating a virtual modifier THEN the engine SHALL set the corresponding bit in the modifier state
3. WHEN checking modifier state THEN scripts SHALL be able to query any combination of virtual modifiers
4. IF a virtual modifier is activated THEN it SHALL NOT be sent to the OS (engine-internal only)
5. WHEN the engine state is queried THEN it SHALL report both standard and virtual modifier states

### REQ-4: Layer Stack System

**User Story:** As a power user, I want multiple keyboard layers that I can switch between, so that I can have navigation, symbol, and gaming modes on one keyboard.

#### Acceptance Criteria

1. WHEN creating a layer THEN the engine SHALL store it with a unique name and keymap
2. WHEN pushing a layer THEN the engine SHALL add it to the top of the stack (highest priority)
3. WHEN popping a layer THEN the engine SHALL remove it from the stack
4. WHEN processing a key THEN the engine SHALL check layers from top to bottom until a mapping is found
5. IF a layer is marked transparent THEN the engine SHALL fall through to lower layers for unmapped keys
6. WHEN the base layer is reached THEN unmapped keys SHALL pass through unchanged

### REQ-5: Tap-Hold Detection

**User Story:** As a power user, I want a key to do one thing on tap and another on hold (e.g., CapsLock = Escape/Ctrl), so that I can maximize efficiency of each key.

#### Acceptance Criteria

1. WHEN a tap-hold key is pressed THEN the engine SHALL create a pending decision with the configured timeout
2. WHEN the key is released before timeout THEN the engine SHALL emit the tap action
3. WHEN the timeout expires while key is held THEN the engine SHALL activate the hold action
4. IF `eager_tap` is enabled THEN the engine SHALL emit tap immediately and cancel if becomes hold
5. IF `permissive_hold` is enabled THEN interrupting key SHALL resolve pending decision as hold
6. IF `retro_tap` is enabled THEN releasing after hold SHALL still emit tap

### REQ-6: Combo Detection

**User Story:** As a power user, I want to press two keys simultaneously to produce a different output (e.g., J+K = Escape), so that I can access more functions without extra keys.

#### Acceptance Criteria

1. WHEN defining a combo THEN the engine SHALL accept a set of 2+ keys and an action
2. WHEN keys are pressed within `combo_timeout_ms` THEN the engine SHALL check for matching combos
3. WHEN a combo matches THEN the engine SHALL block the original keys and execute the combo action
4. IF combo keys are released in different order THEN the combo SHALL still trigger
5. IF only some combo keys are pressed THEN the engine SHALL pass them through after timeout

### REQ-7: One-Shot Modifiers

**User Story:** As a power user, I want a "sticky" modifier that applies only to the next key, so that I don't have to hold modifier keys.

#### Acceptance Criteria

1. WHEN a one-shot modifier is activated THEN the engine SHALL mark it as "armed"
2. WHEN the next non-modifier key is pressed THEN the engine SHALL apply the modifier and deactivate it
3. IF the one-shot modifier is pressed again THEN it SHALL become "locked" (persistent)
4. IF a third press occurs THEN it SHALL be deactivated completely
5. WHEN reporting state THEN the engine SHALL distinguish between armed, locked, and inactive

### REQ-8: Timing Configuration

**User Story:** As a power user, I want to configure timing thresholds to match my typing speed, so that behaviors feel natural to me.

#### Acceptance Criteria

1. WHEN configuring the engine THEN users SHALL be able to set:
   - `tap_timeout_ms` (default: 200) - tap vs hold threshold
   - `combo_timeout_ms` (default: 50) - simultaneous key window
   - `hold_delay_ms` (default: 0) - prevent accidental holds
   - `eager_tap` (default: false) - emit tap immediately
   - `permissive_hold` (default: true) - interrupt = hold
   - `retro_tap` (default: false) - tap on release after hold
2. WHEN timing config is changed THEN it SHALL take effect immediately
3. WHEN timing config is queried THEN the current values SHALL be returned for GUI visualization

### REQ-9: Rhai Script Integration

**User Story:** As a power user, I want to define advanced behaviors in Rhai scripts, so that I have full programmable control.

#### Acceptance Criteria

1. WHEN registering tap-hold in Rhai THEN the function `tap_hold(key, tap, hold)` SHALL be available
2. WHEN registering combos in Rhai THEN the function `combo([keys], action)` SHALL be available
3. WHEN registering one-shot in Rhai THEN the function `one_shot(modifier)` SHALL be available
4. WHEN activating layers in Rhai THEN functions `layer_push(name)`, `layer_pop()`, `layer_toggle(name)` SHALL be available
5. WHEN querying state in Rhai THEN functions `is_layer_active(name)`, `is_modifier_active(mod)` SHALL be available

### REQ-10: Pending Decision Queue

**User Story:** As a developer, I want the engine to track multiple pending timing decisions, so that complex overlapping behaviors work correctly.

#### Acceptance Criteria

1. WHEN multiple tap-hold keys are pressed THEN each SHALL have its own pending decision
2. WHEN processing events THEN the engine SHALL check all pending decisions for resolution
3. WHEN a decision resolves THEN it SHALL be removed from the queue
4. IF decisions conflict THEN the engine SHALL resolve in order of creation (FIFO)
5. WHEN reporting state THEN all pending decisions SHALL be visible for debugging

### REQ-11: Emergency Exit (Safe Mode)

**User Story:** As a user, I want an always-working escape hatch, so that I can never be locked out by a bad config.

#### Acceptance Criteria

1. WHEN Ctrl+Alt+Shift+Escape is pressed THEN the engine SHALL immediately disable all remapping
2. WHEN safe mode is active THEN all keys SHALL pass through unchanged
3. WHEN the same combo is pressed again THEN the engine SHALL re-enable remapping
4. IF the engine crashes THEN the keyboard SHALL automatically return to normal (OS handles this)

## Non-Functional Requirements

### Performance
- All timing decisions must be < 100 microseconds
- Pending decision queue lookup must be O(1) or O(log n)
- No heap allocations in hot path (event processing)
- Timer checks must not block event processing

### Latency Budget
```
Total budget: 1000 microseconds
├── Event receive:     ~100 us (OS + channel)
├── State lookup:       ~10 us (key state, modifiers)
├── Decision check:     ~50 us (pending queue)
├── Script lookup:      ~50 us (remap registry)
├── Action execute:     ~10 us (emit/block)
├── Output send:       ~100 us (uinput/SendInput)
└── Margin:            ~680 us (safety buffer)
```

### Testability
- All timing logic must be testable with mock clocks
- Pending decision queue must be inspectable
- Layer stack must be queryable
- Virtual modifier state must be readable

### Reliability
- No panics in timing code paths
- Graceful handling of timer overflow
- Safe mode must never fail

### Code Quality
- Functions < 50 lines
- Files < 500 lines
- 80% test coverage on decision logic
