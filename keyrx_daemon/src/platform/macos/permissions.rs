//! macOS Accessibility permission checking.
//!
//! This module provides functions to check if the application has been
//! granted Accessibility permission, which is required for keyboard event
//! capture on macOS.
//!
//! # Safety
//!
//! Uses unsafe FFI calls to the macOS Accessibility API via the
//! `accessibility_sys` crate. The API is well-documented and stable.

/// Checks if the application has Accessibility permission.
///
/// This function uses the macOS Accessibility API to determine if the current
/// process is trusted to access accessibility features. This permission is
/// required for keyboard event capture.
///
/// # Returns
///
/// `true` if the application is trusted to access Accessibility features,
/// `false` otherwise.
///
/// # Safety
///
/// This function uses an unsafe FFI call to `AXIsProcessTrusted()`, which is
/// a stable macOS API. The function has no side effects and is safe to call.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::permissions::check_accessibility_permission;
///
/// if !check_accessibility_permission() {
///     eprintln!("Accessibility permission not granted");
/// }
/// ```
pub fn check_accessibility_permission() -> bool {
    // SAFETY: AXIsProcessTrusted() is a stable macOS API with no side effects.
    // It returns a boolean indicating whether the current process has been
    // granted Accessibility permission.
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}

/// Returns a user-friendly error message with setup instructions.
///
/// Provides step-by-step instructions for granting Accessibility permission
/// to the application. These instructions are current as of macOS Sonoma (14.0)
/// and are compatible with earlier versions.
///
/// # Returns
///
/// A string containing step-by-step instructions for granting
/// Accessibility permission, including the path through System Settings
/// and troubleshooting tips.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::platform::macos::permissions;
///
/// if !permissions::check_accessibility_permission() {
///     eprintln!("{}", permissions::get_permission_error_message());
/// }
/// ```
pub fn get_permission_error_message() -> String {
    format!(
        "Accessibility permission required but not granted.\n\
        \n\
        KeyRx requires Accessibility permission to capture and remap keyboard events.\n\
        \n\
        To grant permission:\n\
        \n\
        1. Open System Settings (System Preferences on older macOS versions)\n\
        2. Navigate to Privacy & Security > Accessibility\n\
        3. Click the lock icon and authenticate\n\
        4. Find 'keyrx_daemon' (or Terminal/your IDE if running from there)\n\
        5. Enable the toggle next to the application\n\
        6. Restart keyrx_daemon\n\
        \n\
        Note: If running from Terminal or an IDE, you must grant permission to\n\
        that application, not keyrx_daemon directly.\n\
        \n\
        For more information, see: docs/setup/macos.md\n\
        \n\
        Troubleshooting:\n\
        - If the application doesn't appear, try running it first to trigger\n\
          the permission dialog\n\
        - If permission is granted but still not working, try removing and\n\
          re-adding the application\n\
        - Ensure you're running macOS 10.9 or later"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_permission_returns_bool() {
        // Test that the function returns a boolean value
        // We can't assert true/false without actually having permission
        let result = check_accessibility_permission();
        // Just verify it returns a boolean (compilation test essentially)
        assert!(result == true || result == false);
    }

    #[test]
    fn test_get_permission_error_message_not_empty() {
        let message = get_permission_error_message();
        assert!(!message.is_empty());
    }

    #[test]
    fn test_get_permission_error_message_contains_instructions() {
        let message = get_permission_error_message();

        // Verify the message contains key setup steps
        assert!(message.contains("System Settings"));
        assert!(message.contains("Privacy & Security"));
        assert!(message.contains("Accessibility"));

        // Verify it mentions the application name
        assert!(message.contains("keyrx_daemon"));

        // Verify it has troubleshooting info
        assert!(message.contains("Troubleshooting"));

        // Verify it references documentation
        assert!(message.contains("docs/setup/macos.md"));
    }

    #[test]
    fn test_permission_error_message_is_user_friendly() {
        let message = get_permission_error_message();

        // Verify message is formatted well
        assert!(message.len() > 100); // Should be detailed
        assert!(message.contains('\n')); // Should be multi-line
        assert!(message.contains("1.")); // Should have numbered steps
        assert!(message.contains("2."));
        assert!(message.contains("3."));
    }
}
