//! FFI call metrics aggregation and tracking.
//!
//! This module provides:
//! - Call count tracking for each FFI function
//! - Average latency measurement
//! - Error rate tracking
//! - Thread-safe metrics aggregation
//!
//! # Usage
//!
//! The metrics are automatically collected when using the `#[instrument]` attribute
//! with our FFI tracing infrastructure. To view aggregated metrics:
//!
//! ```ignore
//! use keyrx_core::ffi::metrics;
//!
//! let summary = metrics::get_summary();
//! println!("{}", summary);
//! ```

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Global metrics registry
static METRICS_REGISTRY: OnceLock<Mutex<MetricsRegistry>> = OnceLock::new();

fn get_registry() -> &'static Mutex<MetricsRegistry> {
    METRICS_REGISTRY.get_or_init(|| Mutex::new(MetricsRegistry::new()))
}

/// Metrics for a single FFI function
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    /// Function name
    pub name: String,

    /// Domain (e.g., "engine", "device", "compat")
    pub domain: String,

    /// Total number of calls
    pub call_count: u64,

    /// Number of successful calls
    pub success_count: u64,

    /// Number of failed calls
    pub error_count: u64,

    /// Sum of all latencies in microseconds
    pub total_latency_us: u64,

    /// Minimum latency observed (microseconds)
    pub min_latency_us: u64,

    /// Maximum latency observed (microseconds)
    pub max_latency_us: u64,
}

impl FunctionMetrics {
    fn new(name: String, domain: String) -> Self {
        Self {
            name,
            domain,
            call_count: 0,
            success_count: 0,
            error_count: 0,
            total_latency_us: 0,
            min_latency_us: u64::MAX,
            max_latency_us: 0,
        }
    }

    /// Get average latency in microseconds
    pub fn avg_latency_us(&self) -> u64 {
        if self.call_count == 0 {
            0
        } else {
            self.total_latency_us / self.call_count
        }
    }

    /// Get error rate as a percentage (0.0 to 100.0)
    pub fn error_rate(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.call_count as f64) * 100.0
        }
    }

    /// Get success rate as a percentage (0.0 to 100.0)
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.call_count as f64) * 100.0
        }
    }
}

/// Central metrics registry
#[derive(Debug)]
struct MetricsRegistry {
    /// Metrics by function name
    metrics: HashMap<String, FunctionMetrics>,

    /// Registry creation time
    start_time: Instant,
}

impl MetricsRegistry {
    fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn record_call(&mut self, function: &str, domain: &str, latency_us: u64, is_error: bool) {
        let metrics = self
            .metrics
            .entry(function.to_string())
            .or_insert_with(|| FunctionMetrics::new(function.to_string(), domain.to_string()));

        metrics.call_count += 1;

        if is_error {
            metrics.error_count += 1;
        } else {
            metrics.success_count += 1;
        }

        metrics.total_latency_us += latency_us;
        metrics.min_latency_us = metrics.min_latency_us.min(latency_us);
        metrics.max_latency_us = metrics.max_latency_us.max(latency_us);
    }

    fn get_all_metrics(&self) -> Vec<FunctionMetrics> {
        self.metrics.values().cloned().collect()
    }

    fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn reset(&mut self) {
        self.metrics.clear();
        self.start_time = Instant::now();
    }
}

/// Record a function call with its metrics
///
/// # Arguments
/// * `function` - Function name (e.g., "keyrx_on_state")
/// * `domain` - Domain name (e.g., "engine")
/// * `latency_us` - Call latency in microseconds
/// * `is_error` - Whether the call resulted in an error
pub fn record_call(function: &str, domain: &str, latency_us: u64, is_error: bool) {
    if let Ok(mut registry) = get_registry().lock() {
        registry.record_call(function, domain, latency_us, is_error);
    }
}

/// Get metrics for a specific function
///
/// # Arguments
/// * `function` - Function name to query
///
/// # Returns
/// * `Some(FunctionMetrics)` if the function has been called
/// * `None` if the function has not been called yet
pub fn get_function_metrics(function: &str) -> Option<FunctionMetrics> {
    get_registry().lock().ok()?.metrics.get(function).cloned()
}

/// Get all metrics for all functions
pub fn get_all_metrics() -> Vec<FunctionMetrics> {
    get_registry()
        .lock()
        .map(|registry| registry.get_all_metrics())
        .unwrap_or_default()
}

