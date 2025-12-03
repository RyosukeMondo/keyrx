# Requirements Document

## Introduction

Device keymap resolution happens on every event processing cycle without caching. The Linux driver reads from evdev keymaps repeatedly, and Windows hook callbacks convert scan codes to virtual keys without memoization. With 100+ events/second, this creates unnecessary lookups costing 1-5μs each.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Reduce per-event latency
- **Low Latency**: Sub-millisecond targets require optimization
- **Efficiency**: Don't repeat work unnecessarily

Per tech.md: "Hook callbacks SHALL complete in < 100 microseconds"

## Requirements

### Requirement 1: Keymap Caching

**User Story:** As a user, I want fast key lookups, so that there's no perceptible lag.

#### Acceptance Criteria

1. WHEN keymap is queried THEN cached result SHALL be returned if available
2. IF cache miss occurs THEN lookup SHALL populate cache
3. WHEN cache is full THEN LRU eviction SHALL occur
4. IF cache hit rate measured THEN it SHALL exceed 95%

### Requirement 2: Cache Invalidation

**User Story:** As a user, I want correct behavior after device changes, so that keymaps stay accurate.

#### Acceptance Criteria

1. WHEN device is added THEN cache SHALL be invalidated for that device
2. IF layout changes THEN affected cache entries SHALL be cleared
3. WHEN invalidation occurs THEN it SHALL be O(1) or O(log n)
4. IF stale entry used THEN fallback SHALL be correct

### Requirement 3: Platform Support

**User Story:** As a user, I want caching on all platforms, so that performance is consistent.

#### Acceptance Criteria

1. WHEN on Linux THEN evdev keymap lookups SHALL be cached
2. IF on Windows THEN scan-to-vk conversion SHALL be memoized
3. WHEN platform differs THEN cache strategy SHALL be appropriate
4. IF platform lacks caching THEN graceful fallback SHALL exist

## Non-Functional Requirements

### Performance
- Cache lookup SHALL be O(1)
- Cache overhead SHALL be < 100 bytes per entry
- Cache hit rate SHALL exceed 95% in typical usage
- Memory usage SHALL be bounded (max 10KB for cache)

### Reliability
- Cache corruption SHALL not affect correctness
- Fallback to uncached lookup SHALL always work
