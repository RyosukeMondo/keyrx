# FFI Instrumentation Guide

This guide explains how to use FFI instrumentation in KeyRx for observability, debugging, and performance analysis.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Using Instrumentation](#using-instrumentation)
- [Viewing Metrics](#viewing-metrics)
- [Debugging with Instrumentation](#debugging-with-instrumentation)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Overview

KeyRx's FFI layer is fully instrumented with distributed tracing and performance metrics. Every FFI function call is tracked with:

- **Correlation IDs** - Unique identifiers for request flows
- **Structured Spans** - Hierarchical tracing for nested calls
- **Latency Measurement** - Microsecond-precision timing
- **Error Tracking** - Automatic error detection and logging
- **Metrics Aggregation** - Call counts, success/error rates, average latency

### Benefits

- **Observability**: See what's happening across the FFI boundary
- **Performance Analysis**: Identify slow FFI calls
- **Debugging**: Trace request flows from Dart to Rust
- **Production Monitoring**: Track error rates and latency in production
- **DevTools Integration**: Visualize FFI calls in Flutter DevTools timeline

## Architecture

### Components

1. **Tracing Infrastructure** (`core/src/ffi/tracing.rs`)
   - Correlation ID generation and management
   - Tracing subscriber initialization
   - Span creation helpers

2. **Metrics Aggregation** (`core/src/ffi/metrics.rs`)
   - Per-function call counting
   - Latency tracking (min/max/avg)
   - Success/error rate calculation
   - Thread-safe metrics storage

3. **Instrumentation Macros**
   - `#[instrument]` - Tracing attribute from `tracing` crate
   - `ffi_record_metrics!` - Custom macro for metrics recording

### Data Flow

```
Dart → FFI Call → [#[instrument] attribute]
                       ↓
                   Generate correlation ID
                       ↓
                   Start timing
                       ↓
                   Execute function
                       ↓
                   Record metrics
                       ↓
                   Return to Dart
```

## Using Instrumentation

### Instrumenting a New FFI Function

When adding a new FFI export function, follow this pattern:

```rust
use std::time::Instant;
use tracing::instrument;
use ffi_export::ffi_export;

#[instrument(skip_all, fields(
    correlation_id = %crate::ffi::tracing::correlation_id(),
    vendor_id,      // Include relevant params
    product_id
))]
#[ffi_export(domain = "device", version = "1.0.0")]
#[no_mangle]
pub extern "C" fn keyrx_get_device_info(vendor_id: u16, product_id: u16) -> *mut c_char {
    // 1. Start timing and generate correlation ID
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    // 2. Execute function logic
    let result = device::get_device_info(vendor_id, product_id);

    // 3. Record metrics
    crate::ffi_record_metrics!("keyrx_get_device_info", "device", start, &result);

    // 4. Return result
    ffi_json(result)
}
```

### Instrumentation Attributes

#### `#[instrument]` Configuration

```rust
#[instrument(
    skip_all,  // Skip auto-logging all parameters (we control what to log)
    fields(
        correlation_id = %crate::ffi::tracing::correlation_id(),  // Always include
        param1,        // Include non-sensitive params
        param2,
        // Skip sensitive params like passwords, tokens
    )
)]
```

**Field Types:**
- `%expr` - Display formatting (for correlation_id)
- `field` - Debug formatting (for regular params)
- `?field` - Debug formatting (explicit)
- `field = value` - Custom field with value

### Metrics Recording

The `ffi_record_metrics!` macro records:

```rust
crate::ffi_record_metrics!(
    "function_name",  // FFI function name
    "domain",         // Domain (engine, device, compat, etc.)
    start,            // Instant::now() from function start
    &result           // Result<T, E> reference
);
```

This automatically tracks:
- Call count
- Success count
- Error count
- Latency (min, max, avg)

### Handling Errors

Metrics should be recorded even in error paths:

```rust
let result = match some_operation() {
    Ok(value) => {
        crate::ffi_record_metrics!("my_function", "my_domain", start, &Ok::<_, ()>(value));
        value
    }
    Err(e) => {
        crate::ffi_record_metrics!("my_function", "my_domain", start, &Err::<(), _>(&e));
        return ffi_error(e);
    }
};
```

## Viewing Metrics

### Command Line

Display aggregated metrics:

```bash
# Text format (human-readable)
just ffi-metrics

# JSON format (for scripts/tools)
just ffi-metrics json
```

**Example Output:**

```
FFI Metrics Summary
===================

Uptime: 125.3s
Total functions tracked: 15

Domain: engine
------------------------------------------------------------
  keyrx_on_state
    Calls: 523, Success: 523, Errors: 0 (0.0%)
    Latency: avg=0.15ms, min=0.08ms, max=2.34ms

Domain: compat
------------------------------------------------------------
  keyrx_list_keys
    Calls: 12, Success: 12, Errors: 0 (0.0%)
    Latency: avg=1.23ms, min=0.95ms, max=1.87ms

  keyrx_load_script
    Calls: 8, Success: 7, Errors: 1 (12.5%)
    Latency: avg=45.2ms, min=32.1ms, max=78.9ms
```

### Flutter DevTools Timeline

FFI calls appear in the DevTools timeline when tracing is enabled:

```bash
# Run with FFI tracing enabled
just dev-trace
```

Then open Flutter DevTools and navigate to the Timeline tab. FFI calls will appear with:
- Function name
- Duration
- Arguments (sanitized)
- Success/error status

### Programmatic Access

```rust
use keyrx_core::ffi::metrics;

// Get metrics for a specific function
if let Some(metrics) = metrics::get_function_metrics("keyrx_on_state") {
    println!("Call count: {}", metrics.call_count);
    println!("Avg latency: {}μs", metrics.avg_latency_us());
    println!("Error rate: {:.1}%", metrics.error_rate());
}

// Get all metrics
let all = metrics::get_all_metrics();
for metric in all {
    println!("{}: {} calls", metric.name, metric.call_count);
}

// Get formatted summary
println!("{}", metrics::get_summary());

// Get JSON summary
println!("{}", metrics::get_summary_json());

// Reset metrics
metrics::reset();
```

## Debugging with Instrumentation

### Enable Detailed Logging

Set the `RUST_LOG` environment variable:

```bash
# Debug level for FFI
RUST_LOG=keyrx_core::ffi=debug just dev

# Trace level (very verbose)
RUST_LOG=keyrx_core::ffi=trace just dev

# Or use the convenience command
just dev-trace
```

### Correlation IDs

Every FFI call gets a unique correlation ID. Use this to trace a request through the system:

```
[2024-12-15T10:30:45.123Z DEBUG keyrx_core::ffi::exports_compat] correlation_id=42 FFI call: keyrx_load_script
[2024-12-15T10:30:45.145Z DEBUG keyrx_core::ffi::domains::script] correlation_id=42 Loading script: /path/to/script.rhai
[2024-12-15T10:30:45.167Z DEBUG keyrx_core::ffi::exports_compat] correlation_id=42 FFI exit: keyrx_load_script
```

### Structured Logging

Use structured logging in your FFI functions:

```rust
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
pub extern "C" fn my_ffi_function() -> i32 {
    tracing::info!("Starting FFI operation");

    if let Err(e) = some_operation() {
        tracing::error!(error = %e, "Operation failed");
        return -1;
    }

    tracing::debug!(result = 42, "Operation succeeded");
    0
}
```

### Finding Slow FFI Calls

Use metrics to identify performance bottlenecks:

```bash
# View metrics sorted by average latency
just ffi-metrics | grep "avg="
```

Or programmatically:

```rust
let mut metrics = keyrx_core::ffi::metrics::get_all_metrics();
metrics.sort_by_key(|m| std::cmp::Reverse(m.avg_latency_us()));

for metric in metrics.iter().take(10) {
    println!("{}: {:.2}ms avg", metric.name, metric.avg_latency_us() as f64 / 1000.0);
}
```

### Identifying Error-Prone Functions

```bash
# View metrics sorted by error rate
just ffi-metrics | grep "Errors"
```

## Best Practices

### DO

✅ **Always instrument new FFI functions**
```rust
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
```

✅ **Record metrics before returning**
```rust
crate::ffi_record_metrics!("function_name", "domain", start, &result);
```

✅ **Include relevant parameters in spans**
```rust
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id(), path, device_id))]
```

✅ **Use structured logging for important events**
```rust
tracing::info!(device_id = %id, "Device connected");
```

✅ **Log errors with context**
```rust
tracing::error!(error = %e, path = %path, "Failed to load script");
```

✅ **Use appropriate log levels**
- `ERROR` - Actual errors that need attention
- `WARN` - Unexpected but handled situations
- `INFO` - Important business events
- `DEBUG` - Detailed debugging information
- `TRACE` - Very verbose tracing

### DON'T

❌ **Don't log sensitive data**
```rust
// BAD - exposes password
#[instrument(skip_all, fields(username, password))]

// GOOD - skip sensitive params
#[instrument(skip_all, fields(username))]
```

❌ **Don't forget to record metrics in error paths**
```rust
// BAD - metrics not recorded on error
if let Err(e) = operation() {
    return ffi_error(e);
}
crate::ffi_record_metrics!(...);

// GOOD - metrics recorded for all paths
let result = operation();
crate::ffi_record_metrics!("fn_name", "domain", start, &result);
if let Err(e) = result {
    return ffi_error(e);
}
```

❌ **Don't use println! or eprintln! in FFI code**
```rust
// BAD - not structured, no context
println!("Error: {}", e);

// GOOD - structured logging
tracing::error!(error = %e, "Operation failed");
```

❌ **Don't block on async operations in FFI**
FFI functions are synchronous. Use proper async bridges if needed.

❌ **Don't panic in instrumented functions**
Panics bypass metrics recording. Use `Result` and handle errors properly.

## Examples

### Example 1: Simple FFI Function

```rust
use std::time::Instant;
use tracing::instrument;

#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[ffi_export(domain = "device", version = "1.0.0")]
#[no_mangle]
pub extern "C" fn keyrx_list_devices() -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    tracing::debug!("Listing all devices");

    let result = device::list_devices();

    crate::ffi_record_metrics!("keyrx_list_devices", "device", start, &result);

    ffi_json(result)
}
```

### Example 2: FFI Function with Parameters

```rust
#[instrument(skip_all, fields(
    correlation_id = %crate::ffi::tracing::correlation_id(),
    vendor_id,
    product_id
))]
#[ffi_export(domain = "device", version = "1.0.0")]
#[no_mangle]
pub extern "C" fn keyrx_get_device_profile(vendor_id: u16, product_id: u16) -> *mut c_char {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    tracing::info!(
        vendor_id = vendor_id,
        product_id = product_id,
        "Fetching device profile"
    );

    let result = device::get_device_profile(vendor_id, product_id);

    crate::ffi_record_metrics!("keyrx_get_device_profile", "device", start, &result);

    ffi_json(result)
}
```

### Example 3: FFI Function with Error Handling

```rust
#[instrument(skip_all, fields(correlation_id = %crate::ffi::tracing::correlation_id()))]
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32 {
    let start = Instant::now();
    crate::ffi::tracing::new_correlation_id();

    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => {
            tracing::error!(error_code = code, "Invalid path string");
            crate::ffi_record_metrics!("keyrx_load_script", "compat", start, &Err::<(), _>(code));
            return code;
        }
    };

    tracing::info!(path = %path, "Loading script");

    let result = match script::load_script(path) {
        Ok(_) => {
            tracing::debug!("Script loaded successfully");
            0
        }
        Err(e) => {
            tracing::error!(error = %e, path = %path, "Failed to load script");
            -1
        }
    };

    crate::ffi_record_metrics!("keyrx_load_script", "compat", start, &Ok::<_, ()>(result));
    result
}
```

### Example 4: Viewing Metrics Programmatically

```rust
use keyrx_core::ffi::metrics;

fn analyze_ffi_performance() {
    let all_metrics = metrics::get_all_metrics();

    // Find slowest functions
    let mut sorted = all_metrics.clone();
    sorted.sort_by_key(|m| std::cmp::Reverse(m.avg_latency_us()));

    println!("Slowest FFI functions:");
    for metric in sorted.iter().take(5) {
        println!(
            "  {} ({}) - avg: {:.2}ms, calls: {}",
            metric.name,
            metric.domain,
            metric.avg_latency_us() as f64 / 1000.0,
            metric.call_count
        );
    }

    // Find functions with highest error rates
    let mut by_errors = all_metrics.clone();
    by_errors.sort_by(|a, b| b.error_rate().partial_cmp(&a.error_rate()).unwrap());

    println!("\nFunctions with highest error rates:");
    for metric in by_errors.iter().take(5) {
        if metric.error_count > 0 {
            println!(
                "  {} ({}) - error rate: {:.1}%, errors: {}/{}",
                metric.name,
                metric.domain,
                metric.error_rate(),
                metric.error_count,
                metric.call_count
            );
        }
    }

    // Overall statistics
    let total_calls: u64 = all_metrics.iter().map(|m| m.call_count).sum();
    let total_errors: u64 = all_metrics.iter().map(|m| m.error_count).sum();

    println!("\nOverall FFI Statistics:");
    println!("  Total functions: {}", all_metrics.len());
    println!("  Total calls: {}", total_calls);
    println!("  Total errors: {}", total_errors);
    println!("  Overall error rate: {:.2}%", (total_errors as f64 / total_calls as f64) * 100.0);
}
```

## Performance Impact

The instrumentation infrastructure is designed for minimal overhead:

- **Tracing overhead**: < 1% in production
- **Metrics recording**: < 5% latency increase
- **Memory usage**: ~1KB per tracked function
- **Thread safety**: Lock-free atomic operations where possible

### Disabling in Production

If you need to disable instrumentation in production:

```rust
// In core/src/ffi/tracing.rs, set default config:
impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enable_file_output: false,
            enable_chrome_trace: false,
            min_level: Level::WARN,  // Only log warnings and errors
            enable_console: false,    // Disable console output
        }
    }
}
```

## Troubleshooting

### "Metrics not showing up"

- Ensure you're calling functions that are instrumented
- Check that tracing is initialized: `ffi::tracing::init(...)`
- Verify metrics are recorded: look for `ffi_record_metrics!` calls

### "Correlation IDs are all 0"

- Make sure `new_correlation_id()` is called at function start
- Check for thread-local storage issues in FFI context

### "High latency reported"

- Verify timing is measured correctly (start at function entry)
- Check if blocking I/O is happening in FFI functions
- Use `just dev-trace` and DevTools to see detailed timing

### "Memory leak from metrics"

- Metrics are stored in-memory and grow over time
- Call `metrics::reset()` periodically if needed
- Monitor memory usage with production profiling

## See Also

- [FFI Development Guide](./FFI_DEVELOPMENT.md)
- [FFI Tracing Documentation](./FFI_TRACING.md)
- [FFI Architecture](./ffi-architecture.md)
- [DevTools Timeline Integration](./FFI_TRACING.md#devtools-integration)
