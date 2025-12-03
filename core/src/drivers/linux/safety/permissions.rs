//! Permission checking and error reporting for Linux input devices.
//!
//! This module provides utilities for checking access to Linux input devices
//! and generating helpful error messages when permission issues are detected.
//!
//! # Permission Model
//!
//! Linux input devices typically require:
//! - Read access to `/dev/input/eventX` files (usually via `input` group membership)
//! - Write access to `/dev/uinput` for creating virtual devices
//! - Proper udev rules configuration
//!
//! # Usage
//!
//! ```no_run
//! use keyrx_core::drivers::linux::safety::permissions;
//! use std::path::Path;
//!
//! // Check if we have access to a device
//! if let Err(e) = permissions::check_device_access(Path::new("/dev/input/event3")) {
//!     eprintln!("Permission error: {}", e);
//!     // The error contains actionable hints for the user
//! }
//!
//! // Check uinput access
//! permissions::check_uinput_access()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::drivers::common::error::DriverError;
use std::fs::{self, File};
use std::io;
use std::path::Path;

/// Checks if the current process has read access to a device file.
///
/// This function attempts to check permissions without actually opening the device,
/// but will fall back to attempting an open if necessary. It provides detailed
/// error messages with actionable hints when access is denied.
///
/// # Arguments
///
/// * `device_path` - Path to the input device (e.g., `/dev/input/event3`)
///
/// # Returns
///
/// * `Ok(())` if access is available
/// * `Err(DriverError::PermissionDenied)` with helpful hints if access is denied
/// * `Err(DriverError::DeviceNotFound)` if the device doesn't exist
///
/// # Example
///
/// ```no_run
/// use keyrx_core::drivers::linux::safety::permissions;
/// use std::path::Path;
///
/// match permissions::check_device_access(Path::new("/dev/input/event3")) {
///     Ok(()) => println!("Device accessible"),
///     Err(e) => eprintln!("Cannot access device: {}", e),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)] // Infrastructure for task 10 (LinuxInputSource integration)
pub fn check_device_access(device_path: &Path) -> Result<(), DriverError> {
    // First check if the device exists
    if !device_path.exists() {
        return Err(DriverError::DeviceNotFound {
            path: device_path.to_path_buf(),
        });
    }

    // Try to open the device to check actual access
    // This is more reliable than just checking file permissions
    match File::open(device_path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            Err(create_device_permission_error(device_path))
        }
        Err(e) => Err(DriverError::Platform(e)),
    }
}

/// Checks if the current process has access to create virtual input devices via uinput.
///
/// This checks access to `/dev/uinput`, which is required for creating virtual
/// keyboard devices for event injection.
///
/// # Returns
///
/// * `Ok(())` if uinput access is available
/// * `Err(DriverError::PermissionDenied)` with helpful hints if access is denied
/// * `Err(DriverError::DeviceNotFound)` if /dev/uinput doesn't exist
///
/// # Example
///
/// ```no_run
/// use keyrx_core::drivers::linux::safety::permissions;
///
/// match permissions::check_uinput_access() {
///     Ok(()) => println!("Can create virtual devices"),
///     Err(e) => eprintln!("Cannot access uinput: {}", e),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)] // Infrastructure for task 10 (LinuxInputSource integration)
pub fn check_uinput_access() -> Result<(), DriverError> {
    let uinput_path = Path::new("/dev/uinput");

    // Check if uinput exists
    if !uinput_path.exists() {
        return Err(DriverError::VirtualDeviceError {
            message: "/dev/uinput not found. The uinput kernel module may not be loaded.\n\
                     Try: sudo modprobe uinput"
                .to_string(),
        });
    }

    // Try to open uinput with write access
    match fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(uinput_path)
    {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            Err(create_uinput_permission_error())
        }
        Err(e) => Err(DriverError::Platform(e)),
    }
}

