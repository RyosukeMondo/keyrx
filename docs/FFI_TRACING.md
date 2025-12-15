# FFI Tracing Infrastructure

This document describes the FFI tracing infrastructure for observability and debugging.

## Overview

The FFI tracing infrastructure provides:
- **Structured tracing** for all FFI calls
- **Correlation ID tracking** for request flows
- **Chrome DevTools compatible** JSON output
- **Minimal performance overhead** (<5%)
- **Flexible configuration** for development and production

## Quick Start

### 1. Initialize Tracing

Initialize tracing once at application startup:

```rust
use keyrx_core::ffi::tracing::{init, TracingConfig};

fn main() {
    // Initialize with default configuration
    init(TracingConfig::default());

    // Or customize configuration
    init(TracingConfig {
        enable_file_output: true,
        output_dir: ".keyrx/traces".to_string(),
        enable_chrome_trace: true,
        min_level: tracing::Level::DEBUG,
        enable_console: true,
    });
}
```

### 2. Add Instrumentation to FFI Functions

Use the `#[instrument]` macro with correlation ID tracking:

```rust
use tracing::instrument;
use ffi_export::ffi_export;
use crate::ffi::tracing::{new_correlation_id, correlation_id};

#[instrument(skip(context), fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "engine", version = "1.0")]
pub extern "C" fn keyrx_engine_start(context: *mut EngineContext) -> i32 {
    // Generate a new correlation ID for this request
    new_correlation_id();

    // Function implementation...

    tracing::info!("Engine started successfully");
    0 // Success
}
```

### 3. Use Helper Macros

The tracing module provides convenient helper macros:

```rust
use keyrx_core::{ffi_span, ffi_trace_entry, ffi_trace_exit};

pub extern "C" fn keyrx_some_function() -> i32 {
    // Create a span with correlation ID
    let _span = ffi_span!("some_function", domain = "engine");

    // Or use entry/exit macros
    ffi_trace_entry!("keyrx_some_function");

    let result = do_work();

    ffi_trace_exit!("keyrx_some_function", result);

    result
}
```

## Configuration

### TracingConfig

The `TracingConfig` struct controls tracing behavior:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable_file_output` | `bool` | `false` | Enable JSON file output for traces |
| `output_dir` | `String` | `".keyrx/traces"` | Directory for trace files |
| `enable_chrome_trace` | `bool` | `false` | Enable Chrome DevTools compatible output |
| `min_level` | `Level` | `INFO` | Minimum log level (TRACE, DEBUG, INFO, WARN, ERROR) |
| `enable_console` | `bool` | `true` | Enable console output |

### Environment Variables

Tracing can be configured via environment variables:

```bash
# Set log level
export RUST_LOG=keyrx_core::ffi=trace

# Enable specific modules
export RUST_LOG=keyrx_core::ffi::exports_engine=debug

# Multiple modules
export RUST_LOG=keyrx_core::ffi=info,keyrx_core::engine=debug
```

## Correlation IDs

Correlation IDs help track request flows across FFI boundaries.

### Creating Correlation IDs

```rust
use keyrx_core::ffi::tracing::new_correlation_id;

// Generate a new correlation ID
let id = new_correlation_id();
println!("Request ID: {}", id);
```

### Getting Current Correlation ID

```rust
use keyrx_core::ffi::tracing::correlation_id;

// Get the current thread's correlation ID
let id = correlation_id();
```

### Setting Correlation ID from Dart

When receiving a correlation ID from Dart:

```rust
use keyrx_core::ffi::tracing::set_correlation_id;

#[ffi_export(domain = "engine", version = "1.0")]
pub extern "C" fn keyrx_set_correlation_id(id: u64) {
    set_correlation_id(id);
}
```

### Clearing Correlation ID

```rust
use keyrx_core::ffi::tracing::clear_correlation_id;

// Clear the correlation ID when request completes
clear_correlation_id();
```

## Structured Logging

Use structured fields for rich logging:

```rust
use tracing::{info, warn, error, debug, trace};

