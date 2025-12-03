//! Driver error definitions.
//!
//! This module defines all errors related to platform-specific drivers
//! for Windows and Linux. Driver errors use the KRX-D3xxx range.

use crate::define_errors;

define_errors! {
    category: Driver,
    base: 3000,

    errors: {
        DRIVER_NOT_FOUND = 1 => {
            message: "Driver not found or not installed",
            hint: "Install the required driver for your platform. On Linux, ensure the input subsystem is available. On Windows, check the filter driver installation",
            severity: Error,
        },

        DRIVER_LOAD_FAILED = 2 => {
            message: "Failed to load driver: {reason}",
            hint: "Check driver installation and system logs. You may need to reinstall the driver or reboot",
            severity: Error,
        },

        DRIVER_INIT_FAILED = 3 => {
            message: "Driver initialization failed: {reason}",
            hint: "Verify driver compatibility with your system version. Check for conflicting drivers",
            severity: Error,
        },

        DRIVER_PERMISSION_DENIED = 4 => {
            message: "Permission denied accessing driver: {device}",
            hint: "On Linux, you may need to run as root or add your user to the 'input' group. On Windows, ensure admin privileges or proper driver installation",
            severity: Error,
        },

        DRIVER_DEVICE_NOT_FOUND = 5 => {
            message: "Input device not found: {device}",
            hint: "Check that the device is connected and recognized by the system. Use device manager or lsusb to verify",
            severity: Error,
        },

        DRIVER_DEVICE_BUSY = 6 => {
            message: "Device is busy or already in use: {device}",
            hint: "Another application may be using the device. Close other key remapping tools or restart the system",
            severity: Error,
        },

        DRIVER_DEVICE_DISCONNECTED = 7 => {
            message: "Input device disconnected: {device}",
            hint: "The device was unplugged or lost connection. Reconnect the device to continue",
            severity: Warning,
        },

        DRIVER_READ_FAILED = 8 => {
            message: "Failed to read from device: {reason}",
            hint: "Device may be disconnected or the driver may be unstable. Try reconnecting the device",
            severity: Error,
        },

        DRIVER_WRITE_FAILED = 9 => {
            message: "Failed to write to device: {reason}",
            hint: "Device may be read-only or the driver may not support output. Check device permissions",
            severity: Error,
        },

        DRIVER_IOCTL_FAILED = 10 => {
            message: "Driver ioctl operation failed: {operation}",
            hint: "This is a low-level driver error. Check system logs and driver compatibility",
            severity: Error,
        },

        DRIVER_INCOMPATIBLE_VERSION = 11 => {
            message: "Driver version incompatible: found {found}, required {required}",
            hint: "Update the driver to the required version or downgrade KeyRx to match your driver",
            severity: Error,
        },

        DRIVER_NOT_SUPPORTED = 12 => {
            message: "Driver operation not supported on this platform: {operation}",
            hint: "This feature requires a different operating system or driver version",
            severity: Error,
        },

        DRIVER_BUFFER_OVERFLOW = 13 => {
            message: "Driver buffer overflow: {size} bytes exceeded limit",
            hint: "Too many events in the driver buffer. Events may be dropped. Reduce input rate",
            severity: Warning,
        },

        DRIVER_TIMEOUT = 14 => {
            message: "Driver operation timed out after {timeout_ms}ms",
            hint: "Driver may be unresponsive. Try reloading the driver or restarting the system",
            severity: Error,
        },

        DRIVER_COMMUNICATION_ERROR = 15 => {
            message: "Communication error with driver: {reason}",
            hint: "Driver may be in an inconsistent state. Reload the driver or restart the application",
            severity: Error,
        },

        EVDEV_DEVICE_GRAB_FAILED = 16 => {
            message: "Failed to grab evdev device: {device}",
            hint: "Another process may have exclusive access. On Linux, ensure no other input tools are running",
            severity: Error,
        },

        EVDEV_DEVICE_UNGRAB_FAILED = 17 => {
            message: "Failed to ungrab evdev device: {device}",
            hint: "Device may already be released. This is usually safe to ignore",
            severity: Warning,
        },

        EVDEV_UINPUT_CREATE_FAILED = 18 => {
            message: "Failed to create uinput device: {reason}",
            hint: "Ensure the uinput module is loaded: 'sudo modprobe uinput'. Check /dev/uinput permissions",
            severity: Error,
        },

        EVDEV_UINPUT_SETUP_FAILED = 19 => {
            message: "Failed to setup uinput device: {reason}",
            hint: "uinput device configuration failed. Check kernel version compatibility",
            severity: Error,
        },

        EVDEV_EVENT_CODE_INVALID = 20 => {
            message: "Invalid evdev event code: {code}",
            hint: "The event code is not recognized by the evdev subsystem. Check your key mappings",
            severity: Error,
        },

        EVDEV_SYNC_ERROR = 21 => {
            message: "evdev synchronization error: {reason}",
            hint: "Event stream is out of sync. This may cause missed or duplicate events",
            severity: Warning,
        },

        WINDOWS_DRIVER_NOT_INSTALLED = 22 => {
            message: "Windows filter driver not installed",
            hint: "Run the installer with admin privileges to install the filter driver, or use the portable mode with limited features",
            severity: Error,
        },

        WINDOWS_DRIVER_START_FAILED = 23 => {
            message: "Failed to start Windows filter driver: {reason}",
            hint: "Open Services and check the driver status. You may need to enable the service or check driver signature enforcement",
            severity: Error,
        },

        WINDOWS_DRIVER_STOP_FAILED = 24 => {
            message: "Failed to stop Windows filter driver: {reason}",
            hint: "The driver may have active handles. Close all applications using the driver",
            severity: Warning,
        },

        WINDOWS_HOOK_FAILED = 25 => {
            message: "Failed to install Windows keyboard hook: {reason}",
            hint: "Another low-level hook may be interfering. Close other keyboard tools or check antivirus settings",
            severity: Error,
        },

        WINDOWS_UNHOOK_FAILED = 26 => {
            message: "Failed to remove Windows keyboard hook: {reason}",
            hint: "The hook may already be removed. This is usually safe to ignore",
            severity: Warning,
        },

        WINDOWS_SENDMESSAGE_FAILED = 27 => {
            message: "Windows SendMessage failed: {reason}",
            hint: "Failed to send keyboard input. The target window may be unresponsive",
            severity: Error,
        },

        WINDOWS_REGISTRY_ACCESS_FAILED = 28 => {
            message: "Failed to access Windows registry: {key}",
            hint: "Run with administrator privileges or check registry permissions",
            severity: Error,
        },

        WINDOWS_SERVICE_NOT_RUNNING = 29 => {
            message: "Required Windows service not running: {service}",
            hint: "Start the service manually or configure it to start automatically",
            severity: Error,
        },

        WINDOWS_DRIVER_SIGNATURE_INVALID = 30 => {
            message: "Driver signature verification failed",
            hint: "Disable driver signature enforcement or reinstall with a properly signed driver. See documentation for details",
            severity: Error,
        },

        LINUX_INPUT_SUBSYSTEM_ERROR = 31 => {
            message: "Linux input subsystem error: {reason}",
            hint: "Check that the input subsystem is properly initialized. Verify kernel module loading",
            severity: Error,
        },

        LINUX_UDEV_ERROR = 32 => {
            message: "udev error: {reason}",
            hint: "Check udev rules and permissions. Ensure udev is running and configured correctly",
            severity: Error,
        },

        LINUX_DEVICE_NODE_ERROR = 33 => {
            message: "Failed to access device node: {path}",
            hint: "Check that /dev/{input,uinput} nodes exist and have correct permissions. You may need to add udev rules",
            severity: Error,
        },

        LINUX_MODULE_NOT_LOADED = 34 => {
            message: "Required kernel module not loaded: {module}",
            hint: "Load the module with 'sudo modprobe {module}' or configure it to load at boot",
            severity: Error,
        },

        LINUX_CAPABILITY_MISSING = 35 => {
            message: "Required Linux capability missing: {capability}",
            hint: "Grant the capability with 'sudo setcap {capability}+ep /path/to/keyrx' or run as root",
            severity: Error,
        },

        PLATFORM_NOT_SUPPORTED = 36 => {
            message: "Platform not supported: {platform}",
            hint: "KeyRx currently only supports Windows and Linux. macOS support is planned",
            severity: Error,
        },

        PLATFORM_DETECTION_FAILED = 37 => {
            message: "Failed to detect platform: {reason}",
            hint: "This is an internal error. Please report this bug with your system information",
            severity: Error,
        },

        HOTPLUG_DETECTION_FAILED = 38 => {
            message: "Failed to detect device hotplug: {reason}",
            hint: "Device hotplug monitoring may not work. You may need to restart to detect new devices",
            severity: Warning,
        },

        HOTPLUG_DEVICE_ADDED = 39 => {
            message: "New input device detected: {device}",
            hint: "A new device was connected. Restart KeyRx to use it, or configure hotplug handling",
            severity: Info,
        },

        HOTPLUG_DEVICE_REMOVED = 40 => {
            message: "Input device removed: {device}",
            hint: "A device was disconnected. This is informational unless it was the active device",
            severity: Info,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCategory;
    use crate::keyrx_err;

    #[test]
    fn driver_error_codes_in_range() {
        assert_eq!(DRIVER_NOT_FOUND.code().number(), 3001);
        assert_eq!(DRIVER_LOAD_FAILED.code().number(), 3002);
        assert_eq!(HOTPLUG_DEVICE_REMOVED.code().number(), 3040);

        // Verify all are in Driver category range
        assert!(ErrorCategory::Driver.contains(DRIVER_NOT_FOUND.code().number()));
        assert!(ErrorCategory::Driver.contains(HOTPLUG_DEVICE_REMOVED.code().number()));
    }

    #[test]
    fn driver_error_categories() {
        assert_eq!(DRIVER_NOT_FOUND.code().category(), ErrorCategory::Driver);
        assert_eq!(
            EVDEV_UINPUT_CREATE_FAILED.code().category(),
            ErrorCategory::Driver
        );
        assert_eq!(
            WINDOWS_DRIVER_NOT_INSTALLED.code().category(),
            ErrorCategory::Driver
        );
    }

    #[test]
    fn driver_error_messages() {
        let err = keyrx_err!(DRIVER_LOAD_FAILED, reason = "incompatible version");
        assert_eq!(err.code(), "KRX-D3002");
        assert!(err.message().contains("incompatible version"));
    }

    #[test]
    fn driver_error_hints() {
        assert!(DRIVER_NOT_FOUND.hint().is_some());
        assert!(DRIVER_PERMISSION_DENIED.hint().unwrap().contains("admin"));
        assert!(EVDEV_UINPUT_CREATE_FAILED
            .hint()
            .unwrap()
            .contains("uinput"));
    }

    #[test]
    fn driver_error_severities() {
        use crate::errors::ErrorSeverity;

        assert_eq!(DRIVER_NOT_FOUND.severity(), ErrorSeverity::Error);
        assert_eq!(
            DRIVER_DEVICE_DISCONNECTED.severity(),
            ErrorSeverity::Warning
        );
        assert_eq!(HOTPLUG_DEVICE_ADDED.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn driver_error_formatting() {
        let err = keyrx_err!(
            DRIVER_INCOMPATIBLE_VERSION,
            found = "1.0.0",
            required = "2.0.0"
        );
        assert!(err.message().contains("1.0.0"));
        assert!(err.message().contains("2.0.0"));
        assert!(err.message().contains("incompatible"));
    }

    #[test]
    fn driver_error_context_substitution() {
        let err = keyrx_err!(DRIVER_DEVICE_NOT_FOUND, device = "/dev/input/event0");
        assert_eq!(err.code(), "KRX-D3005");
        assert!(err.message().contains("/dev/input/event0"));
    }

    #[test]
    fn evdev_specific_errors() {
        let grab_err = keyrx_err!(EVDEV_DEVICE_GRAB_FAILED, device = "/dev/input/event5");
        assert!(grab_err.message().contains("/dev/input/event5"));

        let uinput_err = keyrx_err!(EVDEV_UINPUT_CREATE_FAILED, reason = "permission denied");
        assert!(uinput_err.message().contains("permission denied"));
    }

    #[test]
    fn windows_specific_errors() {
        let hook_err = keyrx_err!(WINDOWS_HOOK_FAILED, reason = "access denied");
        assert!(hook_err.message().contains("access denied"));

        let registry_err = keyrx_err!(
            WINDOWS_REGISTRY_ACCESS_FAILED,
            key = "HKLM\\Software\\KeyRx"
        );
        assert!(registry_err.message().contains("HKLM\\Software\\KeyRx"));
    }

    #[test]
    fn linux_specific_errors() {
        let module_err = keyrx_err!(LINUX_MODULE_NOT_LOADED, module = "uinput");
        assert!(module_err.message().contains("uinput"));

        let cap_err = keyrx_err!(LINUX_CAPABILITY_MISSING, capability = "CAP_SYS_ADMIN");
        assert!(cap_err.message().contains("CAP_SYS_ADMIN"));
    }

    #[test]
    fn platform_errors() {
        let platform_err = keyrx_err!(PLATFORM_NOT_SUPPORTED, platform = "macOS");
        assert!(platform_err.message().contains("macOS"));

        let hotplug_err = keyrx_err!(HOTPLUG_DEVICE_ADDED, device = "USB Keyboard");
        assert!(hotplug_err.message().contains("USB Keyboard"));
    }
}
