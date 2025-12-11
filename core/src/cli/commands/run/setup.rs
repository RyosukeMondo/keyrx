//! Setup and initialization logic for the run command.
//!
//! This module handles:
//! - Profile registry initialization
//! - Device definition loading
//! - Revolutionary runtime installation
//! - Device path configuration

use super::RunCommand;
use crate::definitions::DeviceDefinitionLibrary;
use crate::engine::InputEvent;
use crate::ffi::runtime::{RevolutionaryRuntime, RevolutionaryRuntimeGuard};
use crate::identity::DeviceIdentity;
use crate::registry::{DeviceRegistry, ProfileRegistry, ProfileRegistryStorage};
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

impl RunCommand {
    /// Prepare and load the profile registry.
    pub(crate) async fn prepare_profile_registry(&self) -> Arc<ProfileRegistry> {
        let registry = Arc::new(ProfileRegistry::new());
        match registry.load_all_profiles().await {
            Ok(count) => {
                info!(
                    service = "keyrx",
                    component = "cli_run",
                    event = "profiles_loaded",
                    count,
                    "Loaded profiles for revolutionary runtime"
                );
            }
            Err(err) => {
                self.output.warning(&format!(
                    "Failed to load profiles, continuing with empty registry: {err}"
                ));
                warn!(
                    service = "keyrx",
                    component = "cli_run",
                    event = "profiles_load_failed",
                    error = %err,
                    "Profile registry failed to load"
                );
            }
        }
        registry
    }

    /// Get search paths for device definitions.
    pub(crate) fn device_definition_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Ok(exe_path) = env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                paths.push(dir.join("device_definitions"));
            }
        }

        if let Some(root) = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent() {
            paths.push(root.join("device_definitions"));
        }

        paths.push(crate::config::config_dir().join("device_definitions"));

        let mut seen = std::collections::HashSet::new();
        paths
            .into_iter()
            .filter(|path| seen.insert(path.clone()))
            .filter(|path| path.exists())
            .collect()
    }

    /// Load device definitions from all available paths.
    pub(crate) fn load_device_definitions(&self) -> Arc<DeviceDefinitionLibrary> {
        let mut library = DeviceDefinitionLibrary::new();
        let mut total_loaded = 0usize;

        for path in self.device_definition_paths() {
            match library.load_from_directory(&path) {
                Ok(count) => {
                    total_loaded += count;
                    info!(
                        service = "keyrx",
                        component = "cli_run",
                        event = "device_definitions_loaded",
                        path = %path.display(),
                        count,
                        "Loaded device definitions"
                    );
                }
                Err(err) => {
                    self.output.warning(&format!(
                        "Failed to load device definitions from {}: {err}",
                        path.display()
                    ));
                    warn!(
                        service = "keyrx",
                        component = "cli_run",
                        event = "device_definitions_load_failed",
                        path = %path.display(),
                        error = %err,
                        "Unable to load device definitions"
                    );
                }
            }
        }

        if total_loaded == 0 {
            self.output.warning(
                "No device definitions loaded; definition FFI calls will return NOT_FOUND until definitions are available.",
            );
        }

        Arc::new(library)
    }

    /// Install the revolutionary FFI runtime for cross-language bindings.
    pub(crate) fn install_revolutionary_runtime(
        &self,
        device_registry: &DeviceRegistry,
        profile_registry: &Arc<ProfileRegistry>,
        device_definitions: &Arc<DeviceDefinitionLibrary>,
    ) -> Result<RevolutionaryRuntimeGuard> {
        let rhai_runtime = crate::scripting::RhaiRuntime::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize scripting runtime: {}", e))?;

        let shared_runtime = Arc::new(Mutex::new(rhai_runtime));

        RevolutionaryRuntimeGuard::install(RevolutionaryRuntime::new(
            device_registry.clone(),
            profile_registry.clone(),
            device_definitions.clone(),
            shared_runtime,
        ))
        .map_err(|err| anyhow::anyhow!("failed to install revolutionary runtime: {err}"))
    }
}

/// Extract device identity from an input event if available.
pub fn identity_from_event(event: &InputEvent) -> Option<DeviceIdentity> {
    let vendor_id = event.vendor_id?;
    let product_id = event.product_id?;
    let serial_number = event.serial_number.as_ref()?;

    Some(DeviceIdentity::new(
        vendor_id,
        product_id,
        serial_number.clone(),
    ))
}
