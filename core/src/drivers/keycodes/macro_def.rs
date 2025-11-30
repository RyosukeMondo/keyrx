//! Macro definition for generating keycode enum and conversion functions.
//!
//! This module contains the `define_keycodes!` macro which generates:
//! - The `KeyCode` enum with all keyboard key variants
//! - `Display` implementation for human-readable output
//! - `FromStr` implementation with alias support
//! - Platform-specific conversion functions
//! - Helper functions for device registration

/// Macro that generates the KeyCode enum and all related implementations.
///
/// # Syntax
///
/// ```text
/// define_keycodes! {
///     // Variant => "DisplayName", evdev_code, vk_code, ["alias1", "alias2", ...]
///     A => "A", 30, 0x41, ["A"],
///     ...
/// }
/// ```
///
/// The macro generates:
/// - `KeyCode` enum with all specified variants plus `Unknown(u16)`
/// - `Display` impl using the display name
/// - `FromStr` impl matching aliases (case-insensitive)
/// - `evdev_to_keycode(u16) -> KeyCode` for Linux
/// - `keycode_to_evdev(KeyCode) -> u16` for Linux
/// - `vk_to_keycode(u16) -> KeyCode` for Windows
/// - `keycode_to_vk(KeyCode) -> u16` for Windows
/// - `all_keycodes() -> Vec<KeyCode>` for device registration
macro_rules! define_keycodes {
    (
        $(
            $variant:ident => $display:literal, $evdev:expr, $vk:expr, [$($alias:literal),* $(,)?]
        ),* $(,)?
    ) => {
        /// Physical key code representing keyboard keys.
        ///
        /// This enum covers all standard keyboard keys including letters,
        /// numbers, function keys, modifiers, navigation, numpad, and media keys.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum KeyCode {
            $(
                $variant,
            )*
            /// Unknown key with raw scan code.
            Unknown(u16),
        }

        impl KeyCode {
            /// Parse a key code from a string name.
            #[inline]
            pub fn from_name(name: &str) -> Option<Self> {
                Self::from_str(name).ok()
            }

            /// Get the string name of this key code.
            #[inline]
            pub fn name(&self) -> String {
                format!("{self}")
            }
        }

        impl fmt::Display for KeyCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, $display),
                    )*
                    Self::Unknown(code) => write!(f, "Unknown({code})"),
                }
            }
        }

        impl FromStr for KeyCode {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s_upper = s.to_uppercase();
                match s_upper.as_str() {
                    $(
                        $($alias)|* => Ok(Self::$variant),
                    )*
                    _ => Err(format!("Unknown key: {s}")),
                }
            }
        }

        /// Convert evdev key code to KeyCode (Linux).
        ///
        /// Maps Linux evdev event codes (from input-event-codes.h) to the
        /// internal KeyCode representation.
        #[cfg(target_os = "linux")]
        pub fn evdev_to_keycode(code: u16) -> KeyCode {
            match code {
                $(
                    $evdev => KeyCode::$variant,
                )*
                _ => KeyCode::Unknown(code),
            }
        }

        /// Convert KeyCode to evdev key code (Linux).
        ///
        /// Maps internal KeyCode to Linux evdev event codes for key injection.
        #[cfg(target_os = "linux")]
        pub fn keycode_to_evdev(key: KeyCode) -> u16 {
            match key {
                $(
                    KeyCode::$variant => $evdev,
                )*
                KeyCode::Unknown(code) => code,
            }
        }

        /// Convert Windows virtual key code to KeyCode.
        ///
        /// Maps Windows VK_* constants to the internal KeyCode representation.
        /// Available on all platforms for cross-platform testing.
        ///
        /// Note: NumpadEnter and Enter share VK code 0x0D on Windows.
        /// They are distinguished by the extended key flag (KEYEVENTF_EXTENDEDKEY),
        /// not by the VK code itself. This function returns Enter for 0x0D.
        #[allow(unreachable_patterns)]
        pub fn vk_to_keycode(vk: u16) -> KeyCode {
            match vk {
                $(
                    $vk => KeyCode::$variant,
                )*
                _ => KeyCode::Unknown(vk),
            }
        }

        /// Convert KeyCode to Windows virtual key code.
        ///
        /// Maps internal KeyCode to Windows VK_* constants for key injection.
        /// Available on all platforms for cross-platform testing.
        pub fn keycode_to_vk(key: KeyCode) -> u16 {
            match key {
                $(
                    KeyCode::$variant => $vk,
                )*
                KeyCode::Unknown(vk) => vk,
            }
        }

        /// Returns a vector of all known keycodes (excluding Unknown).
        ///
        /// Used for uinput device registration on Linux to enable all keys.
        pub fn all_keycodes() -> Vec<KeyCode> {
            vec![
                $(
                    KeyCode::$variant,
                )*
            ]
        }

        /// Returns all evdev key codes for uinput registration (Linux only).
        ///
        /// Returns a vector of (evdev_code, KeyCode) pairs for registering
        /// all supported keys with a uinput virtual device.
        #[cfg(target_os = "linux")]
        pub fn all_evdev_codes() -> Vec<u16> {
            vec![
                $(
                    $evdev,
                )*
            ]
        }
    };
}

pub(crate) use define_keycodes;
