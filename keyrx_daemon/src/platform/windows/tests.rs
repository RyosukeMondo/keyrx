#[cfg(test)]
mod tests {
    use crate::platform::windows::keycode::{keycode_to_vk, vk_to_keycode};
    use keyrx_core::config::KeyCode;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn test_vk_mapping_completeness() {
        // Test a few common keys
        assert_eq!(vk_to_keycode(VK_A as u16), Some(KeyCode::A));
        assert_eq!(vk_to_keycode(VK_SPACE as u16), Some(KeyCode::Space));
        assert_eq!(vk_to_keycode(VK_LSHIFT as u16), Some(KeyCode::LShift));

        assert_eq!(keycode_to_vk(KeyCode::A), Some(VK_A as u16));
        assert_eq!(keycode_to_vk(KeyCode::Space), Some(VK_SPACE as u16));
        assert_eq!(keycode_to_vk(KeyCode::LShift), Some(VK_LSHIFT as u16));
    }

    #[test]
    fn test_unmapped_vk() {
        assert_eq!(vk_to_keycode(0x07), None); // Undefined VK code
    }

    #[test]
    fn test_extended_keys() {
        use crate::platform::windows::inject::is_extended_key;
        assert!(is_extended_key(VK_RMENU as u16));
        assert!(is_extended_key(VK_RCONTROL as u16));
        assert!(is_extended_key(VK_LEFT as u16));
        assert!(!is_extended_key(VK_A as u16));
    }

    #[test]
    fn test_platform_trait_usage() {
        use crate::platform::{Platform, WindowsPlatform};

        // Verify that WindowsPlatform can be used as a trait object (Box<dyn Platform>)
        let platform: Box<dyn Platform> = Box::new(WindowsPlatform::new());

        // This compile-time check verifies that the trait implementation is correct
        let _ = platform;
    }

    #[test]
    fn test_parse_vid_pid() {
        use super::parse_vid_pid;

        // Test with a typical Windows HID device path
        let path = r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        let (vendor_id, product_id) = parse_vid_pid(path);
        assert_eq!(vendor_id, 0x046D);
        assert_eq!(product_id, 0xC52B);

        // Test with partial path
        let path2 = r"\\?\HID#VID_1234&PID_5678";
        let (vendor_id2, product_id2) = parse_vid_pid(path2);
        assert_eq!(vendor_id2, 0x1234);
        assert_eq!(product_id2, 0x5678);

        // Test with no VID/PID
        let path3 = r"\\?\HID#SOMEDEVICE";
        let (vendor_id3, product_id3) = parse_vid_pid(path3);
        assert_eq!(vendor_id3, 0);
        assert_eq!(product_id3, 0);
    }

    #[test]
    fn test_convert_device_info() {
        use super::convert_device_info;
        use crate::platform::windows::device_map::DeviceInfo;

        let device = DeviceInfo {
            handle: 0x1234,
            path: r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}".to_string(),
            serial: Some("7&2a00c76d&0&0000".to_string()),
        };

        let common = convert_device_info(&device);
        assert_eq!(common.id, "serial-7&2a00c76d&0&0000");
        assert_eq!(common.vendor_id, 0x046D);
        assert_eq!(common.product_id, 0xC52B);
        assert_eq!(common.path, device.path);
    }
}
