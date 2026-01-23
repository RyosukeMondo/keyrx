# IPC Test Mode for E2E Testing

## Quick Summary

**Goal:** Fix 5 failing IPC-dependent tests by adding test mode with full IPC infrastructure.

**Impact:** Improves E2E test pass rate from 90.4% to 96.4%

**Effort:** 1-2 days (new IPC infrastructure)

## Problem

5 tests fail because they require IPC for profile activation and daemon status queries:
- `status-001` - daemon_running field requires IPC ❌
- `integration-001` - Profile activation requires IPC ❌
- `workflow-002` - Profile activation requires IPC ❌
- `workflow-003` - Profile activation requires IPC ❌
- `workflow-007` - Profile activation requires IPC ❌

**Root Cause:** E2E tests run daemon in 'run' mode without full IPC socket.

## Solution

Add `--test-mode` flag that:
1. Starts daemon with IPC socket
2. Enables profile activation via REST API → IPC
3. Enables daemon status queries via REST API → IPC
4. Skips keyboard capture (test-only mode)

## Tasks

1. Add --test-mode CLI flag to daemon
2. Create IPC module structure
3. Implement IPC server for test mode
4. Implement profile activation via IPC
5. Implement daemon status query via IPC
6. Integrate IPC with REST API handlers
7. Update daemon startup for test mode
8. Test IPC-dependent E2E tests

## Success Criteria

- ✅ All 5 IPC-dependent tests pass with --test-mode
- ✅ 100% E2E test pass rate (83/83 → 100%)
- ✅ IPC latency < 50ms
- ✅ Test mode startup < 2 seconds

## Documents

- [requirements.md](./requirements.md) - Complete functional requirements
- [design.md](./design.md) - IPC architecture and protocol
- [tasks.md](./tasks.md) - 8 tasks, 1-2 day estimate
