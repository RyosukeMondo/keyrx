# How to Run Windows E2E Tests - 5 Minute Guide

## Prerequisites

- ✅ Windows 10/11 or Vagrant VM
- ✅ Rust installed (`rustup.rs`)
- ✅ Project cloned to `C:\Users\ryosu\repos\keyrx`

## Method 1: PowerShell Script (Easiest) ⭐

Open PowerShell and run:

```powershell
cd C:\Users\ryosu\repos\keyrx
.\scripts\windows\Run-E2E-Remap-Tests.ps1
```

That's it! The script will:
1. Check your environment
2. Build the project
3. Run all tests
4. Show results with colors

## Method 2: Cargo Command (Manual)

```bash
# 1. Build
cargo build --package keyrx_daemon --features windows --release

# 2. Run tests
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows
```

## Method 3: Vagrant VM (From Linux)

```bash
# 1. Start VM
cd vagrant/windows
vagrant up

# 2. SSH in
vagrant ssh

# 3. Run tests
cd C:\vagrant_project
cargo test -p keyrx_daemon --test windows_e2e_remap --features windows
```

## What You'll See

### ✅ Success
```
╔══════════════════════════════════════════════════════════════════╗
║                    ✅ ALL TESTS PASSED                            ║
╚══════════════════════════════════════════════════════════════════╝

Test Summary:
  ✓ Key event simulation working
  ✓ Remapping applied correctly (A → B)
  ✓ Metrics endpoints detecting events
  ✓ Latency tracking functional
```

### ❌ Failure
```
✗ test_windows_key_remap_e2e ... FAILED

Error: Remapping A→B did not occur

Solution: Check WINDOWS_E2E_TESTING_GUIDE.md troubleshooting section
```

## Test Details

### What Gets Tested

1. **Input Detection**: Pressing "A" is captured
2. **Remapping**: "A" is converted to "B"
3. **Metrics**: Events appear in `/metrics/events` endpoint
4. **Performance**: Latency < 10ms average

### API Endpoints Used

- `POST /api/simulator/load-profile` - Load remapping rules
- `POST /api/simulator/events` - Simulate key presses
- `GET /api/metrics/events` - Check event log
- `GET /api/metrics/latency` - Check performance

## Debugging

### Enable Logging
```powershell
$env:RUST_LOG="debug"
cargo test --test windows_e2e_remap --features windows -- --nocapture
```

### Run Specific Test
```powershell
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -TestName "test_windows_key_remap_e2e"
```

### Check Metrics Manually
```powershell
# 1. Start daemon
.\target\release\keyrx_daemon.exe

# 2. Open browser
start http://localhost:3030/metrics

# 3. Try simulator
start http://localhost:3030/simulator
```

## Common Issues

### Issue: "Rust not found"
**Solution**: Install Rust from https://rustup.rs

### Issue: "Build failed"
**Solution**: Run `cargo clean` then rebuild

### Issue: "No events in metrics"
**Solution**:
1. Enable debug logging: `$env:RUST_LOG="debug"`
2. Check if daemon is running
3. Increase wait time in test

## Next Steps

### ✅ Tests Passed - What Now?

1. **Manual Testing**: Start daemon and test in browser
   ```powershell
   .\target\release\keyrx_daemon.exe
   # Open: http://localhost:3030/metrics
   ```

2. **Real Hardware**: Test with actual keyboard
   - See `keyrx_daemon/tests/e2e_tests.rs` for examples

3. **Add More Tests**: Extend coverage
   - Modifier keys (Ctrl, Shift, Alt)
   - Tap-hold behavior
   - Layer switching

### ❌ Tests Failed - How to Fix?

1. **Check Guide**: `WINDOWS_E2E_TESTING_GUIDE.md` has detailed troubleshooting

2. **Enable Debug Logging**:
   ```powershell
   $env:RUST_LOG="trace"
   cargo test --test windows_e2e_remap --features windows -- --nocapture
   ```

3. **Review Error Messages**: Look for specific failure reason

4. **Check Prerequisites**: Ensure Rust, Cargo installed correctly

## Documentation

| Document | Purpose |
|----------|---------|
| **This File** | Quick 5-minute guide |
| `WINDOWS_E2E_TESTING_GUIDE.md` | Comprehensive 700+ line guide |
| `WINDOWS_E2E_SUMMARY.md` | Implementation details |
| `QUICK_TEST_REFERENCE.md` | Command cheat sheet |

## Help & Support

- **Troubleshooting**: See `WINDOWS_E2E_TESTING_GUIDE.md` section 9
- **API Details**: See `WINDOWS_E2E_TESTING_GUIDE.md` section 5
- **Examples**: Check `keyrx_daemon/tests/simulator_api_test.rs`
- **Issues**: Open GitHub issue with error logs

---

**TL;DR**: Run `.\scripts\windows\Run-E2E-Remap-Tests.ps1` and you're done! ✨
