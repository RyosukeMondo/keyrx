//! Desktop-shell helpers: browser, About box, elevation check.

/// Check if running with administrative privileges
pub(super) fn is_admin() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // SAFETY: the token handle is checked and closed; TOKEN_ELEVATION is a
    // plain struct sized for the TokenElevation query.
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token);
        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// Opens a URL in the default web browser.
pub(super) fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()?;
    Ok(())
}

/// Show About dialog with version information
pub(super) fn show_about_dialog() {
    use crate::version;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

    let message = format!(
        "KeyRx - Advanced Keyboard Remapping\n\n\
         Version: {}\n\
         Build: {}\n\
         Commit: {}\n\n\
         Copyright © 2024 KeyRx Contributors\n\
         Licensed under AGPL-3.0-or-later",
        version::VERSION,
        version::BUILD_DATE,
        version::GIT_HASH
    );
    let message_wide = to_wide(&message);
    let title_wide = to_wide("About KeyRx");

    // SAFETY: both buffers are NUL-terminated UTF-16 and outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(), // No parent window
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

/// NUL-terminated UTF-16 for Win32 wide-string APIs.
fn to_wide(text: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_wide_is_nul_terminated() {
        assert_eq!(to_wide("ab"), vec![u16::from(b'a'), u16::from(b'b'), 0]);
    }
}