#[instrument(skip(config), fields(correlation_id = %correlation_id(), config_size = config.len()))]
pub extern "C" fn keyrx_apply_config(config: *const u8, len: usize) -> i32 {
    debug!(config_ptr = ?config, "Received config pointer");

    info!(
        config_version = "1.0",
        config_size = len,
        "Applying configuration"
    );

    if len > MAX_CONFIG_SIZE {
        warn!(
            config_size = len,
            max_size = MAX_CONFIG_SIZE,
            "Configuration exceeds maximum size"
        );
        return -1;
    }

    match apply_config_internal(config, len) {
        Ok(_) => {
            info!("Configuration applied successfully");
            0
        }
        Err(e) => {
            error!(error = %e, "Failed to apply configuration");
            -2
        }
    }
}
```

## Output Formats

### Console Output

Default human-readable format:

```
2025-12-15T08:00:00.123Z DEBUG keyrx_core::ffi::exports_engine correlation_id=42: Engine started
2025-12-15T08:00:00.456Z  INFO keyrx_core::ffi::exports_engine correlation_id=42: Configuration applied
```

### JSON File Output

Structured JSON format for analysis:

```json
{
  "timestamp": "2025-12-15T08:00:00.123Z",
  "level": "DEBUG",
  "target": "keyrx_core::ffi::exports_engine",
  "fields": {
    "correlation_id": 42,
    "message": "Engine started"
  },
  "span": {
    "name": "keyrx_engine_start",
    "correlation_id": 42
  }
}
```

### Chrome DevTools Format (Future)

Timeline-compatible format for visualization in DevTools (to be implemented in task 18):

```json
{
  "name": "keyrx_engine_start",
  "cat": "ffi",
  "ph": "X",
  "pid": 1,
  "tid": 1,
  "ts": 1234567890,
  "dur": 1500,
  "args": {
    "correlation_id": 42,
    "domain": "engine"
  }
}
```

## Best Practices

### 1. Always Use Correlation IDs

Generate correlation IDs at FFI entry points:

```rust
#[instrument(fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "engine", version = "1.0")]
pub extern "C" fn keyrx_entry_point() -> i32 {
    new_correlation_id(); // Generate at entry
    // ... implementation
}
```

### 2. Skip Sensitive Parameters

Don't log sensitive data in spans:

```rust
#[instrument(
    skip(password, api_key),
    fields(
        correlation_id = %correlation_id(),
        username = %username
    )
)]
pub fn authenticate(username: &str, password: &str, api_key: &str) -> Result<()> {
    // password and api_key are not logged
}
```

### 3. Use Appropriate Log Levels

- `TRACE`: Very detailed debugging (function entry/exit)
- `DEBUG`: Debugging information (state changes, decisions)
- `INFO`: Normal operations (successful operations)
- `WARN`: Warning conditions (degraded performance, recoverable errors)
- `ERROR`: Error conditions (unrecoverable errors)

### 4. Add Context with Fields

```rust
info!(
    device_id = %device.id,
    device_name = %device.name,
    "Device connected"
);
```

### 5. Use Spans for Timing

```rust
#[instrument]
pub fn expensive_operation() -> Result<()> {
    // Span automatically measures duration
    do_work()?;
    Ok(())
}
```

## Performance Considerations

### Target: <5% Overhead

The tracing infrastructure is designed to have minimal impact:

1. **Conditional compilation**: Use `#[instrument]` selectively
2. **Lazy evaluation**: Structured fields are only evaluated when logged
3. **Efficient correlation IDs**: Thread-local storage for fast access
4. **Filtered output**: Only log at configured levels

### Measuring Overhead

Use benchmarks to measure tracing overhead:

```bash
# Benchmark without tracing
cargo bench --bench ffi_bench -- --baseline baseline

# Enable tracing
export RUST_LOG=keyrx_core::ffi=debug

# Benchmark with tracing
cargo bench --bench ffi_bench -- --baseline tracing

# Compare results
# Overhead should be <5%
```

## Troubleshooting

### No Output

If tracing produces no output:

1. Check initialization: `init()` must be called once
2. Check log level: Ensure `RUST_LOG` or `min_level` is set appropriately
3. Check filters: Verify target modules are enabled

### Performance Issues

If tracing causes performance problems:

