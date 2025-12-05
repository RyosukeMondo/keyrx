# Tasks Document

## Phase 1: Core Enforcement

- [x] 1. Create ResourceEnforcer
  - File: `core/src/engine/limits/enforcer.rs`
  - Timeout, memory, queue tracking
  - Atomic counters for thread safety
  - Purpose: Resource monitoring
  - _Requirements: 1.1, 2.1, 3.1_

- [x] 2. Create ResourceLimits config
  - File: `core/src/engine/limits/config.rs`
  - Configurable limits with defaults
  - Serialize/deserialize support
  - Purpose: Limit configuration
  - _Requirements: 1.3, 2.3, 3.3_

## Phase 2: Integration

- [x] 3. Add timeout enforcement
  - File: `core/src/engine/event_loop.rs`
  - ExecutionGuard for script calls
  - Async interrupt on timeout
  - Purpose: Timeout enforcement
  - _Requirements: 1.1, 1.2, 1.4_

- [x] 4. Add memory tracking
  - File: `core/src/scripting/runtime.rs`
  - Track allocations in scripts
  - Terminate on limit
  - Purpose: Memory enforcement
  - _Requirements: 2.1, 2.2, 2.4_

- [x] 5. Add queue limits
  - File: `core/src/engine/output.rs`
  - Bound output queue
  - Drop oldest on overflow
  - Purpose: Queue enforcement
  - _Requirements: 3.1, 3.2, 3.4_
