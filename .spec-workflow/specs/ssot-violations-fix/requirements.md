# Requirements Document

## Introduction

This specification addresses the remaining Single Source of Truth (SSOT) violations in the KeyRX codebase. SSOT violations occur when classes, business logic, or interfaces are duplicated between the UI and core_logic layers, violating the principle that each piece of knowledge should exist in exactly one location. These violations create maintenance burden, potential inconsistencies, and architectural debt.

The remaining 37 violations break down as:
- **3 duplicate classes** (LogEntry, DeviceRegistryService, DeviceRegistryServiceImpl)
- **32 business logic in UI violations** (direct FFI bridge usage)
- **2 interface inconsistency violations**

This work builds on previous SSOT cleanup that successfully eliminated 19 violations (34% reduction from 56 to 37).

## Alignment with Product Vision

This refactoring supports the architectural goal of maintaining clean separation between UI and core logic layers, ensuring:
- **Maintainability**: Changes need to happen in one place only
- **Testability**: Core logic can be tested independently of UI
- **Reusability**: Core logic can be shared across Flutter UI and CLI tools
- **Platform Independence**: Core logic remains pure Dart without Flutter dependencies

## Requirements

### Requirement 1: Eliminate Duplicate Classes

**User Story:** As a developer, I want duplicate classes consolidated into core_logic, so that I maintain a single source of truth for data structures

#### Acceptance Criteria

1. WHEN LogEntry class exists in both UI and core_logic THEN the UI SHALL import from core_logic OR rename its class to avoid conflict
2. WHEN DeviceRegistryService exists in both layers THEN the UI version SHALL be renamed or refactored to clearly indicate its role as a UI adapter
3. WHEN DeviceRegistryServiceImpl exists in both layers THEN duplicate implementation SHALL be eliminated while preserving UI-specific functionality

### Requirement 2: Move Business Logic to Core Layer

**User Story:** As a developer, I want FFI bridge usage isolated in core_logic services, so that UI components remain platform-independent and testable

#### Acceptance Criteria

1. WHEN UI code directly calls FFI bridge methods THEN it SHALL use core_logic service wrappers instead
2. WHEN device_stream_provider uses FFI bridge THEN it SHALL delegate to core_logic DeviceService
3. WHEN providers_setup uses FFI bridge THEN it SHALL use dependency injection with core_logic services
4. WHEN keyflow_repository uses FFI bridge THEN it SHALL delegate to appropriate core_logic services
5. WHEN FlowValidator contains business logic THEN it SHALL move to core_logic OR be clearly identified as UI-specific validation
6. WHEN MappingValidator contains business logic THEN it SHALL move to core_logic OR be clearly identified as UI-specific validation

### Requirement 3: Fix Interface Inconsistencies

**User Story:** As a developer, I want service interfaces to be consistent across implementations, so that services are properly polymorphic and testable

#### Acceptance Criteria

1. WHEN a service has FFI and web implementations THEN both SHALL implement the same interface with matching signatures
2. WHEN interface inconsistencies exist THEN they SHALL be resolved by aligning method signatures
3. IF platform-specific methods are needed THEN they SHALL be documented as extensions with clear reasoning

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Source of Truth**: Each class exists in exactly one location
- **Layer Separation**: UI depends on core_logic, never the reverse
- **Platform Independence**: Core logic contains no Flutter dependencies
- **Service Wrappers**: All FFI access goes through well-defined service interfaces

### Performance
- Refactoring SHALL NOT degrade runtime performance
- Service indirection SHALL be minimal (inline or simple delegation)

### Compatibility
- Existing functionality SHALL be preserved
- UI behavior SHALL remain unchanged from user perspective
- Breaking changes SHALL be avoided in public APIs

### Testing
- Core logic services SHALL have unit tests
- UI components SHALL be testable with mocked services
- Integration tests SHALL verify UI-to-core communication
