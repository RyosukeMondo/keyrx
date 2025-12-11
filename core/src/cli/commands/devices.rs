//! Device management commands backed by `DeviceService`.
//!
//! This command surfaces per-device operations (list, show, remap, label,
//! assign/unassign) using the `DeviceService` which acts as the SSOT.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::ffi::runtime::{with_revolutionary_runtime, RevolutionaryRuntime};
use crate::registry::DeviceBindings;
use crate::services::device::DeviceView;
use crate::services::traits::DeviceServiceTrait;
use crate::services::{DeviceService, DeviceServiceError};
use std::future::Future;
use std::path::PathBuf;
use tokio::runtime::{Handle, Runtime};
use tokio::task;

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

struct DevicesRenderer {
    output: OutputWriter,
    bindings_path: PathBuf,
}

impl DevicesRenderer {
    fn render_devices(&self, devices: &[DeviceView]) -> CommandResult<()> {
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
                        "No devices or bindings found ({}).",
                        self.bindings_path.display()
                    );
                    println!("Run `keyrx run` with devices attached to hydrate bindings or use `keyrx devices assign` to create one.");
                    return CommandResult::success(());
                }

                println!("Devices ({}):", devices.len());
                for device in devices {
                    println!("  {}", device.key);
                    println!(
                        "    Connected: {}",
                        if device.connected { "yes" } else { "no" }
                    );
                    if let Some(label) = &device.label {
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
                }
                CommandResult::success(())
            }
        }
    }

    fn render_device(&self, device: &DeviceView) -> CommandResult<()> {
        self.render_devices(std::slice::from_ref(device))
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
        let registry = self.live_runtime().map(|rt| rt.device_registry().clone());
        let bindings = DeviceBindings::with_path(self.bindings_path.clone());
        let service = DeviceService::new(registry, bindings);

        let renderer = DevicesRenderer {
            output: self.output.clone(),
            bindings_path: self.bindings_path.clone(),
        };
        let action = self.action.clone();

        let result = self.block_on_future(async move {
            let res: Result<CommandResult<()>, DeviceServiceError> = match action {
                DeviceAction::List => {
                    let devices = service.list_devices().await?;
                    Ok(renderer.render_devices(&devices))
                }
                DeviceAction::Show { device_key } => {
                    let device = service.get_device(&device_key).await?;
                    Ok(renderer.render_device(&device))
                }
                DeviceAction::Remap {
                    device_key,
                    enabled,
                } => {
                    let device = service.set_remap_enabled(&device_key, enabled).await?;
                    Ok(renderer.render_device(&device))
                }
                DeviceAction::Assign {
                    device_key,
                    profile_id,
                } => {
                    let device = service.assign_profile(&device_key, &profile_id).await?;
                    Ok(renderer.render_device(&device))
                }
                DeviceAction::Unassign { device_key } => {
                    let device = service.unassign_profile(&device_key).await?;
                    Ok(renderer.render_device(&device))
                }
                DeviceAction::Label { device_key, label } => {
                    let device = service.set_label(&device_key, label).await?;
                    Ok(renderer.render_device(&device))
                }
            };
            res
        });

        match result {
            Ok(Ok(cmd_result)) => cmd_result,
            Ok(Err(e)) => CommandResult::failure(ExitCode::GeneralError, e.to_string()),
            Err(cmd_result) => cmd_result, // Runtime creation failure
        }
    }

    fn live_runtime(&self) -> Option<RevolutionaryRuntime> {
        with_revolutionary_runtime(|runtime| Ok(runtime.clone())).ok()
    }

    fn block_on_future<F, T>(&self, future: F) -> Result<T, CommandResult<()>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        if let Ok(handle) = Handle::try_current() {
            Ok(task::block_in_place(|| handle.block_on(future)))
        } else {
            let runtime = Runtime::new().map_err(|e| {
                CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Failed to create async runtime: {}", e),
                )
            })?;

            Ok(runtime.block_on(future))
        }
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
    use crate::definitions::DeviceDefinitionLibrary;
    use crate::ffi::runtime::RevolutionaryRuntimeGuard;
    use crate::identity::DeviceIdentity;
    use crate::registry::{DeviceBindings, DeviceRegistry, ProfileRegistry};
    use crate::scripting::RhaiRuntime;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

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

    #[test]
    fn remap_updates_live_registry_when_runtime_available() {
        let dir = tempdir().unwrap();
        let bindings_path = dir.path().join("bindings.json");
        let profiles_dir = dir.path().join("profiles");

        let rt = Runtime::new().unwrap();
        let (registry, _rx) = DeviceRegistry::new();
        let identity = DeviceIdentity::new(0x1234, 0x5678, "SERIAL123".to_string());

        rt.block_on(async { registry.register_device(identity.clone()).await });

        let profile_registry = Arc::new(ProfileRegistry::with_directory(profiles_dir));
        let device_definitions = Arc::new(DeviceDefinitionLibrary::new());
        let rhai_runtime = Arc::new(Mutex::new(RhaiRuntime::new().unwrap()));

        let _guard = RevolutionaryRuntimeGuard::install(RevolutionaryRuntime::new(
            registry.clone(),
            profile_registry.clone(),
            device_definitions.clone(),
            rhai_runtime,
        ))
        .expect("runtime install");

        let cmd = DevicesCommand::new(
            OutputFormat::Human,
            DeviceAction::Remap {
                device_key: identity.to_key(),
                enabled: true,
            },
        )
        .with_bindings_path(bindings_path.clone());

        assert!(cmd.run().is_success());

        let state = rt
            .block_on(async { registry.get_device_state(&identity).await })
            .unwrap();
        assert!(state.remap_enabled);

        let mut bindings = DeviceBindings::with_path(bindings_path);
        bindings.load().unwrap();
        let persisted = bindings.get_binding(&identity).unwrap();
        assert!(persisted.remap_enabled);
    }
}
