# Tasks Document

## Phase 1: Core Cache

- [ ] 1. Create KeymapCache trait
  - File: `core/src/drivers/common/cache.rs`
  - Define cache interface with get/insert/invalidate
  - Add CacheStats type
  - Purpose: Platform-agnostic cache interface
  - _Requirements: 1.1, 1.2_

- [ ] 2. Implement LruKeymapCache
  - File: `core/src/drivers/common/cache.rs`
  - Use lru crate for LRU eviction
  - Thread-safe with Mutex
  - Purpose: Generic LRU cache implementation
  - _Requirements: 1.1, 1.3, 1.4_

## Phase 2: Platform Integration

- [ ] 3. Add cache to Linux keymap
  - File: `core/src/drivers/linux/keymap.rs`
  - Cache evdev scan code lookups
  - Invalidate on device changes
  - Purpose: Linux keymap caching
  - _Requirements: 3.1_

- [ ] 4. Add cache to Windows keymap
  - File: `core/src/drivers/windows/keymap.rs`
  - Cache scan-to-vk conversions
  - Memoize MapVirtualKey calls
  - Purpose: Windows keymap caching
  - _Requirements: 3.2_

## Phase 3: Validation

- [ ] 5. Add cache benchmarks and metrics
  - File: `core/benches/keymap_cache_bench.rs`
  - Measure hit rate under load
  - Verify latency improvement
  - Purpose: Performance validation
  - _Requirements: 1.4, Non-functional (performance)_
