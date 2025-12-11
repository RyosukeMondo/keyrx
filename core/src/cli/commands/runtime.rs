//! Runtime profile slot management commands.
//!
//! Provides `keyrx runtime devices` to inspect runtime state and slot
//! subcommands to add, remove, and toggle slots persisted in `runtime.json`.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::models::{
    DeviceInstanceId, DeviceSlots, HardwareProfile, Keymap, ProfileSlot, RuntimeConfig,
};
use crate::config::{ConfigManager, StorageError};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Actions supported by the runtime command.
#[derive(Debug, Clone)]
pub enum RuntimeAction {
    /// List all devices and their profile slots.
    ListDevices,
    /// Add or update a profile slot for a device.
    AddSlot {
        /// USB vendor ID of the target device.
        vendor_id: u16,
        /// USB product ID of the target device.
        product_id: u16,
        /// Optional serial number to distinguish identical devices.
        serial: Option<String>,
        /// Unique identifier for this slot.
        slot_id: String,
        /// Hardware profile to use for this slot.
        hardware_profile_id: String,
        /// Keymap to use for this slot.
        keymap_id: String,
        /// Whether this slot is active.
        active: bool,
        /// Priority for slot ordering (higher = more important).
        priority: Option<u32>,
    },
    /// Remove a profile slot from a device.
    RemoveSlot {
        /// USB vendor ID of the target device.
        vendor_id: u16,
        /// USB product ID of the target device.
        product_id: u16,
        /// Optional serial number to distinguish identical devices.
        serial: Option<String>,
        /// ID of the slot to remove.
        slot_id: String,
    },
    /// Set a slot's active state.
    SetSlotActive {
        /// USB vendor ID of the target device.
        vendor_id: u16,
        /// USB product ID of the target device.
        product_id: u16,
        /// Optional serial number to distinguish identical devices.
        serial: Option<String>,
        /// ID of the slot to modify.
        slot_id: String,
        /// New active state for the slot.
        active: bool,
    },
}

#[derive(Debug, Clone)]
struct DeviceSelector {
    vendor_id: u16,
    product_id: u16,
    serial: Option<String>,
}

/// Runtime command entry point.
///
/// Manages runtime profile slots that associate hardware profiles and keymaps
/// with specific keyboard devices.
pub struct RuntimeCommand {
    output: OutputWriter,
    action: RuntimeAction,
    config_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct RuntimeSlotView {
    id: String,
    hardware_profile_id: String,
    hardware_profile_name: Option<String>,
    keymap_id: String,
    keymap_name: Option<String>,
    active: bool,
    priority: u32,
}

#[derive(Debug, Clone, Serialize)]
struct RuntimeDeviceView {
    vendor_id: u16,
    product_id: u16,
    serial: Option<String>,
    slots: Vec<RuntimeSlotView>,
}

#[derive(Debug, Serialize)]
struct RuntimeListOutput {
    devices: Vec<RuntimeDeviceView>,
}

#[derive(Debug, Serialize)]
struct RuntimeUpdateOutput {
    saved_path: String,
    device: RuntimeDeviceView,
}

impl RuntimeCommand {
    /// Creates a new runtime command with the specified output format and action.
    pub fn new(format: OutputFormat, action: RuntimeAction) -> Self {
        Self {
            output: OutputWriter::new(format),
            action,
            config_root: None,
        }
    }

    /// Override config root (useful for tests).
    pub fn with_config_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.config_root = Some(root.into());
        self
    }

    fn run(&self) -> CommandResult<()> {
        match &self.action {
            RuntimeAction::ListDevices => self.list_devices(),
            RuntimeAction::AddSlot {
                vendor_id,
                product_id,
                serial,
                slot_id,
                hardware_profile_id,
                keymap_id,
                active,
                priority,
            } => self.add_slot(
                DeviceSelector {
                    vendor_id: *vendor_id,
                    product_id: *product_id,
                    serial: serial.clone(),
                },
                slot_id,
                hardware_profile_id,
                keymap_id,
                *active,
                *priority,
            ),
            RuntimeAction::RemoveSlot {
                vendor_id,
                product_id,
                serial,
                slot_id,
            } => self.remove_slot(
                DeviceSelector {
                    vendor_id: *vendor_id,
                    product_id: *product_id,
                    serial: serial.clone(),
                },
                slot_id,
            ),
            RuntimeAction::SetSlotActive {
                vendor_id,
                product_id,
                serial,
                slot_id,
                active,
            } => self.set_slot_active(
                DeviceSelector {
                    vendor_id: *vendor_id,
                    product_id: *product_id,
                    serial: serial.clone(),
                },
                slot_id,
                *active,
            ),
        }
    }

