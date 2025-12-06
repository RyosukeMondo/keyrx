//! Device management commands backed by persisted bindings.
//!
//! This command surfaces per-device operations (list, show, remap, label,
//! assign/unassign) keyed by full VID:PID:Serial identities and persists
//! changes to `device_bindings.json`.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::identity::DeviceIdentity;
use crate::registry::{DeviceBinding, DeviceBindings};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Actions supported by the devices command.
#[derive(Debug, Clone)]
pub enum DeviceAction {
    List,
    Show {
        device_key: String,
    },
    Remap {
        device_key: String,
        enabled: bool,
    },
    Assign {
        device_key: String,
        profile_id: String,
    },
    Unassign {
        device_key: String,
    },
    Label {
        device_key: String,
        label: Option<String>,
    },
}

/// Serializable view of a device binding for human/JSON output.
#[derive(Debug, Clone, Serialize)]
struct DeviceBindingView {
    device_key: String,
    vendor_id: u16,
    product_id: u16,
    serial_number: String,
    user_label: Option<String>,
    remap_enabled: bool,
    profile_id: Option<String>,
    bound_at: Option<String>,
    persisted: bool,
    bindings_path: String,
}

impl DeviceBindingView {
    fn from_binding(
        identity: DeviceIdentity,
        binding: DeviceBinding,
        persisted: bool,
        bindings_path: &Path,
    ) -> Self {
        Self {
            device_key: identity.to_key(),
            vendor_id: identity.vendor_id,
            product_id: identity.product_id,
            serial_number: identity.serial_number,
            user_label: binding.user_label,
            remap_enabled: binding.remap_enabled,
            profile_id: binding.profile_id,
            bound_at: binding.bound_at,
            persisted,
            bindings_path: bindings_path.display().to_string(),
        }
    }

    fn empty(identity: DeviceIdentity, bindings_path: &Path) -> Self {
        let binding = DeviceBinding::new();
        Self::from_binding(identity, binding, false, bindings_path)
    }
}

/// Device command implementation.
pub struct DevicesCommand {
    pub output: OutputWriter,
    action: DeviceAction,
    bindings_path: PathBuf,
}

impl DevicesCommand {
    pub fn new(format: OutputFormat, action: DeviceAction) -> Self {
        Self {
            output: OutputWriter::new(format),
            action,
            bindings_path: DeviceBindings::default_path(),
        }
    }

    /// Override bindings path (used by tests).
    pub fn with_bindings_path(mut self, path: PathBuf) -> Self {
        self.bindings_path = path;
        self
    }

    pub fn run(&self) -> CommandResult<()> {
        match &self.action {
            DeviceAction::List => self.list_devices(),
            DeviceAction::Show { device_key } => self.show_device(device_key),
            DeviceAction::Remap {
                device_key,
                enabled,
            } => self.update_binding(device_key, |binding| binding.remap_enabled = *enabled),
            DeviceAction::Assign {
                device_key,
                profile_id,
            } => self.update_binding(device_key, |binding| {
                binding.profile_id = Some(profile_id.clone())
            }),
            DeviceAction::Unassign { device_key } => {
                self.update_binding(device_key, |binding| binding.profile_id = None)
            }
            DeviceAction::Label { device_key, label } => {
                let label_value = label.clone();
                self.update_binding(device_key, |binding| {
                    binding.user_label = label_value.clone();
                })
            }
        }
    }

    fn list_devices(&self) -> CommandResult<()> {
        let bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let mut views: Vec<_> = bindings
            .all_bindings()
            .iter()
            .map(|(id, binding)| {
                DeviceBindingView::from_binding(id.clone(), binding.clone(), true, bindings.path())
            })
            .collect();

        views.sort_by(|a, b| a.device_key.cmp(&b.device_key));

        self.render_devices(&views)
    }

    fn show_device(&self, device_key: &str) -> CommandResult<()> {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let view = bindings
            .get_binding(&identity)
            .cloned()
            .map(|binding| {
                DeviceBindingView::from_binding(identity.clone(), binding, true, bindings.path())
            })
            .unwrap_or_else(|| DeviceBindingView::empty(identity.clone(), bindings.path()));

        self.render_device(&view)
    }

    fn update_binding<F>(&self, device_key: &str, updater: F) -> CommandResult<()>
    where
        F: FnOnce(&mut DeviceBinding),
    {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let mut bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);

        updater(&mut binding);
        bindings.set_binding(identity.clone(), binding.clone());

