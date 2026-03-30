//! Windows IME state detection via IMM32 and keyboard layout APIs.
//!
//! Uses two approaches for IME open/close detection:
//! 1. `ImmGetDefaultIMEWnd` + `SendMessage(WM_IME_CONTROL, IMC_GETOPENSTATUS)`
//!    (reliable on Windows 11 new IME, used by AutoHotkey/Yamy)
//! 2. `GetKeyboardLayout` low word for input language detection

use keyrx_core::config::ImeState;
use std::collections::HashMap;
use windows_sys::Win32::UI::Input::Ime::{
    ImmGetContext, ImmGetConversionStatus, ImmGetDefaultIMEWnd, ImmGetOpenStatus, ImmReleaseContext,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, SendMessageW,
};

/// WM_IME_CONTROL message
const WM_IME_CONTROL: u32 = 0x0283;
/// IMC_GETOPENSTATUS subcommand
const IMC_GETOPENSTATUS: usize = 0x0005;

/// Query the current IME state on Windows.
///
/// Uses the SendMessage(WM_IME_CONTROL) approach for reliable detection
/// on Windows 11's new TSF-based IME. Falls back gracefully if no
/// foreground window or IME window exists.
pub fn query_windows_ime_state() -> ImeState {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return ImeState::default();
        }

        let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        if thread_id == 0 {
            return ImeState::default();
        }

        // Get keyboard layout for the foreground thread
        let hkl = GetKeyboardLayout(thread_id);
        let lang_id = (hkl as usize & 0xFFFF) as u16;
        let language = langid_to_bcp47(lang_id);

        // Get the default IME window for the foreground window
        // and query open status via WM_IME_CONTROL message.
        // This is more reliable than ImmGetOpenStatus on Windows 11.
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        let active = if !ime_hwnd.is_null() {
            SendMessageW(ime_hwnd, WM_IME_CONTROL, IMC_GETOPENSTATUS, 0) != 0
        } else {
            false
        };

        ImeState { active, language }
    }
}

/// Query detailed IME state for diagnostics.
pub fn query_windows_ime_debug() -> HashMap<String, serde_json::Value> {
    use serde_json::json;
    let mut result = HashMap::new();

    unsafe {
        let hwnd = GetForegroundWindow();
        result.insert("foreground_hwnd".into(), json!(format!("{:?}", hwnd)));

        if hwnd.is_null() {
            result.insert("error".into(), json!("No foreground window"));
            return result;
        }

        let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        result.insert("thread_id".into(), json!(thread_id));

        let hkl = GetKeyboardLayout(thread_id);
        let lang_id = (hkl as usize & 0xFFFF) as u16;
        result.insert("hkl".into(), json!(format!("0x{:08X}", hkl as usize)));
        result.insert("lang_id".into(), json!(format!("0x{:04X}", lang_id)));
        result.insert("language".into(), json!(langid_to_bcp47(lang_id)));

        // Method 1: SendMessage to default IME window
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        result.insert("ime_hwnd".into(), json!(format!("{:?}", ime_hwnd)));
        if !ime_hwnd.is_null() {
            let open = SendMessageW(ime_hwnd, WM_IME_CONTROL, IMC_GETOPENSTATUS, 0);
            result.insert("sendmsg_open_status".into(), json!(open));
        }

        // Method 2: ImmGetOpenStatus
        let himc = ImmGetContext(hwnd);
        result.insert("himc".into(), json!(format!("{:?}", himc)));
        if !himc.is_null() {
            let imm_open = ImmGetOpenStatus(himc);
            result.insert("imm_open_status".into(), json!(imm_open));

            let mut conversion = 0u32;
            let mut sentence = 0u32;
            ImmGetConversionStatus(himc, &mut conversion, &mut sentence);
            result.insert(
                "conversion_mode".into(),
                json!(format!("0x{:08X}", conversion)),
            );
            result.insert("sentence_mode".into(), json!(format!("0x{:08X}", sentence)));

            ImmReleaseContext(hwnd, himc);
        }

        // Combined result
        let state = query_windows_ime_state();
        result.insert("active".into(), json!(state.active));
        result.insert("platform".into(), json!("windows"));
    }

    result
}

/// Convert a Windows LANGID to a BCP 47 language tag.
///
/// Maps the primary language ID (low 10 bits) to standard tags.
/// Falls back to hex LANGID for unmapped languages.
fn langid_to_bcp47(langid: u16) -> String {
    // Check full LANGID first for sub-language specifics
    match langid {
        0x0404 => return "zh-TW".to_string(),
        0x0804 => return "zh-CN".to_string(),
        0x0C04 => return "zh-HK".to_string(),
        0x1004 => return "zh-SG".to_string(),
        _ => {}
    }

    // Primary language ID (low 10 bits)
    let primary = langid & 0x3FF;
    match primary {
        0x0011 => "ja".to_string(),
        0x0012 => "ko".to_string(),
        0x0004 => "zh".to_string(),
        0x0009 => "en".to_string(),
        0x000A => "es".to_string(),
        0x000C => "fr".to_string(),
        0x0007 => "de".to_string(),
        0x0010 => "it".to_string(),
        0x0016 => "pt".to_string(),
        0x0019 => "ru".to_string(),
        _ => format!("0x{:04X}", langid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_langid_japanese() {
        assert_eq!(langid_to_bcp47(0x0411), "ja");
    }

    #[test]
    fn test_langid_korean() {
        assert_eq!(langid_to_bcp47(0x0412), "ko");
    }

    #[test]
    fn test_langid_chinese_simplified() {
        assert_eq!(langid_to_bcp47(0x0804), "zh-CN");
    }

    #[test]
    fn test_langid_chinese_traditional() {
        assert_eq!(langid_to_bcp47(0x0404), "zh-TW");
    }

    #[test]
    fn test_langid_english() {
        assert_eq!(langid_to_bcp47(0x0409), "en");
    }

    #[test]
    fn test_langid_unknown_fallback() {
        let result = langid_to_bcp47(0x9999);
        assert!(result.starts_with("0x"));
    }
}
