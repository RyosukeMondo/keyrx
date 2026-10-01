//! Naming of the daemon's own virtual output keyboard, and the one rule that
//! recognises it.
//!
//! Every daemon instance used to call its uinput device plain `keyrx`, so a
//! tool (or a test) that looked the output up by name could grab ANOTHER
//! instance's device, including the user's real daemon. Now each instance
//! names its device `keyrx-out-<pid>`, and [`is_keyrx_output`] is the single
//! loop-protection rule: such a device is never an input source, whichever
//! instance created it, and the legacy plain `keyrx` name stays covered so an
//! older daemon running next to a newer one cannot be captured either.

use serde::{Deserialize, Serialize};

/// Prefix of every output keyboard this daemon creates by default.
pub const OUTPUT_NAME_PREFIX: &str = "keyrx-out-";

/// The name daemons before per-instance naming gave their output device.
pub const LEGACY_OUTPUT_NAME: &str = "keyrx";

/// The default name for this process's output device: `keyrx-out-<pid>`.
#[must_use]
pub fn default_output_name() -> String {
    format!("{OUTPUT_NAME_PREFIX}{}", std::process::id())
}

/// Whether a device called `name` was created by a keyrx daemon as its
/// output keyboard (current or legacy naming). Such a device must never be
/// grabbed as input.
#[must_use]
pub fn is_keyrx_output(name: &str) -> bool {
    name == LEGACY_OUTPUT_NAME || name.starts_with(OUTPUT_NAME_PREFIX)
}

/// Where a running daemon injects its output, as reported in status on every
/// transport so a tool can find the right device without guessing by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDeviceInfo {
    /// The uinput device name (e.g. `keyrx-out-4242`).
    pub name: String,
    /// The `/dev/input/eventN` node, once the kernel has published it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_current_and_legacy_output_names() {
        assert!(is_keyrx_output("keyrx"));
        assert!(is_keyrx_output(&default_output_name()));
        assert!(is_keyrx_output("keyrx-out-1"));
    }

    #[test]
    fn real_and_test_keyboards_are_not_outputs() {
        assert!(!is_keyrx_output("AT Translated Set 2 keyboard"));
        assert!(!is_keyrx_output("keyrx-md-a-1234"));
        assert!(!is_keyrx_output("keyrx-live-kbd"));
        assert!(!is_keyrx_output("keyrx2"));
    }

    #[test]
    fn default_name_is_unique_per_process() {
        assert!(default_output_name().ends_with(&std::process::id().to_string()));
    }
}
