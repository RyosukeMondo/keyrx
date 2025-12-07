//! Hardware CLI commands: detection, profile lookup, and calibration.
//!
//! This command surfaces the new hardware detection stack, profile database,
//! and calibration pipeline to the CLI with human and machine-readable output.

use crate::cli::HasExitCode;
use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::models::HardwareProfile;
use crate::config::{ConfigManager, StorageError};
use crate::hardware::{
    CalibrationComparison, CalibrationConfig, CalibrationRunner, Calibrator, DeviceClass,
    HardwareDetector, HardwareInfo, ProfileDatabase, ProfileSource, TimingConfig,
};
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Handle;

/// Actions supported by the hardware command.
#[derive(Debug, Clone)]
pub enum HardwareAction {
    List,
    Define {
        source: HardwareSource,
    },
    Wire {
        profile_id: String,
        scancode: u16,
        virtual_key: Option<String>,
        clear: bool,
    },
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

/// Source for hardware profile creation (file path or stdin).
#[derive(Debug, Clone)]
pub enum HardwareSource {
    File(PathBuf),
    Stdin,
}

/// Hardware command entry point.
pub struct HardwareCommand {
    output: OutputWriter,
    action: HardwareAction,
    config_root: Option<PathBuf>,
}

impl HardwareCommand {
    pub fn new(format: OutputFormat, action: HardwareAction) -> Self {
        Self {
            output: OutputWriter::new(format),
            action,
            config_root: None,
        }
    }

    /// Override the config root (useful for tests).
    pub fn with_config_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.config_root = Some(root.into());
        self
    }

    fn run(&self) -> CommandResult<()> {
        match &self.action {
            HardwareAction::List => self.list(),
            HardwareAction::Define { source } => self.define(source),
            HardwareAction::Wire {
                profile_id,
                scancode,
                virtual_key,
                clear,
            } => self.wire(profile_id, *scancode, virtual_key.as_deref(), *clear),
            HardwareAction::Detect => self.detect(),
            HardwareAction::Profile {
                vendor_id,
                product_id,
            } => self.profile(*vendor_id, *product_id),
            HardwareAction::Calibrate {
                vendor_id,
                product_id,
                warmup_samples,
                sample_count,
                max_duration_secs,
                latencies_us,
                samples_file,
            } => self.calibrate(CalibrationRequest {
                vendor_id: *vendor_id,
                product_id: *product_id,
                warmup_samples: *warmup_samples,
                sample_count: *sample_count,
                max_duration_secs: *max_duration_secs,
                latencies_us: latencies_us.clone(),
                samples_file: samples_file.clone(),
            }),
        }
    }

    fn list(&self) -> CommandResult<()> {
        let manager = self.manager();
        let profiles = match manager.load_hardware_profiles() {
            Ok(map) => map.into_values().collect::<Vec<_>>(),
            Err(err) => return self.storage_failure("load hardware profiles", err),
        };

        let mut summaries: Vec<HardwareSummary> =
            profiles.into_iter().map(HardwareSummary::from).collect();
        summaries.sort_by(|a, b| a.id.cmp(&b.id));

        if let Err(err) = self.output.data(&HardwareListOutput {
            hardware_profiles: summaries,
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render hardware list: {err}"),
            );
        }

        CommandResult::success(())
    }

    fn define(&self, source: &HardwareSource) -> CommandResult<()> {
        let json = match self.read_source(source) {
            Ok(content) => content,
            Err(result) => return result,
        };

        let profile: HardwareProfile = match serde_json::from_str(&json) {
            Ok(parsed) => parsed,
            Err(err) => {
                return CommandResult::failure(
                    ExitCode::ValidationFailed,
                    format!("Invalid hardware profile JSON: {err}"),
                )
            }
        };

        let manager = self.manager();
        let saved_path = match manager.save_hardware_profile(&profile) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save hardware profile", err),
        };

        let summary = HardwareSummary::from(profile);

        if let Err(err) = self.output.data(&HardwareSaveOutput {
            saved_path: saved_path.display().to_string(),
            profile: summary.clone(),
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render hardware output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success(&format!(
                "Saved hardware profile '{}' to {}",
                summary.id,
                saved_path.display()
            ));
        }

        CommandResult::success(())
    }

    fn wire(
        &self,
        profile_id: &str,
        scancode: u16,
        virtual_key: Option<&str>,
        clear: bool,
    ) -> CommandResult<()> {
        if clear && virtual_key.is_some() {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Use either --virtual-key to set or --clear to remove a mapping, not both",
            );
        }

