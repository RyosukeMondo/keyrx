# Security Quick Reference Guide

**KeyRx Daemon v0.1.1 - Security Hardening Complete**

---

## 🚀 Quick Start

### Development Mode (No Auth)
```bash
# Default - all endpoints accessible
keyrx_daemon

# Test
curl http://localhost:9867/api/profiles
```

### Production Mode (Auth Required)
```bash
# Set admin password
export KEYRX_ADMIN_PASSWORD=your_secure_password

# Start daemon
keyrx_daemon

# Make authenticated request
curl -H "Authorization: Bearer your_secure_password" http://localhost:9867/api/profiles
```

---

## 🔐 Authentication

### Environment Variable
```bash
export KEYRX_ADMIN_PASSWORD=your_password
```

### Authorization Header Format
```
Authorization: Bearer your_password
```

### Example Requests
```bash
# List profiles (authenticated)
curl -H "Authorization: Bearer mypassword" http://localhost:9867/api/profiles

# Activate profile (authenticated)
curl -X POST -H "Authorization: Bearer mypassword" \
  http://localhost:9867/api/profiles/gaming/activate

# Health check (no auth required)
curl http://localhost:9867/health
```

---

## 🛡️ Security Features Summary

| Feature | Status | Default Config |
|---------|--------|----------------|
| Authentication | ✅ Enabled | Password or DevMode |
| CORS | ✅ Restricted | Localhost-only |
| Path Traversal Protection | ✅ Enabled | Always active |
| Rate Limiting | ✅ Enabled | 10 req/sec per IP |
| Request Size Limits | ✅ Enabled | 1MB body, 10KB URL |
| Timeout | ✅ Enabled | 5 seconds |
| Input Sanitization | ✅ Enabled | HTML, null bytes, control chars |
| DoS Protection | ✅ Enabled | Max 100 WebSocket connections |
| Audit Logging | ✅ Enabled | Security events logged |

---

## 📋 Test Verification

```bash
# Run security tests (16 tests)
cargo test --test security_hardening_test

# Run validation tests (36 tests)
cargo test --test data_validation_test

# Expected: All tests pass ✅
```

---

## 🔍 Monitoring

### Log Messages to Monitor

```bash
# Authentication failures
[WARN] Authentication failed: Missing Authorization header
[WARN] Authentication failed: Invalid password

# Path traversal attempts
[WARN] Path traversal attempt detected: /api/../secret

# Rate limiting
[INFO] Rate limit exceeded for IP: 127.0.0.1:8080
```

### Check Logs
```bash
# With journalctl (systemd)
journalctl -u keyrx_daemon -f | grep -E "WARN|ERROR"

# With log files
tail -f /var/log/keyrx/daemon.log | grep -E "Authentication|traversal|Rate limit"
```

---

## ⚙️ Configuration

### Rate Limiting (Default)
- **Max Requests:** 10 per second per IP
- **Window:** 1 second sliding window

### Request Limits (Default)
- **Body Size:** 1MB
- **URL Length:** 10KB
- **WebSocket Connections:** 100 concurrent

### Profile Limits (Default)
- **Max Profile Size:** 100KB
- **Max Profile Count:** 10

### Timeout (Default)
- **Request Timeout:** 5 seconds

---

## 🚨 Common Issues

### Issue: "Authentication failed" with correct password

**Solution:** Check password encoding
```bash
# Ensure no trailing whitespace
export KEYRX_ADMIN_PASSWORD=$(echo -n "mypassword")

# Verify environment variable
echo $KEYRX_ADMIN_PASSWORD
```

### Issue: CORS errors in browser

**Solution:** Check allowed origins
```rust
// In keyrx_daemon/src/web/mod.rs
// Add your origin if needed (localhost variants only)
.allow_origin("http://localhost:3000".parse().unwrap())
```

### Issue: "Rate limit exceeded"

**Solution:** Wait for window to reset (1 second) or adjust rate limit