    fn list_devices(&self) -> CommandResult<()> {
        let manager = self.manager();
        let resources = match manager.load_all() {
            Ok(res) => res,
            Err(err) => return self.storage_failure("load runtime configuration", err),
        };

        let mut devices: Vec<_> = resources
            .runtime
            .devices
            .iter()
            .map(|device_slots| {
                let mut slots: Vec<_> = device_slots
                    .slots
                    .iter()
                    .map(|slot| RuntimeSlotView {
                        id: slot.id.clone(),
                        hardware_profile_id: slot.hardware_profile_id.clone(),
                        hardware_profile_name: resources
                            .hardware_profiles
                            .get(&slot.hardware_profile_id)
                            .and_then(|hp| hp.name.clone()),
                        keymap_id: slot.keymap_id.clone(),
                        keymap_name: resources
                            .keymaps
                            .get(&slot.keymap_id)
                            .map(|km| km.name.clone()),
                        active: slot.active,
                        priority: slot.priority,
                    })
                    .collect();
                slots.sort_by(|a, b| b.priority.cmp(&a.priority));

                RuntimeDeviceView {
                    vendor_id: device_slots.device.vendor_id,
                    product_id: device_slots.device.product_id,
                    serial: device_slots.device.serial.clone(),
                    slots,
                }
            })
            .collect();

        devices.sort_by(|a, b| {
            (a.vendor_id, a.product_id, a.serial.clone()).cmp(&(
                b.vendor_id,
                b.product_id,
                b.serial.clone(),
            ))
        });

        if let Err(err) = self.output.data(&RuntimeListOutput { devices }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render runtime devices: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success("Runtime devices listed");
        }
        CommandResult::success(())
    }

