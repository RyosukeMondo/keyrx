//! Custom error types for KeyRx.
//!
//! This module provides structured error handling with actionable error messages.
//! All errors implement `std::error::Error` via `thiserror` and can be converted
//! from common error sources like `std::io::Error` and Rhai's `EvalAltResult`.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for KeyRx operations.
///
/// Each variant provides specific context about what went wrong and suggests
/// corrective action where possible.
#[derive(Debug, Error)]
pub enum KeyRxError {
    /// A key name was not recognized.
    ///
    /// This occurs when a script uses an invalid key name in `remap()`, `block()`,
    /// or `pass()`. See `.spec-workflow/steering/tech.md` (Key Naming & Aliases) for valid key names.
    #[error("Unknown key '{key}'. See .spec-workflow/steering/tech.md (Key Naming & Aliases).")]
    UnknownKey { key: String },

    /// A script failed to compile.
    ///
    /// This indicates a syntax error or other compilation issue. The message
    /// contains details from the Rhai compiler, and line/column information
    /// when available.
    #[error("Script compilation failed: {message}{}", format_position(*.line, *.column))]
    ScriptCompileError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    /// A script failed during execution.
    ///
    /// This occurs when a script encounters a runtime error, such as calling
    /// an undefined function or a type mismatch.
    #[error("Script runtime error: {message}")]
    ScriptRuntimeError { message: String },

    /// A file path was invalid.
    ///
    /// This can occur with non-UTF8 paths or paths that don't meet other
    /// requirements (e.g., must be a file, not a directory).
    #[error("Invalid path '{path}': {reason}")]
    InvalidPath { path: String, reason: String },

    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A platform-specific operation failed.
    ///
    /// This covers OS-level errors from input drivers or other platform APIs.
    #[error("Platform error: {message}")]
    PlatformError { message: String },

