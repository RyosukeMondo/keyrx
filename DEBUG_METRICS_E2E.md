# Debugging Metrics E2E Event Flow

## Problem

The metrics page (`/metrics`) shows no key events even though:
- Daemon is running as admin ✓
- Keyboard input is being captured (shown in debug logs) ✓
- Config is loaded (or pass-through mode should still show events) ✓

## Event Flow (Expected)

```
Windows Keyboard
    ↓
Raw Input Hook (rawinput.rs)
    ↓
Global Event Queue (crossbeam channel)
    ↓
Event Loop (daemon/event_loop.rs)
    ↓
EventBroadcaster (daemon/event_broadcaster.rs)
    ↓
Tokio Broadcast Channel (event_tx)
    ↓
WebSocket Handler (web/ws.rs)
    ↓
WebSocket Client (Browser)
    ↓
Metrics Store (metricsStore.ts)
    ↓
Metrics Page (/metrics)
```

## Debug Logging Added

I've added comprehensive debug logging at each step:

### 1. Windows Raw Input (rawinput.rs:428)
```
[DEBUG] Raw input event: KeyEvent { ... }
```

### 2. Event Loop Broadcasting (event_loop.rs:260-269)
```
[DEBUG] Event loop: About to broadcast press event for key A (mapping_triggered: false, output: A)
```

### 3. EventBroadcaster (event_broadcaster.rs:40-48)
```
[DEBUG] Broadcasting key event (subscribers: 1): "A"
[DEBUG] Successfully broadcast key event to 1 receivers
```

### 4. WebSocket Connection (ws.rs:37-41)
```
[INFO] WebSocket client connected (active senders: 1)
[INFO] WebSocket subscribed to daemon events (total receivers: 1)
```

### 5. WebSocket Receiving Events (ws.rs:69-82)
```
[DEBUG] WebSocket received daemon event: KeyEvent("A")
[DEBUG] WebSocket successfully sent event to client
```

## Testing Steps

### Option A: Automated Script

```powershell
.\scripts\windows\Debug-Metrics-E2E.ps1
```

This script will:
1. Build daemon with debug logging
2. Stop existing daemon
3. Start daemon with `RUST_LOG=debug`
4. Open browser to `/metrics`
5. Tail the log file

### Option B: Manual Testing

1. **Build daemon with debug logging:**
   ```powershell
   cargo build --release --features windows
   ```

2. **Start daemon with debug logging:**
   ```powershell
   $env:RUST_LOG = "debug"
   .\target\release\keyrx_daemon.exe run 2>&1 | Tee-Object -FilePath "$env:TEMP\keyrx-debug.log"
   ```

3. **Open browser:**
   ```
   http://localhost:9867/metrics
   ```

4. **Press some keys and watch the daemon window**

5. **Tail the log:**
   ```powershell
   Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait
   ```

## What to Look For

### ✅ Success Indicators

If you see ALL of these in sequence, the flow is working:
1. `Raw input event: KeyEvent`
2. `Event loop: About to broadcast`
3. `Broadcasting key event (subscribers: 1)`
4. `Successfully broadcast key event to 1 receivers`
5. `WebSocket client connected`
6. `WebSocket received daemon event: KeyEvent`
7. Events appear on `/metrics` page

### ❌ Failure Points

| Missing Log | Problem | Solution |
|-------------|---------|----------|
| No "Raw input event" | Keyboard hooks not capturing | Check admin privileges, check if input device is recognized |
| No "About to broadcast" | Event loop not calling broadcaster | Check if EventBroadcaster is set in daemon |
| "subscribers: 0" | No WebSocket clients | Check if browser is connected, check WebSocket connection status |
| No "WebSocket client connected" | WebSocket not established | Check if web server is running on port 9867 |
| No "WebSocket received" | Events not reaching WebSocket | Check if event_tx channel is working |
| Events in logs but not in UI | Frontend issue | Check browser console, check metricsStore subscribeToEvents() |

## Common Issues & Fixes

### Issue 1: "subscribers: 0"

**Cause:** WebSocket client hasn't connected yet or disconnected.

**Fix:**
1. Check browser console for WebSocket errors
2. Verify WebSocket URL: `ws://localhost:9867/ws`
3. Check if CORS is blocking connection
4. Refresh the `/metrics` page

### Issue 2: "Raw input event" but no "About to broadcast"

**Cause:** EventBroadcaster is None (not set in daemon).

**Fix:** This should be impossible after v0.1.1, but if it happens:
1. Check `daemon.set_event_broadcaster()` is called in main.rs:1070
2. Verify EventBroadcaster is passed to `event_loop::process_one_event()`

### Issue 3: "WebSocket received" but no events in UI

**Cause:** Frontend not subscribed or not processing events correctly.

**Fix:**
1. Open browser DevTools console (F12)
2. Check for WebSocket connection: `connected: true` should show in metrics page header
3. Check for JavaScript errors
4. Verify `useMetricsStore().subscribeToEvents()` is called on mount

### Issue 4: All logs present but events show wrong data

**Cause:** Data transformation issue between backend and frontend.

**Fix:**
1. Check `KeyEventData` serialization in event_broadcaster.rs
2. Check `WSMessage` type in metricsStore.ts
3. Verify timestamp format (microseconds vs milliseconds)

## Next Steps

After running the debug script:

1. **If all logs are present:** Issue is in the frontend (browser console errors)
2. **If "subscribers: 0":** WebSocket connection issue
3. **If events don't reach event loop:** Raw Input hook issue
4. **If events reach loop but not broadcast:** EventBroadcaster not set

Report findings with:
- Full debug log output
- Browser console errors (if any)
- WebSocket connection status from metrics page
