//! Linux-specific device serial number extraction.
//!
//! This module extracts serial numbers from Linux evdev devices using:
//! 1. EVIOCGUNIQ ioctl to get USB unique ID (preferred)
//! 2. udev sysfs properties (serial, uniq) (fallback)
//! 3. Synthetic ID based on physical path hash (last resort)
//!
//! The extraction strategy ensures stable device identification across reboots,
//! even for devices without manufacturer-assigned serial numbers.

use anyhow::{Context, Result};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Extract serial number from a Linux evdev device path.
///
/// Attempts multiple methods in order of preference:
/// 1. EVIOCGUNIQ ioctl (via evdev crate) - reads USB unique ID
/// 2. udev sysfs properties - reads serial/uniq from device properties
/// 3. Synthetic ID - generates stable hash from physical path
///
/// # Arguments
/// * `device_path` - Path to evdev device (e.g., `/dev/input/event0`)
///
/// # Returns
/// Serial number string or error if all extraction methods fail.
///
/// # Examples
/// ```ignore
/// let serial = extract_serial_number(Path::new("/dev/input/event0"))?;
/// ```
pub fn extract_serial_number(device_path: &Path) -> Result<String> {
    // Try EVIOCGUNIQ ioctl first (preferred method)
    if let Ok(serial) = read_eviocguniq(device_path) {
        if !serial.is_empty() && serial != "0" {
            tracing::debug!(
                path = %device_path.display(),
                serial,
                method = "EVIOCGUNIQ",
                "Extracted serial number via ioctl"
            );
            return Ok(serial);
        }
    }

    // Try udev sysfs properties
    if let Ok(serial) = read_udev_serial(device_path) {
        if !serial.is_empty() && serial != "0" {
            tracing::debug!(
                path = %device_path.display(),
                serial,
                method = "udev",
                "Extracted serial number from sysfs"
            );
            return Ok(serial);
        }
    }

    // Fallback to synthetic ID based on physical path
    let synthetic_id = generate_synthetic_id(device_path)?;
    tracing::debug!(
        path = %device_path.display(),
        serial = synthetic_id,
        method = "synthetic",
        "Generated synthetic ID from physical path"
    );
    Ok(synthetic_id)
}

/// Read USB unique ID using EVIOCGUNIQ ioctl via evdev crate.
///
/// This is the preferred method as it reads the actual USB unique ID
/// that devices report through their descriptors.
///
/// # Errors
/// Returns error if:
/// - Device cannot be opened
/// - Device doesn't support EVIOCGUNIQ (non-USB devices)
/// - Unique ID is not available
fn read_eviocguniq(device_path: &Path) -> Result<String> {
    let device =
        evdev::Device::open(device_path).context("Failed to open evdev device for EVIOCGUNIQ")?;

    // evdev crate exposes unique_name() which uses EVIOCGUNIQ internally
    device
        .unique_name()
        .map(|s| s.to_string())
        .context("Device does not provide unique ID via EVIOCGUNIQ")
}

