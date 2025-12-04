# KeyRx Performance Metrics System

## Overview

The KeyRx metrics system provides comprehensive performance monitoring with minimal overhead (< 1 microsecond per recording). It tracks latency percentiles, memory usage, and hot path profiling to ensure the keyboard remapping engine meets its sub-millisecond latency targets.

## Architecture

The metrics system is built around a pluggable trait-based architecture:

- **MetricsCollector Trait**: Abstract interface for metrics collection
- **NoOpCollector**: Zero-overhead implementation for release builds (inlined to nothing)
- **FullMetricsCollector**: Complete metrics collection with histograms and profiling

All metrics use:
- **Zero Allocation**: Hot paths use pre-allocated storage
- **Thread Safety**: All collectors are `Send + Sync`
- **Bounded Memory**: Fixed-size buffers and histograms prevent unbounded growth

## Metric Types

### 1. Latency Tracking

Latency metrics track the time taken for key operations using HDR histograms for accurate percentile calculation.

#### Tracked Operations

| Operation | Description | Location |
|-----------|-------------|----------|
| `EventProcess` | Time from receiving input to starting processing | `core/src/engine/mod.rs` |
| `RuleMatch` | Time to find matching rules | `core/src/engine/mod.rs` |
| `ActionExecute` | Time to execute matched actions | `core/src/engine/mod.rs` |
| `DriverRead` | Time to read input from OS driver | `core/src/drivers/*/mod.rs` |
| `DriverWrite` | Time to write output to OS driver | `core/src/drivers/*/mod.rs` |

#### Statistics Available

- **p50** (median): 50th percentile latency
- **p95**: 95th percentile latency
- **p99**: 99th percentile latency
- **mean**: Average latency
- **min**: Minimum recorded latency
- **max**: Maximum recorded latency
- **count**: Total number of samples

All latency values are in **microseconds** (µs).

#### Example Usage

```rust
use keyrx_core::metrics::{MetricsCollector, Operation};

// Record latency manually
collector.record_latency(Operation::EventProcess, 150);

// Or use RAII guards for automatic timing
{
    let _guard = collector.start_profile("expensive_function");
    // ... code to profile ...
} // Automatically records elapsed time on drop
```

### 2. Memory Monitoring

Memory metrics track current, peak, and baseline memory usage with leak detection heuristics.

#### Statistics Available

- **current**: Current memory usage in bytes
- **peak**: Peak memory usage since start in bytes
- **baseline**: Initial memory usage in bytes
- **growth**: Growth from baseline (current - baseline) in bytes
- **has_potential_leak**: Boolean indicating potential memory leak

#### Leak Detection

Memory is sampled periodically (default: every 1 second). A potential leak is detected when:
1. Memory growth exceeds a threshold
2. Growth trend is consistently upward
3. No corresponding increase in workload

#### Example Usage

```rust
// Memory is tracked automatically by the collector
// Access via snapshot
let snapshot = collector.snapshot();
println!("Current memory: {} bytes", snapshot.memory.current);
println!("Peak memory: {} bytes", snapshot.memory.peak);
if snapshot.memory.has_potential_leak {
    println!("Warning: Potential memory leak detected!");
}
```

### 3. Hot Path Profiling

Profile points enable function-level timing with RAII guards for automatic measurement.

#### Statistics Available

- **count**: Number of times the profile point was recorded
- **total_micros**: Total accumulated time across all calls
- **avg_micros**: Average time per call
- **min_micros**: Minimum recorded time
- **max_micros**: Maximum recorded time

All timing values are in **microseconds** (µs).

#### Example Usage

```rust
use keyrx_core::metrics::MetricsCollector;

fn expensive_operation(collector: &dyn MetricsCollector) {
    let _guard = collector.start_profile("expensive_operation");

    // Your code here
    // Guard automatically records time on drop
}

// Get statistics later
let snapshot = collector.snapshot();
if let Some(stats) = snapshot.profiles.get("expensive_operation") {
    println!("Called {} times, avg {} µs", stats.count, stats.avg_micros);
}
```

## Thresholds and Alerts

The metrics system supports configurable thresholds for automated alerts.

### Threshold Types

#### Latency Thresholds

- **Warning**: Latency exceeds warning threshold (e.g., 50 µs)
- **Error**: Latency exceeds error threshold (e.g., 100 µs)

#### Memory Thresholds

- **Warning**: Memory usage exceeds warning threshold (e.g., 100 MB)
- **Error**: Memory usage exceeds error threshold (e.g., 500 MB)

