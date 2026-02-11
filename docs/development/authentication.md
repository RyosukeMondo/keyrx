# Authentication System

## Overview

KeyRx implements a comprehensive JWT-based authentication system with password hashing, rate limiting, and secure session management.

## Features

### 1. JWT Token Management
- **Access Tokens**: 15-minute expiry for short-lived access
- **Refresh Tokens**: 7-day expiry for long-term sessions
- **Token Types**: Separate validation for access vs refresh tokens
- **Secure Signing**: Uses HMAC-SHA256 with configurable secrets

### 2. Password Security
- **Argon2id Hashing**: Industry-standard password hashing algorithm
- **Salt Generation**: Unique salt per password using secure random
- **Complexity Requirements**:
  - Minimum 12 characters
  - At least 3 of: uppercase, lowercase, digit, special character

### 3. Rate Limiting
- **Login Attempts**: 5 attempts per minute per IP address
- **Automatic Cleanup**: Old entries removed after window expires
- **Reset on Success**: Successful login resets counter

### 4. Secure Session Management
- **httpOnly Cookies**: Prevents XSS attacks
- **Secure Flag**: HTTPS-only transmission
- **SameSite**: CSRF protection
- **Automatic Refresh**: Tokens refreshed before expiry

## Configuration

### Environment Variables

```bash
# JWT Authentication (recommended)
export KEYRX_JWT_SECRET="your-secure-secret-key-min-32-chars"
export KEYRX_ADMIN_PASSWORD="YourSecureP@ssw0rd123"

# Legacy Password Authentication (backward compatibility)
export KEYRX_ADMIN_PASSWORD="your_secure_password"

# Development Mode (no authentication)
# Don't set any of the above variables
```

### Security Best Practices

1. **JWT Secret**: Use a strong random string (32+ characters)
2. **Admin Password**: Follow complexity requirements
3. **HTTPS**: Always use HTTPS in production
4. **Rotate Secrets**: Periodically rotate JWT secrets
5. **Monitor Logs**: Watch for suspicious login attempts

## API Endpoints

### POST /api/auth/login
Login with username and password.

**Request:**
```json
{
  "username": "admin",
  "password": "YourSecureP@ssw0rd123"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Errors:**
- `401 Unauthorized`: Invalid credentials
- `429 Too Many Requests`: Rate limit exceeded

### POST /api/auth/logout
Logout (invalidate session).

**Request:**
```
Authorization: Bearer <access_token>
```

**Response:**
- `204 No Content`: Success

### POST /api/auth/refresh
Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "eyJhbGc..."
}
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Errors:**
- `401 Unauthorized`: Invalid or expired refresh token

### GET /api/auth/validate
Validate current access token.

**Request:**
```
Authorization: Bearer <access_token>
```

**Response:**
```json
{
  "valid": true,
  "user_id": "admin",
  "expires_at": 1234567890
}
```

## Frontend Integration

### AuthContext Setup

```tsx
import { AuthProvider } from './contexts/AuthContext';

function App() {
  return (
    <AuthProvider>
      {/* Your app components */}
    </AuthProvider>
  );
}
```

### Using Authentication

```tsx
import { useAuth } from './contexts/AuthContext';

function LoginPage() {
  const { login, error, isLoading } = useAuth();

  const handleSubmit = async (username: string, password: string) => {
    await login(username, password);
    // Redirected to / on success
  };

  return <LoginForm onSubmit={handleSubmit} />;
}
```

### Protected Routes

```tsx
import { useAuth } from './contexts/AuthContext';
import { Navigate } from 'react-router-dom';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) return <div>Loading...</div>;
  if (!isAuthenticated) return <Navigate to="/login" />;

  return <>{children}</>;
}
```

### API Requests with Authentication

```tsx
import { getAuthHeader } from './contexts/AuthContext';

async function fetchProfiles() {
  const response = await fetch('/api/profiles', {
    headers: {
      ...getAuthHeader(),
      'Content-Type': 'application/json',
    },
  });

  if (response.status === 401) {
    // Token expired - AuthContext will handle redirect
    return;
  }

  return response.json();
}
```

## Testing

### Backend Tests

```bash
# Run all authentication tests
cargo test --package keyrx_daemon auth

# Run specific test module
cargo test --package keyrx_daemon auth::jwt::tests
cargo test --package keyrx_daemon auth::password::tests
cargo test --package keyrx_daemon auth::rate_limit::tests
```

### Frontend Tests

```bash
# Run authentication context tests
cd keyrx_ui
npm test src/contexts/AuthContext.test.tsx

# Run login form tests
npm test src/components/LoginForm.test.tsx
```

### Integration Tests

```bash
# Test full authentication flow
cargo test --package keyrx_daemon test_full_authentication_flow
```

## Security Considerations

### Token Storage
- **Access Token**: Stored in localStorage (short-lived, acceptable risk)
- **Refresh Token**: Stored in localStorage (consider httpOnly cookies for production)
- **Never Log Tokens**: Tokens should never appear in logs

### Rate Limiting
- **IP-based**: Prevents brute-force attacks per IP
- **Consider**: Add user-based rate limiting for additional security

### Password Reset
- Not implemented in v1.0
- Requires email/SMS verification for production use

### Multi-Factor Authentication
- Not implemented in v1.0
- Consider adding TOTP/WebAuthn for enhanced security

## Troubleshooting

### "Invalid token" errors
- Check JWT secret is set correctly
- Verify token hasn't expired
- Ensure token is passed in Authorization header

### "Too many login attempts"
- Wait for rate limit window to expire (60 seconds)
- Check for IP address issues in logs
- Verify rate limiter configuration

### "Password too weak" errors
- Ensure password meets complexity requirements
- Minimum 12 characters
- At least 3 character types

## Migration from Legacy Auth

### V1 (Legacy Password Auth)
```bash
export KEYRX_ADMIN_PASSWORD="your_password"
```

### V2 (JWT Auth - Recommended)
```bash
export KEYRX_JWT_SECRET="your-secure-secret-key"
export KEYRX_ADMIN_PASSWORD="YourSecureP@ssw0rd123"
```

The system automatically detects JWT secret and upgrades to JWT authentication while maintaining backward compatibility with legacy password auth.
