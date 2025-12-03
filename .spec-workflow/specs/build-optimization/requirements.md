# Requirements Document

## Introduction

The Rust build uses tokio "full" feature despite only needing async runtime. There are no platform-specific feature gates, causing unnecessary compilation. Build times are longer than necessary. This spec optimizes build configuration with minimal features and proper platform gates.

## Alignment with Product Vision

This feature supports KeyRx's product principles:
- **Performance > Features**: Smaller binaries, faster builds
- **Platform Focus**: Only compile what's needed per platform
- **Developer Experience**: Faster iteration cycles

Per tech.md: "No unnecessary dependencies"

## Requirements

### Requirement 1: Dependency Optimization

**User Story:** As a developer, I want minimal dependencies, so that builds are fast and binaries small.

#### Acceptance Criteria

1. WHEN a dependency is used THEN only needed features SHALL be enabled
2. IF a feature is not used THEN it SHALL be disabled
3. WHEN tokio is needed THEN only required features SHALL be included
4. IF serde is used THEN derive feature SHALL be conditional

### Requirement 2: Platform Feature Gates

**User Story:** As a developer, I want platform-specific code gated, so that cross-compilation works cleanly.

#### Acceptance Criteria

1. WHEN Windows code exists THEN it SHALL be behind #[cfg(windows)]
2. IF Linux code exists THEN it SHALL be behind #[cfg(target_os = "linux")]
3. WHEN dependencies are platform-specific THEN Cargo.toml SHALL reflect this
4. IF a feature is platform-only THEN it SHALL be documented

### Requirement 3: Build Time Optimization

**User Story:** As a developer, I want fast builds, so that I can iterate quickly.

#### Acceptance Criteria

1. WHEN building in dev mode THEN incremental builds SHALL be fast
2. IF unused code exists THEN it SHALL not be compiled
3. WHEN dependencies update THEN only affected crates SHALL rebuild
4. IF parallel compilation helps THEN it SHALL be configured

### Requirement 4: Binary Size Optimization

**User Story:** As a user, I want small binaries, so that downloads are fast.

#### Acceptance Criteria

1. WHEN releasing THEN binaries SHALL be optimized for size
2. IF symbols are not needed THEN they SHALL be stripped
3. WHEN LTO is beneficial THEN it SHALL be enabled for release
4. IF dead code exists THEN it SHALL be eliminated

## Non-Functional Requirements

### Code Architecture and Modularity
- **Feature Flags**: Clear feature boundaries
- **Conditional Compilation**: Platform code isolated
- **Dependency Management**: Minimal feature sets

### Build Performance
- Dev build time SHALL decrease by > 20%
- Release build time SHALL not regress significantly
- Incremental builds SHALL complete in < 10 seconds

### Binary Size
- Release binaries SHALL decrease by > 10%
- Debug symbols SHALL be separate files
- Unused code SHALL be eliminated
