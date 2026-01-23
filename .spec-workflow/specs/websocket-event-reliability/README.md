# WebSocket Event Notification Reliability

## Quick Summary

**Goal:** Fix 2 failing WebSocket event tests by adding event publishing to device/profile update endpoints.

**Impact:** Improves E2E test pass rate from 90.4% to 92.8%

**Effort:** 2-3 hours (simple fix)

## Problem

WebSocket tests fail because device updates and profile activations don't publish events to the event bus:
- `websocket-003` - Device update events not received ❌
- `websocket-004` - Profile activation events not received ❌

## Solution

Add `event_tx.send()` calls after successful operations:
1. Device update (PATCH /api/devices/:id) → publish DeviceUpdated event
2. Profile activation (POST /api/profiles/:name/activate) → publish ProfileActivated event

## Tasks

1. Add event publishing to device update endpoint
2. Add event publishing to profile activation endpoint
3. Verify WebSocket handler event processing
4. Test device update event notification
5. Test profile activation event notification
6. Run full WebSocket test suite (verify 5/5 passing)

## Success Criteria

- ✅ 100% WebSocket test pass rate (5/5)
- ✅ Event delivery latency < 100ms
- ✅ Zero flaky test failures

## Documents

- [requirements.md](./requirements.md) - Complete functional requirements
- [design.md](./design.md) - Architecture and implementation
- [tasks.md](./tasks.md) - 6 tasks, 2-3 hour estimate