### Configuring Thresholds

#### From Rust

```rust
use keyrx_core::observability::metrics_bridge::set_thresholds;

// Set thresholds: 50µs warn, 100µs error, 100MB warn, 500MB error
set_thresholds(50, 100, 100 * 1024 * 1024, 500 * 1024 * 1024);
```

#### From FFI (C/Flutter)

```c
// Set thresholds
keyrx_metrics_set_thresholds(50, 100, 104857600, 524288000);

// Get current thresholds
uint64_t latency_warn, latency_error, memory_warn, memory_error;
keyrx_metrics_get_thresholds(&latency_warn, &latency_error,
                              &memory_warn, &memory_error);
```

### Alert Callbacks

Register a callback to receive threshold violation notifications:

```c
void on_threshold_violation(const ThresholdViolation* violation) {
    switch (violation->violation_type) {
        case 0: // LatencyWarning
            printf("Warning: Latency %llu µs exceeds %llu µs\n",
                   violation->actual_value, violation->threshold_value);
            break;
        case 1: // LatencyError
            printf("Error: Latency %llu µs exceeds %llu µs\n",
                   violation->actual_value, violation->threshold_value);
            break;
        case 2: // MemoryWarning
            printf("Warning: Memory %llu bytes exceeds %llu bytes\n",
                   violation->actual_value, violation->threshold_value);
            break;
        case 3: // MemoryError
            printf("Error: Memory %llu bytes exceeds %llu bytes\n",
                   violation->actual_value, violation->threshold_value);
            break;
    }
}

keyrx_metrics_set_threshold_callback(on_threshold_violation);
```

## Accessing Metrics

### From Rust

```rust
use keyrx_core::metrics::MetricsCollector;

// Get a snapshot
let snapshot = collector.snapshot();

// Access latency stats
if let Some(stats) = snapshot.latencies.get("event_process") {
    println!("Event processing p95: {} µs", stats.p95);
    println!("Event processing p99: {} µs", stats.p99);
}

// Access memory stats
println!("Current memory: {} bytes", snapshot.memory.current);
println!("Peak memory: {} bytes", snapshot.memory.peak);

// Access profile stats
if let Some(stats) = snapshot.profiles.get("my_function") {
    println!("Function called {} times", stats.count);
    println!("Average time: {} µs", stats.avg_micros);
}

// Export to JSON
let json = snapshot.to_json().unwrap();
println!("{}", json);
```

### From FFI (C/Flutter)

#### Polling Approach (Recommended)

```c
// Get metrics as JSON
char* json = keyrx_metrics_snapshot_json();
if (json) {
    process_metrics(json);
    keyrx_free_string(json);
}
```

#### Struct Approach

```c
// Get metrics as struct
MetricsSnapshotFfi* snapshot = keyrx_metrics_snapshot();
if (snapshot) {
    printf("Events processed: %llu\n", snapshot->events_processed);
    printf("P95 latency: %llu µs\n", snapshot->event_latency_p95);
    printf("P99 latency: %llu µs\n", snapshot->event_latency_p99);
    printf("Memory used: %llu bytes\n", snapshot->memory_used);

    keyrx_metrics_free_snapshot(snapshot);
}
```

#### Callback Approach (Real-time Updates)

```c
void on_metrics_update(const MetricsSnapshotFfi* snapshot) {
    printf("P95: %llu µs, Memory: %llu bytes\n",
           snapshot->event_latency_p95, snapshot->memory_used);
}

// Register callback
keyrx_metrics_set_callback(on_metrics_update);

// Start periodic updates
keyrx_metrics_start_updates();

// Trigger immediate update
keyrx_metrics_trigger_callback();

// Stop updates when done
keyrx_metrics_stop_updates();

// Unregister callback
keyrx_metrics_set_callback(NULL);
```

### From Flutter

```dart
import 'package:keyrx/services/metrics_service.dart';

class MyWidget extends StatefulWidget {
  @override
  _MyWidgetState createState() => _MyWidgetState();
}

class _MyWidgetState extends State<MyWidget> {
  final MetricsService _metricsService = MetricsService();

  @override
  void initState() {
    super.initState();
    _metricsService.startPeriodicUpdates();
  }

  @override
  void dispose() {
    _metricsService.stopPeriodicUpdates();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<MetricsSnapshot>(
      stream: _metricsService.metricsStream,
      builder: (context, snapshot) {
        if (!snapshot.hasData) return CircularProgressIndicator();

        final metrics = snapshot.data!;
        return Column(
          children: [
            Text('P95 Latency: ${metrics.eventLatencyP95} µs'),
            Text('P99 Latency: ${metrics.eventLatencyP99} µs'),
            Text('Memory: ${metrics.memoryUsed} bytes'),
          ],
        );
      },
    );
  }
}
```

