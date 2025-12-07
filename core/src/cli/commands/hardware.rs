//! Hardware CLI commands: detection, profile lookup, and calibration.
//!
//! This command surfaces the new hardware detection stack, profile database,
//! and calibration pipeline to the CLI with human and machine-readable output.

use crate::cli::HasExitCode;
use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::hardware::{
    CalibrationComparison, CalibrationConfig, CalibrationRunner, Calibrator, DeviceClass,
    HardwareDetector, HardwareInfo, ProfileDatabase, ProfileSource, TimingConfig,
};
use serde::Serialize;
use std::fs;
use std::time::Duration;
use tokio::runtime::Handle;

/// Actions supported by the hardware command.
#[derive(Debug, Clone)]
pub enum HardwareAction {
    Detect,
    Profile {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
    },
    Calibrate {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        warmup_samples: usize,
        sample_count: usize,
        max_duration_secs: u64,
        latencies_us: Vec<u64>,
        samples_file: Option<String>,
    },
}

#[derive(Debug, Clone)]
struct CalibrationRequest {
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    warmup_samples: usize,
    sample_count: usize,
    max_duration_secs: u64,
    latencies_us: Vec<u64>,
    samples_file: Option<String>,
}

/// Hardware command entry point.
pub struct HardwareCommand {
    output: OutputWriter,
    action: HardwareAction,
}

impl HardwareCommand {
    pub fn new(format: OutputFormat, action: HardwareAction) -> Self {
        Self {
            output: OutputWriter::new(format),
            action,
        }
    }

    fn detect(&self) -> CommandResult<()> {
        let database = ProfileDatabase::with_builtin();
        let detected = match HardwareDetector::detect_all() {
            Ok(devices) if devices.is_empty() => {
                return CommandResult::failure(
                    ExitCode::DeviceNotFound,
                    "No keyboard devices detected",
                )
            }
            Ok(devices) => devices,
            Err(err) => return CommandResult::failure(err.exit_code(), format!("{err:#}")),
        };

        let entries: Vec<DetectionEntry> = detected
            .iter()
            .map(|info| DetectionEntry::from(info, database.resolve(info)))
            .collect();

        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Table => {
                let payload = DetectionOutput { devices: entries };
                let _ = self.output.data(&payload);
            }
            OutputFormat::Human => {
                println!("Detected hardware ({} devices):", entries.len());
                for (idx, entry) in entries.iter().enumerate() {
                    println!(
                        "  {}. {:04x}:{:04x} {} ({})",
                        idx + 1,
                        entry.vendor_id,
                        entry.product_id,
                        entry.product_name.as_deref().unwrap_or("Unknown keyboard"),
                        entry.device_class
                    );

                    if let Some(profile) = &entry.suggested_profile {
                        println!(
                        "     Suggested profile: {} [{:?}] debounce={}ms repeat={}ms/{}ms scan={}us",
                        profile.name,
                        profile.source,
                        profile.timing.debounce_ms,
                        profile.timing.repeat_delay_ms,
                        profile.timing.repeat_rate_ms,
                        profile.timing.scan_interval_us
                    );
                    } else {
                        println!("     Suggested profile: <none>");
                    }
                }
            }
        }

