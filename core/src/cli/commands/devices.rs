//! Device management commands backed by persisted bindings and the live registry.
//!
//! This command surfaces per-device operations (list, show, remap, label,
//! assign/unassign) keyed by full VID:PID:Serial identities and persists
//! changes to `device_bindings.json`. When the revolutionary runtime is
//! active, operations are routed to the live `DeviceRegistry` and then
//! persisted so CLI output reflects actual runtime state.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::ffi::runtime::{with_revolutionary_runtime, RevolutionaryRuntime};
use crate::identity::DeviceIdentity;
use crate::registry::{
    DeviceBinding, DeviceBindings, DeviceRegistry, DeviceRegistryError, DeviceState,
};
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
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
    connected: bool,
}

impl DeviceBindingView {
    fn from_binding(
        identity: DeviceIdentity,
        binding: DeviceBinding,
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
            persisted: true,
            bindings_path: bindings_path.display().to_string(),
            connected: false,
        }
    }

    fn from_state(
        state: &DeviceState,
        binding: Option<DeviceBinding>,
        bindings_path: &Path,
    ) -> Self {
        let persisted = binding.is_some();
        let (bound_at, fallback_label) = binding
            .map(|b| (b.bound_at, b.user_label))
            .unwrap_or((None, None));

        Self {
            device_key: state.identity.to_key(),
            vendor_id: state.identity.vendor_id,
            product_id: state.identity.product_id,
            serial_number: state.identity.serial_number.clone(),
            user_label: state.identity.user_label.clone().or(fallback_label),
            remap_enabled: state.remap_enabled,
            profile_id: state.profile_id.clone(),
            bound_at,
            persisted,
            bindings_path: bindings_path.display().to_string(),
            connected: true,
        }
    }

    fn empty(identity: DeviceIdentity, bindings_path: &Path) -> Self {
        let binding = DeviceBinding::new();
        Self {
            device_key: identity.to_key(),
            vendor_id: identity.vendor_id,
            product_id: identity.product_id,
            serial_number: identity.serial_number,
            user_label: binding.user_label,
            remap_enabled: binding.remap_enabled,
            profile_id: binding.profile_id,
            bound_at: binding.bound_at,
            persisted: false,
            bindings_path: bindings_path.display().to_string(),
            connected: false,
        }
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
            } => self.remap_device(device_key, *enabled),
            DeviceAction::Assign {
                device_key,
                profile_id,
            } => self.assign_profile(device_key, profile_id),
            DeviceAction::Unassign { device_key } => self.unassign_profile(device_key),
            DeviceAction::Label { device_key, label } => self.label_device(device_key, label),
        }
    }

    fn list_devices(&self) -> CommandResult<()> {
        let bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        if let Some(runtime) = self.live_runtime() {
            return self.list_with_runtime(runtime, bindings);
        }

        self.list_from_bindings(bindings)
    }

    fn list_with_runtime(
        &self,
        runtime: RevolutionaryRuntime,
        bindings: DeviceBindings,
    ) -> CommandResult<()> {
        let bindings_path = bindings.path().to_path_buf();
        let mut remaining: HashMap<_, _> = bindings.all_bindings().clone();

        let states = match self.block_on_future({
            let registry = runtime.device_registry().clone();
            async move { registry.list_devices().await }
        }) {
            Ok(states) => states,
            Err(err) => return err,
        };

        let mut views: Vec<_> = states
            .into_iter()
            .map(|state| {
                let binding = remaining.remove(&state.identity);
                DeviceBindingView::from_state(&state, binding, &bindings_path)
            })
            .collect();

        for (identity, binding) in remaining {
            views.push(DeviceBindingView::from_binding(
                identity,
                binding,
                &bindings_path,
            ));
        }

        views.sort_by(|a, b| a.device_key.cmp(&b.device_key));
        self.render_devices(&views)
    }

    fn list_from_bindings(&self, bindings: DeviceBindings) -> CommandResult<()> {
        let bindings_path = bindings.path().to_path_buf();
        let mut views: Vec<_> = bindings
            .all_bindings()
            .iter()
            .map(|(id, binding)| {
                DeviceBindingView::from_binding(id.clone(), binding.clone(), &bindings_path)
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

        if let Some(runtime) = self.live_runtime() {
            return self.show_with_runtime(runtime, bindings, identity);
        }

        let view = bindings
            .get_binding(&identity)
            .cloned()
            .map(|binding| {
                DeviceBindingView::from_binding(identity.clone(), binding, bindings.path())
            })
            .unwrap_or_else(|| DeviceBindingView::empty(identity.clone(), bindings.path()));

        self.render_device(&view)
    }

    fn show_with_runtime(
        &self,
        runtime: RevolutionaryRuntime,
        bindings: DeviceBindings,
        identity: DeviceIdentity,
    ) -> CommandResult<()> {
        let bindings_path = bindings.path().to_path_buf();
        let binding = bindings.get_binding(&identity).cloned();
        let state = match self.block_on_future({
            let registry = runtime.device_registry().clone();
            let id = identity.clone();
            async move { registry.get_device_state(&id).await }
        }) {
            Ok(state) => state,
            Err(err) => return err,
        };

        let view = if let Some(state) = state {
            DeviceBindingView::from_state(&state, binding, &bindings_path)
        } else if let Some(binding) = binding {
            self.output.warning(&format!(
                "Device {} is not connected; showing persisted binding.",
                identity
            ));
            DeviceBindingView::from_binding(identity, binding, &bindings_path)
        } else {
            self.output.warning(&format!(
                "Device {} is not connected and has no persisted binding.",
                identity
            ));
            DeviceBindingView::empty(identity, &bindings_path)
        };

        self.render_device(&view)
    }

    fn remap_device(&self, device_key: &str, enabled: bool) -> CommandResult<()> {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let mut bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let runtime = self.live_runtime();
        let live_state = if let Some(runtime) = runtime.as_ref() {
            match self.apply_live_update(runtime, &identity, move |id, registry| async move {
                registry.set_remap_enabled(&id, enabled).await
            }) {
                Ok(state) => state,
                Err(err) => return err,
            }
        } else {
            None
        };

        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.remap_enabled = enabled;
        bindings.set_binding(identity.clone(), binding.clone());

        if let Err(e) = bindings.save() {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to save device bindings: {}", e),
            );
        }

        let view = if let Some(state) = live_state {
            DeviceBindingView::from_state(&state, Some(binding), bindings.path())
        } else {
            DeviceBindingView::from_binding(identity, binding, bindings.path())
        };

        self.render_device(&view)
    }

    fn assign_profile(&self, device_key: &str, profile_id: &str) -> CommandResult<()> {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let mut bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let runtime = self.live_runtime();
        let live_state = if let Some(runtime) = runtime.as_ref() {
            match self.apply_live_update(runtime, &identity, {
                let profile_id = profile_id.to_string();
                move |id, registry| async move {
                    registry.assign_profile(&id, profile_id.clone()).await
                }
            }) {
                Ok(state) => state,
                Err(err) => return err,
            }
        } else {
            None
        };

        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.profile_id = Some(profile_id.to_string());
        bindings.set_binding(identity.clone(), binding.clone());

        if let Err(e) = bindings.save() {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to save device bindings: {}", e),
            );
        }

        let view = if let Some(state) = live_state {
            DeviceBindingView::from_state(&state, Some(binding), bindings.path())
        } else {
            DeviceBindingView::from_binding(identity, binding, bindings.path())
        };

        self.render_device(&view)
    }

    fn unassign_profile(&self, device_key: &str) -> CommandResult<()> {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let mut bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let runtime = self.live_runtime();
        let live_state = if let Some(runtime) = runtime.as_ref() {
            match self.apply_live_update(runtime, &identity, move |id, registry| async move {
                registry.unassign_profile(&id).await
            }) {
                Ok(state) => state,
                Err(err) => return err,
            }
        } else {
            None
        };

        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.profile_id = None;
        bindings.set_binding(identity.clone(), binding.clone());

        if let Err(e) = bindings.save() {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to save device bindings: {}", e),
            );
        }

        let view = if let Some(state) = live_state {
            DeviceBindingView::from_state(&state, Some(binding), bindings.path())
        } else {
            DeviceBindingView::from_binding(identity, binding, bindings.path())
        };

        self.render_device(&view)
    }

    fn label_device(&self, device_key: &str, label: &Option<String>) -> CommandResult<()> {
        let identity = match self.parse_identity(device_key) {
            Ok(id) => id,
            Err(err) => return err,
        };

        let mut bindings = match self.load_bindings() {
            Ok(b) => b,
            Err(err) => return err,
        };

        let label_value = label.clone();
        let runtime = self.live_runtime();
        let live_state = if let Some(runtime) = runtime.as_ref() {
            match self.apply_live_update(runtime, &identity, move |id, registry| async move {
                registry.set_user_label(&id, label_value.clone()).await
            }) {
                Ok(state) => state,
                Err(err) => return err,
            }
        } else {
            None
        };

        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.user_label = label.clone();
        bindings.set_binding(identity.clone(), binding.clone());

        if let Err(e) = bindings.save() {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to save device bindings: {}", e),
            );
        }

        let view = if let Some(state) = live_state {
            DeviceBindingView::from_state(&state, Some(binding), bindings.path())
        } else {
            DeviceBindingView::from_binding(identity, binding, bindings.path())
        };

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
                        "No devices or bindings found ({}).",
                        self.bindings_path.display()
                    );
                    println!("Run `keyrx run` with devices attached to hydrate bindings or use `keyrx devices assign` to create one.");
                    return CommandResult::success(());
                }

                println!("Devices ({}):", devices.len());
                for device in devices {
                    println!("  {}", device.device_key);
                    println!(
                        "    Connected: {}",
                        if device.connected { "yes" } else { "no" }
                    );
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

    fn apply_live_update<F, Fut>(
        &self,
        runtime: &RevolutionaryRuntime,
        identity: &DeviceIdentity,
        op: F,
    ) -> Result<Option<DeviceState>, CommandResult<()>>
    where
        F: FnOnce(DeviceIdentity, DeviceRegistry) -> Fut,
        Fut: Future<Output = Result<(), DeviceRegistryError>> + Send + 'static,
    {
        let registry = runtime.device_registry().clone();
        let id_for_op = identity.clone();

        match self.block_on_future(op(id_for_op.clone(), registry.clone()))? {
            Ok(()) => {
                self.block_on_future(async move { registry.get_device_state(&id_for_op).await })
            }
            Err(DeviceRegistryError::DeviceNotFound(_)) => {
                self.output.warning(&format!(
                    "Device {} is not connected; updated persisted binding only.",
                    identity
                ));
                Ok(None)
            }
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
    use std::sync::Arc;
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
        let _guard = RevolutionaryRuntimeGuard::install(RevolutionaryRuntime::new(
            registry.clone(),
            profile_registry.clone(),
            device_definitions.clone(),
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
