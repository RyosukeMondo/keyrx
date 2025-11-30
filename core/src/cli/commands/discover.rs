//! Device discovery CLI command.
//!
//! Drives the discovery state machine for a target keyboard, captures progress,
//! and writes the resulting profile to the registry.

use crate::cli::{OutputFormat, OutputWriter};
use crate::discovery::{
    default_schema_version, DeviceId, DeviceProfile, DeviceRegistry, DiscoverySession,
    DiscoverySummary, ProfileSource, RegistryEntry, RegistryStatus, SessionStatus, SessionUpdate,
};
use crate::drivers;
use crate::drivers::DeviceInfo;
use crate::traits::InputSource;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

/// Exit reasons that map to specific CLI exit codes.
#[derive(Debug, Error)]
pub enum DiscoverExit {
    #[error("discovery cancelled")]
    Cancelled,
    #[error("discovery validation failed: {0}")]
    Validation(String),
}

/// Discover command dependencies (device + input source).
pub struct DiscoverInput {
    pub input: Box<dyn InputSource>,
}

type DeviceLister = dyn Fn() -> Result<Vec<DeviceInfo>> + Send + Sync;
type InputBuilder = dyn Fn(&DeviceInfo) -> Result<DiscoverInput> + Send + Sync;

/// CLI command for `keyrx discover`.
pub struct DiscoverCommand {
    pub device: Option<String>,
    pub force: bool,
    pub assume_yes: bool,
    pub output: OutputWriter,
    list_devices: Box<DeviceLister>,
    build_input: Box<InputBuilder>,
}

impl DiscoverCommand {
    pub fn new(
        device: Option<String>,
        force: bool,
        assume_yes: bool,
        format: OutputFormat,
    ) -> Self {
        Self {
            device,
            force,
            assume_yes,
            output: OutputWriter::new(format),
            list_devices: Box::new(drivers::list_keyboards),
            build_input: Box::new(default_input_builder),
        }
    }

