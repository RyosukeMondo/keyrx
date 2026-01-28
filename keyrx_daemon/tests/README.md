# Daemon Test Suite Documentation

Comprehensive test infrastructure for bug remediation verification (WS8).

## Test Categories

### 1. Memory Leak Tests (`memory_leak_test.rs`)

**Purpose:** Detect and prevent memory leaks in WebSocket subscriptions and event handling.

**Tests:**
- `test_websocket_subscription_cleanup_single_cycle` - Verify single cleanup
- `test_websocket_subscription_cleanup_1000_cycles` - 1000 connection cycles (long-running)
- `test_event_broadcaster_queue_bounded` - Queue stays bounded under load
- `test_no_subscription_leaks_under_concurrent_load` - Concurrent connection safety
- `test_memory_stable_during_profile_operations` - Profile operations don't leak
- `test_websocket_broadcast_performance` - Broadcasting to multiple subscribers
- `test_cleanup_on_abnormal_websocket_termination` - Cleanup on crashes

**Run:**
```bash
# Quick tests
cargo test --test memory_leak_test

# Long-running stress test
cargo test --test memory_leak_test -- --ignored --nocapture
```

### 2. Concurrency Tests (`concurrency_test.rs`)

**Purpose:** Verify thread safety and race condition handling.

**Tests:**
- `test_concurrent_profile_activations` - 10 threads activating profiles
- `test_100_concurrent_websocket_connections` - 100 concurrent WebSockets (long-running)
- `test_event_broadcasting_race_conditions` - No message loss/duplication
- `test_message_ordering_under_concurrent_load` - Messages ordered correctly
- `test_concurrent_api_endpoint_access` - Multiple endpoint access
- `test_concurrent_profile_create_delete` - Creation/deletion races
- `test_concurrent_websocket_subscribe_unsubscribe` - Subscription races
- `test_concurrent_shared_state_access` - Readers and writers

**Run:**
```bash
# Quick tests
cargo test --test concurrency_test

# Long-running tests
cargo test --test concurrency_test -- --ignored --nocapture
```

### 3. E2E Bug Remediation Tests (`bug_remediation_e2e_test.rs`)

**Purpose:** End-to-end workflow verification for all bug fixes.

**Tests:**
- `test_profile_creation_activation_workflow` - Full lifecycle
- `test_websocket_subscription_workflow` - Subscribe/unsubscribe flow
- `test_profile_error_handling` - Error responses
- `test_device_management_workflow` - Device operations
- `test_settings_operations` - Settings get/update
- `test_concurrent_multi_endpoint_operations` - Multi-endpoint access
- `test_websocket_rpc_error_handling` - RPC error handling
- `test_profile_activation_state_persistence` - State management
- `test_multiple_websocket_clients_broadcast` - Multi-client broadcasting
- `test_api_authentication` - Auth handling
- `test_rate_limiting_normal_operations` - Rate limiting
- `test_cors_headers` - CORS configuration
- `test_graceful_error_recovery` - Error recovery

**Run:**
```bash
cargo test --test bug_remediation_e2e_test
```

### 4. Stress Tests (`stress_test.rs`)

**Purpose:** Long-running stability and performance under load.

**Tests:**
- `test_24_hour_stability` - 24-hour continuous operation
- `test_1000_operations_per_second` - High throughput test
- `test_100_concurrent_websockets_under_load` - 100 WebSockets + load
- `test_memory_stability_monitoring` - 1-hour memory monitoring
- `test_cpu_stability_under_load` - 30-minute CPU test
- `test_performance_degradation` - 2-hour degradation detection
- `test_concurrent_mixed_workload` - 10-minute mixed operations

**Run:**
```bash
# All stress tests (long-running, requires --ignored)
cargo test --test stress_test -- --ignored --nocapture

# Individual tests
cargo test --test stress_test test_24_hour_stability -- --ignored --nocapture
cargo test --test stress_test test_1000_operations_per_second -- --ignored --nocapture
```

**Note:** Stress tests are marked `#[ignore]` due to long runtime. Run with `--ignored` flag.

### 5. Security Tests (`security_test.rs`)

**Purpose:** Verify security measures and attack resistance.

**Tests:**
- `test_authentication_bypass_attempts` - Auth bypass prevention
- `test_path_traversal_attempts` - Directory traversal blocking
- `test_sql_injection_attempts` - SQL injection prevention
- `test_command_injection_attempts` - Command injection blocking
- `test_xss_attempts` - XSS payload sanitization
- `test_dos_resistance_large_payloads` - Large payload rejection
- `test_dos_resistance_rapid_requests` - Request flood handling
- `test_dos_resistance_websocket_flood` - WebSocket flood protection
- `test_cors_enforcement` - CORS header verification
- `test_rate_limiting_per_endpoint` - Rate limiting application
- `test_input_validation` - Input validation correctness
- `test_sensitive_data_exposure` - No sensitive data in responses
- `test_websocket_security` - WebSocket message security
- `test_authorization_checks` - Authorization enforcement

**Run:**
```bash
cargo test --test security_test
```

### 6. Performance Tests (`performance_test.rs`)

