# KeyRx Test Quick Reference

## 🚀 Run All Tests

```bash
# Backend unit + integration tests
cargo test --workspace

# Frontend tests
cd keyrx_ui && npm test

# Comprehensive REST API E2E tests (NEW - 36 tests)
cargo test --test rest_api_comprehensive_e2e_test -- --test-threads=1
```

## ✅ REST API E2E Tests (36 tests)

**New comprehensive test suite covering all working features!**

```bash
# Run all REST API E2E tests
cargo test --test rest_api_comprehensive_e2e_test -- --test-threads=1

# Verbose output
cargo test --test rest_api_comprehensive_e2e_test -- --test-threads=1 --nocapture

# Specific test
cargo test --test rest_api_comprehensive_e2e_test test_device_detection -- --test-threads=1
```

### Coverage Summary
✅ Device detection with serial numbers (3 tests)
✅ Profile activation & persistence (6 tests)
✅ Config rendering (3 tests)
✅ Rhai mapping visualization (4 tests)
✅ Metrics (7 tests)
✅ Event simulation (6 tests)
✅ Edge cases & error handling (7 tests)

**Full details:** `keyrx_daemon/tests/REST_API_E2E_TEST_COVERAGE.md`

---

# Windows E2E Testing - Quick Reference

## 🚀 Most Common Commands

### Run All Tests (Recommended)
```powershell
.\scripts\windows\Run-E2E-Remap-Tests.ps1
```

### Run Tests with Verbose Output
```powershell
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -Verbose
```

### Run Specific Test
```powershell
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -TestName "test_windows_key_remap_e2e"
```

### Skip Build (if already built)
```powershell
.\scripts\windows\Run-E2E-Remap-Tests.ps1 -NoBuild
```

## 🔧 Direct Cargo Commands

### Build
```bash
cargo build --package keyrx_daemon --features windows --release
```

### Test
```bash
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows
```

### Test with Output
```bash
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows -- --nocapture
```

### Specific Test
```bash
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows test_windows_key_remap_e2e
```

## 🐞 Debug Mode

### Enable Logging
```powershell
$env:RUST_LOG="debug"
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows -- --nocapture
```

### Trace Level
```powershell
$env:RUST_LOG="trace"
cargo test --package keyrx_daemon --test windows_e2e_remap --features windows -- --nocapture
```

## 🖥️ Manual Testing

### 1. Start Daemon
```powershell
.\target\release\keyrx_daemon.exe
```

### 2. Test Endpoints

**Metrics Events**:
```bash
curl http://localhost:3030/api/metrics/events?count=100
```

**Metrics Latency**:
```bash
curl http://localhost:3030/api/metrics/latency
```

**Load Profile**:
```bash
curl -X POST http://localhost:3030/api/simulator/load-profile -H "Content-Type: application/json" -d "{\"name\":\"test-remap\"}"
```

**Simulate Events**:
```bash
curl -X POST http://localhost:3030/api/simulator/events -H "Content-Type: application/json" -d "{\"dsl\":\"press:A,wait:50,release:A\"}"
```

### 3. Browser Testing
```
http://localhost:3030/metrics
http://localhost:3030/simulator
```

## 🏗️ Vagrant VM Testing

### Start VM
```bash
cd vagrant/windows
vagrant up
```

### SSH into VM
```bash
vagrant ssh
```

### Inside VM
```powershell
cd C:\vagrant_project
cargo test -p keyrx_daemon --test windows_e2e_remap --features windows
```

### Stop VM
```bash
vagrant halt
```

### Automated Testing
```bash
./scripts/windows_test_vm.sh
```

## 📊 What Each Test Does

### test_windows_key_remap_e2e
- Creates A→B remapping profile
- Simulates key events via API
- Verifies events in /metrics/events
- Checks remapping occurred
- Validates latency < 10ms

### test_windows_metrics_endpoint_available
- Tests /api/metrics/events accessibility
- Tests /api/metrics/latency accessibility
- Accepts 200 or 500 status

### test_windows_simulator_integration
- Loads remapping profile
- Simulates multiple keys (A, B)
- Verifies outputs match expectations
- Checks remapping accuracy

## 🎯 Expected Results

### Success Output
```
✓ Profile loaded
✓ Events simulated
✓ Metrics retrieved
✓ Press A detected: true
✓ Release A detected: true
✓ Output B detected: true
✓ Average latency: 10μs

✅ All Windows E2E tests passed!
```

### Failure Indicators
```
✗ Failed to load profile
✗ No events recorded in metrics
✗ Remapping A→B did not occur
✗ Average latency too high
```

## 🚨 Troubleshooting

### Issue: Profile not loading
```bash
# Check config directory
ls ./config

# Verify .krx files
file ./config/*.krx
```

### Issue: No events in metrics
```powershell
# Increase debug level
$env:RUST_LOG="keyrx_daemon=trace"

# Run test with output
cargo test --test windows_e2e_remap --features windows -- --nocapture
```

### Issue: High latency
```bash
# Check system load
tasklist | findstr /i "cpu"

# Close other applications
# Re-run test
```

### Issue: Build errors
```bash
# Clean build
cargo clean

# Rebuild
cargo build --package keyrx_daemon --features windows --release

# Update dependencies
cargo update
```

## 📁 Key Files

```
tests/
  └─ windows_e2e_remap.rs          (E2E test suite)

scripts/windows/
  └─ Run-E2E-Remap-Tests.ps1      (PowerShell runner)

src/web/
  └─ mod.rs                        (AppState testing support)

docs/
  ├─ WINDOWS_E2E_TESTING_GUIDE.md (Comprehensive guide)
  └─ WINDOWS_E2E_SUMMARY.md       (Implementation summary)
```

## 🔗 Quick Links

| Resource | Location |
|----------|----------|
| Full Guide | `WINDOWS_E2E_TESTING_GUIDE.md` |
| Summary | `WINDOWS_E2E_SUMMARY.md` |
| Test Code | `keyrx_daemon/tests/windows_e2e_remap.rs` |
| Runner Script | `scripts/windows/Run-E2E-Remap-Tests.ps1` |
| Vagrant Setup | `vagrant/windows/README.md` |

## ⚡ Pro Tips

1. **Always use PowerShell script** for best experience
2. **Check RUST_LOG** for detailed debugging
3. **Use -Verbose flag** when troubleshooting
4. **Test in Vagrant VM** for clean environment
5. **Monitor /metrics page** while testing

## 📞 Getting Help

1. Check `WINDOWS_E2E_TESTING_GUIDE.md` (700+ lines)
2. Review error messages carefully
3. Enable debug logging (`RUST_LOG=debug`)
4. Check existing test files for examples
5. Open GitHub issue if needed

---

**Quick Start**: `.\scripts\windows\Run-E2E-Remap-Tests.ps1`
