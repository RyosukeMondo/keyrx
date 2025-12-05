# Tasks Document

## Phase 1: Core Cache

- [x] 1. Create ScriptCache
  - File: `core/src/scripting/cache.rs`
  - Content-addressable storage
  - LRU eviction
  - Purpose: AST caching
  - _Requirements: 1.1, 1.2, 3.2_

- [x] 2. Implement AST serialization
  - File: `core/src/scripting/cache.rs`
  - Serialize/deserialize Rhai AST
  - Handle version compatibility
  - Purpose: Persistent cache
  - _Requirements: 1.1_

## Phase 2: Integration

- [x] 3. Integrate cache into runtime
  - File: `core/src/scripting/runtime.rs`
  - Check cache before compile
  - Store compiled AST
  - Purpose: Runtime integration
  - _Requirements: 1.1, 1.4_

- [x] 4. Add CLI cache options
  - File: `core/src/cli/commands/run.rs`
  - Add `--no-cache` and `--clear-cache`
  - Purpose: User control
  - _Requirements: 3.3, 3.4_

- [ ] 5. Add cache metrics
  - File: `core/src/scripting/cache.rs`
  - Track hit rate, size, startup improvement
  - Purpose: Performance validation
  - _Requirements: Non-functional_
