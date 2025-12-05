//! Device discovery CLI command.
//!
//! Drives the discovery state machine for a target keyboard, captures progress,
//! and writes the resulting profile to the registry.

use crate::cli::{OutputFormat, OutputWriter};
use crate::discovery::{DeviceId, DeviceRegistry, DiscoverySession, RegistryEntry, RegistryStatus};
use crate::drivers;
use crate::drivers::DeviceInfo;
use crate::traits::InputSource;
use anyhow::{Context, Result};

use super::discover_session::{
    capture_session, create_emergency_detector, handle_summary, ExistingProfileJson,
};
use super::discover_validation::{default_layout, parse_device_id, prompt_for_layout};

// Re-export DiscoverExit from validation module for public API
pub use super::discover_validation::DiscoverExit;

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
        fn adapter() -> Result<Vec<DeviceInfo>> {
            drivers::list_keyboards().map_err(|e| anyhow::anyhow!("{}", e))
        }
        Self {
            device,
            force,
            assume_yes,
            output: OutputWriter::new(format),
            list_devices: Box::new(adapter),
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

        let (_tracker, detector) = create_emergency_detector();
        session = session.with_emergency_exit_detector(detector);

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

        let summary = capture_session(&mut discover_input.input, session, &self.output).await?;

        // Best-effort cleanup; prefer reporting discovery outcome over stop errors.
        if let Err(err) = discover_input.input.stop().await {
            self.output
                .warning(&format!("Failed to stop input capture cleanly: {err:#}"));
        }

        handle_summary(
            &mut registry,
            summary,
            &device,
            &self.output,
            self.assume_yes,
        )
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

        if self.assume_yes
            || matches!(
                self.output.format(),
                OutputFormat::Json | OutputFormat::Yaml
            )
        {
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
            OutputFormat::Json | OutputFormat::Yaml => {
                let payload = ExistingProfileJson {
                    status: "ready",
                    path: entry.path.display().to_string(),
                    vendor_id: entry.device_id.vendor_id,
                    product_id: entry.device_id.product_id,
                };
                self.output.data(&payload)?;
            }
            _ => self.output.success(&message),
        }
        Ok(())
    }
}

fn default_input_builder(_device: &DeviceInfo) -> Result<DiscoverInput> {
    #[cfg(all(target_os = "linux", feature = "linux-driver"))]
    {
        use crate::drivers::LinuxInput;
        let input = LinuxInput::new(Some(_device.path.clone()))?;
        Ok(DiscoverInput {
            input: Box::new(input),
        })
    }

    #[cfg(all(target_os = "windows", feature = "windows-driver"))]
    {
        use crate::drivers::{WindowsInput, WindowsRawInput};
        let path_str = _device.path.to_string_lossy();

        // If we are targeting a specific HID device, use Raw Input driver
        // which can distinguish between devices.
        if !path_str.contains("System#Keyboard") && !path_str.contains("Global Hook") {
            let input = WindowsRawInput::new(Some(path_str.to_string()))?;
            Ok(DiscoverInput {
                input: Box::new(input),
            })
        } else {
            // Fallback to Global Hook for system keyboard or if generic
            let input = WindowsInput::new()?;
            Ok(DiscoverInput {
                input: Box::new(input),
            })
        }
    }

    #[cfg(not(any(
        all(target_os = "linux", feature = "linux-driver"),
        all(target_os = "windows", feature = "windows-driver")
    )))]
    {
        anyhow::bail!(
            "Device discovery requires platform driver support. \
             Enable 'linux-driver' or 'windows-driver' feature."
        )
    }
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
            |_dev| unreachable!("input builder should not be called when profile exists"),
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
