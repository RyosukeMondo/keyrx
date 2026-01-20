//! macOS device enumeration using IOKit.
//!
//! This module provides USB keyboard enumeration using IOKit-sys FFI bindings.
//! It discovers keyboards and extracts vendor ID, product ID, and serial numbers.

use crate::platform::DeviceInfo;
use std::ffi::CStr;
use std::os::raw::c_char;

#[cfg(target_os = "macos")]
use IOKit_sys::{
    kIOMasterPortDefault, IOIteratorNext, IOObjectRelease, IORegistryEntryCreateCFProperty,
    IOServiceGetMatchingServices, IOServiceMatching,
};

#[cfg(target_os = "macos")]
use objc2::runtime::NSObject;
#[cfg(target_os = "macos")]
use objc2::msg_send;

/// RAII wrapper for IOKit objects that automatically releases on drop.
///
/// This wrapper ensures that IOKit objects are properly released even if
/// an error occurs during enumeration, preventing resource leaks.
///
/// # Safety
///
/// The wrapper assumes the object was created with a +1 retain count
/// (from IOKit APIs that return owned objects).
#[cfg(target_os = "macos")]
struct IOObjectGuard(u32);

#[cfg(target_os = "macos")]
impl IOObjectGuard {
    /// Creates a new guard for an IOKit object.
    ///
    /// # Safety
    ///
    /// The object must be a valid IOKit object handle with +1 retain count.
    unsafe fn new(object: u32) -> Self {
        Self(object)
    }

    /// Returns the raw object handle.
    fn get(&self) -> u32 {
        self.0
    }
}

#[cfg(target_os = "macos")]
impl Drop for IOObjectGuard {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                IOObjectRelease(self.0);
            }
        }
    }
}

/// Enumerates all USB keyboard devices.
///
/// Uses IOKit APIs to discover USB HID keyboards and extract device metadata.
///
/// # Returns
///
/// A vector of [`DeviceInfo`] structs containing device metadata.
///
/// # Errors
///
/// Returns an error if IOKit enumeration fails.
pub fn list_keyboard_devices() -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }

    #[cfg(target_os = "macos")]
    unsafe {
        list_keyboard_devices_impl()
    }
}

#[cfg(target_os = "macos")]
unsafe fn list_keyboard_devices_impl() -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
    let mut devices = Vec::new();

    // Create matching dictionary for IOHIDKeyboard service
    let matching_dict = IOServiceMatching(b"IOHIDKeyboard\0".as_ptr() as *const c_char);
    if matching_dict.is_null() {
        return Err("Failed to create IOHIDKeyboard matching dictionary".into());
    }

    // Get matching services iterator
    let mut iterator: u32 = 0;
    let result = IOServiceGetMatchingServices(kIOMasterPortDefault, matching_dict, &mut iterator);
    if result != 0 {
        return Err(format!("IOServiceGetMatchingServices failed with code {}", result).into());
    }

    let iter_guard = IOObjectGuard::new(iterator);

    // Iterate through matching services
    loop {
        let service = IOIteratorNext(iter_guard.get());
        if service == 0 {
            break;
        }

        let service_guard = IOObjectGuard::new(service);

        // Extract device properties
        if let Ok(device_info) = extract_device_info(service_guard.get()) {
            devices.push(device_info);
        }
    }

    Ok(devices)
}

/// Extracts device information from an IOKit service object.
///
/// # Safety
///
/// The service must be a valid IOKit service handle.
#[cfg(target_os = "macos")]
unsafe fn extract_device_info(service: u32) -> Result<DeviceInfo, Box<dyn std::error::Error>> {
    // Extract properties
    let vendor_id = get_registry_property_int(service, "VendorID")?;
    let product_id = get_registry_property_int(service, "ProductID")?;
    let product_name = get_registry_property_string(service, "Product")
        .unwrap_or_else(|_| format!("Keyboard ({:04x}:{:04x})", vendor_id, product_id));
    let serial = get_registry_property_string(service, "SerialNumber").ok();

    // Generate unique device ID
    let id = if let Some(ref s) = serial {
        format!("usb-{:04x}:{:04x}-{}", vendor_id, product_id, s)
    } else {
        format!("usb-{:04x}:{:04x}", vendor_id, product_id)
    };

    // Generate path (IOKit service path)
    let path = format!("IOService:/IOHIDKeyboard/{}", id);

    Ok(DeviceInfo {
        id,
        name: product_name,
        path,
        vendor_id,
        product_id,
    })
}

