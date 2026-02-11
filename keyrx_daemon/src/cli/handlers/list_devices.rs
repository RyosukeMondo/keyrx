//! List devices command handler.

use crate::cli::dispatcher::exit_codes;

#[cfg(target_os = "linux")]
/// Handles the `list-devices` subcommand - lists input devices.
pub fn handle_list_devices() -> Result<(), (i32, String)> {
    use crate::device_manager::enumerate_keyboards;

    // Get all keyboard devices
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("No keyboard devices found.");
        println!();
        println!("This could mean:");
        println!("  - No keyboards are connected");
        println!("  - Permission denied to read /dev/input/event* devices");
        println!();
        println!("To fix permission issues, either:");
        println!("  - Run as root (for testing only)");
        println!("  - Add your user to the 'input' group: sudo usermod -aG input $USER");
        println!("  - Install the udev rules: see docs/LINUX_SETUP.md");
        return Ok(());
    }

    println!("Available keyboard devices:");
    println!();
    println!("{:<30} {:<25} SERIAL", "PATH", "NAME");
    println!("{}", "-".repeat(80));

    for keyboard in &keyboards {
        let serial_display = keyboard.serial.as_deref().unwrap_or("-");
        println!(
            "{:<30} {:<25} {}",
            keyboard.path.display(),
            truncate_string(&keyboard.name, 24),
            serial_display
        );
    }

    println!();
    println!("Found {} keyboard device(s).", keyboards.len());
    println!();
    println!("Tip: Use patterns in your configuration to match devices:");
    println!("  - \"*\" matches all keyboards");
    println!("  - \"USB*\" matches devices with USB in name/serial");
    println!("  - Exact name match for specific devices");

    Ok(())
}

#[cfg(target_os = "linux")]
/// Truncates a string to the specified length, adding "..." if truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(not(target_os = "linux"))]
pub fn handle_list_devices() -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'list-devices' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string_no_truncation() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_string_exact_length() {
        assert_eq!(truncate_string("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_string_truncation() {
        assert_eq!(truncate_string("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_string_short_max_len() {
        assert_eq!(truncate_string("hello", 3), "hel");
    }
}