**Purpose:** Benchmark performance and detect regressions.

**Tests:**
- `test_api_endpoint_performance` - All API endpoint latency
- `test_profile_creation_performance` - Profile creation speed
- `test_profile_activation_performance` - Activation speed
- `test_websocket_connection_performance` - Connection latency
- `test_websocket_subscription_performance` - Subscription speed
- `test_websocket_broadcast_latency` - Broadcast latency
- `test_concurrent_request_performance` - Concurrent request handling
- `test_memory_allocation_performance` - Resource allocation speed
- `test_json_serialization_performance` - JSON parsing speed
- `test_performance_regression_detection` - Regression detection
- `test_cold_start_vs_warm_performance` - Cold vs warm performance

**Run:**
```bash
cargo test --test performance_test
```

**Performance Thresholds:**
- API latency: <100ms average
- WebSocket connection: <500ms
- Profile activation: <200ms
- Subscription: <100ms

## Running All Tests

### Quick Test Suite (No Long-Running Tests)
```bash
cargo test --workspace
```

### Full Test Suite (Including Stress Tests)
```bash
# Run all quick tests
cargo test --workspace

# Run all ignored long-running tests
cargo test --workspace -- --ignored --nocapture
```

### Specific Test Categories
```bash
# Memory leak tests
cargo test --test memory_leak_test

# Concurrency tests
cargo test --test concurrency_test

# E2E tests
cargo test --test bug_remediation_e2e_test

# Security tests
cargo test --test security_test

# Performance tests
cargo test --test performance_test

# Stress tests (long-running)
cargo test --test stress_test -- --ignored --nocapture
```

## Test Infrastructure

### Common Test Utilities (`tests/common/`)

**TestApp** - Main test fixture providing:
- Isolated temp directories per test
- HTTP client helpers (GET, POST, PATCH, DELETE)
- WebSocket connection helpers
- Automatic cleanup on drop
- Parallel test support (different ports per test)

**Usage:**
```rust
use common::test_app::TestApp;

#[tokio::test]
async fn test_example() {
    let app = TestApp::new().await;

    // Make HTTP request
    let response = app.get("/api/status").await;
    assert!(response.status().is_success());

    // Connect WebSocket
    let ws = app.connect_ws().await;
    ws.send_text("message").await.unwrap();
}
```

## Continuous Integration

### GitHub Actions Integration

```yaml
- name: Run Quick Tests
  run: cargo test --workspace

- name: Run Security Tests
  run: cargo test --test security_test

- name: Run Performance Tests
  run: cargo test --test performance_test

# Optional: Run stress tests on nightly builds
- name: Run Stress Tests
  run: cargo test --workspace -- --ignored --nocapture
  if: github.event_name == 'schedule'
```

## Test Results Reporting

### Generate Test Report
```bash
# Run all tests and capture output
cargo test --workspace 2>&1 | tee TEST_RESULTS.txt

# With coverage
cargo tarpaulin --out Html --output-dir coverage/
```

### Test Metrics
- Total tests: 962+ backend tests
- Coverage target: ≥80% overall, ≥90% for critical paths
- Performance baseline: Tracked in performance_test.rs

## Debugging Failed Tests

### Verbose Output
```bash
# Show test output
cargo test test_name -- --nocapture

# Show all logs
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Run Single Test
```bash
cargo test --test memory_leak_test test_websocket_subscription_cleanup_single_cycle
```

### Debug WebSocket Tests
```bash
# Enable WebSocket logging
RUST_LOG=keyrx_daemon::web::ws=debug cargo test --test memory_leak_test -- --nocapture
```

## Test Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View report
open coverage/index.html
```

### Coverage Targets
- Overall: ≥80%
- keyrx_core: ≥90%
- Critical paths (auth, security): ≥90%
- Bug fixes: 100% coverage of fixed code

## Test Maintenance

### Adding New Tests
1. Identify test category (memory, concurrency, e2e, etc.)
2. Add test function to appropriate file
3. Use `TestApp` for integration tests
4. Mark long-running tests with `#[ignore]`
5. Document in this README

### Test Naming Convention
- `test_<feature>_<scenario>` - Regular tests
- `test_<feature>_<scenario>_<duration>` - Long-running tests
- Use descriptive names explaining what is tested

### Best Practices
- Keep tests isolated (no shared state)
- Use deterministic test data
- Clean up resources (TestApp does this automatically)
- Assert meaningful conditions
- Add comments for complex test logic

## Troubleshooting

### "Address already in use" Error
Each TestApp instance uses a unique port. If you see this error:
```bash
# Kill processes using port range
pkill -f keyrx_daemon
```

### Tests Hanging
Check for:
- Infinite loops in test code
- Missing timeout on async operations
- Deadlocks in concurrent tests

### Flaky Tests
- Add explicit waits (`sleep`) where needed
- Increase timeouts for slow CI environments
- Check for race conditions

## References

- Test infrastructure: `tests/common/test_app.rs`
- CI configuration: `.github/workflows/ci.yml`
- Coverage reports: `coverage/index.html` (after running tarpaulin)