        CommandResult::success(())
    }

    fn profile(&self, vendor_id: Option<u16>, product_id: Option<u16>) -> CommandResult<()> {
        if vendor_id.is_some() ^ product_id.is_some() {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Provide both --vendor-id and --product-id to resolve a specific device",
            );
        }

        let database = ProfileDatabase::with_builtin();

        let targets: Vec<HardwareInfo> = if let (Some(vid), Some(pid)) = (vendor_id, product_id) {
            vec![HardwareInfo {
                vendor_id: vid,
                product_id: pid,
                vendor_name: None,
                product_name: None,
                device_class: DeviceClass::Unknown,
            }]
        } else {
            match HardwareDetector::detect_all() {
                Ok(devices) if devices.is_empty() => {
                    return CommandResult::failure(
                        ExitCode::DeviceNotFound,
                        "No keyboard devices detected",
                    )
                }
                Ok(devices) => devices,
                Err(err) => return CommandResult::failure(err.exit_code(), format!("{err:#}")),
            }
        };

        let profiles: Vec<ProfileView> = targets
            .iter()
            .map(|hw| {
                let resolved = database.resolve(hw);
                ProfileView::from_hardware(hw, resolved)
            })
            .collect();

        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Table => {
                let payload = ProfileOutput { profiles };
                let _ = self.output.data(&payload);
            }
            OutputFormat::Human => {
                println!("Resolved profiles:");
                for profile in &profiles {
                    println!(
                        "  {:04x}:{:04x} -> {} [{:?}]",
                        profile.vendor_id, profile.product_id, profile.name, profile.source
                    );
                    println!(
                        "     debounce={}ms repeat={}ms/{}ms scan={}us class={}",
                        profile.timing.debounce_ms,
                        profile.timing.repeat_delay_ms,
                        profile.timing.repeat_rate_ms,
                        profile.timing.scan_interval_us,
                        profile.device_class
                    );
                }
            }
        }

        CommandResult::success(())
    }

    fn calibrate(&self, request: CalibrationRequest) -> CommandResult<()> {
        if request.vendor_id.is_some() ^ request.product_id.is_some() {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Provide both --vendor-id and --product-id when specifying device identity",
            );
        }

        let samples = match self.load_samples(request.latencies_us, request.samples_file.as_deref())
        {
            Ok(data) => data,
            Err(err) => return err,
        };

        let total_needed = request.warmup_samples + request.sample_count;
        if samples.len() < total_needed {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!(
                    "Not enough samples: provided {}, need at least {} (warmup {} + sample {})",
                    samples.len(),
                    total_needed,
                    request.warmup_samples,
                    request.sample_count
                ),
            );
        }

        let runner = StaticLatencyRunner::new(samples);
        let config = CalibrationConfig {
            warmup_samples: request.warmup_samples,
            sample_count: request.sample_count,
            max_duration: Duration::from_secs(request.max_duration_secs),
        };
        let calibrator = Calibrator::new(config);

        let result = match Handle::try_current() {
            Ok(handle) => handle.block_on(calibrator.run(&runner)),
            Err(_) => {
                let runtime = tokio::runtime::Runtime::new().map_err(|err| {
                    CommandResult::failure(
                        ExitCode::GeneralError,
                        format!("Failed to start runtime: {err}"),
                    )
                });
                match runtime {
                    Ok(rt) => rt.block_on(calibrator.run(&runner)),
                    Err(res) => return res,
                }
            }
        };

        let result = match result {
            Ok(res) => res,
            Err(err) => return CommandResult::failure(err.exit_code(), format!("{err:#}")),
        };

        let before_profile =
            request
                .vendor_id
                .zip(request.product_id)
                .map(|(vid, pid)| HardwareInfo {
                    vendor_id: vid,
                    product_id: pid,
                    vendor_name: None,
                    product_name: None,
                    device_class: DeviceClass::Unknown,
                });

        let mut comparison: Option<CalibrationComparison> = None;
        let mut baseline: Option<TimingConfig> = None;

        if let Some(hw) = before_profile {
            let baseline_profile = ProfileDatabase::with_builtin().resolve(&hw);
            comparison = Some(calibrator.compare(&baseline_profile.timing, &result.optimal_timing));
            baseline = Some(baseline_profile.timing);
        }

        let view = CalibrationOutput::new(result, baseline, comparison);

        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml | OutputFormat::Table => {
                let _ = self.output.data(&view);
            }
            OutputFormat::Human => {
                self.output.success("Calibration complete");
                println!(
                    "Measured latency: {} µs (confidence {:.2})",
                    view.measured_latency_us, view.confidence
                );
                println!(
                    "Optimal timing: debounce={}ms repeat={}ms/{}ms scan={}us",
                    view.optimal_timing.debounce_ms,
                    view.optimal_timing.repeat_delay_ms,
                    view.optimal_timing.repeat_rate_ms,
                    view.optimal_timing.scan_interval_us
                );

                if let (Some(base), Some(deltas)) = (&view.before_timing, &view.deltas) {
                    println!(
                        "Baseline: debounce={}ms repeat={}ms/{}ms scan={}us",
                        base.debounce_ms,
                        base.repeat_delay_ms,
                        base.repeat_rate_ms,
                        base.scan_interval_us
                    );
                    println!(
                        "Deltas: debounce {:+}ms repeat_delay {:+}ms repeat_rate {:+}ms scan {:+}us",
                        deltas.debounce_delta_ms,
                        deltas.repeat_delay_delta_ms,
                        deltas.repeat_rate_delta_ms,
                        deltas.scan_interval_delta_us
                    );
                }

                println!("Samples used: {}", view.samples_us.len());
            }
        }

        CommandResult::success(())
    }

    fn load_samples(
        &self,
        latencies_us: Vec<u64>,
        samples_file: Option<&str>,
    ) -> Result<Vec<Duration>, CommandResult<()>> {
        let mut samples: Vec<u64> = latencies_us;

        if let Some(path) = samples_file {
            let contents = fs::read_to_string(path).map_err(|err| {
                CommandResult::failure(
                    err.exit_code(),
                    format!("Failed to read samples file {path}: {err}"),
                )
            })?;

            for (idx, line) in contents.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match trimmed.parse::<u64>() {
                    Ok(value) => samples.push(value),
                    Err(err) => {
                        return Err(CommandResult::failure(
                            ExitCode::ValidationFailed,
                            format!("Invalid sample on line {}: {}", idx + 1, err),
                        ))
                    }
                }
            }
        }

        if samples.is_empty() {
            return Err(CommandResult::failure(
                ExitCode::ValidationFailed,
                "Provide latency samples via --latency-us or --samples-file",
            ));
        }

        let durations: Vec<Duration> = samples.into_iter().map(Duration::from_micros).collect();

        Ok(durations)
    }
}