    #[cfg(test)]
    pub fn with_providers(
        device: Option<String>,
        force: bool,
        assume_yes: bool,
        format: OutputFormat,
        list_devices: impl Fn() -> Result<Vec<DeviceInfo>> + Send + Sync + 'static,
        build_input: impl Fn(&DeviceInfo) -> Result<DiscoverInput> + Send + Sync + 'static,
    ) -> Self {
        Self {
            device,
            force,
            assume_yes,
            output: OutputWriter::new(format),
            list_devices: Box::new(list_devices),
            build_input: Box::new(build_input),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let device = self.select_device()?;
        let mut registry = DeviceRegistry::new();
        let device_id = DeviceId::new(device.vendor_id, device.product_id);
        let entry = registry.load_or_default(device_id);

        if entry.status == RegistryStatus::Ready && !self.force {
            self.report_existing_profile(&entry)?;
            return Ok(());
        }

        if self.force {
            self.output
                .warning("Forcing discovery even though a profile exists");
        } else {
            self.output
                .warning("Profile missing or invalid; starting discovery");
        }

        let (rows, cols_per_row) = self.determine_layout(&entry)?;
        let mut session = DiscoverySession::new(device_id, rows, cols_per_row)
            .map_err(|err| DiscoverExit::Validation(format!("invalid layout: {err}")))?;

        if let Some(path) = device.path.to_str() {
            session = session.with_target_device_id(path);
        }
        let emergency = Arc::new(Mutex::new(EmergencyTracker::new()));
        let emergency_clone = emergency.clone();
        session = session.with_emergency_exit_detector(move |event| {
            emergency_clone
                .lock()
                .map(|mut tracker| tracker.update(event))
                .unwrap_or(false)
        });

        let mut discover_input = (self.build_input)(&device)?;
        let start_msg = format!(
            "Starting discovery for {} ({:04x}:{:04x}) at {}",
            device.name,
            device.vendor_id,
            device.product_id,
            device.path.display()
        );
        self.output.success(&start_msg);
        discover_input
            .input
            .start()
            .await
            .context("Failed to start input capture")?;

        let summary = self
            .capture_session(&mut discover_input.input, session)
            .await?;

        // Best-effort cleanup; prefer reporting discovery outcome over stop errors.
        if let Err(err) = discover_input.input.stop().await {
            self.output
                .warning(&format!("Failed to stop input capture cleanly: {err:#}"));
        }

        self.handle_summary(&mut registry, summary, &device)
    }

    fn select_device(&self) -> Result<DeviceInfo, DiscoverExit> {
        let devices = (self.list_devices)().map_err(|err| {
            DiscoverExit::Validation(format!("device enumeration failed: {err:#}"))
        })?;

        if devices.is_empty() {
            return Err(DiscoverExit::Validation(
                "No keyboard devices detected".to_string(),
            ));
        }

        if let Some(device_str) = &self.device {
            let id = parse_device_id(device_str)?;
            devices
                .into_iter()
                .find(|dev| dev.vendor_id == id.vendor_id && dev.product_id == id.product_id)
                .ok_or_else(|| {
                    DiscoverExit::Validation(format!(
                        "No device found for {:04x}:{:04x}",
                        id.vendor_id, id.product_id
                    ))
                })
        } else {
            Ok(devices[0].clone())
        }
    }

    fn determine_layout(&self, entry: &RegistryEntry) -> Result<(u8, Vec<u8>), DiscoverExit> {
        if entry.profile.rows > 0 && !entry.profile.cols_per_row.is_empty() {
            return Ok((entry.profile.rows, entry.profile.cols_per_row.clone()));
        }

        if self.assume_yes || matches!(self.output.format(), OutputFormat::Json) {
            let (rows, cols) = default_layout();
            return Ok((rows, cols));
        }

        prompt_for_layout()
    }

    fn report_existing_profile(&self, entry: &RegistryEntry) -> Result<()> {
        let message = format!(
            "Profile already exists for {:04x}:{:04x} at {}. Use --force to re-discover.",
            entry.device_id.vendor_id,
            entry.device_id.product_id,
            entry.path.display()
        );

        match self.output.format() {
            OutputFormat::Human => self.output.success(&message),
            OutputFormat::Json => {
                let payload = ExistingProfileJson {
                    status: "ready",
                    path: entry.path.display().to_string(),
                    vendor_id: entry.device_id.vendor_id,
                    product_id: entry.device_id.product_id,
                };
                self.output.data(&payload)?;
            }
        }
        Ok(())
    }

    async fn capture_session(
        &self,
        input: &mut Box<dyn InputSource>,
        mut session: DiscoverySession,
    ) -> Result<DiscoverySummary> {
        loop {
            let events = input.poll_events().await?;
            if events.is_empty() {
                sleep(Duration::from_millis(25)).await;
                continue;
            }

            for event in events {
                match session.handle_event(&event) {
                    SessionUpdate::Ignored => {}
                    SessionUpdate::Progress(progress) => self.report_progress(&progress)?,
                    SessionUpdate::Duplicate(dup) => self.report_duplicate(&dup)?,
                    SessionUpdate::Finished(summary) => return Ok(summary),
                }
            }
        }
    }

    fn handle_summary(
        &self,
        registry: &mut DeviceRegistry,
        summary: DiscoverySummary,
        device: &DeviceInfo,
    ) -> Result<()> {
        self.report_summary(&summary)?;

        match summary.status {
            SessionStatus::Completed => {
                if !self.assume_yes
                    && !matches!(self.output.format(), OutputFormat::Json)
                    && !confirm("Save discovered profile? [y/N]: ")?
                {
                    return Err(DiscoverExit::Cancelled.into());
                }

                let profile = DeviceProfile {
                    schema_version: default_schema_version(),
                    vendor_id: summary.device_id.vendor_id,
                    product_id: summary.device_id.product_id,
                    name: Some(device.name.clone()),
                    discovered_at: Utc::now(),
                    rows: summary.rows,
                    cols_per_row: summary.cols_per_row.clone(),
                    keymap: summary.keymap.clone(),
                    aliases: summary.aliases.clone(),
                    source: ProfileSource::Discovered,
                };
                let path = registry.save_profile(profile)?;
                self.output
                    .success(&format!("Saved profile to {}", path.display()));

                if matches!(self.output.format(), OutputFormat::Json) {
                    let payload = DiscoverResultJson {
                        status: "saved",
                        path: path.display().to_string(),
                        cols_per_row: summary.cols_per_row.clone(),
                    };
                    self.output.data(&payload)?;
                }
                Ok(())
            }
            SessionStatus::Cancelled | SessionStatus::Bypassed => {
                Err(DiscoverExit::Cancelled.into())
            }
            SessionStatus::InProgress => {
                Err(DiscoverExit::Validation("discovery did not complete".to_string()).into())
            }
        }
    }

    fn report_progress(&self, progress: &crate::discovery::DiscoveryProgress) -> Result<()> {
        match self.output.format() {
            OutputFormat::Human => {
                if let Some(next) = progress.next {
                    self.output.success(&format!(
                        "Captured {}/{} keys. Next: row {}, col {}",
                        progress.captured, progress.total, next.row, next.col
                    ));
                } else {
                    self.output.success(&format!(
                        "Captured {}/{} keys.",
                        progress.captured, progress.total
                    ));
                }
            }
            OutputFormat::Json => {
                let payload = ProgressJson {
                    status: "progress",
                    captured: progress.captured,
                    total: progress.total,
                    next: progress
                        .next
                        .as_ref()
                        .and_then(|pos| serde_json::to_value(pos).ok()),
                };
                self.output.data(&payload)?;
            }
        }
        Ok(())
    }

    fn report_duplicate(&self, dup: &crate::discovery::DuplicateWarning) -> Result<()> {
        match self.output.format() {
            OutputFormat::Human => self.output.warning(&format!(
                "Duplicate scan code {} (existing r{},c{} attempted r{},c{})",
                dup.scan_code,
                dup.existing.row,
                dup.existing.col,
                dup.attempted.row,
                dup.attempted.col
            )),
            OutputFormat::Json => {
                let payload = DuplicateJson {
                    status: "duplicate",
                    scan_code: dup.scan_code,
                    existing: serde_json::json!(dup.existing),
                    attempted: serde_json::json!(dup.attempted),
                };
                self.output.data(&payload)?;
            }
        }
        Ok(())
    }

    fn report_summary(&self, summary: &DiscoverySummary) -> Result<()> {
        match self.output.format() {
            OutputFormat::Human => {
                let headline = match summary.status {
                    SessionStatus::Completed => "Discovery completed",
                    SessionStatus::Cancelled => "Discovery cancelled",
                    SessionStatus::Bypassed => "Discovery bypassed (emergency exit)",
                    SessionStatus::InProgress => "Discovery incomplete",
                };
                self.output.success(headline);
                self.output.success(&format!(
                    "Captured {}/{} keys; duplicates: {}; unmapped: {}",
                    summary.captured,
                    summary.total,
                    summary.duplicates.len(),
                    summary.unmapped.len()
                ));
                if let Some(msg) = &summary.message {
                    self.output.warning(msg);
                }
            }
            OutputFormat::Json => {
                let payload = SummaryJson {
                    status: "summary",
                    summary,
                };
                self.output.data(&payload)?;
            }
        }
        Ok(())
    }
}

fn default_input_builder(device: &DeviceInfo) -> Result<DiscoverInput> {
    #[cfg(target_os = "linux")]
    {
        use crate::drivers::LinuxInput;
        let input = LinuxInput::new(Some(device.path.clone()))?;
        Ok(DiscoverInput {
            input: Box::new(input),
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        use crate::drivers::PlatformInput;
        let input = PlatformInput::new()?;
        Ok(DiscoverInput {
            input: Box::new(input),
        })
    }
}

fn parse_device_id(device: &str) -> Result<DeviceId, DiscoverExit> {
    let parts: Vec<_> = device.split(':').collect();
    if parts.len() != 2 {
        return Err(DiscoverExit::Validation(
            "Device must be in format vendor:product (hex or decimal)".to_string(),
        ));
    }

    let vendor_id = parse_u16(parts[0])?;
    let product_id = parse_u16(parts[1])?;
    Ok(DeviceId::new(vendor_id, product_id))
}

fn parse_u16(value: &str) -> Result<u16, DiscoverExit> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
            .map_err(|_| DiscoverExit::Validation(format!("Invalid hex value: {value}")))
    } else {
        value
            .parse::<u16>()
            .map_err(|_| DiscoverExit::Validation(format!("Invalid number: {value}")))
    }
}

fn prompt_for_layout() -> Result<(u8, Vec<u8>), DiscoverExit> {
    let (default_rows, default_cols) = default_layout();
    println!("Enter number of rows for this keyboard [default {default_rows}]: ");
    print_flush();

    let mut rows_input = String::new();
    io::stdin()
        .read_line(&mut rows_input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;
    let rows = if rows_input.trim().is_empty() {
        default_rows
    } else {
        rows_input
            .trim()
            .parse::<u8>()
            .map_err(|_| DiscoverExit::Validation("Rows must be a number".to_string()))?
    };

    println!(
        "Enter columns per row separated by commas [default {}]: ",
        default_cols
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    print_flush();

    let mut cols_input = String::new();
    io::stdin()
        .read_line(&mut cols_input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;

    let cols_per_row = if cols_input.trim().is_empty() {
        default_cols
    } else {
        cols_input
            .trim()
            .split(',')
            .map(|v| {
                v.trim()
                    .parse::<u8>()
                    .map_err(|_| DiscoverExit::Validation("Columns must be numbers".to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    if cols_per_row.len() != rows as usize || rows == 0 || cols_per_row.contains(&0) {
        return Err(DiscoverExit::Validation(
            "Rows must match columns length and all rows must be non-zero".to_string(),
        ));
    }

    Ok((rows, cols_per_row))
}

fn confirm(prompt: &str) -> Result<bool, DiscoverExit> {
    print!("{prompt}");
    print_flush();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;
    let accepted = matches!(input.to_lowercase().trim(), "y" | "yes");
    Ok(accepted)
}

fn print_flush() {
    let _ = io::stdout().flush();
}

fn default_layout() -> (u8, Vec<u8>) {
    (5, vec![14, 14, 13, 13, 7])
}

#[derive(Default)]
struct EmergencyTracker {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl EmergencyTracker {
    fn new() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    fn update(&mut self, event: &crate::engine::InputEvent) -> bool {
        use crate::engine::KeyCode;

        match event.key {
            KeyCode::LeftCtrl | KeyCode::RightCtrl => self.ctrl = event.pressed,
            KeyCode::LeftAlt | KeyCode::RightAlt => self.alt = event.pressed,
            KeyCode::LeftShift | KeyCode::RightShift => self.shift = event.pressed,
            KeyCode::Escape => {
                if event.pressed && self.ctrl && self.alt && self.shift {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[derive(Serialize)]
struct ExistingProfileJson<'a> {
    status: &'a str,
    path: String,
    vendor_id: u16,
    product_id: u16,
}

#[derive(Serialize)]
struct ProgressJson<'a> {
    status: &'a str,
    captured: usize,
    total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct DuplicateJson<'a> {
    status: &'a str,
    scan_code: u16,
    existing: serde_json::Value,
    attempted: serde_json::Value,
}

#[derive(Serialize)]
struct SummaryJson<'a> {
    status: &'a str,
    summary: &'a DiscoverySummary,
}

#[derive(Serialize)]
struct DiscoverResultJson<'a> {
    status: &'a str,
    path: String,
    cols_per_row: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::device_profiles_dir;
    use crate::discovery::storage::{default_profile_for, write_profile};
    use crate::discovery::test_utils::config_env_lock;
    use crate::engine::{InputEvent, KeyCode};
    use crate::mocks::MockInput;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn mock_device() -> DeviceInfo {
        DeviceInfo::new(
            PathBuf::from("/dev/input/mock0"),
            "Mock Keyboard".to_string(),
            0x1,
            0x2,
            true,
        )
    }

    #[test]
    fn parse_device_id_hex_and_dec() {
        let hex = parse_device_id("0x1234:0xabcd").unwrap();
        assert_eq!(hex.vendor_id, 0x1234);
        assert_eq!(hex.product_id, 0xABCD);

        let dec = parse_device_id("4660:43981").unwrap();
        assert_eq!(dec.vendor_id, 0x1234);
        assert_eq!(dec.product_id, 0xABCD);

        let err = parse_device_id("bad").unwrap_err();
        assert!(matches!(err, DiscoverExit::Validation(_)));
    }

    #[tokio::test]
    async fn skips_when_profile_ready_and_not_forced() {
        let _guard = config_env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        std::env::remove_var("HOME");

        let device = mock_device();
        let id = DeviceId::new(device.vendor_id, device.product_id);
        let mut profile = default_profile_for(id);
        profile.rows = 1;
        profile.cols_per_row = vec![1];
        write_profile(&profile).unwrap();

        let cmd = DiscoverCommand::with_providers(
            None,
            false,
            true,
            OutputFormat::Human,
            move || Ok(vec![device.clone()]),
            |_dev| panic!("input should not be built when skipping"),
        );

        let result = cmd.run().await;
        assert!(result.is_ok());

        if let Some(xdg) = prev_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(home) = prev_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[tokio::test]
    async fn discovers_and_writes_profile() {
        let _guard = config_env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        std::env::remove_var("HOME");

        let device = mock_device();
        let id = DeviceId::new(device.vendor_id, device.product_id);
        let mut profile = default_profile_for(id);
        profile.rows = 1;
        profile.cols_per_row = vec![2];
        write_profile(&profile).unwrap();

        let input_builder = |dev: &DeviceInfo| -> Result<DiscoverInput> {
            let mut mock = MockInput::new();
            mock.queue_event(InputEvent::with_metadata(
                KeyCode::A,
                true,
                1,
                dev.path.to_str().map(String::from),
                false,
                false,
                30,
            ));
            mock.queue_event(InputEvent::with_metadata(
                KeyCode::B,
                true,
                2,
                dev.path.to_str().map(String::from),
                false,
                false,
                31,
            ));
            Ok(DiscoverInput {
                input: Box::new(mock),
            })
        };

        let cmd = DiscoverCommand::with_providers(
            None,
            true,
            true,
            OutputFormat::Human,
            move || Ok(vec![device.clone()]),
            input_builder,
        );

        cmd.run().await.unwrap();

        let path = device_profiles_dir().join(id.to_filename());
        assert!(path.exists(), "profile should be written");
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(contents.contains("\"vendor_id\": 1"));
        assert!(contents.contains("\"product_id\": 2"));

        if let Some(xdg) = prev_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(home) = prev_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