1. Reduce log level: Use `INFO` or `WARN` in production
2. Disable file output: Console-only is faster
3. Remove unnecessary `#[instrument]` macros
4. Filter targets: Only enable specific modules

### Missing Correlation IDs

If correlation IDs are 0 or missing:

1. Call `new_correlation_id()` at FFI entry points
2. Ensure correlation ID is propagated across threads
3. Check that `correlation_id()` is called in the same thread

## Integration with Dart DevTools Timeline

FFI traces can be visualized in Flutter DevTools Timeline using the `FfiTracingBridge`.

### Quick Start

```dart
import 'package:keyrx/ffi/tracing_bridge.dart';

void main() async {
  // Enable FFI trace collection
  FfiTracingBridge.enable();

  // Start Timeline integration (polls every 100ms by default)
  await FfiTracingBridge.startTimelineIntegration();

  // Your app code here...
  // FFI calls will now appear in DevTools Timeline

  // When done:
  FfiTracingBridge.stopTimelineIntegration();
  FfiTracingBridge.disable();
}
```

### Manual Event Access

You can also access trace events directly without Timeline integration:

```dart
// Get all events
final events = FfiTracingBridge.getEvents();
for (final event in events) {
  print('FFI call: ${event.name} took ${event.durationUs}μs');
}

// Get filtered events
final slowCalls = FfiTracingBridge.getFilteredEvents(
  FfiTraceFilter(
    domain: 'engine',
    minDurationUs: 1000, // Only calls > 1ms
  ),
);

// Get statistics
final stats = FfiTracingBridge.getStats();
print(stats); // Shows total events, by domain, by status, etc.
```

### Filtering Options

```dart
const filter = FfiTraceFilter(
  domain: 'engine',           // Filter by domain (null = all)
  minDurationUs: 1000,        // Minimum duration in μs
  functionPattern: r'keyrx_engine_.*', // Regex pattern
  resultStatus: FfiResultStatus.error, // Only errors
);

final filtered = FfiTracingBridge.getFilteredEvents(filter);
```

### Running with Tracing

Use the `just dev-trace` command to run your app with tracing enabled:

```bash
# Start development with tracing
just dev-trace

# Or manually with environment variable
RUST_LOG=keyrx_core::ffi=trace,info just dev
```

### Viewing in DevTools

1. Start your app with tracing enabled: `just dev-trace`
2. Open Flutter DevTools in your browser
3. Navigate to the Timeline tab
4. Enable timeline integration in your app:
   ```dart
   FfiTracingBridge.enable();
   await FfiTracingBridge.startTimelineIntegration();
   ```
5. Interact with your app
6. FFI calls will appear in the Timeline with:
   - Function name as the event name
   - Duration visualization
   - Arguments (sanitized)
   - Correlation IDs
   - Result status (success/error)

### Performance Considerations

- **Polling Interval**: Default 100ms. Adjust based on your needs:
  ```dart
  await FfiTracingBridge.startTimelineIntegration(
    pollInterval: Duration(milliseconds: 50), // More frequent
  );
  ```
- **Buffer Size**: The Rust side buffers up to 1000 events (circular buffer)
- **Timeline Impact**: Timeline events themselves have minimal overhead (<1%)
- **Production**: Disable tracing in production builds for best performance

## Examples

### Example 1: Simple FFI Function

```rust
use tracing::instrument;
use keyrx_core::ffi::tracing::{new_correlation_id, correlation_id};

#[instrument(fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "device", version = "1.0")]
pub extern "C" fn keyrx_device_connect(device_id: u32) -> i32 {
    new_correlation_id();

    tracing::info!(device_id = device_id, "Connecting to device");

    match connect_device_internal(device_id) {
        Ok(_) => {
            tracing::info!(device_id = device_id, "Device connected successfully");
            0
        }
        Err(e) => {
            tracing::error!(device_id = device_id, error = %e, "Failed to connect to device");
            -1
        }
    }
}
```

### Example 2: Complex FFI Operation with Spans

