//! Keyboard layout detection and scan-code-to-label mapping.

use axum::Json;

use crate::error::DaemonError;

use super::types::KeyboardLabelsResponse;

/// GET /api/keyboard/labels - Detect keyboard layout and return display labels
#[cfg(target_os = "windows")]
pub(super) async fn get_keyboard_labels() -> Result<Json<KeyboardLabelsResponse>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let labels = detect_keyboard_labels();
        Ok::<Json<KeyboardLabelsResponse>, DaemonError>(Json(labels))
    })
    .await
    .map_err(|e| {
        DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("keyboard-labels"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

#[cfg(not(target_os = "windows"))]
pub(super) async fn get_keyboard_labels() -> Result<Json<KeyboardLabelsResponse>, DaemonError> {
    Ok(Json(KeyboardLabelsResponse {
        detected_layout: "Unknown (non-Windows)".to_string(),
        labels: std::collections::HashMap::new(),
    }))
}

/// Detect the current keyboard layout and compute display labels
/// for each scan code.
#[cfg(target_os = "windows")]
fn detect_keyboard_labels() -> KeyboardLabelsResponse {
    use std::collections::HashMap;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyboardLayout, MapVirtualKeyW, ToUnicodeEx, MAPVK_VSC_TO_VK_EX,
    };

    let hkl = unsafe { GetKeyboardLayout(0) };

    // Extract layout ID for display name
    let layout_id = (hkl as usize) & 0xFFFF;
    let detected_layout = match layout_id {
        0x0411 => "Japanese (109-key)".to_string(),
        0x0409 => "US English (ANSI)".to_string(),
        0x0809 => "UK English (ISO)".to_string(),
        0x0407 => "German (QWERTZ)".to_string(),
        0x040C => "French (AZERTY)".to_string(),
        _ => format!("Layout 0x{:04X}", layout_id),
    };

    // Scan codes to probe (all layout-dependent keys)
    let scan_label_pairs: &[(u32, &str)] = &[
        (0x29, "Grave"),
        (0x0C, "Minus"),
        (0x0D, "Equal"),
        (0x1A, "LeftBracket"),
        (0x1B, "RightBracket"),
        (0x2B, "Backslash"),
        (0x27, "Semicolon"),
        (0x28, "Quote"),
        (0x33, "Comma"),
        (0x34, "Period"),
        (0x35, "Slash"),
        (0x02, "Num1"),
        (0x03, "Num2"),
        (0x04, "Num3"),
        (0x05, "Num4"),
        (0x06, "Num5"),
        (0x07, "Num6"),
        (0x08, "Num7"),
        (0x09, "Num8"),
        (0x0A, "Num9"),
        (0x0B, "Num0"),
    ];

    let mut labels = HashMap::new();
    let key_state = [0u8; 256];

    for &(scancode, name) in scan_label_pairs {
        let vk = unsafe { MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK_EX) };
        if vk == 0 {
            continue;
        }

        let mut buf = [0u16; 4];
        let result = unsafe {
            ToUnicodeEx(
                vk,
                scancode,
                key_state.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as i32,
                0,
                hkl,
            )
        };

        if result > 0 {
            let label = String::from_utf16_lossy(&buf[..result as usize]);
            labels.insert(name.to_string(), label);
        }
    }

    KeyboardLabelsResponse {
        detected_layout,
        labels,
    }
}