```rust
// Custom rate limit (in code)
let rate_limiter = RateLimitLayer::with_config(RateLimitConfig {
    max_requests: 100,
    window: Duration::from_secs(60),  // 100 req/min
});
```

### Issue: "URI too long"

**Solution:** Reduce query parameter size (10KB limit)

---

## 📚 API Examples

### Profile Management (All Require Auth)

```bash
# List all profiles
curl -H "Authorization: Bearer $KEYRX_ADMIN_PASSWORD" \
  http://localhost:9867/api/profiles

# Get profile details
curl -H "Authorization: Bearer $KEYRX_ADMIN_PASSWORD" \
  http://localhost:9867/api/profiles/gaming

# Create profile
curl -X POST -H "Authorization: Bearer $KEYRX_ADMIN_PASSWORD" \
  -H "Content-Type: application/json" \
  -d '{"name":"myprofile","template":"blank"}' \
  http://localhost:9867/api/profiles

# Activate profile
curl -X POST -H "Authorization: Bearer $KEYRX_ADMIN_PASSWORD" \
  http://localhost:9867/api/profiles/myprofile/activate

# Delete profile
curl -X DELETE -H "Authorization: Bearer $KEYRX_ADMIN_PASSWORD" \
  http://localhost:9867/api/profiles/myprofile
```

### Health Endpoint (No Auth Required)

```bash
# Daemon health check
curl http://localhost:9867/health

# API health check
curl http://localhost:9867/api/health
```

---

## 🔒 Security Best Practices

### 1. Password Management
✅ Use strong passwords (≥16 chars, mixed case, symbols)
✅ Store in credential managers (1Password, Bitwarden)
✅ Rotate regularly (every 90 days)
❌ Never commit passwords to version control
❌ Never share passwords in plain text

### 2. Network Exposure
✅ Keep daemon on localhost (default: 127.0.0.1:9867)
✅ Use VPN for remote access
❌ Don't expose to public internet without TLS
❌ Don't allow remote connections without VPN

### 3. Systemd Service
```ini
[Unit]
Description=KeyRx Keyboard Remapping Daemon
After=network.target

[Service]
Type=simple
User=myuser
Environment="KEYRX_ADMIN_PASSWORD=your_secure_password"
ExecStart=/usr/bin/keyrx_daemon
Restart=on-failure
PrivateTmp=yes
NoNewPrivileges=yes

[Install]
WantedBy=multi-user.target
```

### 4. Log Rotation
```bash
# /etc/logrotate.d/keyrx
/var/log/keyrx/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

---

## 📊 Security Test Coverage

| Category | Tests | Status |
|----------|-------|--------|
| Authentication | 3 | ✅ Passing |
| CORS | 1 | ✅ Passing |
| Path Traversal | 4 | ✅ Passing |
| Rate Limiting | 4 | ✅ Passing |
| Request Limits | 2 | ✅ Passing |
| Timeout | 3 | ✅ Passing |
| Sanitization | 6 | ✅ Passing |
| DoS Protection | 1 | ✅ Passing |
| File Operations | 2 | ✅ Passing |
| Error Messages | 1 | ✅ Passing |
| Resource Limits | 3 | ✅ Passing |
| Audit Logging | 1 | ✅ Passing |
| **Validation Tests** | **36** | **✅ Passing** |
| **Total** | **52** | **✅ All Passing** |

---

## 🔗 Related Documentation

- **Full Security Report:** `WS5_SECURITY_COMPLETE.md`
- **Security Audit:** `SECURITY_AUDIT_REPORT.md`
- **API Documentation:** `keyrx_daemon/src/web/api/README.md`
- **Bug Remediation:** `.spec-workflow/specs/bug-remediation-sweep/`

---

## 📞 Support

**Security Issues:** Report security vulnerabilities privately to the maintainers

**General Issues:** https://github.com/yourusername/keyrx/issues

---

**Last Updated:** 2026-01-28
**Version:** KeyRx v0.1.1
**Status:** ✅ Production Ready (with admin password configured)