impl Command for HardwareCommand {
    fn name(&self) -> &str {
        "hardware"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        match self.action.clone() {
            HardwareAction::Detect => self.detect(),
            HardwareAction::Profile {
                vendor_id,
                product_id,
            } => self.profile(vendor_id, product_id),
            HardwareAction::Calibrate {
                vendor_id,
                product_id,
                warmup_samples,
                sample_count,
                max_duration_secs,
                latencies_us,
                samples_file,
            } => self.calibrate(CalibrationRequest {
                vendor_id,
                product_id,
                warmup_samples,
                sample_count,
                max_duration_secs,
                latencies_us,
                samples_file,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
struct DetectionEntry {
    vendor_id: u16,
    product_id: u16,
    vendor_name: Option<String>,
    product_name: Option<String>,
    device_class: String,
    suggested_profile: Option<ProfileView>,
}

impl DetectionEntry {
    fn from(info: &HardwareInfo, resolved: crate::hardware::HardwareProfile) -> Self {
        Self {
            vendor_id: info.vendor_id,
            product_id: info.product_id,
            vendor_name: info.vendor_name.clone(),
            product_name: info.product_name.clone(),
            device_class: format!("{:?}", info.device_class),
            suggested_profile: Some(ProfileView::from(resolved, info.device_class)),
        }
    }
}

#[derive(Debug, Serialize)]
struct DetectionOutput {
    devices: Vec<DetectionEntry>,
}

#[derive(Debug, Serialize)]
struct ProfileView {
    vendor_id: u16,
    product_id: u16,
    name: String,
    source: ProfileSource,
    timing: TimingConfig,
    device_class: String,
}

impl ProfileView {
    fn from(profile: crate::hardware::HardwareProfile, device_class: DeviceClass) -> Self {
        Self {
            vendor_id: profile.vendor_id,
            product_id: profile.product_id,
            name: profile.name,
            source: profile.source,
            timing: profile.timing,
            device_class: format!("{:?}", device_class),
        }
    }

    fn from_hardware(hw: &HardwareInfo, profile: crate::hardware::HardwareProfile) -> Self {
        Self::from(profile, hw.device_class)
    }
}

#[derive(Debug, Serialize)]
struct ProfileOutput {
    profiles: Vec<ProfileView>,
}

#[derive(Debug, Serialize)]
struct CalibrationOutput {
    measured_latency_us: u64,
    confidence: f64,
    optimal_timing: TimingConfig,
    before_timing: Option<TimingConfig>,
    deltas: Option<DeltaView>,
    samples_us: Vec<u64>,
}

impl CalibrationOutput {
    fn new(
        result: crate::hardware::CalibrationResult,
        before_timing: Option<TimingConfig>,
        deltas: Option<CalibrationComparison>,
    ) -> Self {
        Self {
            measured_latency_us: result.measured_latency.as_micros() as u64,
            confidence: result.confidence,
            optimal_timing: result.optimal_timing.clone(),
            before_timing,
            deltas: deltas.map(DeltaView::from),
            samples_us: result
                .samples
                .iter()
                .map(|d| d.as_micros() as u64)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct DeltaView {
    debounce_delta_ms: i32,
    repeat_delay_delta_ms: i32,
    repeat_rate_delta_ms: i32,
    scan_interval_delta_us: i32,
}

impl From<CalibrationComparison> for DeltaView {
    fn from(value: CalibrationComparison) -> Self {
        Self {
            debounce_delta_ms: value.debounce_delta_ms,
            repeat_delay_delta_ms: value.repeat_delay_delta_ms,
            repeat_rate_delta_ms: value.repeat_rate_delta_ms,
            scan_interval_delta_us: value.scan_interval_delta_us,
        }
    }
}

struct StaticLatencyRunner {
    samples: Vec<Duration>,
}

impl StaticLatencyRunner {
    fn new(samples: Vec<Duration>) -> Self {
        Self { samples }
    }
}

#[async_trait::async_trait]
impl CalibrationRunner for StaticLatencyRunner {
    async fn run_sequence(
        &self,
        total_samples: usize,
        _max_duration: Duration,
    ) -> Result<Vec<Duration>, crate::errors::KeyrxError> {
        Ok(self.samples.iter().take(total_samples).cloned().collect())
    }
}
