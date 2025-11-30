//! Device listing command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::drivers;
use anyhow::Result;

/// List all available keyboard devices.
pub struct DevicesCommand {
    output: OutputWriter,
}

impl DevicesCommand {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        let devices = drivers::list_keyboards()?;

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
                for device in &devices {
                    println!(
                        "  {} ({:04x}:{:04x})",
                        device.name, device.vendor_id, device.product_id
                    );
                    println!("    Path: {}", device.path.display());
                    println!();
                }
            }
            OutputFormat::Json => {
                self.output.data(&devices)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
