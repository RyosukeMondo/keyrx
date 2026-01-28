# Bug Remediation Sweep - Task Breakdown

## WS1: Memory Management (Critical)

### MEM-001: Dashboard Subscription Memory Leak
**File**: keyrx_ui/src/components/Dashboard.tsx:75-150
**Priority**: Critical
**Status**: Pending

**Problem**: Subscriptions multiply on pause/unpause without cleanup
**Fix**:
```typescript
useEffect(() => {
  const subscription = wsContext.subscribe('metrics', handleMetrics);
  return () => subscription.unsubscribe(); // Add cleanup
}, [wsContext]); // Add dependencies
```

**Tests**:
- [ ] Test subscription count stays constant on pause/unpause cycles
- [ ] Test subscriptions cleaned up on unmount
- [ ] Memory leak detection test (monitor heap over 100 pause/unpause cycles)

### MEM-002: WebSocket Server-Side Subscription Leak
**File**: keyrx_daemon/src/web/ws.rs:120-180
**Priority**: Critical
**Status**: Pending

**Problem**: Orphaned subscriptions when clients disconnect
**Fix**:
- Add Drop guard for Subscription
- Track subscriptions in HashMap<ConnectionId, Vec<SubscriptionId>>
- Clean up on connection close

**Tests**:
- [ ] Test subscriptions removed on disconnect
- [ ] Test multiple clients don't leak subscriptions
- [ ] Stress test: 1000 connect/disconnect cycles

### MEM-003: Unbounded WebSocket Queue Growth
**File**: keyrx_daemon/src/daemon/event_broadcaster.rs:45-90
**Priority**: Critical
**Status**: Pending

**Problem**: Slow clients cause unbounded queue growth → OOM
**Fix**:
- Add bounded channel (capacity: 1000)
- Implement backpressure strategy (drop oldest or disconnect slow clients)
- Add queue size metrics

**Tests**:
- [ ] Test queue stays bounded under slow client
- [ ] Test backpressure triggers correctly
- [ ] Test metrics report queue size

## WS2: WebSocket Infrastructure (Critical/High)

### WS-001: Missing Health Check Responses
**File**: keyrx_daemon/src/web/ws.rs:200-220
**Priority**: High
**Status**: Pending

**Fix**: Add ping/pong frame handling with timeout detection

### WS-002: Incorrect Reconnection Logic
**File**: keyrx_ui/src/hooks/useWebSocket.ts:80-120
**Priority**: High
**Status**: Pending

**Fix**: Implement exponential backoff with max attempts

### WS-003: Race Conditions in Event Broadcasting
**File**: keyrx_daemon/src/daemon/event_broadcaster.rs:120-180
**Priority**: Critical
**Status**: Pending

**Fix**: Add RwLock around subscribers map, ensure atomic operations

### WS-004: Message Ordering Issues
**File**: keyrx_daemon/src/web/ws.rs:250-300
**Priority**: High
**Status**: Pending

**Fix**: Add sequence numbers to messages, buffer out-of-order messages

### WS-005: Duplicate Message Delivery
**File**: keyrx_daemon/src/daemon/event_broadcaster.rs:200-250
**Priority**: High
**Status**: Pending

**Fix**: Track delivered message IDs per subscriber, deduplicate

## WS3: Profile Management (High)

### PROF-001: Profile Switching Race Conditions
**File**: keyrx_daemon/src/profiles/service.rs:150-200
**Priority**: High
**Status**: Pending

**Fix**: Add Mutex around profile switching, serialize activate() calls

### PROF-002: Missing Validation in Profile Operations
**File**: keyrx_daemon/src/profiles/manager.rs:100-150
**Priority**: High
**Status**: Pending

**Fix**: Validate profile names (regex: ^[a-zA-Z0-9_-]{1,64}$)

### PROF-003: Incomplete Error Handling
**File**: keyrx_daemon/src/web/api/profiles.rs:All endpoints
**Priority**: Medium
**Status**: Pending

**Fix**: Return structured errors with error codes

### PROF-004: Missing Activation Metadata
**File**: keyrx_daemon/src/profiles/manager.rs:activate()
**Priority**: Medium
**Status**: Pending

**Fix**: Store activation timestamp, activator info

### PROF-005: Duplicate Profile Names Allowed
**File**: keyrx_daemon/src/profiles/manager.rs:create()
**Priority**: Medium
**Status**: Pending

**Fix**: Check for existing profile before creating

## WS4: API Layer (High/Medium)

### API-001: Type Mismatches in Responses
**Priority**: High
**Status**: Pending

**Files**:
- keyrx_daemon/src/web/api/profiles.rs:list_profiles
- keyrx_daemon/src/web/api/simulator.rs:get_status

**Fix**: Ensure frontend types match backend schemas

### API-002 through API-010: Various API Issues
**Priority**: Medium
**Status**: Pending

**Fixes**:
- Add missing fields to responses
- Standardize error format
- Add request validation
- Add path parameter validation

## WS5: Security Hardening (Critical/High)

### SEC-001: Missing Authentication Layer
**Priority**: Critical
**Status**: Pending

**Fix**: Add JWT-based authentication middleware
**Files**:
- keyrx_daemon/src/auth/mod.rs (new)
- keyrx_daemon/src/web/middleware/auth.rs (new)

### SEC-002: CORS Misconfiguration
**Priority**: Critical
**Status**: Pending

**Fix**: Restrict CORS to localhost only in production mode

### SEC-003: Path Traversal Vulnerabilities
**Priority**: Critical
**Status**: Pending

**Fix**: Use PathBuf::canonicalize() and validate paths

### SEC-004 through SEC-012: Additional Security Issues
**Priority**: High
**Status**: Pending

**Fixes**:
- Add rate limiting
- Add request size limits
- Add timeout protection
- Add input sanitization

## WS6: UI Component Fixes (Medium)

### UI-001: Missing Null Checks
**Files**: Multiple components
**Priority**: Medium
**Status**: Pending

### UI-002: Unsafe Type Assertions
**Files**: Multiple components
**Priority**: Medium
**Status**: Pending

### UI-003 through UI-015: Various UI Issues
**Priority**: Medium/Low
**Status**: Pending

## WS7: Data Validation (High)

### VAL-001 through VAL-005: Validation Issues
**Priority**: High
**Status**: Pending

**Fixes**:
- Add comprehensive input validation
- Add file size limits
- Add content validation
- Add sanitization

## WS8: Testing Infrastructure (Medium)

### TEST-001: Memory Leak Detection Tests
**Priority**: High
**Status**: Pending

**Create**:
- keyrx_daemon/tests/memory_leak_test.rs
- keyrx_ui/tests/memory-leak.test.tsx

### TEST-002: Concurrency Tests
**Priority**: High
**Status**: Pending

**Create**:
- keyrx_daemon/tests/concurrency_test.rs

### TEST-003: E2E Integration Tests
**Priority**: High
**Status**: Pending

**Create**:
- keyrx_daemon/tests/bug_remediation_e2e_test.rs

## Summary

**Total Bugs**: 67+
- Critical: 15
- High: 19
- Medium: 23
- Low: 10

**Parallel Workstreams**: 8
**Estimated Timeline**: 7-9 days with 8 agents
**Test Coverage Target**: 100% for fixed bugs
