//! Configuration subcommand definitions.
//!
//! Contains subcommand enums for devices, hardware, layout, keymap, runtime, and golden commands.

use clap::{ArgAction, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Hardware subcommands.
#[derive(Subcommand, Clone)]
pub enum HardwareCommands {
    /// List stored hardware wiring profiles
    List,

    /// Create or update a hardware wiring profile from JSON (file path or "-")
    Define {
        /// Path to hardware profile JSON (use "-" to read from stdin)
        #[arg(value_name = "PATH")]
        source: PathBuf,
    },

    /// Wire a scancode to a virtual key within a hardware profile
    Wire {
        /// Hardware profile identifier
        #[arg(long)]
        profile: String,

        /// Scancode to map (hex like 0x04 or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        scancode: u16,

        /// Virtual key identifier to assign (omit with --clear to remove)
        #[arg(long)]
        virtual_key: Option<String>,

        /// Clear an existing mapping for the scancode
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },

    /// Detect connected keyboards and suggest timing profiles
    Detect,

    /// Resolve profiles for detected or specific hardware
    Profile {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        vendor_id: Option<u16>,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        product_id: Option<u16>,
    },

    /// Run calibration using latency samples (microseconds)
    Calibrate {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        vendor_id: Option<u16>,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        product_id: Option<u16>,

        /// Warmup samples to discard
        #[arg(long, default_value_t = 3)]
        warmup_samples: usize,

        /// Samples to keep for optimization
        #[arg(long, default_value_t = 25)]
        sample_count: usize,

        /// Max duration for calibration run (seconds)
        #[arg(long, default_value_t = 30)]
        max_duration_secs: u64,

        /// Latency samples in microseconds (repeatable)
        #[arg(long = "latency-us")]
        latencies: Vec<u64>,

        /// Path to newline-delimited latency samples (microseconds)
        #[arg(long)]
        samples_file: Option<PathBuf>,
    },
}

/// Layout subcommands.
#[derive(Subcommand, Clone, Default)]
pub enum LayoutCommands {
    /// List all virtual layouts
    #[default]
    List,

    /// Show a specific virtual layout by id
    Show {
        /// Layout identifier
        id: String,
    },

    /// Create or update a virtual layout from JSON (file path or "-")
    Create {
        /// Path to layout JSON (use "-" to read from stdin)
        #[arg(value_name = "PATH")]
        source: PathBuf,
    },
}

/// Keymap subcommands.
#[derive(Subcommand, Clone, Default)]
pub enum KeymapCommands {
    /// List all keymaps
    #[default]
    List,

    /// Show a specific keymap by id
    Show {
        /// Keymap identifier
        id: String,
    },

    /// Set or clear a binding in a keymap layer
    Map {
        /// Keymap identifier
        #[arg(long)]
        keymap: String,

        /// Layer name (creates the layer if missing)
        #[arg(long)]
        layer: String,

        /// Virtual key identifier to bind
        #[arg(long = "virtual-key")]
        virtual_key: String,

        /// Action binding to apply (`key:<code>`, `macro:<text>`, `layer-toggle:<layer>`, or `transparent`)
        #[arg(long, required_unless_present = "clear")]
        action: Option<String>,

        /// Clear the binding instead of setting it
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },
}

/// Runtime subcommands.
#[derive(Subcommand, Clone, Default)]
pub enum RuntimeCommands {
    /// List runtime devices and active slots
    #[default]
    Devices,

    /// Add or update a runtime slot for a device
    SlotAdd {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier
        #[arg(long)]
        slot: String,

        /// Hardware profile id to attach
        #[arg(long = "hardware-profile")]
        hardware_profile: String,

        /// Keymap id to attach
        #[arg(long)]
        keymap: String,

        /// Slot priority (higher wins). Defaults to append order.
        #[arg(long)]
        priority: Option<u32>,

        /// Whether the slot is active
        #[arg(long, default_value_t = true)]
        active: bool,
    },

    /// Remove a runtime slot
    SlotRemove {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier to remove
        #[arg(long)]
        slot: String,
    },

    /// Toggle a runtime slot's active flag
    SlotActive {
        /// Vendor ID (hex like 0x1b1c or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        vendor_id: u16,

        /// Product ID (hex like 0x1b2e or decimal)
        #[arg(long, value_parser = crate::parse_hex_or_decimal_u16)]
        product_id: u16,

        /// Optional device serial number to disambiguate identical models
        #[arg(long)]
        serial: Option<String>,

        /// Slot identifier to toggle
        #[arg(long)]
        slot: String,

        /// Desired active state (true/false)
        #[arg(long)]
        active: bool,
    },
}

/// Device subcommands.
#[derive(Subcommand, Clone, Default)]
pub enum DeviceCommands {
    /// List all persisted device bindings
    #[default]
    List,

    /// Show details for a device identity
    Show {
        /// Device identity key (VID:PID:SERIAL)
        device: String,
    },

    /// Set or clear a user label for a device
    Label {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// New label value
        #[arg(required_unless_present = "clear")]
        label: Option<String>,

        /// Clear the label instead of setting one
        #[arg(long, action = ArgAction::SetTrue)]
        clear: bool,
    },

    /// Toggle remapping for a device
    Remap {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// Desired remap state
        #[arg(value_enum)]
        state: RemapState,
    },

    /// Assign a profile to a device
    Assign {
        /// Device identity key (VID:PID:SERIAL)
        device: String,

        /// Profile ID to assign
        profile: String,
    },

    /// Remove any profile assignment for a device
    Unassign {
        /// Device identity key (VID:PID:SERIAL)
        device: String,
    },
}

/// Remap state for device command.
#[derive(Clone, ValueEnum)]
pub enum RemapState {
    On,
    Off,
}

impl RemapState {
    pub fn enabled(&self) -> bool {
        matches!(self, RemapState::On)
    }
}

/// Golden session subcommands.
#[derive(Subcommand)]
pub enum GoldenCommands {
    /// Record a new golden session from a script
    Record {
        /// Name of the golden session (alphanumeric, underscores, hyphens)
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,
    },

    /// Verify an existing golden session
    Verify {
        /// Name of the golden session to verify
        name: String,

        /// Path to the script to run for verification (optional)
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Update an existing golden session
    Update {
        /// Name of the golden session to update
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,

        /// Confirm the update (required to overwrite existing session)
        #[arg(long)]
        confirm: bool,
    },

    /// List all golden sessions
    List,
}