        if let Err(e) = bindings.save() {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to save device bindings: {}", e),
            );
        }

        let view = DeviceBindingView::from_binding(identity, binding, true, bindings.path());
        self.render_device(&view)
    }

    fn render_devices(&self, devices: &[DeviceBindingView]) -> CommandResult<()> {
        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml => {
                if let Err(e) = self.output.data(devices) {
                    return CommandResult::failure(ExitCode::GeneralError, e.to_string());
                }
                CommandResult::success(())
            }
            _ => {
                if devices.is_empty() {
                    println!(
                        "No device bindings found at {}",
                        self.bindings_path.display()
                    );
                    println!("Run `keyrx run` with devices attached to hydrate bindings or use `keyrx devices assign` to create one.");
                    return CommandResult::success(());
                }

                println!("Device bindings ({}):", devices.len());
                for device in devices {
                    println!("  {}", device.device_key);
                    if let Some(label) = &device.user_label {
                        println!("    Label: {}", label);
                    }
                    println!(
                        "    Remap: {}",
                        if device.remap_enabled { "on" } else { "off" }
                    );
                    println!(
                        "    Profile: {}",
                        device.profile_id.as_deref().unwrap_or("none")
                    );
                    if let Some(bound_at) = &device.bound_at {
                        println!("    Updated: {}", bound_at);
                    }
                    println!(
                        "    Persisted: {} ({})",
                        if device.persisted { "yes" } else { "no" },
                        device.bindings_path
                    );
                }
                CommandResult::success(())
            }
        }
    }

    fn render_device(&self, device: &DeviceBindingView) -> CommandResult<()> {
        self.render_devices(std::slice::from_ref(device))
    }

    fn parse_identity(&self, device_key: &str) -> Result<DeviceIdentity, CommandResult<()>> {
        DeviceIdentity::from_key(device_key)
            .map_err(|msg| CommandResult::failure(ExitCode::ValidationFailed, msg))
    }

    fn load_bindings(&self) -> Result<DeviceBindings, CommandResult<()>> {
        let mut bindings = DeviceBindings::with_path(self.bindings_path.clone());
        if let Err(e) = bindings.load() {
            return Err(CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to load device bindings: {}", e),
            ));
        }
        Ok(bindings)
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
    use crate::cli::OutputFormat;
    use crate::identity::DeviceIdentity;
    use crate::registry::DeviceBindings;
    use tempfile::tempdir;

    #[test]
    fn list_empty_bindings_succeeds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bindings.json");
        let cmd =
            DevicesCommand::new(OutputFormat::Human, DeviceAction::List).with_bindings_path(path);

        let result = cmd.run();
        assert!(result.is_success());
    }

    #[test]
    fn assign_creates_binding() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bindings.json");
        let device_key = "046d:c52b:ABC123";

        let cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Assign {
                device_key: device_key.to_string(),
                profile_id: "profile-1".to_string(),
            },
        )
        .with_bindings_path(path.clone());

        assert!(cmd.run().is_success());

        let mut bindings = DeviceBindings::with_path(path.clone());
        bindings.load().unwrap();
        let identity = DeviceIdentity::from_key(device_key).unwrap();
        let binding = bindings.get_binding(&identity).unwrap();
        assert_eq!(binding.profile_id.as_deref(), Some("profile-1"));
        assert!(binding.remap_enabled);
    }

    #[test]
    fn label_and_remap_update_binding() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bindings.json");
        let device_key = "0fd9:0080:CL12345678";

        let label_cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Label {
                device_key: device_key.to_string(),
                label: Some("My Deck".to_string()),
            },
        )
        .with_bindings_path(path.clone());
        assert!(label_cmd.run().is_success());

        let remap_cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Remap {
                device_key: device_key.to_string(),
                enabled: false,
            },
        )
        .with_bindings_path(path.clone());
        assert!(remap_cmd.run().is_success());

        let mut bindings = DeviceBindings::with_path(path.clone());
        bindings.load().unwrap();
        let identity = DeviceIdentity::from_key(device_key).unwrap();
        let binding = bindings.get_binding(&identity).unwrap();
        assert_eq!(binding.user_label.as_deref(), Some("My Deck"));
        assert!(!binding.remap_enabled);

        let clear_label_cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Label {
                device_key: device_key.to_string(),
                label: None,
            },
        )
        .with_bindings_path(path.clone());
        assert!(clear_label_cmd.run().is_success());

        let mut bindings = DeviceBindings::with_path(path);
        bindings.load().unwrap();
        let binding = bindings.get_binding(&identity).unwrap();
        assert!(binding.user_label.is_none());
    }

    #[test]
    fn show_missing_binding_does_not_create_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bindings.json");
        let device_key = "1234:5678:SERIAL";

        let cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Show {
                device_key: device_key.to_string(),
            },
        )
        .with_bindings_path(path.clone());
        assert!(cmd.run().is_success());

        assert!(!path.exists());
    }
}
