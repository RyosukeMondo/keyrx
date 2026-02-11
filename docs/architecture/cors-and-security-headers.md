# CORS and Security Headers Configuration

## Overview

This document describes keyrx's CORS (Cross-Origin Resource Sharing) configuration and security headers implementation, which provide production-ready protection against common web vulnerabilities.

## CORS Configuration

### Default Behavior

**Development Mode** (default):
- Automatically allows common localhost origins
- Default origins: `http://localhost:3000`, `http://localhost:5173`, `http://localhost:8080`, `http://127.0.0.1:3000`, `http://127.0.0.1:5173`, `http://127.0.0.1:8080`
- Enables credentials (cookies, authorization headers)

**Production Mode**:
- No default origins - must be explicitly configured
- Enforces whitelist from `KEYRX_ALLOWED_ORIGINS` environment variable
- Requires explicit origin configuration to prevent accidental exposure

### Configuration via Environment Variable

Set `KEYRX_ALLOWED_ORIGINS` to a comma-separated list of allowed origins:

```bash
# Single origin
export KEYRX_ALLOWED_ORIGINS="https://app.example.com"

# Multiple origins
export KEYRX_ALLOWED_ORIGINS="https://app.example.com, https://dashboard.example.com, https://admin.example.com"

# Development (with spaces)
export KEYRX_ALLOWED_ORIGINS="http://localhost:3000, http://localhost:5173"
```

### Environment Detection

keyrx automatically detects production environment via:
1. `RUST_ENV=production`
2. `ENVIRONMENT=production`

```bash
# Production configuration
RUST_ENV=production KEYRX_ALLOWED_ORIGINS="https://app.example.com" keyrx-daemon

# Development configuration (default)
keyrx-daemon  # Uses default dev origins
```

### CORS Headers Applied

All responses include:
- `Access-Control-Allow-Origin`: Whitelist from configuration
- `Access-Control-Allow-Methods`: GET, POST, PUT, DELETE, PATCH, OPTIONS
- `Access-Control-Allow-Headers`: Content-Type, Authorization
- `Access-Control-Allow-Credentials`: true

## Security Headers

All responses include comprehensive security headers:

### Content-Security-Policy (CSP)

Prevents XSS and injection attacks by restricting resource loading:

**Production CSP:**
```
default-src 'self'
script-src 'self' 'wasm-unsafe-eval'
style-src 'self'
img-src 'self' data:
font-src 'self'
connect-src 'self'
frame-ancestors 'none'
base-uri 'self'
form-action 'self'
```

**Development CSP** (more permissive for development):
```
default-src 'self'
script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'
style-src 'self' 'unsafe-inline'
img-src 'self' data:
font-src 'self'
connect-src 'self' ws: wss:
frame-ancestors 'none'
base-uri 'self'
form-action 'self'
```

### X-Content-Type-Options

Prevents MIME type sniffing:
```
X-Content-Type-Options: nosniff
```

### X-Frame-Options

Prevents clickjacking attacks:
```
X-Frame-Options: DENY
```

### Referrer-Policy

Controls referrer information sent to other sites:
```
Referrer-Policy: strict-origin-when-cross-origin
```

### Strict-Transport-Security (HSTS)

Enforces HTTPS in production (disabled in development):
```
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
```

Parameters:
- `max-age=63072000` - 2 years (production)
- `includeSubDomains` - Apply to all subdomains
- `preload` - Eligible for browser HSTS preload list

### X-Permitted-Cross-Domain-Policies

Restricts Flash/PDF cross-domain access:
```
X-Permitted-Cross-Domain-Policies: none
```

### Permissions-Policy

Controls browser features (formerly Feature-Policy):
```
Permissions-Policy: geolocation=(), microphone=(), camera=(), magnetometer=(), gyroscope=(), accelerometer=(), usb=(), payment=()
```

## Implementation Details

### Architecture

```
┌─ HTTP Request
│
├─ Security Headers Middleware (outermost)
│  └─ Adds all security headers to response
│
├─ Timeout Middleware
│  └─ Enforces request timeout (30s)
│
├─ Security Middleware
│  └─ Validates URL length, checks path traversal
│
├─ Input Validation Middleware
│  └─ Validates request structure, size limits
│
├─ Rate Limit Middleware
│  └─ Enforces rate limiting (10 req/sec)
│
├─ Auth Middleware (innermost handler layer)
│  └─ Validates authentication tokens
│
└─ CORS Layer (first to apply)
   └─ Validates origin, sets CORS headers
```