    /// A Linux-specific driver error occurred.
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    LinuxDriver(#[from] LinuxDriverError),

    /// A Windows-specific driver error occurred.
    #[cfg(windows)]
    #[error(transparent)]
    WindowsDriver(#[from] WindowsDriverError),
}

/// Linux-specific driver errors.
///
/// These errors occur during evdev/uinput operations on Linux.
/// Each variant includes remediation hints to help users resolve the issue.
#[cfg(target_os = "linux")]
#[derive(Debug, Error)]
pub enum LinuxDriverError {
    /// The specified device was not found.
    ///
    /// Verify the device path exists and is a valid input device.
    /// Run `keyrx devices` to list available keyboards.
    #[error("Device not found: {path}. Run 'keyrx devices' to list available keyboards.")]
    DeviceNotFound { path: PathBuf },

    /// Permission denied when accessing the device.
    ///
    /// The user needs read/write access to the input device.
    /// Add user to the 'input' group: `sudo usermod -aG input $USER`
    #[error("Permission denied for {path}. Add yourself to the 'input' group: sudo usermod -aG input $USER (then log out and back in)")]
    PermissionDenied { path: PathBuf },

    /// Failed to grab exclusive access to the keyboard.
    ///
    /// Another process may have the keyboard grabbed, or permissions are insufficient.
    #[error("Failed to grab keyboard: {0}. Another process may have exclusive access, or try running with elevated permissions.")]
    GrabFailed(#[source] std::io::Error),

    /// Failed to create or use the uinput virtual keyboard.
    ///
    /// Ensure the uinput module is loaded: `sudo modprobe uinput`
    /// The user needs write access to /dev/uinput.
    #[error("Failed to create virtual keyboard: {0}. Ensure uinput module is loaded: sudo modprobe uinput")]
    UinputFailed(#[source] std::io::Error),
}

/// Windows-specific driver errors.
///
/// These errors occur during Windows hook and SendInput operations.
#[cfg(windows)]
#[derive(Debug, Error)]
pub enum WindowsDriverError {
    /// Failed to install the low-level keyboard hook.
    ///
    /// The error code is from GetLastError().
    #[error("Failed to install keyboard hook (error code: {0}). Ensure the application has focus or try running as administrator.")]
    HookInstallFailed(u32),

    /// Failed to inject a key via SendInput.
    ///
    /// The error code is from GetLastError().
    #[error("Failed to send input (error code: {0}). Another application may be blocking input injection.")]
    SendInputFailed(u32),

    /// The message pump thread panicked.
    ///
    /// This is a fatal error indicating internal driver failure.
    #[error("Message pump thread panicked. This is an internal error - please report this issue.")]
    MessagePumpPanic,
}

impl KeyRxError {
    /// Create an UnknownKey error.
    pub fn unknown_key(key: impl Into<String>) -> Self {
        Self::UnknownKey { key: key.into() }
    }

    /// Create a ScriptCompileError with position information.
    pub fn script_compile(
        message: impl Into<String>,
        line: Option<usize>,
        column: Option<usize>,
    ) -> Self {
        Self::ScriptCompileError {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a ScriptCompileError without position information.
    pub fn script_compile_simple(message: impl Into<String>) -> Self {
        Self::ScriptCompileError {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    /// Create a ScriptRuntimeError.
    pub fn script_runtime(message: impl Into<String>) -> Self {
        Self::ScriptRuntimeError {
            message: message.into(),
        }
    }

    /// Create an InvalidPath error.
    pub fn invalid_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a PlatformError.
    pub fn platform(message: impl Into<String>) -> Self {
        Self::PlatformError {
            message: message.into(),
        }
    }
}

#[cfg(target_os = "linux")]
impl LinuxDriverError {
    /// Create a DeviceNotFound error.
    pub fn device_not_found(path: impl Into<PathBuf>) -> Self {
        Self::DeviceNotFound { path: path.into() }
    }

    /// Create a PermissionDenied error.
    pub fn permission_denied(path: impl Into<PathBuf>) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    /// Create a GrabFailed error.
    pub fn grab_failed(err: std::io::Error) -> Self {
        Self::GrabFailed(err)
    }

    /// Create a UinputFailed error.
    pub fn uinput_failed(err: std::io::Error) -> Self {
        Self::UinputFailed(err)
    }
}

#[cfg(windows)]
impl WindowsDriverError {
    /// Create a HookInstallFailed error.
    pub fn hook_install_failed(error_code: u32) -> Self {
        Self::HookInstallFailed(error_code)
    }

    /// Create a SendInputFailed error.
    pub fn send_input_failed(error_code: u32) -> Self {
        Self::SendInputFailed(error_code)
    }

    /// Create a MessagePumpPanic error.
    pub fn message_pump_panic() -> Self {
        Self::MessagePumpPanic
    }
}

impl From<Box<rhai::EvalAltResult>> for KeyRxError {
    fn from(err: Box<rhai::EvalAltResult>) -> Self {
        let position = err.position();
        if position.is_none() {
            Self::ScriptRuntimeError {
                message: err.to_string(),
            }
        } else {
            Self::ScriptRuntimeError {
                message: format!("{} at {}", err, position),
            }
        }
    }
}

/// Helper to format optional position information.
fn format_position(line: Option<usize>, column: Option<usize>) -> String {
    match (line, column) {
        (Some(l), Some(c)) => format!(" at line {}, column {}", l, c),
        (Some(l), None) => format!(" at line {}", l),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_key_error_message() {
        let err = KeyRxError::unknown_key("BadKey");
        assert!(err.to_string().contains("BadKey"));
        assert!(err
            .to_string()
            .contains(".spec-workflow/steering/tech.md (Key Naming & Aliases)."));
    }

    #[test]
    fn script_compile_error_with_position() {
        let err = KeyRxError::script_compile("syntax error", Some(10), Some(5));
        let msg = err.to_string();
        assert!(msg.contains("syntax error"));
        assert!(msg.contains("line 10"));
        assert!(msg.contains("column 5"));
    }

    #[test]
    fn script_compile_error_without_position() {
        let err = KeyRxError::script_compile_simple("unknown token");
        let msg = err.to_string();
        assert!(msg.contains("unknown token"));
        assert!(!msg.contains("line"));
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: KeyRxError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn invalid_path_error() {
        let err = KeyRxError::invalid_path("/some/path", "not UTF-8");
        assert!(err.to_string().contains("/some/path"));
        assert!(err.to_string().contains("not UTF-8"));
    }

    #[test]
    fn platform_error() {
        let err = KeyRxError::platform("device not found");
        assert!(err.to_string().contains("device not found"));
    }

    #[cfg(target_os = "linux")]
    mod linux_driver_errors {
        use super::*;

        #[test]
        fn device_not_found_error() {
            let err = LinuxDriverError::device_not_found("/dev/input/event99");
            let msg = err.to_string();
            assert!(msg.contains("/dev/input/event99"));
            assert!(msg.contains("keyrx devices"));
        }

        #[test]
        fn permission_denied_error() {
            let err = LinuxDriverError::permission_denied("/dev/input/event0");
            let msg = err.to_string();
            assert!(msg.contains("/dev/input/event0"));
            assert!(msg.contains("input"));
            assert!(msg.contains("usermod"));
        }

        #[test]
        fn grab_failed_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::Other, "device busy");
            let err = LinuxDriverError::grab_failed(io_err);
            let msg = err.to_string();
            assert!(msg.contains("device busy"));
            assert!(msg.contains("exclusive access"));
        }

        #[test]
        fn uinput_failed_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
            let err = LinuxDriverError::uinput_failed(io_err);
            let msg = err.to_string();
            assert!(msg.contains("no access"));
            assert!(msg.contains("modprobe uinput"));
        }

        #[test]
        fn linux_error_converts_to_keyrx_error() {
            let linux_err = LinuxDriverError::device_not_found("/dev/input/event0");
            let keyrx_err: KeyRxError = linux_err.into();
            assert!(matches!(keyrx_err, KeyRxError::LinuxDriver(_)));
            assert!(keyrx_err.to_string().contains("/dev/input/event0"));
        }
    }

    #[cfg(windows)]
    mod windows_driver_errors {
        use super::*;

        #[test]
        fn hook_install_failed_error() {
            let err = WindowsDriverError::hook_install_failed(5);
            let msg = err.to_string();
            assert!(msg.contains("error code: 5"));
            assert!(msg.contains("administrator"));
        }

        #[test]
        fn send_input_failed_error() {
            let err = WindowsDriverError::send_input_failed(1);
            let msg = err.to_string();
            assert!(msg.contains("error code: 1"));
            assert!(msg.contains("blocking"));
        }

        #[test]
        fn message_pump_panic_error() {
            let err = WindowsDriverError::message_pump_panic();
            let msg = err.to_string();
            assert!(msg.contains("panicked"));
            assert!(msg.contains("report"));
        }

        #[test]
        fn windows_error_converts_to_keyrx_error() {
            let win_err = WindowsDriverError::hook_install_failed(5);
            let keyrx_err: KeyRxError = win_err.into();
            assert!(matches!(keyrx_err, KeyRxError::WindowsDriver(_)));
            assert!(keyrx_err.to_string().contains("error code: 5"));
        }
    }
}
