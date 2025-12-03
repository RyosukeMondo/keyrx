# Tasks Document

## Phase 1: Core Buffer

- [ ] 1. Create EventBuffer
  - File: `core/src/engine/coalescing/buffer.rs`
  - Implement time-based and size-based flushing
  - Add coalescing rules for repeats
  - Purpose: Event batching buffer
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 2. Create CoalescingConfig
  - File: `core/src/engine/coalescing/config.rs`
  - Configurable batch size and timeout
  - Sensible defaults
  - Purpose: Coalescing configuration
  - _Requirements: 3.1, 3.2, 3.4_

## Phase 2: Integration

- [ ] 3. Create CoalescingEngine wrapper
  - File: `core/src/engine/coalescing/mod.rs`
  - Wrap engine with coalescing layer
  - Preserve timing semantics
  - Purpose: Engine integration
  - _Requirements: 2.2, 2.4_

- [ ] 4. Add coalescing to event loop
  - File: `core/src/engine/event_loop.rs`
  - Integrate buffer into main loop
  - Handle flush triggers
  - Purpose: Runtime integration
  - _Requirements: 1.1, 2.3_
