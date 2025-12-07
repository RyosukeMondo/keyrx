use crate::hardware::TimingConfig;
use std::time::Duration;

/// Result of optimizing timing parameters based on observed latency samples.
#[derive(Debug, Clone)]
pub struct TimingOptimization {
    pub measured_latency: Duration,
    pub timing: TimingConfig,
    pub confidence: f64,
}

/// Calculates optimal timing parameters and confidence scores from latency samples.
#[derive(Debug, Default, Clone)]
pub struct TimingOptimizer;

impl TimingOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Derive tuned timing settings from measured samples.
    ///
    /// `expected_samples` is used to score confidence; pass the intended
    /// sample count for the calibration run.
    pub fn optimize(&self, samples: &[Duration], expected_samples: usize) -> TimingOptimization {
        if samples.is_empty() {
            return TimingOptimization {
                measured_latency: Duration::from_micros(0),
                timing: TimingConfig::default(),
                confidence: 0.0,
            };
        }

        let mean_us = mean_us(samples);
        let p95_us = percentile_us(samples, 0.95).unwrap_or(mean_us);

        let mut timing = TimingConfig::default();
        timing.debounce_ms = timing.debounce_ms.max(((p95_us / 1_000).max(1)) as u32);
        timing.scan_interval_us = timing.scan_interval_us.max(((mean_us / 2).max(250)) as u32);

        TimingOptimization {
            measured_latency: Duration::from_micros(mean_us),
            timing,
            confidence: confidence(samples, expected_samples),
        }
    }
}

fn mean_us(samples: &[Duration]) -> u64 {
    if samples.is_empty() {
        return 0;
    }

    let sum: u128 = samples.iter().map(|d| d.as_micros()).sum();
    (sum / samples.len() as u128) as u64
}

fn percentile_us(samples: &[Duration], percentile: f64) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }

    let mut micros: Vec<u64> = samples.iter().map(|d| d.as_micros() as u64).collect();
    micros.sort_unstable();

    let idx = ((micros.len() as f64 * percentile).ceil() as usize).saturating_sub(1);
    micros.get(idx).copied()
}

fn confidence(samples: &[Duration], expected_samples: usize) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }

    let mean = mean_us(samples) as f64;
    if mean == 0.0 {
        return 0.0;
    }

    let variance = samples
        .iter()
        .map(|d| {
            let delta = d.as_micros() as f64 - mean;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    let std_dev = variance.sqrt();
    let jitter_ratio = (std_dev / mean).min(1.0);

    let expected = expected_samples.max(1) as f64;
    let coverage = (samples.len() as f64 / expected).min(1.0);

    (0.6 * (1.0 - jitter_ratio) + 0.4 * coverage).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_defaults_when_no_samples() {
        let optimizer = TimingOptimizer::new();
        let result = optimizer.optimize(&[], 10);

        assert_eq!(result.measured_latency, Duration::from_micros(0));
        assert_eq!(result.timing, TimingConfig::default());
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn derives_timing_with_reasonable_bounds() {
        let optimizer = TimingOptimizer::new();
        let samples: Vec<Duration> = [800, 900, 1000, 1200, 1400]
            .iter()
            .copied()
            .map(Duration::from_micros)
            .collect();

        let result = optimizer.optimize(&samples, 5);

        assert!(result.measured_latency >= Duration::from_micros(900));
        assert!(result.timing.debounce_ms >= 1);
        assert!(result.timing.scan_interval_us >= 400);
    }

    #[test]
    fn confidence_rewards_stable_coverage() {
        let optimizer = TimingOptimizer::new();
        let stable_samples: Vec<Duration> = vec![Duration::from_micros(1_000); 5];
        let jittery_samples: Vec<Duration> = vec![
            Duration::from_micros(100),
            Duration::from_micros(5_000),
            Duration::from_micros(10_000),
        ];

        let stable = optimizer.optimize(&stable_samples, 5).confidence;
        let jittery = optimizer.optimize(&jittery_samples, 5).confidence;

        assert!(stable > jittery);
        assert!(stable <= 1.0);
    }
}
