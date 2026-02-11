use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::sync::{Arc, RwLock};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::UI::Input::{
    GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST, RIDI_DEVICENAME,
    RIM_TYPEKEYBOARD,
};

/// Information about a raw input device.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub handle: usize,
    pub path: String,
    pub serial: Option<String>,
}

impl DeviceInfo {
    /// Returns a unique device ID for this device.
    #[must_use]
    pub fn device_id(&self) -> String {
        if let Some(ref serial) = self.serial {
            if !serial.is_empty() {
                return format!("serial-{}", serial);
            }
        }
        // Fallback to path-based ID
        format!("path-{}", self.path)
    }
}

/// Manages mapping between Raw Input handles and device information.
#[derive(Clone)]
pub struct DeviceMap {
    pub devices: Arc<RwLock<HashMap<usize, DeviceInfo>>>,
}

impl Default for DeviceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceMap {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enumerates all connected keyboard devices and populates the map.
    pub fn enumerate(&self) -> Result<(), String> {
        let devices = unsafe {
            let mut device_count: u32 = 0;
            if GetRawInputDeviceList(
                ptr::null_mut(),
                &mut device_count,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            ) == u32::MAX
            {
                // It is normal for this to return -1? No, for getting size it returns 0.
                // Windows docs say: "If pRawInputDeviceList is NULL, the function returns 0"
                // But if error, it returns -1.
            }

            if device_count == 0 {
                return Ok(());
            }

            let mut devices = vec![
                RAWINPUTDEVICELIST {
                    hDevice: ptr::null_mut(),
                    dwType: 0
                };
                device_count as usize
            ];

            if GetRawInputDeviceList(
                devices.as_mut_ptr(),
                &mut device_count,
                size_of::<RAWINPUTDEVICELIST>() as u32,
            ) == u32::MAX
            {
                return Err("GetRawInputDeviceList failed".to_string());
            }
            devices
        };

        for device in devices {
            if device.dwType == RIM_TYPEKEYBOARD {
                // We don't care about the result here, just try to add
                let _ = self.add_device(device.hDevice);
            }
        }
        Ok(())
    }

    /// Adds or updates a device in the map.
    pub fn add_device(&self, handle: HANDLE) -> Result<DeviceInfo, String> {
        let path = self.get_device_path(handle)?;
        let serial = self.extract_serial(&path);

        let info = DeviceInfo {
            handle: handle as usize,
            path: path.clone(),
            serial,
        };

        // Store using handle as key (cast to usize for hashing)
        match self.devices.write() {
            Ok(mut devices) => {
                if devices.len() >= 50 && !devices.contains_key(&(handle as usize)) {
                    log::warn!(
                        "Device map limit reached (50), ignoring new device: {}",
                        info.path
                    );
                    return Err("Device map limit reached".to_string());
                }
                devices.insert(handle as usize, info.clone());
            }
            Err(_) => {
                log::error!("Device map lock poisoned in add_device");
                return Err("Device map lock poisoned".to_string());
            }
        }

        Ok(info)
    }

    /// Adds a synthetic device for testing purposes.
    /// This bypasses Win32 API calls and directly registers the device info.
    /// Adds a synthetic device for testing purposes.
    /// This bypasses Win32 API calls and directly registers the device info.
    pub fn add_synthetic_device(&self, handle: usize, path: String, serial: Option<String>) {
        let info = DeviceInfo {
            handle,
            path,
            serial,
        };
        match self.devices.write() {
            Ok(mut devices) => {
                if devices.len() >= 50 && !devices.contains_key(&handle) {
                    log::warn!(
                        "Device map limit reached (50), ignoring synthetic device: {}",
                        info.path
                    );
                    return;
                }
                devices.insert(handle, info);
            }
            Err(_) => {
                log::error!("Device map lock poisoned in add_synthetic_device");
            }
        }
    }

    /// Removes a device from the map.
    pub fn remove_device(&self, handle: HANDLE) {
        match self.devices.write() {
            Ok(mut devices) => {
                devices.remove(&(handle as usize));
            }
            Err(_) => {
                log::error!("Device map lock poisoned in remove_device");
            }
        }
    }

    /// Looks up device info by handle.
    pub fn get(&self, handle: HANDLE) -> Option<DeviceInfo> {
        match self.devices.read() {
            Ok(devices) => devices.get(&(handle as usize)).cloned(),
            Err(_) => {
                log::error!("Device map lock poisoned in get");
                None
            }
        }
    }

