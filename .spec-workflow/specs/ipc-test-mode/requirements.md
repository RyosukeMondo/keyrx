# Requirements: IPC Test Mode for E2E Testing

## 1. Overview

Enable 5 failing IPC-dependent E2E tests by adding test mode that runs daemon with full IPC infrastructure, allowing profile activation and daemon status queries.

## 2. Problem Statement

**Current Status:** 5 tests failing due to IPC dependency
**Failing Tests:**
- `status-001` - GET /api/status (daemon_running field requires IPC)
- `integration-001` - Profile lifecycle (profile activation requires IPC)
- `workflow-002` - Profile duplicate→rename→activate (profile activation)
- `workflow-003` - Profile validation→fix→activate (profile activation)
- `workflow-007` - Simulator event → mapping → output (profile activation)

**Root Cause:** E2E tests run daemon in 'run' mode without full IPC socket. Profile activation and daemon status queries require IPC communication.

## 3. Functional Requirements

### 3.1 Test Mode Launch
- **REQ-3.1.1**: Add `--test-mode` flag to daemon CLI
- **REQ-3.1.2**: Test mode must initialize full IPC infrastructure
- **REQ-3.1.3**: Test mode must enable profile activation via REST API
- **REQ-3.1.4**: Test mode must populate daemon_running field in status responses

### 3.2 IPC Infrastructure
- **REQ-3.2.1**: Create IPC socket for inter-process communication
- **REQ-3.2.2**: Start IPC message handler in test mode
- **REQ-3.2.3**: Support profile activation commands via IPC
- **REQ-3.2.4**: Support daemon status queries via IPC

### 3.3 REST API Integration
- **REQ-3.3.1**: POST /api/profiles/:name/activate must use IPC in test mode
- **REQ-3.3.2**: GET /api/status must query IPC for daemon_running field
- **REQ-3.3.3**: API calls must timeout after 5 seconds if IPC unavailable
- **REQ-3.3.4**: Clear error messages if IPC required but unavailable

## 4. Non-Functional Requirements

### 4.1 Performance
- **REQ-4.1.1**: IPC round-trip < 50ms
- **REQ-4.1.2**: Test mode startup < 2 seconds
- **REQ-4.1.3**: No performance impact on non-test modes

### 4.2 Reliability
- **REQ-4.2.1**: IPC socket must cleanup on daemon shutdown
- **REQ-4.2.2**: Graceful degradation if IPC fails
- **REQ-4.2.3**: Clear error messages for IPC communication failures

### 4.3 Security
- **REQ-4.3.1**: Test mode only enabled via explicit --test-mode flag
- **REQ-4.3.2**: IPC socket restricted to local connections only
- **REQ-4.3.3**: Test mode disabled in production builds (debug assertions)

## 5. Acceptance Criteria

- ✅ All 5 IPC-dependent tests pass in test mode
- ✅ `status-001` test passes (daemon_running field populated)
- ✅ Profile activation works via REST API in test mode
- ✅ IPC round-trip latency < 50ms
- ✅ Zero security vulnerabilities in test mode

## 6. Out of Scope

- ❌ Full daemon functionality (keyboard capture, event injection)
- ❌ Multi-instance IPC (only single daemon instance)
- ❌ IPC authentication/encryption
- ❌ Cross-platform IPC (focus on Linux first)

## 7. Alternative Solutions

### Option A: Mock IPC Responses (Rejected)
**Why:** Too complex to mock all IPC interactions, brittle

### Option B: Skip IPC-Dependent Tests (Rejected)
**Why:** Loses test coverage, defeats purpose of E2E testing

### Option C: Full Daemon with IPC (Selected)
**Why:** Simple, tests real code paths, minimal changes required

## 8. Success Metrics

- ✅ 100% E2E test pass rate (83/83) in test mode
- ✅ IPC latency < 50ms
- ✅ Test mode startup < 2 seconds
- ✅ Zero test flakiness related to IPC
