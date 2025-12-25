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

/// Manages mapping between Raw Input handles and device information.
#[derive(Clone)]
pub struct DeviceMap {
    devices: Arc<RwLock<HashMap<usize, DeviceInfo>>>,
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
        self.devices
            .write()
            .unwrap()
            .insert(handle as usize, info.clone());

        Ok(info)
    }

    /// Removes a device from the map.
    pub fn remove_device(&self, handle: HANDLE) {
        self.devices.write().unwrap().remove(&(handle as usize));
    }

    /// Looks up device info by handle.
    pub fn get(&self, handle: HANDLE) -> Option<DeviceInfo> {
        self.devices
            .read()
            .unwrap()
            .get(&(handle as usize))
            .cloned()
    }

    /// Gets the list of all tracked devices.
    pub fn all(&self) -> Vec<DeviceInfo> {
        self.devices.read().unwrap().values().cloned().collect()
    }

    fn get_device_path(&self, handle: HANDLE) -> Result<String, String> {
        unsafe {
            let mut size: u32 = 0;
            // First call to get size.
            if GetRawInputDeviceInfoW(
                handle as *mut c_void,
                RIDI_DEVICENAME,
                ptr::null_mut(),
                &mut size,
            ) == u32::MAX
            {
                // Error checking omitted for brevity in size query
            }

            if size == 0 {
                return Err("Failed to get device info size".to_string());
            }

            let mut buffer = vec![0u16; size as usize];
            let result = GetRawInputDeviceInfoW(
                handle as *mut c_void,
                RIDI_DEVICENAME,
                buffer.as_mut_ptr() as *mut c_void,
                &mut size,
            );

            if result == u32::MAX {
                return Err("GetRawInputDeviceInfoW failed".to_string());
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
