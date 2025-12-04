# KeyRx Logging Standards

This document defines the logging standards for the KeyRx project. All developers must follow these guidelines to ensure consistent, searchable, and actionable logs across the codebase.

## Overview

KeyRx uses the [tracing](https://docs.rs/tracing) crate for structured logging. All logging must use the `tracing` macros and our convenience wrappers. **The use of `println!`, `eprintln!`, or `dbg!` for debugging is prohibited** except in user-facing CLI output.

### Core Principles

1. **Structured Over Unstructured**: Always use structured fields instead of string interpolation
2. **Fail Fast**: Log errors at the point they occur, with full context
3. **Performance Aware**: Logging must not block the engine or significantly impact performance
4. **Security First**: Never log secrets, passwords, tokens, or PII

## Log Levels

Use the appropriate log level based on the severity and purpose of the message.

### TRACE (Level 0)

**When to use:**
- Extremely verbose debugging information
- Entry/exit of functions
- Loop iterations
- Low-level protocol details

**Guidelines:**
- Only enable during deep debugging sessions
- Expected to be very noisy
- Not typically enabled in production

**Example:**
```rust
use tracing::trace;

fn process_key(key: u32) {
    trace!(key = key, "Entering process_key");
    // ... processing ...
    trace!(key = key, result = "success", "Exiting process_key");
}
```

### DEBUG (Level 1)

**When to use:**
- Detailed diagnostic information
- State transitions
- Intermediate computation results
- Configuration values
- Non-critical path execution

**Guidelines:**
- Useful for troubleshooting during development
- Can be enabled in production for specific modules
- Should not contain sensitive information

**Example:**
```rust
use tracing::debug;
use keyrx_core::log_event;
use tracing::Level;

log_event!(Level::DEBUG, "key_mapped",
    from_key = from,
    to_key = to,
    layer_id = layer.id(),
);
```

### INFO (Level 2)

**When to use:**
- Significant application events
- Lifecycle changes (started, stopped, reloaded)
- Successful major operations
- Configuration changes
- Normal business operations

**Guidelines:**
- Should be meaningful to operators
- Default production log level
- Every INFO log should answer "what happened?"

**Example:**
```rust
use keyrx_core::log_event;
use tracing::Level;

log_event!(Level::INFO, "engine_started",
    config_path = %config_path.display(),
    layer_count = layers.len(),
    driver = "evdev",
);

log_event!(Level::INFO, "config_reloaded",
    mappings_count = mappings.len(),
    layers_modified = modified_layers.len(),
);
```

### WARN (Level 3)

**When to use:**
- Recoverable errors
- Deprecated functionality usage
- Resource constraints (but not critical)
- Unexpected but handled situations
- Performance degradation

**Guidelines:**
- Indicates something that should be investigated
- The application can continue, but attention may be needed
- Should include enough context to take action

**Example:**
```rust
use tracing::warn;

warn!(
    buffer_size = buffer.len(),
    capacity = buffer.capacity(),
    "Event buffer approaching capacity"
);

warn!(
    latency_ms = latency.as_millis(),
    threshold_ms = threshold.as_millis(),
    "Key processing latency exceeded threshold"
);
```

### ERROR (Level 4)

**When to use:**
- Errors that prevent an operation from completing
- Unexpected failures
- Critical resource issues
- Data corruption or loss
- Unrecoverable errors that require intervention

**Guidelines:**
- Always include the error message
- Include context about what was being attempted
- Include any relevant identifiers or state
- Should be actionable

**Example:**
```rust
use keyrx_core::log_error;

if let Err(e) = driver.grab_device() {
    log_error!(e, "Failed to grab input device",
        device_path = %device_path.display(),
        retry_count = retry,
    );
}

// Alternative with tracing directly:
use tracing::error;

error!(
    error = %err,
    context = "Failed to load configuration",
    config_path = %path.display(),
    "Configuration error"
);
```

## Required Fields

### All Log Entries

Every log entry automatically includes:
- `timestamp`: Unix timestamp in microseconds
- `level`: The log level (TRACE/DEBUG/INFO/WARN/ERROR)
- `target`: The Rust module path where the log originated
- `message`: Human-readable description

### Event Field

For structured events, always include an `event` field that describes what happened:

```rust
use keyrx_core::log_event;
use tracing::Level;

log_event!(Level::INFO, "key_processed",
    key_code = 65,
    latency_us = 150,
);
```

The `event` field should:
- Use snake_case
- Be a verb in past tense (e.g., "key_processed", "config_loaded")
- Be consistent across the codebase for the same event types

### Error Logging

When logging errors, always include:
- `error`: The error message (use `%err` for Display formatting)
- `context`: String describing what was being attempted

```rust
use keyrx_core::log_error;

log_error!(err, "Failed to initialize driver",
    driver_type = "evdev",
    device_path = "/dev/input/event0",
);
```

### Performance Tracking

For performance-sensitive operations, use timed spans:

```rust
use keyrx_core::timed_span;

fn process_batch(events: &[Event]) {
    let _span = timed_span!("process_batch",
        batch_size = events.len()
    ).entered();

    // ... processing ...
}
```

## Convenience Macros

KeyRx provides three convenience macros for common logging patterns:

### `log_event!`

Use for structured event logging with consistent field naming:

```rust
use keyrx_core::log_event;
use tracing::Level;

// Simple event
log_event!(Level::INFO, "engine_started");

// Event with fields
log_event!(Level::DEBUG, "key_mapped",
    from_key = "a",
    to_key = "b",
    layer = 0,
);
```

### `log_error!`

Use for error logging with automatic formatting:

```rust
use keyrx_core::log_error;

if let Err(e) = operation() {
    log_error!(e, "Operation failed");

    // With additional context
    log_error!(e, "Failed to save configuration",
        config_path = "/etc/keyrx/config.toml",
        attempted_backup = true,
    );
}
```

### `timed_span!`

Use for performance tracking:

```rust
use keyrx_core::timed_span;

{
    let _span = timed_span!("database_query",
        query_type = "select",
        table = "mappings"
    ).entered();

    // Query execution
}
// Timing automatically recorded when span drops
```

## Field Naming Conventions

Use consistent field names across the codebase:

### Common Field Names

| Field | Type | Usage | Example |
|-------|------|-------|---------|
| `event` | String | Event type identifier | `"key_processed"` |
| `error` | Display | Error message | `%err` |
| `context` | String | Error context | `"Failed to load config"` |
| `key_code` | u32 | Key code value | `65` |
| `latency_us` | u64 | Latency in microseconds | `150` |
| `latency_ms` | u64 | Latency in milliseconds | `15` |
| `duration_us` | u64 | Duration in microseconds | `200` |
| `path` | Display | File path | `%path.display()` |
| `device_path` | Display | Device file path | `"/dev/input/event0"` |
| `driver` | String | Driver name | `"evdev"` |
| `layer_id` | usize | Layer identifier | `0` |
| `mapping_count` | usize | Number of mappings | `42` |
| `retry_count` | u32 | Retry attempt number | `3` |

### Field Naming Rules

1. **Use snake_case**: All field names use snake_case
2. **Include units**: Include units in the field name (`_us`, `_ms`, `_count`)
3. **Be specific**: Use descriptive names (`key_code` not `key`, `latency_us` not `time`)
4. **Use Display format**: For paths and complex types, use `%value` formatting

## Migration from println!

### Before and After Examples

**Before:**
```rust
println!("Processing key: {}", key);
eprintln!("Error loading config: {}", err);
```

**After:**
```rust
use keyrx_core::log_event;
use keyrx_core::log_error;
use tracing::Level;

log_event!(Level::DEBUG, "key_processing", key_code = key);
log_error!(err, "Failed to load configuration");
```

### Migration Rules

1. **Debug println! → DEBUG level**
   ```rust
   // Before
   println!("Debug: state = {:?}", state);

   // After
   use tracing::debug;
   debug!(state = ?state, "State snapshot");
   ```

2. **Info println! → INFO level**
   ```rust
   // Before
   println!("Started engine with {} layers", count);

   // After
   log_event!(Level::INFO, "engine_started", layer_count = count);
   ```

3. **Error eprintln! → ERROR level**
   ```rust
   // Before
   eprintln!("Failed to open device: {}", err);

   // After
   log_error!(err, "Failed to open device");
   ```

4. **User-facing output → Keep println!**
   ```rust
   // User-facing CLI output should remain println!
   println!("KeyRx v{}", VERSION);  // OK
   println!("Configuration saved successfully");  // OK
   ```

## Performance Considerations

### Disabled Logging Must Be Zero-Cost

Always use the macros, which compile to no-ops when the level is disabled:

```rust
// Good - zero cost when DEBUG is disabled
debug!(expensive_value = %compute_expensive(), "Debug info");

// Bad - always computes even when disabled
let value = compute_expensive();
debug!(value = %value, "Debug info");
```

### Expensive Computations

For expensive operations in log statements:

```rust
use tracing::debug;

// Only compute if DEBUG is enabled
if tracing::enabled!(tracing::Level::DEBUG) {
    let expensive_result = compute_expensive();
    debug!(result = %expensive_result, "Computed result");
}
```

### High-Frequency Logging

For code that logs in tight loops:

1. Use TRACE level (easily disabled)
2. Consider rate limiting
3. Aggregate and log summaries instead

```rust
// Instead of logging each event:
// trace!(key = key, "Key pressed");  // 1000s per second

// Log summaries:
log_event!(Level::INFO, "batch_processed",
    event_count = batch.len(),
    duration_us = elapsed.as_micros(),
);
```

## Security and Privacy

### Never Log Secrets

**NEVER log:**
- Passwords or password hashes
- API keys or tokens
- Private keys
- Session IDs
- Personal identification information (PII)

```rust
// BAD - logs sensitive data
debug!(api_key = api_key, "Making API request");

// GOOD - logs only safe metadata
debug!(api_endpoint = endpoint, "Making API request");
```

### Sanitize User Input

When logging user-provided data:

```rust
use tracing::debug;

// Truncate or sanitize if needed
let safe_input = user_input.chars().take(100).collect::<String>();
debug!(input_preview = %safe_input, "Processing user input");
```

## Output Formats

### JSON Format

For production use, enable JSON output:

```rust
use keyrx_core::observability::StructuredLogger;

StructuredLogger::new()
    .with_json()
    .with_level(tracing::Level::INFO)
    .init()?;
```

Example JSON output:
```json
{
  "timestamp": "2025-12-04T10:30:45.123456Z",
  "level": "INFO",
  "target": "keyrx_core::engine",
  "message": "Engine started",
  "fields": {
    "event": "engine_started",
    "layer_count": 3,
    "driver": "evdev"
  }
}
```

### Pretty Format

For development, use pretty printing:

```rust
StructuredLogger::new()
    .with_level(tracing::Level::DEBUG)
    .init()?;
```

Example pretty output:
```
2025-12-04T10:30:45.123456Z  INFO keyrx_core::engine: Engine started
    event: engine_started
    layer_count: 3
    driver: evdev
```

## FFI Bridge

Logs can be bridged to Flutter via FFI for in-app debugging:

```rust
use keyrx_core::observability::LogBridge;

let bridge = LogBridge::new();
bridge.set_callback(my_log_callback);
bridge.set_enabled(true);

// Logs will now be sent to the callback
```

From Flutter:
```dart
final service = ObservabilityService();
service.logStream.listen((logEntry) {
  print('${logEntry.level}: ${logEntry.message}');
});
```

## Testing

### Capturing Logs in Tests

```rust
#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    fn init_test_logger() {
        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_test_writer())
            .try_init();
    }

    #[test]
    fn test_with_logging() {
        init_test_logger();

        log_event!(Level::INFO, "test_started");
        // ... test code ...
        log_event!(Level::INFO, "test_completed");
    }
}
```

### Asserting Log Output

For tests that need to verify log output:

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

#[test]
fn test_logs_error() {
    let (logs, _handle) = capture_logs();

    // Code that should log an error
    let result = failing_operation();
    assert!(result.is_err());

    let logs = logs.lock().unwrap();
    assert!(logs.iter().any(|l| l.contains("Failed to")));
}
```

## Enforcement

### Linting

A Clippy lint warns against `println!` in non-CLI code:

```toml
# .cargo/config.toml or clippy.toml
[lints]
disallowed-methods = [
    { path = "std::println", reason = "Use tracing instead" },
    { path = "std::eprintln", reason = "Use tracing instead" },
    { path = "std::dbg", reason = "Use tracing::debug! instead" },
]
```

### Code Review

All pull requests must:
1. Use structured logging via `tracing` or our macros
2. Include appropriate log levels
3. Include structured fields
4. Not log sensitive data
5. Not use `println!` for debugging

## Examples

### Complete Example: Engine Module

```rust
use keyrx_core::{log_event, log_error, timed_span};
use tracing::{debug, info, warn, Level};

pub struct Engine {
    // ...
}

impl Engine {
    pub fn new(config: Config) -> Result<Self, EngineError> {
        log_event!(Level::INFO, "engine_initializing",
            layer_count = config.layers.len(),
            driver = %config.driver,
        );

        let engine = Self {
            // ...
        };

        log_event!(Level::INFO, "engine_initialized");
        Ok(engine)
    }

    pub fn process_event(&mut self, event: InputEvent) -> Result<(), EngineError> {
        let _span = timed_span!("process_event",
            event_type = event.type_code(),
            event_code = event.code(),
        ).entered();

        debug!(
            event_type = event.type_code(),
            event_code = event.code(),
            event_value = event.value(),
            "Processing input event"
        );

        match self.transform_event(event) {
            Ok(output) => {
                log_event!(Level::DEBUG, "event_transformed",
                    input_code = event.code(),
                    output_code = output.code(),
                );
                self.emit_event(output)
            }
            Err(e) => {
                log_error!(e, "Failed to transform event",
                    event_type = event.type_code(),
                    event_code = event.code(),
                );
                Err(e)
            }
        }
    }

    fn check_health(&self) {
        if self.event_buffer.len() > self.event_buffer.capacity() * 9 / 10 {
            warn!(
                buffer_size = self.event_buffer.len(),
                capacity = self.event_buffer.capacity(),
                "Event buffer approaching capacity"
            );
        }
    }
}
```

## References

- [tracing documentation](https://docs.rs/tracing)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber)
- KeyRx Requirements: `.spec-workflow/specs/logging-standardization/requirements.md`
- KeyRx Design: `.spec-workflow/specs/logging-standardization/design.md`
