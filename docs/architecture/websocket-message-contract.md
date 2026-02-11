# WebSocket Message Contract

**This document defines the contract between frontend and backend WebSocket communication.**

## Purpose

This contract ensures that frontend and backend stay in sync regarding WebSocket message formats. Any changes to message structure must be reflected in both systems and verified by contract tests.

## Current State (as of 2026-01-03)

### Legacy Format (Currently Used)

Backend sends messages in the `DaemonEvent` format:

```typescript
// General structure
{
  "type": "latency" | "state" | "event",
  "payload": { ...eventData }
}
```

#### Latency Message
```json
{
  "type": "latency",
  "payload": {
    "min": 100,
    "avg": 250,
    "max": 500,
    "p95": 400,
    "p99": 480,
    "timestamp": 1234567890
  }
}
```

#### State Message
```json
{
  "type": "state",
  "payload": {
    "modifiers": ["MD_00"],
    "locks": ["LK_00"],
    "layer": "base"
  }
}
```

#### Key Event Message
```json
{
  "type": "event",
  "payload": {
    "timestamp": 1234567890,
    "key_code": "KEY_A",
    "event_type": "press",
    "input": "KEY_A",
    "output": "KEY_B",
    "latency": 150
  }
}
```

### New RPC Format (Planned)

The new format wraps events in a standardized RPC envelope:

```typescript
{
  "type": "event",
  "channel": "latency" | "daemon-state" | "events",
  "data": { ...eventData }
}
```

#### Examples

**Latency Event:**
```json
{
  "type": "event",
  "channel": "latency",
  "data": {
    "min": 100,
    "avg": 250,
    "max": 500,
    "p95": 400,
    "p99": 480,
    "timestamp": 1234567890
  }
}
```

**State Event:**
```json
{
  "type": "event",
  "channel": "daemon-state",
  "data": {
    "modifiers": ["MD_00"],
    "locks": ["LK_00"],
    "layer": "base"
  }
}
```

**RPC Response:**
```json
{
  "type": "response",
  "id": "req-123",
  "result": { ...data }
}
```

**Error Response:**
```json
{
  "type": "response",
  "id": "req-123",
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

## Message Type Mapping

| Legacy Type | New Channel | Description |
|-------------|-------------|-------------|
| `latency` | `latency` | Latency statistics (1s interval) |
| `state` | `daemon-state` | Daemon state (modifiers, locks, layer) |
| `event` | `events` | Individual key events |
| `heartbeat` | *(ignored)* | Keep-alive messages |

## Testing Strategy

### 1. Contract Tests (Prevent Breaking Changes)

**Backend:** `keyrx_daemon/tests/websocket_contract_test.rs`
- Verify DaemonEvent serialization format
- Document expected message structure
- Act as early warning for format changes

**Frontend:** `keyrx_ui/tests/hooks/useUnifiedApi.test.tsx`
- Verify message parsing and handling
- Test both legacy and new formats
- Ensure backward compatibility

### 2. When to Update Tests

**Add a new event type:**
1. Add variant to `DaemonEvent` enum (backend)
2. Update `websocket_contract_test.rs` with new format test
3. Update `useUnifiedApi.test.tsx` to handle new type
4. Update this contract document

**Change message format:**
1. Update contract tests FIRST (TDD approach)
2. Implement changes in backend
3. Implement changes in frontend
4. Verify all tests pass
5. Update this document

### 3. Running Contract Tests

```bash
# Backend contract tests
cd keyrx_daemon
cargo test websocket_contract

# Frontend contract tests
cd keyrx_ui
npm run test -- useUnifiedApi.test.tsx

# Run all tests
make test
```

## Migration Plan (Legacy → RPC Format)

### Phase 1: Dual Support (Current)
- ✅ Backend sends legacy format
- ✅ Frontend handles both legacy and RPC formats
- ✅ Contract tests document both formats

### Phase 2: Backend Migration
- [ ] Update `keyrx_daemon/src/web/ws.rs` to wrap DaemonEvent in ServerMessage::Event
- [ ] Update backend contract tests to expect new format
- [ ] Verify frontend continues to work (backward compatibility)

### Phase 3: Frontend Cleanup
- [ ] Wait for all clients to update
- [ ] Remove legacy format handler from useUnifiedApi.ts
- [ ] Update frontend contract tests to expect only new format
- [ ] Remove legacy format documentation

### Phase 4: Complete Migration
- [ ] Remove DaemonEvent enum (replace with ServerMessage)
- [ ] Update all documentation
- [ ] Mark migration complete

## Breaking Change Protocol

**If you must make a breaking change:**

1. **Notify team** - Breaking changes affect all developers
2. **Update contract tests** - Make them fail intentionally
3. **Implement dual support** - Support both old and new formats
4. **Version the protocol** - Add version field to messages
5. **Deprecation period** - Warn clients for 2 releases
6. **Remove old format** - Only after deprecation period
7. **Update docs** - Reflect current reality

## Error Codes (JSON-RPC 2.0)

| Code | Meaning | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid request | Not a valid Request object |
| -32601 | Method not found | Method doesn't exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |

## Subscription Channels

Valid channels for `subscribe`/`unsubscribe`:

- `daemon-state` - Daemon state changes
- `events` - Individual key events
- `latency` - Latency statistics

## References

- **Backend RPC Types:** `keyrx_daemon/src/web/rpc_types.rs`
- **Backend Event Types:** `keyrx_daemon/src/web/events.rs`
- **Backend WebSocket Handler:** `keyrx_daemon/src/web/ws.rs`
- **Frontend RPC Types:** `keyrx_ui/src/types/rpc.ts`
- **Frontend WebSocket Handler:** `keyrx_ui/src/hooks/useUnifiedApi.ts`
- **Backend Contract Tests:** `keyrx_daemon/tests/websocket_contract_test.rs`
- **Frontend Contract Tests:** `keyrx_ui/tests/hooks/useUnifiedApi.test.tsx`

## Maintenance

**Review this document:**
- Before making any WebSocket changes
- After adding new event types
- When planning breaking changes
- During code reviews involving WebSocket code

**Last Updated:** 2026-01-03
**Status:** Legacy format in use, dual support enabled