/// Checks if the current process has access to the `/dev/input` directory.
///
/// This is a preliminary check to detect broad permission issues before
/// attempting to access individual device files.
///
/// # Returns
///
/// * `Ok(())` if the directory is readable
/// * `Err(DriverError::PermissionDenied)` if access is denied
#[allow(dead_code)] // Infrastructure for task 10 (LinuxInputSource integration)
pub fn check_input_directory_access() -> Result<(), DriverError> {
    let input_dir = Path::new("/dev/input");

    match fs::read_dir(input_dir) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            Err(DriverError::PermissionDenied {
                resource: "/dev/input".to_string(),
                hint: create_input_group_hint(),
            })
        }
        Err(e) => Err(DriverError::Platform(e)),
    }
}

/// Gets the current user's group memberships as a list of group names.
///
/// This is used to provide more specific hints about missing group membership.
#[allow(dead_code)] // Used by check_device_access and check_uinput_access
fn get_user_groups() -> Vec<String> {
    // Try to read groups from /proc/self/status
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Groups:") {
                // Parse group IDs
                let gids: Vec<u32> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();

                // Convert GIDs to names by reading /etc/group
                if let Ok(group_file) = fs::read_to_string("/etc/group") {
                    let mut groups = Vec::new();
                    for line in group_file.lines() {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 3 {
                            if let Ok(gid) = parts[2].parse::<u32>() {
                                if gids.contains(&gid) {
                                    groups.push(parts[0].to_string());
                                }
                            }
                        }
                    }
                    return groups;
                }
            }
        }
    }

    Vec::new()
}

/// Checks if the current user is a member of the 'input' group.
#[allow(dead_code)] // Used by permission error generation functions
fn is_in_input_group() -> bool {
    get_user_groups().iter().any(|g| g == "input")
}

/// Creates a detailed permission error for device access with context-aware hints.
#[allow(dead_code)] // Used by check_device_access
fn create_device_permission_error(device_path: &Path) -> DriverError {
    let in_input_group = is_in_input_group();
    let device_str = device_path.display().to_string();

    let hint = if !in_input_group {
        format!(
            "Your user is not in the 'input' group.\n\
             \n\
             To fix this:\n\
             1. Add your user to the input group:\n\
                sudo usermod -aG input $USER\n\
             \n\
             2. Log out and log back in for the change to take effect\n\
             \n\
             Or, configure a udev rule for {device_str}:\n\
             Create /etc/udev/rules.d/99-input.rules with:\n\
             KERNEL==\"event*\", SUBSYSTEM==\"input\", MODE=\"0660\", GROUP=\"input\"\n\
             \n\
             Then run: sudo udevadm control --reload-rules && sudo udevadm trigger"
        )
    } else {
        format!(
            "Permission denied even though you're in the 'input' group.\n\
             \n\
             Possible causes:\n\
             1. Group membership not yet active (need to log out/in)\n\
             2. Device has restrictive permissions\n\
             3. SELinux or AppArmor may be blocking access\n\
             \n\
             To check device permissions:\n\
             ls -l {device_str}\n\
             \n\
             To fix with udev rules:\n\
             Create /etc/udev/rules.d/99-input.rules with:\n\
             KERNEL==\"event*\", SUBSYSTEM==\"input\", MODE=\"0660\", GROUP=\"input\"\n\
             \n\
             Then run: sudo udevadm control --reload-rules && sudo udevadm trigger"
        )
    };

    DriverError::PermissionDenied {
        resource: device_str,
        hint,
    }
}