    fn add_slot(
        &self,
        device: DeviceSelector,
        slot_id: &str,
        hardware_profile_id: &str,
        keymap_id: &str,
        active: bool,
        priority: Option<u32>,
    ) -> CommandResult<()> {
        let manager = self.manager();
        let hardware_profiles = match manager.load_hardware_profiles() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load hardware profiles", err),
        };
        let keymaps = match manager.load_keymaps() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load keymaps", err),
        };
        if !hardware_profiles.contains_key(hardware_profile_id) {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Hardware profile '{hardware_profile_id}' not found"),
            );
        }
        if !keymaps.contains_key(keymap_id) {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Keymap '{keymap_id}' not found"),
            );
        }

        let mut runtime = match manager.load_runtime_config() {
            Ok(cfg) => cfg,
            Err(err) => return self.storage_failure("load runtime configuration", err),
        };

        let device = device_instance(&device);

        let slots = self.device_slots_mut(&mut runtime, &device);
        let next_priority = priority.unwrap_or_else(|| {
            slots
                .iter()
                .map(|s| s.priority)
                .max()
                .unwrap_or_default()
                .saturating_add(1)
        });

        let new_slot = ProfileSlot {
            id: slot_id.to_string(),
            hardware_profile_id: hardware_profile_id.to_string(),
            keymap_id: keymap_id.to_string(),
            active,
            priority: next_priority,
        };

        if let Some(existing) = slots.iter_mut().find(|s| s.id == new_slot.id) {
            *existing = new_slot;
        } else {
            slots.push(new_slot);
        }
        reorder_slots(slots);

        let saved_path = match manager.save_runtime_config(&runtime) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save runtime configuration", err),
        };

        let device_view =
            match self.build_device_view(&device, &runtime, &hardware_profiles, &keymaps) {
                Ok(view) => view,
                Err(err) => return err,
            };

        if let Err(err) = self.output.data(&RuntimeUpdateOutput {
            saved_path: saved_path.display().to_string(),
            device: device_view,
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render slot output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success(&format!(
                "Slot '{slot_id}' saved for device {}",
                device_key(&device)
            ));
        }

        CommandResult::success(())
    }

    fn remove_slot(&self, device: DeviceSelector, slot_id: &str) -> CommandResult<()> {
        let manager = self.manager();
        let hardware_profiles = match manager.load_hardware_profiles() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load hardware profiles", err),
        };
        let keymaps = match manager.load_keymaps() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load keymaps", err),
        };

        let mut runtime = match manager.load_runtime_config() {
            Ok(cfg) => cfg,
            Err(err) => return self.storage_failure("load runtime configuration", err),
        };
        let device = device_instance(&device);

        let Some(slots) = runtime
            .devices
            .iter_mut()
            .find(|d| d.device == device)
            .map(|d| &mut d.slots)
        else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Device {} has no runtime slots", device_key(&device)),
            );
        };

        let before = slots.len();
        slots.retain(|slot| slot.id != slot_id);
        if before == slots.len() {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!(
                    "Slot '{slot_id}' not found for device {}",
                    device_key(&device)
                ),
            );
        }

        reorder_slots(slots);

        let saved_path = match manager.save_runtime_config(&runtime) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save runtime configuration", err),
        };

        let device_view =
            match self.build_device_view(&device, &runtime, &hardware_profiles, &keymaps) {
                Ok(view) => view,
                Err(err) => return err,
            };

        if let Err(err) = self.output.data(&RuntimeUpdateOutput {
            saved_path: saved_path.display().to_string(),
            device: device_view,
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render slot removal output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success(&format!(
                "Removed slot '{slot_id}' from device {}",
                device_key(&device)
            ));
        }

        CommandResult::success(())
    }

    fn set_slot_active(
        &self,
        device: DeviceSelector,
        slot_id: &str,
        active: bool,
    ) -> CommandResult<()> {
        let manager = self.manager();
        let hardware_profiles = match manager.load_hardware_profiles() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load hardware profiles", err),
        };
        let keymaps = match manager.load_keymaps() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load keymaps", err),
        };
        let mut runtime = match manager.load_runtime_config() {
            Ok(cfg) => cfg,
            Err(err) => return self.storage_failure("load runtime configuration", err),
        };

        let device = device_instance(&device);
        let Some(slots) = runtime
            .devices
            .iter_mut()
            .find(|d| d.device == device)
            .map(|d| &mut d.slots)
        else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Device {} has no runtime slots", device_key(&device)),
            );
        };

        let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!(
                    "Slot '{slot_id}' not found for device {}",
                    device_key(&device)
                ),
            );
        };

        slot.active = active;
        reorder_slots(slots);

        let saved_path = match manager.save_runtime_config(&runtime) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save runtime configuration", err),
        };

        let device_view =
            match self.build_device_view(&device, &runtime, &hardware_profiles, &keymaps) {
                Ok(view) => view,
                Err(err) => return err,
            };

        if let Err(err) = self.output.data(&RuntimeUpdateOutput {
            saved_path: saved_path.display().to_string(),
            device: device_view,
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render slot toggle output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success(&format!(
                "Slot '{slot_id}' for device {} set to active={active}",
                device_key(&device)
            ));
        }

        CommandResult::success(())
    }

    fn device_slots_mut<'a>(
        &self,
        runtime: &'a mut RuntimeConfig,
        device: &DeviceInstanceId,
    ) -> &'a mut Vec<ProfileSlot> {
        if let Some(index) = runtime.devices.iter().position(|d| d.device == *device) {
            return &mut runtime.devices[index].slots;
        }

        runtime.devices.push(DeviceSlots {
            device: device.clone(),
            slots: vec![],
        });
        let new_index = runtime.devices.len().saturating_sub(1);
        &mut runtime.devices[new_index].slots
    }

    fn build_device_view(
        &self,
        device: &DeviceInstanceId,
        runtime: &RuntimeConfig,
        hardware_profiles: &HashMap<String, HardwareProfile>,
        keymaps: &HashMap<String, Keymap>,
    ) -> Result<RuntimeDeviceView, CommandResult<()>> {
        let Some(device_slots) = runtime.devices.iter().find(|d| &d.device == device) else {
            return Err(CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Device {} not found in runtime config", device_key(device)),
            ));
        };

        let mut slots: Vec<_> = device_slots
            .slots
            .iter()
            .map(|slot| RuntimeSlotView {
                id: slot.id.clone(),
                hardware_profile_id: slot.hardware_profile_id.clone(),
                hardware_profile_name: hardware_profiles
                    .get(&slot.hardware_profile_id)
                    .and_then(|hp| hp.name.clone()),
                keymap_id: slot.keymap_id.clone(),
                keymap_name: keymaps.get(&slot.keymap_id).map(|km| km.name.clone()),
                active: slot.active,
                priority: slot.priority,
            })
            .collect();
        slots.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(RuntimeDeviceView {
            vendor_id: device.vendor_id,
            product_id: device.product_id,
            serial: device.serial.clone(),
            slots,
        })
    }

    fn manager(&self) -> ConfigManager {
        match &self.config_root {
            Some(root) => ConfigManager::new(root),
            None => ConfigManager::default(),
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
}

impl Command for RuntimeCommand {
    fn name(&self) -> &str {
        "runtime"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}

fn reorder_slots(slots: &mut [ProfileSlot]) {
    slots.sort_by(|a, b| b.priority.cmp(&a.priority));
}

fn device_instance(selector: &DeviceSelector) -> DeviceInstanceId {
    DeviceInstanceId {
        vendor_id: selector.vendor_id,
        product_id: selector.product_id,
        serial: selector.serial.clone(),
    }
}

fn device_key(device: &DeviceInstanceId) -> String {
    match &device.serial {
        Some(serial) => format!(
            "{:04x}:{:04x}:{serial}",
            device.vendor_id, device.product_id
        ),
        None => format!("{:04x}:{:04x}", device.vendor_id, device.product_id),
    }
}
