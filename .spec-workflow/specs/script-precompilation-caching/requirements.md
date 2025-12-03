# Requirements Document

## Introduction

Rhai scripts are parsed and compiled fresh on each `keyrx run` invocation without caching ASTs. For complex scripts, this adds 50-200ms to startup. Caching compiled scripts based on content hash eliminates redundant compilation.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Fast startup
- **Developer Experience**: Quick iteration
- **Efficiency**: Don't repeat work

## Requirements

### Requirement 1: Script Caching

**User Story:** As a user, I want fast startup, so that I can quickly test changes.

#### Acceptance Criteria

1. WHEN script unchanged THEN cached AST SHALL be used
2. IF script modified THEN recompilation SHALL occur
3. WHEN cache hit THEN startup SHALL be < 50ms faster
4. IF cache miss THEN normal compilation SHALL proceed

### Requirement 2: Cache Validation

**User Story:** As a developer, I want correct caching, so that stale scripts aren't used.

#### Acceptance Criteria

1. WHEN script content changes THEN cache SHALL invalidate
2. IF file mtime changes THEN validation SHALL check content hash
3. WHEN dependencies change THEN dependent scripts SHALL recompile
4. IF validation fails THEN fallback to fresh compile SHALL occur

### Requirement 3: Cache Management

**User Story:** As a user, I want manageable cache, so that disk space is controlled.

#### Acceptance Criteria

1. WHEN cache directory exists THEN it SHALL be in `.keyrx_cache/`
2. IF cache too large THEN LRU eviction SHALL occur
3. WHEN `--no-cache` passed THEN cache SHALL be bypassed
4. IF `--clear-cache` passed THEN cache SHALL be deleted

## Non-Functional Requirements

### Performance
- Cache lookup SHALL be < 5ms
- Cache storage SHALL be < 10MB total
- Startup improvement SHALL be measurable (> 20%)
