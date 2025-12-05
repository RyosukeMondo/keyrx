//! Device listing command.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::drivers;

/// List all available keyboard devices.
pub struct DevicesCommand {
    pub output: OutputWriter,
    list_devices: fn() -> anyhow::Result<Vec<drivers::DeviceInfo>>,
}

impl DevicesCommand {
    pub fn new(format: OutputFormat) -> Self {
        fn adapter() -> anyhow::Result<Vec<drivers::DeviceInfo>> {
            drivers::list_keyboards().map_err(|e| anyhow::anyhow!("{}", e))
        }
        Self::with_provider(format, adapter)
    }

    pub fn with_provider(
        format: OutputFormat,
        list_devices: fn() -> anyhow::Result<Vec<drivers::DeviceInfo>>,
    ) -> Self {
        Self {
            output: OutputWriter::new(format),
            list_devices,
        }
    }

    pub fn run(&self) -> CommandResult<()> {
        let devices = match (self.list_devices)() {
            Ok(d) => d,
            Err(e) => {
                return CommandResult::failure(
                    ExitCode::DeviceNotFound,
                    format!("Failed to list devices: {}", e),
                )
            }
        };
        self.render_devices(&devices)
    }

    fn render_devices(&self, devices: &[drivers::DeviceInfo]) -> CommandResult<()> {
        if devices.is_empty() {
            match self.output.format() {
                OutputFormat::Json | OutputFormat::Yaml => {
                    println!("[]");
                }
                _ => {
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
            }
            return CommandResult::success(());
        }

        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml => {
                if let Err(e) = self.output.data(devices) {
                    return CommandResult::failure(
                        ExitCode::GeneralError,
                        format!("Failed to output device list: {}", e),
                    );
                }
            }
            _ => {
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
        }

        CommandResult::success(())
    }
}

impl Command for DevicesCommand {
    fn name(&self) -> &str {
        "devices"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
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
        let result = cmd.run();
        assert!(result.is_success());
    }

    #[test]
    fn devices_command_renders_empty_json() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(vec![]));
        let result = cmd.run();
        assert!(result.is_success());
    }

    #[test]
    fn devices_command_renders_devices_human() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Ok(sample_devices()));
        let result = cmd.run();
        assert!(result.is_success());
    }

    #[test]
    fn devices_command_renders_devices_json() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Json, || Ok(sample_devices()));
        let result = cmd.run();
        assert!(result.is_success());
    }

    #[test]
    fn devices_command_propagates_errors_from_provider() {
        let cmd = DevicesCommand::with_provider(OutputFormat::Human, || Err(anyhow!("boom")));
        let result = cmd.run();
        assert!(result.is_failure());
        assert!(result.messages().iter().any(|msg| msg.contains("boom")));
    }
}
