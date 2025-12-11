//! Configuration commands: devices, hardware, layout, keymap, runtime, migrate
//!
//! This module contains command handlers for configuration management:
//! - `devices`: Manage device bindings and profiles
//! - `hardware`: Hardware detection, profiles, and calibration
//! - `layout`: Manage virtual layouts
//! - `keymap`: Manage logical keymaps
//! - `runtime`: Inspect and modify runtime profile slots
//! - `migrate`: Migrate profiles between versions

use keyrx_core::cli::{
    commands::{
        DeviceAction, DevicesCommand, HardwareAction, HardwareCommand, HardwareSource,
        KeymapAction, KeymapCommand, LayoutAction, LayoutCommand, LayoutSource, MapRequest,
        MigrateCommand, RuntimeAction, RuntimeCommand,
    },
    Command, CommandContext, CommandResult,
};
use std::path::PathBuf;

/// Device subcommand actions parsed from CLI
pub enum DeviceCommandAction {
    List,
    Show {
        device: String,
    },
    Label {
        device: String,
        label: Option<String>,
        clear: bool,
    },
    Remap {
        device: String,
        enabled: bool,
    },
    Assign {
        device: String,
        profile: String,
    },
    Unassign {
        device: String,
    },
}

/// Execute the `devices` command
pub fn execute_devices(action: DeviceCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let device_action = match action {
        DeviceCommandAction::List => DeviceAction::List,
        DeviceCommandAction::Show { device } => DeviceAction::Show { device_key: device },
        DeviceCommandAction::Label {
            device,
            label,
            clear,
        } => DeviceAction::Label {
            device_key: device,
            label: if clear { None } else { label },
        },
        DeviceCommandAction::Remap { device, enabled } => DeviceAction::Remap {
            device_key: device,
            enabled,
        },
        DeviceCommandAction::Assign { device, profile } => DeviceAction::Assign {
            device_key: device,
            profile_id: profile,
        },
        DeviceCommandAction::Unassign { device } => DeviceAction::Unassign { device_key: device },
    };
    let mut cmd = DevicesCommand::new(ctx.output_format(), device_action);
    cmd.execute(ctx)
}

/// Hardware subcommand actions parsed from CLI
pub enum HardwareCommandAction {
    List,
    Define {
        source: PathBuf,
    },
    Wire {
        profile: String,
        scancode: u16,
        virtual_key: Option<String>,
        clear: bool,
    },
    Detect,
    Profile {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
    },
    Calibrate {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        warmup_samples: usize,
        sample_count: usize,
        max_duration_secs: u64,
        latencies: Vec<u64>,
        samples_file: Option<PathBuf>,
    },
}

/// Execute the `hardware` command
pub fn execute_hardware(action: HardwareCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let hw_action = match action {
        HardwareCommandAction::List => HardwareAction::List,
        HardwareCommandAction::Define { source } => {
            let src = if source.as_os_str() == "-" {
                HardwareSource::Stdin
            } else {
                HardwareSource::File(source)
            };
            HardwareAction::Define { source: src }
        }
        HardwareCommandAction::Wire {
            profile,
            scancode,
            virtual_key,
            clear,
        } => HardwareAction::Wire {
            profile_id: profile,
            scancode,
            virtual_key,
            clear,
        },
        HardwareCommandAction::Detect => HardwareAction::Detect,
        HardwareCommandAction::Profile {
            vendor_id,
            product_id,
        } => HardwareAction::Profile {
            vendor_id,
            product_id,
        },
        HardwareCommandAction::Calibrate {
            vendor_id,
            product_id,
            warmup_samples,
            sample_count,
            max_duration_secs,
            latencies,
            samples_file,
        } => HardwareAction::Calibrate {
            vendor_id,
            product_id,
            warmup_samples,
            sample_count,
            max_duration_secs,
            latencies_us: latencies,
            samples_file: samples_file.map(|p| p.display().to_string()),
        },
    };
    let mut cmd = HardwareCommand::new(ctx.output_format(), hw_action);
    cmd.execute(ctx)
}

/// Layout subcommand actions parsed from CLI
pub enum LayoutCommandAction {
    List,
    Show { id: String },
    Create { source: PathBuf },
}

/// Execute the `layout` command
pub fn execute_layout(action: LayoutCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let layout_action = match action {
        LayoutCommandAction::List => LayoutAction::List,
        LayoutCommandAction::Show { id } => LayoutAction::Show { id },
        LayoutCommandAction::Create { source } => {
            let src = if source.as_os_str() == "-" {
                LayoutSource::Stdin
            } else {
                LayoutSource::File(source)
            };
            LayoutAction::Create { source: src }
        }
    };
    let mut cmd = LayoutCommand::new(ctx.output_format(), layout_action);
    cmd.execute(ctx)
}

/// Keymap subcommand actions parsed from CLI
pub enum KeymapCommandAction {
    List,
    Show {
        id: String,
    },
    Map {
        keymap: String,
        layer: String,
        virtual_key: String,
        action: Option<String>,
        clear: bool,
    },
}

/// Execute the `keymap` command
pub fn execute_keymap(action: KeymapCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let keymap_action = match action {
        KeymapCommandAction::List => KeymapAction::List,
        KeymapCommandAction::Show { id } => KeymapAction::Show { id },
        KeymapCommandAction::Map {
            keymap,
            layer,
            virtual_key,
            action,
            clear,
        } => KeymapAction::Map {
            request: MapRequest {
                keymap_id: keymap,
                layer,
                virtual_key,
                action,
                clear,
            },
        },
    };
    let mut cmd = KeymapCommand::new(ctx.output_format(), keymap_action);
    cmd.execute(ctx)
}

/// Runtime subcommand actions parsed from CLI
pub enum RuntimeCommandAction {
    Devices,
    SlotAdd {
        vendor_id: u16,
        product_id: u16,
        serial: Option<String>,
        slot: String,
        hardware_profile: String,
        keymap: String,
        priority: Option<u32>,
        active: bool,
    },
    SlotRemove {
        vendor_id: u16,
        product_id: u16,
        serial: Option<String>,
        slot: String,
    },
    SlotActive {
        vendor_id: u16,
        product_id: u16,
        serial: Option<String>,
        slot: String,
        active: bool,
    },
}

/// Execute the `runtime` command
pub fn execute_runtime(action: RuntimeCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let runtime_action = match action {
        RuntimeCommandAction::Devices => RuntimeAction::ListDevices,
        RuntimeCommandAction::SlotAdd {
            vendor_id,
            product_id,
            serial,
            slot,
            hardware_profile,
            keymap,
            priority,
            active,
        } => RuntimeAction::AddSlot {
            vendor_id,
            product_id,
            serial,
            slot_id: slot,
            hardware_profile_id: hardware_profile,
            keymap_id: keymap,
            active,
            priority,
        },
        RuntimeCommandAction::SlotRemove {
            vendor_id,
            product_id,
            serial,
            slot,
        } => RuntimeAction::RemoveSlot {
            vendor_id,
            product_id,
            serial,
            slot_id: slot,
        },
        RuntimeCommandAction::SlotActive {
            vendor_id,
            product_id,
            serial,
            slot,
            active,
        } => RuntimeAction::SetSlotActive {
            vendor_id,
            product_id,
            serial,
            slot_id: slot,
            active,
        },
    };
    let mut cmd = RuntimeCommand::new(ctx.output_format(), runtime_action);
    cmd.execute(ctx)
}

/// Execute the `migrate` command
pub async fn execute_migrate(
    from: String,
    backup: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let cmd = MigrateCommand::new(ctx.output_format(), from, backup);
    cmd.run().await
}
