//! Configuration validation command handler.

use crate::cli::dispatcher::exit_codes;
use std::path::Path;

#[cfg(target_os = "linux")]
/// Handles the `validate` subcommand - validates config without grabbing.
pub fn handle_validate(config_path: &Path) -> Result<(), (i32, String)> {
    use crate::config_loader::load_config;
    use crate::device_manager::{enumerate_keyboards, match_device};

    println!("Validating configuration: {}", config_path.display());
    println!();

    // Step 1: Load and validate the configuration
    println!("1. Loading configuration...");
    let config = load_config(config_path).map_err(|e| {
        (
            exit_codes::CONFIG_ERROR,
            format!("Failed to load configuration: {}", e),
        )
    })?;

    println!(
        "   Configuration loaded: {} device pattern(s)",
        config.devices.len()
    );

    // Print the device patterns
    for (i, device_config) in config.devices.iter().enumerate() {
        println!(
            "   [{:>2}] Pattern: \"{}\" ({} mapping(s))",
            i + 1,
            device_config.identifier.pattern,
            device_config.mappings.len()
        );
    }
    println!();

    // Step 2: Enumerate keyboard devices
    println!("2. Enumerating keyboard devices...");
    let keyboards = enumerate_keyboards().map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to enumerate devices: {}", e),
        )
    })?;

    if keyboards.is_empty() {
        println!("   No keyboard devices found.");
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

    println!("   Found {} keyboard device(s)", keyboards.len());
    println!();

    // Step 3: Match devices against patterns
    println!("3. Matching devices to configuration patterns...");
    println!();

    let mut matched_count = 0;
    let mut unmatched_devices = Vec::new();

    for keyboard in &keyboards {
        // Check each pattern in order (priority)
        let mut matched_pattern: Option<&str> = None;

        for device_config in config.devices.iter() {
            let pattern = device_config.identifier.pattern.as_str();
            if match_device(keyboard, pattern) {
                matched_pattern = Some(pattern);
                break; // First match wins (priority ordering)
            }
        }

        if let Some(pattern) = matched_pattern {
            println!(
                "   [MATCH] {} -> pattern \"{}\"",
                keyboard.path.display(),
                pattern
            );
            println!("           Name: {}", keyboard.name);
            if let Some(ref serial) = keyboard.serial {
                println!("           Serial: {}", serial);
            }
            matched_count += 1;
        } else {
            unmatched_devices.push(keyboard);
        }
    }

    println!();

    // Show unmatched devices as warnings
    if !unmatched_devices.is_empty() {
        println!("   Unmatched devices (will not be remapped):");
        for device in &unmatched_devices {
            println!("   [SKIP]  {}", device.path.display());
            println!("           Name: {}", device.name);
        }
        println!();
    }

    // Final result
    println!("{}", "=".repeat(60));
    if matched_count > 0 {
        println!(
            "RESULT: Configuration is valid. {} of {} device(s) matched.",
            matched_count,
            keyboards.len()
        );
        println!();
        println!(
            "Run 'keyrx_daemon run --config {}' to start remapping.",
            config_path.display()
        );
    } else {
        println!("WARNING: Configuration is valid, but no devices matched any pattern.");
        println!();
        println!(
            "Check your device patterns. Use 'keyrx_daemon list-devices' to see available devices."
        );
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn handle_validate(_config_path: &Path) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'validate' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}