        if virtual_key.is_none() && !clear {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Provide --virtual-key to set a mapping or --clear to remove one",
            );
        }

        let manager = self.manager();
        let mut profiles = match manager.load_hardware_profiles() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load hardware profiles", err),
        };

        let Some(mut profile) = profiles.remove(profile_id) else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Hardware profile '{profile_id}' not found"),
            );
        };

        if clear {
            profile.wiring.remove(&scancode);
        } else if let Some(key) = virtual_key {
            profile.wiring.insert(scancode, key.to_string());
        }

        let saved_path = match manager.save_hardware_profile(&profile) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save hardware profile", err),
        };

        let mapping = profile.wiring.get(&scancode).cloned();
        let summary = HardwareSummary::from(profile);

        if let Err(err) = self.output.data(&HardwareWireOutput {
            saved_path: saved_path.display().to_string(),
            profile: summary.clone(),
            mapping: WireMappingOutput {
                scancode,
                virtual_key: mapping.clone(),
                action: if clear { "cleared" } else { "set" }.to_string(),
            },
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render wiring output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            if clear {
                self.output.success(&format!(
                    "Cleared mapping for scancode 0x{scancode:02x} in profile '{profile_id}'"
                ));
            } else if let Some(key) = mapping {
                self.output.success(&format!(
                    "Mapped scancode 0x{scancode:02x} -> {key} in profile '{profile_id}'"
                ));
            }
        }

        CommandResult::success(())
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

    fn read_source(&self, source: &HardwareSource) -> Result<String, CommandResult<()>> {
        match source {
            HardwareSource::Stdin => {
                let mut buffer = String::new();
                if let Err(err) = io::stdin().read_to_string(&mut buffer) {
                    return Err(CommandResult::failure(
                        ExitCode::GeneralError,
                        format!("Failed to read hardware profile from stdin: {err}"),
                    ));
                }
                Ok(buffer)
            }
            HardwareSource::File(path) => fs::read_to_string(path).map_err(|err| {
                let code = if err.kind() == io::ErrorKind::PermissionDenied {
                    ExitCode::PermissionDenied
                } else {
                    ExitCode::GeneralError
                };
                CommandResult::failure(
                    code,
                    format!(
                        "Failed to read hardware profile file '{}': {err}",
                        path.display()
                    ),
                )
            }),
        }
    }

    fn storage_failure(&self, action: &str, err: StorageError) -> CommandResult<()> {
        let code = match &err {
            StorageError::CreateDir(_, e)
            | StorageError::ReadDir(_, e)
            | StorageError::ReadFile(_, e)
            | StorageError::WriteFile(_, e)
                if e.kind() == io::ErrorKind::PermissionDenied =>
            {
                ExitCode::PermissionDenied
            }
            StorageError::Parse(_, _) => ExitCode::ValidationFailed,
            _ => ExitCode::GeneralError,
        };
        CommandResult::failure(code, format!("Failed to {action}: {err}"))
    }

    fn manager(&self) -> ConfigManager {
        match &self.config_root {
            Some(root) => ConfigManager::new(root.clone()),
            None => ConfigManager::default(),
        }
    }
}

impl Command for HardwareCommand {
    fn name(&self) -> &str {
        "hardware"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}

#[derive(Debug, Serialize, Clone)]
struct HardwareSummary {
    id: String,
    vendor_id: u16,
    product_id: u16,
    name: Option<String>,
    virtual_layout_id: String,
    wiring_count: usize,
}

impl From<HardwareProfile> for HardwareSummary {
    fn from(profile: HardwareProfile) -> Self {
        Self {
            id: profile.id,
            vendor_id: profile.vendor_id,
            product_id: profile.product_id,
            name: profile.name,
            virtual_layout_id: profile.virtual_layout_id,
            wiring_count: profile.wiring.len(),
        }
    }
}

#[derive(Debug, Serialize)]
struct HardwareListOutput {
    hardware_profiles: Vec<HardwareSummary>,
}

#[derive(Debug, Serialize)]
struct HardwareSaveOutput {
    saved_path: String,
    profile: HardwareSummary,
}

#[derive(Debug, Serialize)]
struct WireMappingOutput {
    scancode: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    virtual_key: Option<String>,
    action: String,
}

#[derive(Debug, Serialize)]
struct HardwareWireOutput {
    saved_path: String,
    profile: HardwareSummary,
    mapping: WireMappingOutput,
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
