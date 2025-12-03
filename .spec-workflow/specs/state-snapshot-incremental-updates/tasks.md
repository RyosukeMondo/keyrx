# Tasks Document

## Phase 1: Core Types

- [ ] 1. Create StateChange enum
  - File: `core/src/engine/state/delta.rs`
  - Define all change types
  - Add serialization
  - Purpose: Change representation
  - _Requirements: 1.1_

- [ ] 2. Create StateDelta type
  - File: `core/src/engine/state/delta.rs`
  - Version tracking
  - Change collection
  - Purpose: Delta container
  - _Requirements: 1.1, 2.1, 2.2_

- [ ] 3. Create DeltaTracker
  - File: `core/src/engine/state/tracker.rs`
  - Record changes as they occur
  - Generate deltas on request
  - Purpose: Change tracking
  - _Requirements: 2.1, 3.2_

## Phase 2: Integration

- [ ] 4. Integrate delta tracking into Engine
  - File: `core/src/engine/mod.rs`
  - Record changes on state mutations
  - Purpose: Engine integration
  - _Requirements: 1.1_

- [ ] 5. Update FFI to send deltas
  - File: `core/src/ffi/exports_engine.rs`
  - Send delta instead of full state
  - Fallback to full on mismatch
  - Purpose: FFI integration
  - _Requirements: 1.2, 1.4, 2.3_

- [ ] 6. Update Flutter to apply deltas
  - File: `ui/lib/state/app_state.dart`
  - Apply deltas to local state
  - Request full sync on error
  - Purpose: Flutter integration
  - _Requirements: 1.3, 2.3_
