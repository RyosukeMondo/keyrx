# Requirements: Add Web Support

**Spec Name**: add-web-support
**Created**: 2025-12-20
**Status**: Draft
**Priority**: High (Platform expansion)

---

## Overview

Add web browser support using Flutter Web while maintaining 100% SSOT and zero code redundancy across Windows, Linux, and Web platforms.

---

## Business Goals

1. Enable web browser access to KeyRx functionality
2. Maintain 100% SSOT in core_logic
3. Achieve zero UI code redundancy across platforms
4. Ensure consistent UX across Windows, Linux, Web
5. Detect and prevent SSOT violations at build/test time

---

## Requirements

### REQ-1: Conditional Platform Exports

**Description**: Implement platform-specific service implementations using Dart conditional exports.

**Current State**:
- Services have single implementation: `*_impl.dart`
- Only FFI transport supported
- Desktop-only architecture

**Target State**:
- Services have interface + conditional export
- FFI implementation: `*_ffi.dart` (Desktop)
- HTTP implementation: `*_web.dart` (Web)
- Automatic platform selection at compile time

**Acceptance Criteria**:
- ✅ 11 services refactored to conditional export pattern
- ✅ FFI implementations in `*_ffi.dart` files
- ✅ Web implementations in `*_web.dart` files
- ✅ Same interface for both platforms
- ✅ Zero business logic duplication

---

### REQ-2: Web Transport Layer

**Description**: Create HTTP/WebSocket transport layer for web platform.

**Components**:
- WebApiClient (HTTP REST client)
- WebSocketClient (real-time updates)
- Request/Response models
- Error handling

**Target State**:
- Location: `packages/core_logic/lib/web/`
- HTTP client for API calls
- WebSocket for real-time events
- Same interface as FFI bridge

**Acceptance Criteria**:
- ✅ WebApiClient implements all service operations
- ✅ WebSocketClient provides real-time updates
- ✅ Error handling matches FFI behavior
- ✅ Type-safe request/response models
- ✅ Zero Flutter dependencies

---

### REQ-3: Backend API Service

**Description**: Create REST API server that wraps core_logic with FFI.

**Architecture**:
```
Web Browser → HTTP → Backend API → core_logic (FFI) → Rust Core
```

**Target State**:
- Location: `apps/keyrx_server/`
- REST API endpoints for all services
- WebSocket support for real-time events
- Uses core_logic directly (SSOT maintained)

**Acceptance Criteria**:
- ✅ REST API for all 11 services
- ✅ WebSocket real-time event broadcasting
- ✅ CORS support for web clients
- ✅ Authentication/authorization middleware
- ✅ Uses core_logic (no logic duplication)

---

### REQ-4: Multi-Platform ServiceRegistry

**Description**: Update ServiceRegistry to create platform-appropriate implementations.

**Platform Detection**:
- Desktop: Create FFI-based services
- Web: Create HTTP-based services

**Target State**:
- ServiceRegistry.real() detects platform automatically
- Web mode creates WebApiClient
- Desktop mode creates KeyrxBridge
- Same interface for all consumers

**Acceptance Criteria**:
- ✅ Platform detection using `kIsWeb`
- ✅ Automatic service implementation selection
- ✅ No UI code changes required
- ✅ Environment-based backend URL configuration

---

### REQ-5: SSOT Violation Detection (Build Time)

**Description**: Implement automated checks to detect SSOT violations at build time.

**Violation Types**:
1. Business logic in UI layer
2. Flutter imports in core_logic
3. Duplicate interface definitions
4. Inconsistent method signatures across platforms
5. Direct FFI usage outside core_logic

**Detection Mechanisms**:
- Custom analyzer rules
- Pre-commit hooks
- CI/CD checks

**Target State**:
- Location: `tools/ssot_checker/`
- Runs on pre-commit and CI
- Fails build on violations
- Clear error messages with fix suggestions

**Acceptance Criteria**:
- ✅ Detects Flutter imports in core_logic
- ✅ Detects business logic in UI layer
- ✅ Verifies interface consistency (FFI vs Web)
- ✅ Checks for duplicate implementations
- ✅ Reports violations with file:line locations
- ✅ Integrated into pre-commit hooks

---

### REQ-6: SSOT Violation Detection (Test Time)

**Description**: Create integration tests that verify SSOT compliance.

**Test Categories**:
1. Cross-platform interface consistency
2. Business logic location verification
3. Transport layer isolation
4. Real-time event parity

**Target State**:
- Location: `packages/core_logic/test/ssot/`
- Runs in CI/CD pipeline
- Fails on SSOT violations

**Acceptance Criteria**:
- ✅ Test verifies FFI and Web implement same interface
- ✅ Test ensures no business logic in transport layers
- ✅ Test validates all platforms use same core_logic code
- ✅ Test checks real-time events work on both platforms
- ✅ Mocks verify only transport layer differs

---

### REQ-7: Flutter Web Build Configuration

**Description**: Configure Flutter project for web compilation.

**Configuration**:
- Web build settings
- Backend URL configuration
- CORS headers
- WebSocket connection handling

**Acceptance Criteria**:
- ✅ Flutter web builds successfully
- ✅ Environment-based backend URL
- ✅ Hot reload works for development
- ✅ Production build optimized

---

## Non-Functional Requirements

### NFR-1: Zero Code Redundancy

All business logic must exist only in core_logic. No duplication between platforms.

**Measurement**: Automated SSOT checker reports 0% redundancy.

### NFR-2: Zero Flutter Dependencies in core_logic

core_logic must remain pure Dart for web compatibility.

**Verification**: Build-time check for `import 'package:flutter'` in core_logic.

### NFR-3: Platform-Transparent UI

Flutter UI must work on all platforms without platform-specific code.

**Verification**: Single UI codebase compiles to Windows, Linux, Web.

### NFR-4: Real-Time Parity

Web platform must receive same real-time updates as desktop.

**Verification**: Integration tests verify WebSocket events match FFI events.

### NFR-5: Performance Parity

Web platform should have comparable performance to desktop.

**Benchmark**: API response times within 20% of FFI calls.

---

## Success Metrics

- ✅ Flutter builds for Windows, Linux, Web from same codebase
- ✅ 100% UI code reuse (0% redundancy)
- ✅ 100% business logic in core_logic (0% redundancy)
- ✅ SSOT checker reports 0 violations
- ✅ All integration tests pass on all platforms
- ✅ Web platform fully functional with real-time updates

---

## Out of Scope

- Mobile support (iOS/Android)
- Offline mode for web
- Progressive Web App (PWA) features
- Backend horizontal scaling
- Multi-user collaboration
- Authentication system (use basic auth)

---

## Dependencies

- Flutter 3.16+ (web support)
- Dart 3.2+ (conditional exports)
- shelf 1.4+ (web server)
- web_socket_channel 2.4+ (WebSocket)

---

## Risks & Mitigation

### Risk 1: SSOT Violations During Development

**Impact**: Code duplication, maintenance burden

**Mitigation**:
- Automated SSOT checker in pre-commit
- Clear guidelines in contributing docs
- Code review checklist

### Risk 2: Web Performance vs Desktop

**Impact**: User experience degradation

**Mitigation**:
- Benchmark tests in CI
- WebSocket for real-time updates (not polling)
- Optimize backend API responses

### Risk 3: Platform-Specific Bugs

**Impact**: Inconsistent behavior

**Mitigation**:
- Cross-platform integration tests
- Test on Chrome, Firefox, Safari
- Platform-specific CI jobs
