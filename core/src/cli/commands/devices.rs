//! Device listing command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::drivers;
use anyhow::Result;

/// List all available keyboard devices.
pub struct DevicesCommand {
    pub output: OutputWriter,
    list_devices: fn() -> Result<Vec<drivers::DeviceInfo>>,
}

impl DevicesCommand {
    pub fn new(format: OutputFormat) -> Self {
        Self::with_provider(format, drivers::list_keyboards)
    }

    pub fn with_provider(
        format: OutputFormat,
        list_devices: fn() -> Result<Vec<drivers::DeviceInfo>>,
    ) -> Self {
        Self {
            output: OutputWriter::new(format),
            list_devices,
        }
    }

    pub fn run(&self) -> Result<()> {
        let devices = (self.list_devices)()?;
        self.render_devices(&devices)
    }

    fn render_devices(&self, devices: &[drivers::DeviceInfo]) -> Result<()> {
        if devices.is_empty() {
            match self.output.format() {
                OutputFormat::Human => {
                    println!("No keyboard devices found.");
                    println!();
                    #[cfg(target_os = "linux")]
                    {
                        println!("Troubleshooting:");
                        println!(
                            "  - Ensure you're in the 'input' group: sudo usermod -aG input $USER"
                        );
                        println!("  - Check device permissions: ls -la /dev/input/event*");
                        println!("  - Verify udev rules are installed");
                    }
                    #[cfg(target_os = "windows")]
                    {
                        println!("Troubleshooting:");
                        println!("  - Ensure a keyboard is connected");
                        println!("  - Try running as Administrator");
                    }
                }
                OutputFormat::Json => {
                    println!("[]");
                }
            }
            return Ok(());
        }

        match self.output.format() {
            OutputFormat::Human => {
                println!("Found {} keyboard device(s):\n", devices.len());
                for device in devices {
                    println!(
                        "  {} ({:04x}:{:04x})",
                        device.name, device.vendor_id, device.product_id
                    );
                    println!("    Path: {}", device.path.display());
                    println!();
                }
            }
            OutputFormat::Json => {
                self.output.data(devices)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::DeviceInfo;
    use anyhow::anyhow;
    use std::path::PathBuf;

    #[test]
    fn devices_command_creates_with_human_format() {
        let cmd = DevicesCommand::new(OutputFormat::Human);
        assert!(matches!(cmd.output.format(), OutputFormat::Human));
    }

    #[test]
    fn devices_command_creates_with_json_format() {
        let cmd = DevicesCommand::new(OutputFormat::Json);
        assert!(matches!(cmd.output.format(), OutputFormat::Json));
    }

    fn sample_devices() -> Vec<DeviceInfo> {
        vec![
            DeviceInfo::new(
                PathBuf::from("/dev/input/event0"),
                "Keyboard A".to_string(),
                0x1234,
                0x5678,
                true,
            ),
            DeviceInfo::new(
                PathBuf::from("/dev/input/event1"),
                "Keyboard B".to_string(),
                0x1111,
                0x2222,
                true,
            ),
        ]
    }

    #[test]
    fn devices_command_renders_empty_human() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Ok(vec![]));
        cmd.run().unwrap();
    }

    #[test]
    fn devices_command_renders_empty_json() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(vec![]));
        cmd.run().unwrap();
    }

    #[test]
    fn devices_command_renders_devices_human() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Ok(sample_devices()));
        cmd.run().unwrap();
    }

    #[test]
    fn devices_command_renders_devices_json() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(sample_devices()));
        cmd.run().unwrap();
    }

    #[test]
    fn devices_command_propagates_errors_from_provider() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Err(anyhow!("boom")));
        let err = cmd.run().unwrap_err();
        assert!(err.to_string().contains("boom"));
    }
}