## Metrics Dashboard

KeyRx includes a built-in metrics dashboard in the Flutter UI for real-time visualization.

### Accessing the Dashboard

1. Open the KeyRx application
2. Navigate to the Debug page
3. View the Metrics Dashboard tab

### Dashboard Features

- **Latency Charts**: Real-time line charts showing p50, p95, p99 latencies over time
- **Memory Graph**: Visual representation of current and peak memory usage
- **Profile Stats**: Table of hot path profile points with timing statistics
- **Export**: Button to export current metrics to JSON file

### Dashboard Implementation

Location: `ui/lib/widgets/metrics/metrics_dashboard.dart`

The dashboard uses:
- `fl_chart` for chart visualization
- `MetricsService` for data fetching
- Real-time updates via periodic polling

## JSON Export Format

Metrics can be exported to JSON for integration with external monitoring tools.

### Example JSON Output

```json
{
  "timestamp": 1701234567890,
  "latencies": {
    "event_process": {
      "count": 1000,
      "p50": 100,
      "p95": 250,
      "p99": 500,
      "mean": 120.5,
      "min": 50,
      "max": 1000
    },
    "driver_read": {
      "count": 1000,
      "p50": 80,
      "p95": 200,
      "p99": 400,
      "mean": 95.2,
      "min": 40,
      "max": 800
    }
  },
  "memory": {
    "current": 10485760,
    "peak": 15728640,
    "baseline": 8388608,
    "growth": 2097152,
    "has_potential_leak": false
  },
  "profiles": {
    "process_event": {
      "count": 1000,
      "total_micros": 120000,
      "avg_micros": 120,
      "min_micros": 50,
      "max_micros": 500
    }
  }
}
```

### Field Descriptions

#### Root Fields

- `timestamp`: Unix timestamp in milliseconds when snapshot was taken

#### Latencies Object

Maps operation name (string) to latency statistics:
- `count`: Total number of samples
- `p50`, `p95`, `p99`: Percentile values in microseconds
- `mean`: Average latency in microseconds
- `min`, `max`: Minimum and maximum values in microseconds

#### Memory Object

- `current`: Current memory usage in bytes
- `peak`: Peak memory usage since start in bytes
- `baseline`: Initial memory usage in bytes
- `growth`: Growth from baseline in bytes
- `has_potential_leak`: Boolean indicating potential leak

#### Profiles Object

Maps profile point name (string) to timing statistics:
- `count`: Number of times recorded
- `total_micros`: Total accumulated time in microseconds
- `avg_micros`: Average time per call in microseconds
- `min_micros`, `max_micros`: Minimum and maximum values in microseconds

## Performance Impact

### Recording Overhead

The metrics system is designed for minimal overhead:

- **Target**: < 1 microsecond per recording
- **Verified**: See `core/benches/metrics_bench.rs` for benchmark results
- **NoOpCollector**: Zero overhead in release builds (fully inlined)

### Benchmark Results

Run benchmarks with:

```bash
cargo bench --bench metrics_bench
```

Expected results:
- Latency recording: ~500-800 nanoseconds
- Memory recording: ~200-400 nanoseconds
- Profile guard creation: ~300-500 nanoseconds
- Snapshot generation: ~1-5 microseconds

### Memory Usage

- **Histogram storage**: ~32 KB per operation (fixed)
- **Memory samples**: ~4 KB (ring buffer)
- **Profile points**: ~8 bytes per unique profile point name
- **Total**: ~200 KB for full metrics collection

### Disabling Metrics

For production builds where metrics are not needed:

```rust
use keyrx_core::metrics::default_noop_collector;

// Use no-op collector (zero overhead)
let collector = default_noop_collector();
```

## Best Practices

### 1. Use RAII Guards

Prefer RAII guards over manual timing:

```rust
// Good - automatic, exception-safe
{
    let _guard = collector.start_profile("function");
    do_work();
} // Automatically recorded

// Avoid - manual, error-prone
let start = Instant::now();
do_work();
collector.record_profile("function", start.elapsed().as_micros() as u64);
```

### 2. Static Profile Names

Use `&'static str` for profile point names to avoid allocations:

