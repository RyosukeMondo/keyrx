# Windows E2E Testing Guide - Key Remapping & Metrics

This guide explains how to run end-to-end tests for key remapping on Windows, including verification that events appear in the `/metrics` endpoint.

## Overview

The E2E test suite validates:
1. ✅ Key event simulation via `/api/simulator/events`
2. ✅ Key remapping functionality (A → B)
3. ✅ Event detection in `/api/metrics/events`
4. ✅ Latency tracking via `/api/metrics/latency`
5. ✅ WebSocket event broadcasting (optional)

## Quick Start

### Option 1: Run Tests Locally (Windows)

```powershell
# Navigate to project root
cd C:\Users\ryosu\repos\keyrx

# Run all E2E tests
.\scripts\windows\Run-E2E-Remap-Tests.ps1

# Run with verbose output
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -Verbose

# Run specific test
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -TestName "test_windows_key_remap_e2e"

# Skip build step (if already built)
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -NoBuild
```

### Option 2: Run Tests in Vagrant VM (from Linux)

```bash
# From project root on Linux
./scripts/windows_test_vm.sh

# Or manually
cd vagrant/windows
vagrant up
vagrant ssh

# Inside VM
cd C:\vagrant_project
cargo test -p keyrx_daemon --test windows_e2e_remap --features windows
```

### Option 3: Manual Cargo Commands

```bash
# Build first
cargo build --package keyrx_daemon --features windows --release

# Run all E2E tests
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows

# Run with verbose output
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows -- --nocapture

# Run specific test
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows test_windows_key_remap_e2e
```

## Test Architecture

### Test Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Create Test Profile (A → B remapping)                   │
│    - ConfigRoot with KeyMapping A → B                       │
│    - Serialize to .krx binary                                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. Start Test Server                                        │
│    - AppState with all services                             │
│    - Axum router with /api endpoints                        │
│    - Bind to random port (127.0.0.1:0)                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. Load Profile via API                                     │
│    POST /api/simulator/load-profile                         │
│    {"name": "test-remap"}                                   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Simulate Key Events                                      │
│    POST /api/simulator/events                               │
│    {"dsl": "press:A,wait:50,release:A"}                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. Verify in Metrics Endpoint                               │
│    GET /api/metrics/events?count=100                        │
│    - Check for input event (A pressed/released)             │
│    - Check for output event (B)                             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. Verify Latency Stats                                     │
│    GET /api/metrics/latency                                 │
│    - Check avg_us < 10,000μs                                │
│    - Verify p95, p99 metrics                                │
└─────────────────────────────────────────────────────────────┘
```

### Test Cases

#### 1. `test_windows_key_remap_e2e`
**Complete end-to-end remapping test**

Tests:
- Profile creation and loading
- Key event simulation (A press/release)
- Remapping verification (A → B)
- Metrics endpoint detection
- Latency tracking

Expected Results:
- ✅ Events appear in `/api/metrics/events`
- ✅ Input: `key_code=A`, `event_type=press/release`
- ✅ Output: Contains `B` (remapped)
- ✅ Average latency < 10ms

#### 2. `test_windows_metrics_endpoint_available`
**Verify metrics endpoints are accessible**

Tests:
- `/api/metrics/events` returns 200 or 500
- `/api/metrics/latency` returns 200 or 500

Expected Results:
- ✅ Endpoints respond (500 acceptable if daemon not running)

#### 3. `test_windows_simulator_integration`
**Verify simulator produces correct outputs**

Tests:
- Profile loading
- DSL event parsing
- Multi-key simulation (A, B)
- Output verification

Expected Results:
- ✅ At least 2 B outputs (1 remapped from A, 1 direct)

## API Endpoints Used

### Simulator API

#### POST `/api/simulator/load-profile`
Load a profile for simulation testing.

**Request:**
```json
{
  "name": "test-remap"
}
```

**Response:**
```json
{
  "success": true
}
```

#### POST `/api/simulator/events`
Simulate key events using DSL or explicit event list.

**Request (DSL):**
```json
{
  "dsl": "press:A,wait:50,release:A"
}
```

**Request (Events):**
```json
{
  "events": [
    {"type": "press", "key": "A", "timestamp_us": 0},
    {"type": "release", "key": "A", "timestamp_us": 50000}
  ]
}
```

**Response:**
```json
{
  "outputs": [
    {"type": "press", "key": "B", "timestamp_us": 5, "latency_us": 5},
    {"type": "release", "key": "B", "timestamp_us": 50005, "latency_us": 5}
  ],
  "passed": true
}
```

### Metrics API

#### GET `/api/metrics/events?count=100`
Retrieve recent key events.

**Response:**
```json
{
  "count": 2,
  "events": [
    {
      "event_type": "press",
      "key_code": 65,
      "timestamp": 1234567890,
      "device_id": "test-device",
      "output": "B",
      "mapping_triggered": true
    },
    {
      "event_type": "release",
      "key_code": 65,
      "timestamp": 1234567940,
      "device_id": "test-device",
      "output": "B",
      "mapping_triggered": true
    }
  ]
}
```

#### GET `/api/metrics/latency`
Retrieve latency statistics.

**Response:**
```json
{
  "min_us": 5,
  "avg_us": 10,
  "max_us": 50,
  "p95_us": 20,
  "p99_us": 30
}
```

## Troubleshooting

### Test Failures

#### "Failed to load profile"
**Cause:** Profile file not created or invalid format.

**Solution:**
```rust
// Ensure setup_test_profile() is called
setup_test_profile(&config_dir).await?;
```

#### "No events recorded in metrics!"
**Cause:** Events not being broadcast or stored.

**Solution:**
1. Check if EventBroadcaster is configured
2. Verify event_loop is calling `broadcaster.broadcast_key_event()`
3. Increase wait time: `sleep(Duration::from_millis(500)).await;`

#### "Remapping A→B did not occur"
**Cause:** Remapping logic not applied or profile not loaded.

**Solution:**
1. Verify profile has correct mapping: `KeyCode::A → KeyCode::B`
2. Check SimulationEngine is processing events
3. Enable debug logging: `RUST_LOG=debug cargo test`

#### "Average latency too high"
**Cause:** System under load or slow hardware.

**Solution:**
1. Close other applications
2. Increase threshold: `assert!(avg_us < 50_000);`
3. Run on faster hardware or VM

### Debugging

#### Enable Verbose Logging
```powershell
$env:RUST_LOG="debug"
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows -- --nocapture
```

#### Check Specific Event Details
```rust
for event in events {
    println!("Event: {:?}", event);
}
```

#### Test with Manual Daemon
```powershell
# Terminal 1: Start daemon
.\target\release\keyrx_daemon.exe