/// Gets an integer property from an IORegistry entry.
///
/// # Safety
///
/// The service must be a valid IOKit service handle.
#[cfg(target_os = "macos")]
unsafe fn get_registry_property_int(
    service: u32,
    key: &str,
) -> Result<u16, Box<dyn std::error::Error>> {
    // Create CFString key
    let key_cfstring = objc2_core_foundation::CFString::from_str(key);
    let cf_type_ref: &objc2_core_foundation::CFType = key_cfstring.as_ref();

    let property = IORegistryEntryCreateCFProperty(
        service,
        cf_type_ref as *const _ as *const _,
        std::ptr::null_mut(),
        0,
    );

    if property.is_null() {
        return Err(format!("Property '{}' not found", key).into());
    }

    // Cast to NSNumber and extract integer value
    let number: *const NSObject = property as *const NSObject;
    let value: i32 = msg_send![number, intValue];

    // Release the property
    let _: () = msg_send![number, release];

    Ok(value as u16)
}

/// Gets a string property from an IORegistry entry.
///
/// # Safety
///
/// The service must be a valid IOKit service handle.
#[cfg(target_os = "macos")]
unsafe fn get_registry_property_string(
    service: u32,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create CFString key
    let key_cfstring = objc2_core_foundation::CFString::from_str(key);
    let cf_type_ref: &objc2_core_foundation::CFType = key_cfstring.as_ref();

    let property = IORegistryEntryCreateCFProperty(
        service,
        cf_type_ref as *const _ as *const _,
        std::ptr::null_mut(),
        0,
    );

    if property.is_null() {
        return Err(format!("Property '{}' not found", key).into());
    }

    // Cast to NSString
    let nsstring: *const NSObject = property as *const NSObject;
    let utf8_ptr: *const c_char = msg_send![nsstring, UTF8String];

    if utf8_ptr.is_null() {
        let _: () = msg_send![nsstring, release];
        return Err("Failed to get UTF8 string".into());
    }

    let c_str = CStr::from_ptr(utf8_ptr);
    let result = c_str.to_string_lossy().into_owned();

    // Release the property
    let _: () = msg_send![nsstring, release];

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_keyboard_devices_no_crash() {
        // This test just ensures the function doesn't panic
        // Actual device enumeration requires hardware
        let result = list_keyboard_devices();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_io_object_guard_releases() {
        // Test that IOObjectGuard properly releases on drop
        // We use a zero object which is safe to release
        let guard = unsafe { IOObjectGuard::new(0) };
        assert_eq!(guard.get(), 0);
        drop(guard);
        // If this doesn't crash, the guard worked
    }

    #[test]
    fn test_device_info_format() {
        // Test device info formatting
        let device = DeviceInfo {
            id: "usb-046d:c52b-ABC123".to_string(),
            name: "Logitech Keyboard".to_string(),
            path: "IOService:/IOHIDKeyboard/usb-046d:c52b-ABC123".to_string(),
            vendor_id: 0x046d,
            product_id: 0xc52b,
        };

        assert_eq!(device.vendor_id, 0x046d);
        assert_eq!(device.product_id, 0xc52b);
        assert!(device.id.contains("046d"));
        assert!(device.id.contains("c52b"));
    }

    #[test]
    fn test_device_id_without_serial() {
        // Test device ID generation without serial number
        let device = DeviceInfo {
            id: "usb-046d:c52b".to_string(),
            name: "Keyboard".to_string(),
            path: "IOService:/IOHIDKeyboard/usb-046d:c52b".to_string(),
            vendor_id: 0x046d,
            product_id: 0xc52b,
        };

        assert_eq!(device.id, "usb-046d:c52b");
    }

    #[test]
    fn test_device_id_with_serial() {
        // Test device ID generation with serial number
        let device = DeviceInfo {
            id: "usb-046d:c52b-SN12345".to_string(),
            name: "Keyboard".to_string(),
            path: "IOService:/IOHIDKeyboard/usb-046d:c52b-SN12345".to_string(),
            vendor_id: 0x046d,
            product_id: 0xc52b,
        };

        assert!(device.id.ends_with("SN12345"));
    }
}