    /// Gets the list of all tracked devices.
    pub fn all(&self) -> Vec<DeviceInfo> {
        match self.devices.read() {
            Ok(devices) => devices.values().cloned().collect(),
            Err(_) => {
                log::error!("Device map lock poisoned in all");
                Vec::new()
            }
        }
    }

    fn get_device_path(&self, handle: HANDLE) -> Result<String, String> {
        unsafe {
            let mut size: u32 = 0;
            // First call to get size.
            if GetRawInputDeviceInfoW(handle, RIDI_DEVICENAME, ptr::null_mut(), &mut size)
                == u32::MAX
            {
                let err = std::io::Error::last_os_error();
                return Err(format!("Failed to get device info size: {}", err));
            }

            if size == 0 {
                return Err("Device has no name (size=0)".to_string());
            }

            let mut buffer = vec![0u16; size as usize];
            let result = GetRawInputDeviceInfoW(
                handle,
                RIDI_DEVICENAME,
                buffer.as_mut_ptr() as *mut c_void,
                &mut size,
            );

            if result == u32::MAX {
                let err = std::io::Error::last_os_error();
                return Err(format!("GetRawInputDeviceInfoW failed: {}", err));
            }

            let len = result as usize;
            if len > 0 {
                // Remove null terminator if present
                let actual_len = if buffer[len - 1] == 0 { len - 1 } else { len };
                String::from_utf16(&buffer[0..actual_len])
                    .map_err(|e| format!("Invalid UTF-16: {}", e))
            } else {
                Ok(String::new())
            }
        }
    }

    /// Extracts serial number or instance ID from device path.
    fn extract_serial(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('#').collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_serial_normal() {
        let device_map = DeviceMap::new();
        // Example format: \\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}
        let path = r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        let serial = device_map.extract_serial(path);
        assert_eq!(serial, Some("7&2a00c76d&0&0000".to_string()));
    }

    #[test]
    fn test_extract_serial_short_path() {
        let device_map = DeviceMap::new();
        let path = r"\\?\HID#VID_1234&PID_5678";
        let serial = device_map.extract_serial(path);
        assert_eq!(serial, None);
    }

    #[test]
    fn test_extract_serial_empty() {
        let device_map = DeviceMap::new();
        let path = "";
        let serial = device_map.extract_serial(path);
        assert_eq!(serial, None);
    }

    #[test]
    fn test_extract_serial_malformed() {
        let device_map = DeviceMap::new();
        let path = "not-a-hid-path";
        let serial = device_map.extract_serial(path);
        assert_eq!(serial, None);
    }

    #[test]
    fn test_device_map_basic_operations() {
        let map = DeviceMap::new();
        let all = map.all();
        assert!(all.is_empty());
    }

    #[test]
    fn test_synthetic_device() {
        let map = DeviceMap::new();
        map.add_synthetic_device(
            0x1234,
            "test-path".to_string(),
            Some("test-serial".to_string()),
        );

        let info = map.get(0x1234 as HANDLE).expect("Device should exist");
        assert_eq!(info.handle, 0x1234);
        assert_eq!(info.path, "test-path");
        assert_eq!(info.serial, Some("test-serial".to_string()));

        let all = map.all();
        assert_eq!(all.len(), 1);

        map.remove_device(0x1234 as HANDLE);
        assert!(map.get(0x1234 as HANDLE).is_none());
    }

    #[test]
    fn test_device_map_limit() {
        let map = DeviceMap::new();
        // Fill up to 50
        for i in 0..50 {
            map.add_synthetic_device(i, format!("path-{}", i), None);
        }
        assert_eq!(map.all().len(), 50);

        // Try to add 51st
        map.add_synthetic_device(50, "path-50".to_string(), None);
        assert_eq!(map.all().len(), 50);
        assert!(map.get(50 as HANDLE).is_none());

        // Replace existing device should work
        map.add_synthetic_device(0, "new-path-0".to_string(), None);
        assert_eq!(map.all().len(), 50);
        assert_eq!(map.get(0 as HANDLE).unwrap().path, "new-path-0");
    }

    #[test]
    fn test_device_removal() {
        let map = DeviceMap::new();
        map.add_synthetic_device(1, "path-1".to_string(), None);
        map.add_synthetic_device(2, "path-2".to_string(), None);
        assert_eq!(map.all().len(), 2);

        map.remove_device(1 as HANDLE);
        assert_eq!(map.all().len(), 1);
        assert!(map.get(1 as HANDLE).is_none());
        assert!(map.get(2 as HANDLE).is_some());
    }
}