# Terminal 2: Run tests
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows
```

## Manual Testing in Browser

After running tests, you can manually verify in the web UI:

1. **Start daemon:**
   ```powershell
   .\target\release\keyrx_daemon.exe
   ```

2. **Open browser:**
   ```
   http://localhost:3030/metrics
   ```

3. **Navigate to Simulator page:**
   - Click "Simulator" in navigation
   - Load profile: `test-remap`
   - Enter DSL: `press:A,wait:50,release:A`
   - Click "Run"

4. **Check Metrics page:**
   - Navigate to "Metrics"
   - Verify events appear in event log
   - Verify latency chart shows data
   - Check that A → B remapping is visible

## CI/CD Integration

### GitHub Actions (Windows)

```yaml
name: Windows E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Build
        run: cargo build --package keyrx_daemon --features windows --release

      - name: Run E2E Tests
        run: cargo test --package keyrx_daemon --test windows_e2e_remap --features windows
```

### Vagrant VM Testing

```bash
# In CI/CD pipeline
./scripts/windows_test_vm.sh --headless

# Or with UAT
./scripts/windows_test_vm.sh --uat
```

## Performance Benchmarks

Expected performance on modern hardware:

| Metric | Target | Acceptable | Excellent |
|--------|--------|------------|-----------|
| Average Latency | < 100μs | < 50μs | < 10μs |
| P95 Latency | < 200μs | < 100μs | < 20μs |
| P99 Latency | < 500μs | < 200μs | < 50μs |
| Event Capture | 100% | 100% | 100% |
| Remapping Accuracy | 100% | 100% | 100% |

## Next Steps

1. **Extend to real hardware testing:**
   - Use `keyrx_daemon\tests\e2e_tests.rs` as template
   - Add physical keyboard detection
   - Test with actual key presses

2. **Add more remapping scenarios:**
   - Modifier keys (Ctrl, Shift, Alt)
   - Tap-hold behavior
   - Layer switching
   - Multi-key combos

3. **WebSocket real-time testing:**
   - Connect WebSocket client
   - Subscribe to events
   - Verify real-time broadcasts

4. **Stress testing:**
   - Simulate high-frequency key events (100+ keys/sec)
   - Verify latency remains acceptable
   - Test memory usage under load

## References

- **Simulation Service:** `keyrx_daemon/src/services/simulation_service.rs`
- **Metrics API:** `keyrx_daemon/src/web/api/metrics.rs`
- **Event Loop:** `keyrx_daemon/src/daemon/event_loop.rs`
- **Event Broadcaster:** `keyrx_daemon/src/daemon/event_broadcaster.rs`
- **Windows Platform:** `keyrx_daemon/src/platform/windows/`

## Support

For issues or questions:
- Check existing tests in `keyrx_daemon/tests/`
- Review TROUBLESHOOTING.md
- Open an issue on GitHub
