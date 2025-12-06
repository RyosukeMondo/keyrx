# Tasks Document

_Status: Priority #6 in 2025 implementation order; all items pending. Focus on compositor/cross-layout modifiers with priority handling._

## Phase 1: Core Types

- [x] 1. Create Layout entity
  - File: `core/src/engine/layout/mod.rs`
  - Separate layout from layer stack
  - Add layout metadata
  - Purpose: Layout abstraction
  - _Requirements: 1.1_

- [x] 2. Create LayoutCompositor
  - File: `core/src/engine/layout/compositor.rs`
  - Manage multiple layouts
  - Priority-based resolution
  - Purpose: Layout composition
  - _Requirements: 1.1, 1.2, 1.4, 3.1, 3.2, 3.4_

## Phase 2: Modifier System

- [x] 3. Implement cross-layout modifiers
  - File: `core/src/engine/layout/modifiers.rs`
  - Modifier scoping
  - Shared modifier state
  - Purpose: Modifier coordination
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

## Phase 3: Integration

- [ ] 4. Integrate compositor into engine
  - File: `core/src/engine/mod.rs`
  - Replace single layout with compositor
  - Handle transitions
  - Purpose: Engine integration
  - _Requirements: 1.3_

- [ ] 5. Add Rhai bindings
  - File: `core/src/scripting/bindings.rs`
  - Layout composition functions
  - Priority control
  - Purpose: Script access
  - _Requirements: 1.1, 3.3_

- [ ] 6. Update FFI and Flutter
  - Files: FFI exports, Flutter UI
  - Expose multi-layout state
  - UI for layout management
  - Purpose: Full stack integration
  - _Requirements: 1.1_