### Code Structure

1. **daemon_config.rs**: CORS origin configuration and parsing
   - `DaemonConfig::cors_origins()` - Get configured origins
   - `DaemonConfig::parse_cors_origins()` - Parse from environment variable

2. **middleware/security_headers.rs**: Security headers middleware
   - `SecurityHeadersLayer` - Middleware configuration layer
   - `SecurityHeadersConfig` - Header configuration
   - `security_headers_middleware()` - Middleware handler function

3. **web/mod.rs**: Application router setup
   - Integrates all middleware layers
   - Applies CORS configuration
   - Selects production or development security headers

## Testing

### Unit Tests

Test CORS configuration parsing:
```bash
cargo test -p keyrx_daemon --lib daemon_config
```

Test security headers middleware:
```bash
cargo test -p keyrx_daemon --lib security_headers
```

### Manual Testing

Test CORS preflight request:
```bash
curl -X OPTIONS http://localhost:9867/api/health \
  -H "Origin: https://app.example.com" \
  -H "Access-Control-Request-Method: GET" \
  -v
```

Test security headers:
```bash
curl http://localhost:9867/api/health -v | grep -E "^< [A-Za-z-]+:"
```

Expected headers:
```
< content-security-policy: ...
< x-content-type-options: nosniff
< x-frame-options: DENY
< referrer-policy: strict-origin-when-cross-origin
< x-permitted-cross-domain-policies: none
< permissions-policy: ...
< strict-transport-security: ... (production only)
```

## Security Considerations

### Development vs Production

| Setting | Development | Production |
|---------|-------------|------------|
| **CORS Default** | Permissive localhost | Requires explicit origins |
| **HSTS** | Disabled | Enabled (2-year max-age) |
| **CSP** | Allows unsafe-inline | Blocks unsafe-inline |
| **WebSocket** | Allowed | Same-origin only |

### Best Practices

1. **Never use `allow-all` CORS in production**
   - Always explicitly whitelist origins
   - Remove `CorsLayer::permissive()` from production code

2. **Validate CORS origins early**
   - Browser enforces preflight requests
   - Server validates Origin header on all requests
   - Requests from non-whitelisted origins are rejected

3. **Monitor security headers**
   - Verify headers are present in production
   - Use https://securityheaders.com for audit
   - Test with browser DevTools (F12)

4. **Update HSTS carefully**
   - 2-year max-age requires careful testing
   - Start with shorter periods (1 month) during rollout
   - Only enable on dedicated HTTPS domain

5. **Handle WebSocket CORS**
   - WebSocket upgrade uses same CORS validation
   - Configure `connect-src` in CSP for WebSocket origins
   - Use `wss://` (WebSocket Secure) in production

## Troubleshooting

### CORS errors in browser console

**Error:** `Access to fetch has been blocked by CORS policy`

**Solution:**
1. Check browser console for exact error
2. Verify origin matches `KEYRX_ALLOWED_ORIGINS`
3. Ensure preflight request succeeds (OPTIONS)
4. Check `Access-Control-Allow-Headers` includes required headers

### CSP violations

**Error:** `Refused to load the script because it violates the Content Security Policy`

**Solution:**
1. Check browser console for blocked resource
2. For development: Use `SecurityHeadersLayer::dev()` to allow unsafe-inline
3. For production: Update CSP to allow specific resource or inline content

### HSTS errors

**Error:** `The certificate is not trusted because no issuer chain was provided`

**Solution:**
1. HSTS requires valid HTTPS certificate
2. Disable HSTS in development: Use `SecurityHeadersLayer::dev()`
3. Ensure HTTPS redirect is in place before enabling HSTS

### Configuration not applied

**Symptoms:** CORS headers not in response

**Solution:**
1. Check `KEYRX_ALLOWED_ORIGINS` is set correctly
2. Restart daemon after changing environment variable
3. Verify middleware is registered in router
4. Check logs for configuration load errors

## Related Documentation

- [Security Architecture Audit](../security-architecture-audit.md) - CORS section
- [CLAUDE.md - Security Guidelines](../CLAUDE.md) - Error handling and secrets
- [keyrx Daemon Manual](./daemon.md) - Server configuration

## References

- [OWASP - Cross-Origin Resource Sharing](https://owasp.org/www-community/attacks/csrf)
- [MDN - CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [MDN - Content-Security-Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy)
- [OWASP - Security Headers](https://owasp.org/www-project-secure-headers/)
- [securityheaders.com](https://securityheaders.com) - Security header audit tool