/// Get metrics summary as formatted string
pub fn get_summary() -> String {
    let metrics = get_all_metrics();
    let uptime = get_registry()
        .lock()
        .map(|r| r.get_uptime())
        .unwrap_or_default();

    let mut output = String::new();
    output.push_str("FFI Metrics Summary\n");
    output.push_str("===================\n\n");
    output.push_str(&format!("Uptime: {:?}\n", uptime));
    output.push_str(&format!("Total functions tracked: {}\n\n", metrics.len()));

    if metrics.is_empty() {
        output.push_str("No FFI calls recorded yet.\n");
        return output;
    }

    // Group by domain
    let mut by_domain: HashMap<String, Vec<&FunctionMetrics>> = HashMap::new();
    for metric in &metrics {
        by_domain
            .entry(metric.domain.clone())
            .or_default()
            .push(metric);
    }

    let mut domains: Vec<_> = by_domain.keys().cloned().collect();
    domains.sort();

    for domain in domains {
        let Some(domain_metrics) = by_domain.get(&domain) else {
            continue;
        };

        output.push_str(&format!("Domain: {}\n", domain));
        output.push_str(&"-".repeat(60));
        output.push('\n');

        for metric in domain_metrics {
            output.push_str(&format!(
                "  {}\n    Calls: {}, Success: {}, Errors: {} ({:.1}%)\n    Latency: avg={:.2}ms, min={:.2}ms, max={:.2}ms\n",
                metric.name,
                metric.call_count,
                metric.success_count,
                metric.error_count,
                metric.error_rate(),
                metric.avg_latency_us() as f64 / 1000.0,
                metric.min_latency_us as f64 / 1000.0,
                metric.max_latency_us as f64 / 1000.0,
            ));
        }
        output.push('\n');
    }

    output
}

/// Get metrics summary as JSON
pub fn get_summary_json() -> String {
    let metrics = get_all_metrics();
    let uptime = get_registry()
        .lock()
        .map(|r| r.get_uptime())
        .unwrap_or_default();

    let json = serde_json::json!({
        "uptime_seconds": uptime.as_secs(),
        "functions_tracked": metrics.len(),
        "metrics": metrics.iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "domain": m.domain,
                "call_count": m.call_count,
                "success_count": m.success_count,
                "error_count": m.error_count,
                "error_rate": m.error_rate(),
                "avg_latency_us": m.avg_latency_us(),
                "min_latency_us": m.min_latency_us,
                "max_latency_us": m.max_latency_us,
            })
        }).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

/// Reset all metrics
pub fn reset() {
    if let Ok(mut registry) = get_registry().lock() {
        registry.reset();
    }
}

/// Helper to measure and record function execution
///
/// # Examples
///
/// ```ignore
/// use keyrx_core::ffi::metrics::measure_call;
///
/// let result = measure_call("my_function", "my_domain", || {
///     // Your function logic
///     Ok(42)
/// });
/// ```
pub fn measure_call<F, T, E>(function: &str, domain: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    let start = Instant::now();
    let result = f();
    let latency_us = start.elapsed().as_micros() as u64;
    let is_error = result.is_err();

    record_call(function, domain, latency_us, is_error);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_retrieve_metrics() {
        reset();

        record_call("test_fn", "test_domain", 1000, false);
        record_call("test_fn", "test_domain", 2000, false);
        record_call("test_fn", "test_domain", 3000, true);

        let metrics = get_function_metrics("test_fn").unwrap();
        assert_eq!(metrics.call_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.avg_latency_us(), 2000);
        assert_eq!(metrics.min_latency_us, 1000);
        assert_eq!(metrics.max_latency_us, 3000);
    }

    #[test]
    fn test_error_rate_calculation() {
        reset();

        record_call("error_fn", "test", 100, false);
        record_call("error_fn", "test", 100, true);
        record_call("error_fn", "test", 100, true);
        record_call("error_fn", "test", 100, true);

        let metrics = get_function_metrics("error_fn").unwrap();
        assert_eq!(metrics.error_rate(), 75.0);
        assert_eq!(metrics.success_rate(), 25.0);
    }

    #[test]
    fn test_measure_call() {
        reset();

        let result: Result<i32, ()> = measure_call("measured_fn", "test", || Ok(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let metrics = get_function_metrics("measured_fn").unwrap();
        assert_eq!(metrics.call_count, 1);
        assert_eq!(metrics.success_count, 1);
    }

    #[test]
    fn test_summary_generation() {
        reset();

        record_call("fn1", "domain1", 1000, false);
        record_call("fn2", "domain2", 2000, true);

        let summary = get_summary();
        assert!(summary.contains("FFI Metrics Summary"));
        assert!(summary.contains("fn1"));
        assert!(summary.contains("fn2"));
        assert!(summary.contains("domain1"));
        assert!(summary.contains("domain2"));
    }

    #[test]
    fn test_json_summary() {
        reset();

        record_call("json_fn", "json_domain", 500, false);

        let json = get_summary_json();
        assert!(json.contains("json_fn"));
        assert!(json.contains("json_domain"));
        assert!(json.contains("\"call_count\": 1"));
    }
}
