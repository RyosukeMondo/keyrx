#!/usr/bin/env bash
# Display FFI metrics summary

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if a Rust binary is running that exposes metrics
# For now, we'll create a simple tool that reads metrics from the FFI layer

echo "FFI Metrics Tool"
echo "================"
echo ""

# Build a simple Rust tool to query metrics
cat > /tmp/ffi_metrics_query.rs <<'EOF'
fn main() {
    // Initialize the FFI tracing system
    keyrx_core::ffi::tracing::init(keyrx_core::ffi::tracing::TracingConfig::default());

    // Get and display metrics
    let summary = keyrx_core::ffi::metrics::get_summary();
    println!("{}", summary);

    // Also output JSON for programmatic access
    if std::env::args().any(|arg| arg == "--json") {
        println!("\n--- JSON Output ---");
        println!("{}", keyrx_core::ffi::metrics::get_summary_json());
    }
}
EOF

# For now, provide a placeholder message
cat <<EOF
This command will display FFI call metrics including:
- Call count per function
- Average latency
- Error rates
- Success rates

To use this feature:
1. Start your KeyRx application with FFI tracing enabled
2. The metrics will be collected automatically as FFI functions are called
3. Run 'just ffi-metrics' to view the aggregated data

Note: Metrics are collected in-memory and reset on application restart.

For real-time metrics, consider:
- Enabling FFI tracing with: RUST_LOG=keyrx_core::ffi=debug
- Using the DevTools timeline integration (just dev-trace)

Example output when metrics are available:
-------------------------------------------
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

EOF
