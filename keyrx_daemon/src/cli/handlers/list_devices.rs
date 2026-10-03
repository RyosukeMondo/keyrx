//! List devices command handler.

use crate::cli::dispatcher::exit_codes;

#[cfg(target_os = "linux")]
/// Handles the `list-devices` subcommand - lists input devices.
pub fn handle_list_devices(all: bool) -> Result<(), (i32, String)> {
    use crate::device_manager::enumerate_keyboards;

    // Get all keyboard devices
    let mut keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    // Software keyboards (other tools' virtual devices) are not something a
    // user remaps; keyrx's own outputs are already excluded by enumeration.
    let total = keyboards.len();
    if !all {
        keyboards.retain(|k| !k.is_virtual);
    }
    let hidden = total - keyboards.len();

    if keyboards.is_empty() {
        // Enumeration reads world-readable sysfs metadata, not /dev/input
        // itself (see device_manager::linux_enum), so an empty result means
        // no keyboard-like device is plugged in - it is not a permission
        // symptom. A device present but not *grabbable* is diagnosed below,
        // once devices are actually listed.
        println!("No keyboard devices found: no keyboard-like device is connected.");
        println!("Run `keyrx_daemon doctor` for a full diagnosis.");
        return Ok(());
    }

    println!("Available keyboard devices:");
    println!();
    println!("{:<30} {:<25} {:<10} SERIAL", "PATH", "NAME", "READABLE");
    println!("{}", "-".repeat(90));

    let mut unreadable = 0;
    for keyboard in &keyboards {
        let serial_display = keyboard.serial.as_deref().unwrap_or("-");
        let readable = is_readable(&keyboard.path);
        if !readable {
            unreadable += 1;
        }
        println!(
            "{:<30} {:<25} {:<10} {}",
            keyboard.path.display(),
            truncate_string(&keyboard.name, 24),
            if readable { "yes" } else { "no" },
            serial_display
        );
    }

    println!();
    println!("Found {} keyboard device(s).", keyboards.len());
    if hidden > 0 {
        println!("({hidden} software keyboard(s) hidden; use --all to list them.)");
    }
    if unreadable > 0 {
        println!();
        println!(
            "{unreadable} device(s) can be listed (world-readable metadata) but not opened - \
             the daemon cannot grab a device it matches unless it can open it, so its keys \
             will not be remapped."
        );
        println!(
            "  Fix: {}, then log out and back in.",
            crate::permission_advice::join_group_command("input")
        );
        println!("  Check everything at once: keyrx_daemon doctor");
    }
    println!();
    println!("Tip: Use patterns in your configuration to match devices:");
    println!("  - \"*\" matches all keyboards");
    println!("  - \"USB*\" matches devices with USB in name/serial");
    println!("  - Exact name match for specific devices");

    Ok(())
}

#[cfg(target_os = "linux")]
/// Whether this process can currently open `path` for reading, without
/// actually opening it (a real open+close costs ~15ms per device - see
/// `device_manager::linux_enum`, and this command may list many).
fn is_readable(path: &std::path::Path) -> bool {
    nix::unistd::access(path, nix::unistd::AccessFlags::R_OK).is_ok()
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
pub fn handle_list_devices(_all: bool) -> Result<(), (i32, String)> {
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
