//! Event recording command handler.

use crate::cli::dispatcher::exit_codes;
use std::path::Path;

#[cfg(target_os = "linux")]
/// Handles the `record` subcommand.
pub fn handle_record(output_path: &Path, device_path: Option<&Path>) -> Result<(), (i32, String)> {
    use keyrx_core::runtime::KeyEvent;
    use keyrx_daemon::platform::linux::evdev_to_keycode;
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::SystemTime;

    // If no device provided, list devices and return
    let Some(device_path) = device_path else {
        println!("No input device specified.");
        println!("Please choose a device from the list below and run:");
        println!(
            "  sudo keyrx_daemon record --output {} --device <PATH>",
            output_path.display()
        );
        println!();
        return crate::cli::handlers::list_devices::handle_list_devices();
    };

    println!("Preparing to record from: {}", device_path.display());

    // Open the device
    let mut device = evdev::Device::open(device_path).map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to open device {}: {}", device_path.display(), e),
        )
    })?;

    println!("Recording started. Press Ctrl+C to stop.");
    println!("Warning: Ensure keyrx_daemon is stopped.");

    // Setup signal handler
    let running = Arc::new(AtomicBool::new(true));

    if let Err(e) = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&running)) {
        eprintln!("Failed to register signal handler: {}", e);
    }

    #[derive(Serialize, Deserialize)]
    struct Metadata {
        version: String,
        timestamp: String,
        device_name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Recording {
        metadata: Metadata,
        events: Vec<KeyEvent>,
    }

    let mut captured_events = Vec::new();
    let start_time = std::time::Instant::now();

    // Event loop
    while running.load(Ordering::SeqCst) {
        match device.fetch_events() {
            Ok(iterator) => {
                for ev in iterator {
                    // Filter for key events
                    if ev.event_type() == evdev::EventType::KEY {
                        let code = ev.code();
                        let value = ev.value(); // 0=Release, 1=Press, 2=Repeat

                        if value == 2 {
                            continue;
                        } // Ignore repeats

                        if let Some(keycode) = evdev_to_keycode(code) {
                            let event_type = if value == 1 {
                                keyrx_core::runtime::KeyEventType::Press
                            } else {
                                keyrx_core::runtime::KeyEventType::Release
                            };

                            // Calculate relative time
                            let timestamp_us = start_time.elapsed().as_micros() as u64;

                            let final_event =
                                if event_type == keyrx_core::runtime::KeyEventType::Press {
                                    KeyEvent::press(keycode).with_timestamp(timestamp_us)
                                } else {
                                    KeyEvent::release(keycode).with_timestamp(timestamp_us)
                                };

                            print!("\rCaptured: {:?}     ", final_event.keycode());
                            std::io::stdout().flush().ok();

                            captured_events.push(final_event);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                // Signal received
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("\nError reading device: {}", e);
                break;
            }
        }
    }

    println!(
        "\nRecording stopped. Saving {} events...",
        captured_events.len()
    );

    let recording = Recording {
        metadata: Metadata {
            version: "1.0".to_string(),
            timestamp: humantime::format_rfc3339(SystemTime::now()).to_string(),
            device_name: device.name().unwrap_or("Unknown").to_string(),
        },
        events: captured_events,
    };

    let json = serde_json::to_string_pretty(&recording).map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to serialize recording: {}", e),
        )
    })?;

    let mut file = File::create(output_path).map_err(|e| {
        (
            exit_codes::PERMISSION_ERROR,
            format!("Failed to create output file: {}", e),
        )
    })?;

    file.write_all(json.as_bytes()).map_err(|e| {
        (
            exit_codes::RUNTIME_ERROR,
            format!("Failed to write to file: {}", e),
        )
    })?;

    println!("Saved to {}", output_path.display());
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn handle_record(_output: &Path, _device: Option<&Path>) -> Result<(), (i32, String)> {
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'record' command is only available on Linux. \
         Build with --features linux to enable."
            .to_string(),
    ))
}