/// Read serial number from udev sysfs properties.
///
/// Attempts to read the serial number from sysfs device properties.
/// This works for devices where udev has populated the serial/uniq attributes.
///
/// The sysfs path structure is:
/// `/sys/class/input/event0/device/` contains properties like:
/// - `uniq` - Unique identifier (often USB serial)
/// - `name` - Device name
/// - `phys` - Physical path
///
/// # Errors
/// Returns error if:
/// - sysfs path cannot be constructed
/// - Property files cannot be read
/// - All property files are empty or missing
fn read_udev_serial(device_path: &Path) -> Result<String> {
    // Extract event number from device path (e.g., event0 -> 0)
    let event_name = device_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid device path")?;

    if !event_name.starts_with("event") {
        anyhow::bail!("Device path does not contain event number: {}", event_name);
    }

    // Construct sysfs path: /sys/class/input/eventN/device/
    let sysfs_device_path = format!("/sys/class/input/{}/device", event_name);
    let device_dir = Path::new(&sysfs_device_path);

    // Try to read 'uniq' property first (most reliable for USB devices)
    if let Ok(uniq) = fs::read_to_string(device_dir.join("uniq")) {
        let trimmed = uniq.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    // If uniq is not available, try reading from parent device
    // Follow the 'device' symlink to get to the actual USB device
    if let Ok(real_path) = fs::read_link(device_dir) {
        // Navigate to the USB device directory which may have 'serial' attribute
        let mut current = device_dir.join(&real_path);

        // Walk up the device hierarchy looking for a serial attribute
        for _ in 0..5 {
            if let Ok(serial) = fs::read_to_string(current.join("serial")) {
                let trimmed = serial.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }

            // Try 'uniq' at this level too
            if let Ok(uniq) = fs::read_to_string(current.join("uniq")) {
                let trimmed = uniq.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }

            // Move up to parent
            if !current.pop() {
                break;
            }
        }
    }

    anyhow::bail!("No serial number found in udev sysfs properties")
}

/// Generate synthetic ID based on physical path hash.
///
/// Creates a stable identifier by hashing the device's physical path.
/// The physical path represents the device's position in the USB topology
/// (bus, port, etc.), which remains stable across reboots for devices
/// plugged into the same port.
///
/// This is used as a last resort for devices that don't provide serial numbers.
///
/// # Arguments
/// * `device_path` - Path to evdev device
///
/// # Returns
/// Hex-encoded hash of the physical path (e.g., "phys_a3f5b9c2")
///
/// # Errors
/// Returns error if:
/// - Device cannot be opened
/// - Physical path cannot be read from sysfs
fn generate_synthetic_id(device_path: &Path) -> Result<String> {
    // Extract event name
    let event_name = device_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid device path")?;

    // Read physical path from sysfs
    let sysfs_phys_path = format!("/sys/class/input/{}/device/phys", event_name);
    let phys = fs::read_to_string(&sysfs_phys_path)
        .with_context(|| format!("Failed to read physical path from {}", sysfs_phys_path))?;

    let phys = phys.trim();
    if phys.is_empty() {
        anyhow::bail!("Physical path is empty");
    }

    // Generate stable hash from physical path
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    phys.hash(&mut hasher);
    let hash = hasher.finish();

    // Format as hex with "phys_" prefix to indicate synthetic ID
    Ok(format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_synthetic_id_stability() {
        // Synthetic ID should be deterministic for the same physical path
        let phys1 = "usb-0000:00:14.0-1/input0";
        let phys2 = "usb-0000:00:14.0-1/input0";

        let mut hasher1 = DefaultHasher::new();
        phys1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        phys2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2, "Same physical path should produce same hash");
    }

    #[test]
    fn test_synthetic_id_uniqueness() {
        // Different physical paths should produce different hashes
        let phys1 = "usb-0000:00:14.0-1/input0";
        let phys2 = "usb-0000:00:14.0-2/input0";

        let mut hasher1 = DefaultHasher::new();
        phys1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        phys2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_ne!(
            hash1, hash2,
            "Different physical paths should produce different hashes"
        );
    }

    #[test]
    fn test_synthetic_id_format() {
        // Test that synthetic ID has correct format
        let phys = "usb-0000:00:14.0-1/input0";
        let mut hasher = DefaultHasher::new();
        phys.hash(&mut hasher);
        let hash = hasher.finish();
        let synthetic_id = format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32);

        assert!(synthetic_id.starts_with("phys_"));
        assert_eq!(synthetic_id.len(), 13); // "phys_" (5) + 8 hex chars
    }

    // Note: Integration tests with real devices are in tests/identity_tests.rs
    // to avoid requiring actual hardware in unit tests.

    // The following tests would require real hardware or mocking:
    // - test_read_eviocguniq_with_real_device
    // - test_read_udev_serial_with_real_device
    // - test_extract_serial_number_priority
}