/// Creates a detailed permission error for uinput access with actionable hints.
#[allow(dead_code)] // Used by check_uinput_access
fn create_uinput_permission_error() -> DriverError {
    let in_input_group = is_in_input_group();

    let hint = if !in_input_group {
        "Your user is not in the 'input' group.\n\
         \n\
         To fix this:\n\
         1. Add your user to the input group:\n\
            sudo usermod -aG input $USER\n\
         \n\
         2. Create a udev rule for uinput:\n\
            Create /etc/udev/rules.d/99-uinput.rules with:\n\
            KERNEL==\"uinput\", SUBSYSTEM==\"misc\", MODE=\"0660\", GROUP=\"input\"\n\
         \n\
         3. Reload udev rules:\n\
            sudo udevadm control --reload-rules && sudo udevadm trigger\n\
         \n\
         4. Load the uinput module:\n\
            sudo modprobe uinput\n\
         \n\
         5. Log out and log back in for group changes to take effect"
            .to_string()
    } else {
        "Permission denied for /dev/uinput even though you're in the 'input' group.\n\
         \n\
         To fix this:\n\
         1. Create a udev rule for uinput:\n\
            Create /etc/udev/rules.d/99-uinput.rules with:\n\
            KERNEL==\"uinput\", SUBSYSTEM==\"misc\", MODE=\"0660\", GROUP=\"input\"\n\
         \n\
         2. Reload udev rules:\n\
            sudo udevadm control --reload-rules && sudo udevadm trigger\n\
         \n\
         3. Ensure uinput module is loaded:\n\
            sudo modprobe uinput\n\
            \n\
            To load it automatically on boot, add 'uinput' to /etc/modules\n\
         \n\
         4. Check current permissions:\n\
            ls -l /dev/uinput"
            .to_string()
    };

    DriverError::PermissionDenied {
        resource: "/dev/uinput".to_string(),
        hint,
    }
}

/// Creates the standard hint for adding a user to the input group.
#[allow(dead_code)] // Used by check_input_directory_access
fn create_input_group_hint() -> String {
    let username = std::env::var("USER").unwrap_or_else(|_| "$USER".to_string());

    format!(
        "To access input devices, add your user to the 'input' group:\n\
         \n\
         sudo usermod -aG input {username}\n\
         \n\
         Then log out and log back in for the change to take effect.\n\
         \n\
         Alternatively, run with elevated privileges (not recommended)."
    )
}

/// Checks all necessary permissions for running the Linux input driver.
///
/// This is a comprehensive check that validates:
/// - Access to the `/dev/input` directory
/// - Access to `/dev/uinput` for virtual device creation
///
/// Use this function during driver initialization to provide early, clear
/// feedback about permission issues before attempting device operations.
///
/// # Returns
///
/// * `Ok(())` if all required permissions are available
/// * `Err(DriverError)` with detailed hints if any permission check fails
///
/// # Example
///
/// ```no_run
/// use keyrx_core::drivers::linux::safety::permissions;
///
/// // Check all permissions during driver initialization
/// if let Err(e) = permissions::check_all_permissions() {
///     eprintln!("Permission check failed: {}", e);
///     eprintln!("\n{}", e.suggested_action());
///     std::process::exit(1);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)] // Infrastructure for task 10 (LinuxInputSource integration)
pub fn check_all_permissions() -> Result<(), DriverError> {
    // Check basic input directory access
    check_input_directory_access()?;

    // Check uinput access (required for event injection)
    check_uinput_access()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_device_returns_not_found() {
        let result = check_device_access(Path::new("/dev/input/event999"));
        assert!(matches!(result, Err(DriverError::DeviceNotFound { .. })));
    }

    #[test]
    fn get_user_groups_returns_list() {
        let groups = get_user_groups();
        // Should at least return some groups on a real system
        // In test environment might be empty, so we just check it doesn't panic
        let _ = groups.len();
    }

    #[test]
    fn is_in_input_group_does_not_panic() {
        // Just ensure it doesn't panic when checking
        let _ = is_in_input_group();
    }

    #[test]
    fn input_group_hint_contains_usermod() {
        let hint = create_input_group_hint();
        assert!(hint.contains("usermod"));
        assert!(hint.contains("input"));
    }

    #[test]
    fn device_permission_error_has_useful_hints() {
        let err = create_device_permission_error(Path::new("/dev/input/event0"));
        match err {
            DriverError::PermissionDenied { hint, .. } => {
                assert!(hint.contains("usermod") || hint.contains("udev"));
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn uinput_permission_error_has_useful_hints() {
        let err = create_uinput_permission_error();
        match err {
            DriverError::PermissionDenied { hint, .. } => {
                assert!(hint.contains("uinput"));
                assert!(hint.contains("udev") || hint.contains("modprobe"));
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn check_all_permissions_returns_result() {
        // This will fail in most test environments, but shouldn't panic
        let _ = check_all_permissions();
    }
}
