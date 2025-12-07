use crate::errors::KeyrxError;
use crate::hardware::optimizer::TimingOptimizer;
use crate::hardware::TimingConfig;
use async_trait::async_trait;
use std::time::Duration;

/// Configuration for running a calibration cycle.
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// Number of initial samples to discard to avoid cold-start noise.
    pub warmup_samples: usize,
    /// Number of samples to keep for the final analysis.
    pub sample_count: usize,
    /// Maximum duration allowed for a calibration run.
    pub max_duration: Duration,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            warmup_samples: 3,
            sample_count: 25,
            max_duration: Duration::from_secs(30),
        }
    }
}

/// Result of a calibration pass, including measured latency and tuned timing.
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    pub measured_latency: Duration,
    pub optimal_timing: TimingConfig,
    pub confidence: f64,
    pub samples: Vec<Duration>,
}

/// Differences between two timing configurations for before/after reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationComparison {
    pub debounce_delta_ms: i32,
    pub repeat_delay_delta_ms: i32,
    pub repeat_rate_delta_ms: i32,
    pub scan_interval_delta_us: i32,
}

#[async_trait]
pub trait CalibrationRunner {
    /// Execute the hardware test sequence and return per-sample latencies.
    ///
    /// Implementations should honor the provided limits and return latencies
    /// in microseconds for each observed sample.
    async fn run_sequence(
        &self,
        total_samples: usize,
        max_duration: Duration,
    ) -> Result<Vec<Duration>, KeyrxError>;
}

/// Calibrates a hardware device by running timed test sequences and deriving
/// a tuned timing configuration.
pub struct Calibrator {
    config: CalibrationConfig,
    optimizer: TimingOptimizer,
}

impl Calibrator {
    /// Build a calibrator with the provided configuration.
    pub fn new(config: CalibrationConfig) -> Self {
        Self {
            config,
            optimizer: TimingOptimizer::new(),
        }
    }

    /// Run calibration using the supplied runner implementation.
    pub async fn run<R>(&self, runner: &R) -> Result<CalibrationResult, KeyrxError>
    where
        R: CalibrationRunner + Sync,
    {
        let raw_samples = runner
            .run_sequence(self.config.total_samples(), self.config.max_duration)
            .await?;

        let samples = self.prepare_samples(raw_samples);
        let optimization = self.optimizer.optimize(&samples, self.config.sample_count);

        Ok(CalibrationResult {
            measured_latency: optimization.measured_latency,
            optimal_timing: optimization.timing,
            confidence: optimization.confidence,
            samples,
        })
    }

    /// Compare two timing configurations for presentation in before/after UI.
    pub fn compare(&self, before: &TimingConfig, after: &TimingConfig) -> CalibrationComparison {
        CalibrationComparison {
            debounce_delta_ms: after.debounce_ms as i32 - before.debounce_ms as i32,
            repeat_delay_delta_ms: after.repeat_delay_ms as i32 - before.repeat_delay_ms as i32,
            repeat_rate_delta_ms: after.repeat_rate_ms as i32 - before.repeat_rate_ms as i32,
            scan_interval_delta_us: after.scan_interval_us as i32 - before.scan_interval_us as i32,
        }
    }

    fn prepare_samples(&self, samples: Vec<Duration>) -> Vec<Duration> {
        samples
            .into_iter()
            .skip(self.config.warmup_samples)
            .take(self.config.sample_count)
            .collect()
    }
}

impl CalibrationConfig {
    fn total_samples(&self) -> usize {
        self.warmup_samples + self.sample_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    struct FakeRunner {
        samples: Mutex<Vec<Duration>>,
    }

    impl FakeRunner {
        fn new(samples: Vec<Duration>) -> Self {
            Self {
                samples: Mutex::new(samples),
            }
        }
    }

    #[async_trait]
    impl CalibrationRunner for FakeRunner {
        async fn run_sequence(
            &self,
            total_samples: usize,
            _max_duration: Duration,
        ) -> Result<Vec<Duration>, KeyrxError> {
            let mut guard = self.samples.lock().await;
            guard.truncate(total_samples);
            Ok(guard.clone())
        }
    }

    #[tokio::test]
    async fn drops_warmup_and_limits_samples() {
        let runner = FakeRunner::new(
            vec![2, 4, 6, 8, 10, 12]
                .into_iter()
                .map(|ms| Duration::from_millis(ms))
                .collect(),
        );

        let calibrator = Calibrator::new(CalibrationConfig {
            warmup_samples: 2,
            sample_count: 3,
            ..Default::default()
        });

        let result = calibrator.run(&runner).await.unwrap();
        assert_eq!(result.samples.len(), 3);
        assert_eq!(
            result.samples,
            vec![
                Duration::from_millis(6),
                Duration::from_millis(8),
                Duration::from_millis(10)
            ]
        );
    }

    #[tokio::test]
    async fn derives_timing_from_latency_profile() {
        let runner = FakeRunner::new(
            vec![800, 900, 1000, 1200, 1400]
                .into_iter()
                .map(Duration::from_micros)
                .collect(),
        );

        let calibrator = Calibrator::new(CalibrationConfig {
            warmup_samples: 0,
            sample_count: 5,
            ..Default::default()
        });

        let result = calibrator.run(&runner).await.unwrap();
        assert!(result.measured_latency >= Duration::from_micros(900));
        assert!(result.optimal_timing.debounce_ms >= 1);
        assert!(result.optimal_timing.scan_interval_us >= 400);
    }

    #[tokio::test]
    async fn compare_reports_deltas() {
        let calibrator = Calibrator::new(CalibrationConfig::default());
        let before = TimingConfig::default();
        let after = TimingConfig {
            debounce_ms: before.debounce_ms + 1,
            repeat_delay_ms: before.repeat_delay_ms + 10,
            repeat_rate_ms: before.repeat_rate_ms + 2,
            scan_interval_us: before.scan_interval_us + 150,
        };

        let comparison = calibrator.compare(&before, &after);
        assert_eq!(comparison.debounce_delta_ms, 1);
        assert_eq!(comparison.repeat_delay_delta_ms, 10);
        assert_eq!(comparison.repeat_rate_delta_ms, 2);
        assert_eq!(comparison.scan_interval_delta_us, 150);
    }
}
