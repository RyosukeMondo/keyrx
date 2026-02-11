# Error Handling Standards - KeyRx

## Overview

This document defines standardized error handling patterns for KeyRx frontend and backend.

## Frontend Error Handling (TypeScript/React)

### Error Hierarchy (`utils/errors.ts`)

- `KeyRxError` - Base error class
- `ApiError` - API/HTTP errors (404, 500, timeout)
- `ValidationError` - Input validation errors
- `NetworkError` - Network/connection errors
- `ProfileError` - Profile operation errors
- `ConfigError` - Configuration errors
- `DeviceError` - Device operation errors
- `WebSocketError` - WebSocket errors

### Structured Logging (`utils/logger.ts`)

Use `logger` instead of `console.*`:

```typescript
import { logger } from '@/utils/logger';
import { parseError } from '@/utils/errors';

// ❌ BAD
try {
  await someOperation();
} catch (err) {
  console.error('Operation failed:', err);
}

// ✅ GOOD
try {
  await someOperation();
  logger.info('operation_success', { userId: '123' });
} catch (err) {
  const error = parseError(err);
  logger.error('operation_failed', error, { userId: '123' });
  throw error;
}
```

### User Feedback

Always provide user feedback:

```typescript
import { toast } from '@/components/ui/toast';

try {
  await createProfile(name);
  toast.success('Profile created');
  logger.info('profile_created', { profileName: name });
} catch (err) {
  const error = parseError(err);
  toast.error(error.getUserMessage());
  logger.error('profile_creation_failed', error, { profileName: name });
}
```

### Never Use Silent Catch Blocks

```typescript
// ❌ BAD
try {
  await operation();
} catch (err) {}

// ✅ GOOD
try {
  await operation();
} catch (err) {
  const error = parseError(err);
  logger.error('operation_failed', error);
  toast.error(error.getUserMessage());
}
```

## Backend Error Handling (Rust)

### Structured Logging with Tracing

```rust
use tracing::{error, warn, info, debug};

// ❌ BAD
eprintln!("Failed to load config: {}", err);

// ✅ GOOD
info!(
    event = "server_started",
    port = %port,
    version = env!("CARGO_PKG_VERSION")
);
```

### Error Propagation with Context

```rust
// ❌ BAD - Generic error
fn load_profile(name: &str) -> Result<Profile, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_profile(content))
}

// ✅ GOOD - Contextual error
fn load_profile(name: &str) -> Result<Profile, MyError> {
    let content = std::fs::read_to_string(path)
        .map_err(|source| MyError::ProfileLoad {
            name: name.to_string(),
            source,
        })?;

    info!(event = "profile_loaded", profile = %name);
    Ok(parse_profile(content))
}
```

## Migration Checklist

### Frontend
- [ ] Replace `console.*` with `logger.*`
- [ ] Wrap errors with `parseError()`
- [ ] Add user feedback with `toast.*`
- [ ] Remove empty catch blocks
- [ ] No sensitive data in logs

### Backend
- [ ] Replace `eprintln!` with `tracing::*!` (except CLI)
- [ ] Add event names to log statements
- [ ] Add structured fields
- [ ] Ensure error propagation with context
- [ ] No sensitive data in logs