```rust
use tracing::{instrument, info, warn, debug};
use keyrx_core::ffi::tracing::{new_correlation_id, correlation_id};

#[instrument(skip(config_ptr), fields(correlation_id = %correlation_id(), config_size = config_len))]
#[ffi_export(domain = "engine", version = "1.0")]
pub extern "C" fn keyrx_engine_configure(
    config_ptr: *const u8,
    config_len: usize
) -> i32 {
    new_correlation_id();

    debug!("Validating configuration");
    if config_len > MAX_CONFIG_SIZE {
        warn!(
            config_size = config_len,
            max_size = MAX_CONFIG_SIZE,
            "Configuration too large"
        );
        return -1;
    }

    let config = unsafe {
        std::slice::from_raw_parts(config_ptr, config_len)
    };

    info!("Parsing configuration");
    let parsed_config = match parse_config(config) {
        Ok(cfg) => {
            debug!(config_version = ?cfg.version, "Configuration parsed");
            cfg
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse configuration");
            return -2;
        }
    };

    info!("Applying configuration");
    match apply_config(parsed_config) {
        Ok(_) => {
            info!("Configuration applied successfully");
            0
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to apply configuration");
            -3
        }
    }
}
```

### Example 3: Propagating Correlation IDs

```rust
use keyrx_core::ffi::tracing::{new_correlation_id, correlation_id, set_correlation_id};

// Entry point - create new correlation ID
#[instrument(fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "session", version = "1.0")]
pub extern "C" fn keyrx_session_start() -> i32 {
    let id = new_correlation_id();
    tracing::info!(correlation_id = id, "Session started");

    // Call internal functions - correlation ID propagates automatically
    load_profile();
    initialize_engine();

    0
}

// Internal function - uses existing correlation ID
#[instrument(fields(correlation_id = %correlation_id()))]
fn load_profile() {
    tracing::debug!("Loading profile");
    // ... implementation
}

// Another internal function - same correlation ID
#[instrument(fields(correlation_id = %correlation_id()))]
fn initialize_engine() {
    tracing::debug!("Initializing engine");
    // ... implementation
}
```

## Testing

Run the tracing module tests:

```bash
cargo test --package keyrx_core --lib ffi::tracing
```

Example test:

```rust
#[test]
fn test_correlation_id_generation() {
    let id1 = new_correlation_id();
    let id2 = new_correlation_id();
    assert_ne!(id1, id2);
    assert!(id2 > id1);
}
```

### Example 4: Using Trace Export in FFI Functions

```rust
use keyrx_core::ffi::tracing_export::{
    emit_trace_event_start, emit_trace_event_end, ResultStatus
};

#[ffi_export(domain = "engine", version = "1.0")]
pub extern "C" fn keyrx_engine_process_input(input: u32) -> i32 {
    // Emit trace event start
    let event_id = emit_trace_event_start(
        "keyrx_engine_process_input",
        "engine",
        &[("input", &input.to_string())],
    );

    // Do work
    let result = match process_input_internal(input) {
        Ok(_) => {
            emit_trace_event_end(event_id, ResultStatus::Success);
            0
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to process input");
            emit_trace_event_end(event_id, ResultStatus::Error);
            -1
        }
    };

    result
}
```

## Testing

Run the tracing module tests:

```bash
# Test tracing infrastructure
cargo test --package keyrx_core --lib ffi::tracing

# Test tracing export
cargo test --package keyrx_core --lib ffi::tracing_export
```

Example tests:

```rust
#[test]
fn test_correlation_id_generation() {
    let id1 = new_correlation_id();
    let id2 = new_correlation_id();
    assert_ne!(id1, id2);
    assert!(id2 > id1);
}

#[test]
fn test_trace_event_lifecycle() {
    keyrx_trace_enable();
    keyrx_trace_clear();

    let event_id = emit_trace_event_start("test_fn", "test", &[]);
    assert!(event_id > 0);

    emit_trace_event_end(event_id, ResultStatus::Success);

    let count = keyrx_trace_get_count();
    assert_eq!(count, 1);
}
```

## See Also

- [FFI Architecture](ffi-architecture.md)
- [FFI Development Guide](FFI_DEVELOPMENT.md)
- Task 17: FFI Tracing Infrastructure (completed)
- Task 18: Dart DevTools Timeline Integration (completed)
- Task 19: FFI Call Instrumentation (next)
