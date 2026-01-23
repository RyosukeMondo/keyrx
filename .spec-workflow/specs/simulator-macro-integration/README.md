# Simulator-Macro Recorder Integration

## Quick Summary

**Goal:** Fix 1 failing workflow test by connecting simulator events to macro recorder.

**Impact:** Completes E2E test suite (100% pass rate)

**Effort:** 3-4 hours (event bus integration)

## Problem

Simulator generates keyboard events but doesn't feed them to macro recorder:
- `workflow-006` - Macro record → simulate → playback ❌

**Root Cause:** Simulator and macro recorder use separate event paths with no connection.

## Solution

Route simulator events through event bus:
```
Simulator → Event Bus → Macro Recorder
```

1. Add event_tx to SimulatorService
2. Send simulated events to event bus
3. Connect macro recorder to event bus
4. Wire up at daemon startup

## Tasks

1. Add event bus sender to SimulatorService
2. Send simulated events to event bus
3. Connect macro recorder to event bus
4. Wire up event bus in daemon initialization
5. Test simulator-macro integration

## Success Criteria

- ✅ `workflow-006` test passes
- ✅ Simulated events recorded correctly
- ✅ Event format matches schema
- ✅ Zero flaky test failures

## Documents

- [requirements.md](./requirements.md) - Complete functional requirements
- [design.md](./design.md) - Event flow architecture
- [tasks.md](./tasks.md) - 5 tasks, 3-4 hour estimate