```rust
// Good - static string
const PROFILE_NAME: &str = "my_function";
let _guard = collector.start_profile(PROFILE_NAME);

// Avoid - dynamic string (allocates)
let name = format!("function_{}", id);
let _guard = collector.start_profile(&name); // Won't compile
```

### 3. Choose Appropriate Operations

Use the correct operation type for accurate categorization:

```rust
// Event processing
collector.record_latency(Operation::EventProcess, latency);

// Rule matching
collector.record_latency(Operation::RuleMatch, latency);

// Action execution
collector.record_latency(Operation::ActionExecute, latency);

// Driver operations
collector.record_latency(Operation::DriverRead, latency);
collector.record_latency(Operation::DriverWrite, latency);
```

### 4. Export Regularly

Export metrics periodically to avoid data loss on crashes:

```rust
use std::fs;

// Export every 60 seconds
let snapshot = collector.snapshot();
let json = snapshot.to_json().unwrap();
fs::write("/var/log/keyrx/metrics.json", json).ok();
```

### 5. Set Appropriate Thresholds

Configure thresholds based on your performance requirements:

```rust
// Conservative thresholds for production
set_thresholds(
    100,              // Latency warning: 100 µs
    500,              // Latency error: 500 µs
    100 * 1024 * 1024,  // Memory warning: 100 MB
    500 * 1024 * 1024,  // Memory error: 500 MB
);

// Strict thresholds for debugging
set_thresholds(
    50,               // Latency warning: 50 µs
    100,              // Latency error: 100 µs
    50 * 1024 * 1024,   // Memory warning: 50 MB
    200 * 1024 * 1024,  // Memory error: 200 MB
);
```

## Troubleshooting

### High Latency Warnings

If you see frequent latency threshold violations:

1. **Check the metrics dashboard** to identify which operations are slow
2. **Use profile points** to identify hot spots within slow operations
3. **Review recent code changes** that may have introduced inefficiencies
4. **Consider system load** - high CPU/memory usage can affect latency

### Memory Leak Detected

If `has_potential_leak` is true:

1. **Check memory growth rate** in the dashboard over time
2. **Review recent allocations** in the code
3. **Use profiling tools** like `valgrind` or `heaptrack` for detailed analysis
4. **Check for unclosed resources** (file handles, sockets, etc.)

### Missing Metrics

If metrics are not appearing:

1. **Verify collector is not NoOp** - check that `FullMetricsCollector` is being used
2. **Check callback registration** - ensure FFI callbacks are properly registered
3. **Verify periodic updates** - ensure `keyrx_metrics_start_updates()` was called
4. **Check for errors** - review logs for FFI or serialization errors

### Inaccurate Percentiles

If latency percentiles seem wrong:

1. **Verify sufficient samples** - percentiles are only accurate with many samples
2. **Check histogram bounds** - values outside histogram range are clamped
3. **Review measurement points** - ensure timing is measured correctly
4. **Consider outliers** - occasional spikes can skew statistics

## Related Documentation

- [Architecture](ARCHITECTURE.md) - Overall system architecture
- [FFI Architecture](ffi-architecture.md) - FFI layer design
- [Logging Standards](logging-standards.md) - Logging conventions

## Source Code Locations

### Core Metrics Implementation

- `core/src/metrics/mod.rs` - Module root and re-exports
- `core/src/metrics/collector.rs` - MetricsCollector trait
- `core/src/metrics/operation.rs` - Operation enum
- `core/src/metrics/latency.rs` - Latency histogram implementation
- `core/src/metrics/memory.rs` - Memory monitoring
- `core/src/metrics/profile.rs` - Profile points with RAII guards
- `core/src/metrics/snapshot.rs` - Snapshot types for export
- `core/src/metrics/full_collector.rs` - Full metrics collector
- `core/src/metrics/noop_collector.rs` - No-op collector
- `core/src/metrics/sampler.rs` - Periodic memory sampling

### FFI Integration

- `core/src/ffi/exports_metrics.rs` - FFI exports documentation
- `core/src/ffi/domains/observability/mod.rs` - Observability domain implementation
- `core/src/observability/metrics_bridge.rs` - Metrics bridge for FFI

### Flutter Integration

- `ui/lib/services/metrics_service.dart` - Metrics service for Flutter
- `ui/lib/widgets/metrics/metrics_dashboard.dart` - Metrics dashboard widget
- `ui/lib/pages/debug_page.dart` - Debug page with metrics integration

### Tests and Benchmarks

- `core/tests/unit/metrics/` - Unit tests
- `core/benches/metrics_bench.rs` - Performance benchmarks
