# Tasks Document

_Status: Priority #3 in 2025 implementation order; open items: 4 (in-progress), 5-6 pending. Complete delta pipeline before higher-layer features._

## Phase 1: Core Types

- [x] 1. Create StateChange enum
  - File: `core/src/engine/state/delta.rs`
  - Define all change types
  - Add serialization
  - Purpose: Change representation
  - _Requirements: 1.1_

- [x] 2. Create StateDelta type
  - File: `core/src/engine/state/delta.rs`
  - Version tracking
  - Change collection
  - Purpose: Delta container
  - _Requirements: 1.1, 2.1, 2.2_

- [x] 3. Create DeltaTracker
  - File: `core/src/engine/state/tracker.rs`
  - Record changes as they occur
  - Generate deltas on request
  - Purpose: Change tracking
  - _Requirements: 2.1, 3.2_

## Phase 2: Integration

- [x] 4. Integrate delta tracking into Engine
  - File: `core/src/engine/mod.rs`
  - Record changes on state mutations
  - Purpose: Engine integration
  - _Requirements: 1.1_

- [x] 5. Update FFI to send deltas
  - File: `core/src/ffi/exports_engine.rs`
  - Send delta instead of full state
  - Fallback to full on mismatch
  - Purpose: FFI integration
  - _Requirements: 1.2, 1.4, 2.3_

- [x] 6. Update Flutter to apply deltas
  - File: `ui/lib/state/app_state.dart`
  - Apply deltas to local state
  - Request full sync on error
  - Purpose: Flutter integration
  - _Requirements: 1.3, 2.3_
