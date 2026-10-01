//! Event recording command handler.
//!
//! Writes the SAME `EventSequence` format `simulate --events-file` reads
//! (`crate::config::simulation_engine::EventSequence`), plus a `metadata`
//! object for humans - so "record on real hardware, then replay it" is one
//! file format, not two that drifted apart (the loader ignores the unknown
//! `metadata` key).

use crate::cli::dispatcher::exit_codes;
use crate::config::simulation_engine::{EventSequence, EventType, SimulatedEvent};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata written alongside a recording, for humans reading the file.
// Written only by the Linux recorder (evdev); the format is still tested
// on every platform.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RecordingMetadata {
    pub version: String,
    pub timestamp: String,
    pub device_name: String,
}

/// What `record` writes: an [`EventSequence`] (so `simulate --events-file`
/// loads it directly) plus [`RecordingMetadata`]. `#[serde(flatten)]` puts
/// `seed`/`events` at the top level alongside `metadata`, so the file IS an
/// `EventSequence` with one extra, ignorable key.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Recording {
    pub metadata: RecordingMetadata,
    #[serde(flatten)]
    pub sequence: EventSequence,
}

/// Converts a captured [`keyrx_core::runtime::KeyEvent`] to a
/// [`SimulatedEvent`], formatting its key name with the SAME `{:?}` codec
/// `SimulationEngine` parses back (`keyrx_core::parser::validators::parse_physical_key`
/// accepts every `KeyCode`'s Debug spelling).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn key_event_to_simulated(event: &keyrx_core::runtime::KeyEvent) -> SimulatedEvent {
    SimulatedEvent {
        device_id: event.device_id().map(str::to_string),
        timestamp_us: event.timestamp_us(),
        key: format!("{:?}", event.keycode()),
        event_type: match event.event_type() {
            keyrx_core::runtime::KeyEventType::Press => EventType::Press,
            keyrx_core::runtime::KeyEventType::Release => EventType::Release,
        },
    }
}

#[cfg(target_os = "linux")]
/// Handles the `record` subcommand.
pub fn handle_record(output_path: &Path, device_path: Option<&Path>) -> Result<(), (i32, String)> {
    use crate::platform::linux::evdev_to_keycode;
    use keyrx_core::runtime::KeyEvent;
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
        return crate::cli::handlers::list_devices::handle_list_devices(false);
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

                            captured_events.push(key_event_to_simulated(&final_event));
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
        metadata: RecordingMetadata {
            version: "1.0".to_string(),
            timestamp: humantime::format_rfc3339(SystemTime::now()).to_string(),
            device_name: device.name().unwrap_or("Unknown").to_string(),
        },
        sequence: EventSequence {
            events: captured_events,
            seed: 0,
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::simulation_engine::SimulationEngine;
    use keyrx_core::runtime::KeyEvent;
    use std::io::Write;

    /// The whole point of writing `EventSequence` from `record`: a file it
    /// writes must be loadable by `simulate --events-file` unchanged. This
    /// builds a `Recording` exactly as `handle_record` would, serializes it,
    /// loads it back through `SimulationEngine::load_events_from_file`
    /// (the `metadata` key must be ignored, not rejected), and replays it.
    #[test]
    fn a_recording_round_trips_through_load_events_from_file_and_replays() {
        let events = vec![
            key_event_to_simulated(
                &KeyEvent::press(keyrx_core::config::KeyCode::A).with_timestamp(0),
            ),
            key_event_to_simulated(
                &KeyEvent::release(keyrx_core::config::KeyCode::A).with_timestamp(50_000),
            ),
        ];
        let recording = Recording {
            metadata: RecordingMetadata {
                version: "1.0".to_string(),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                device_name: "Test Keyboard".to_string(),
            },
            sequence: EventSequence {
                events: events.clone(),
                seed: 0,
            },
        };

        let json = serde_json::to_string_pretty(&recording).expect("serialize recording");
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        file.write_all(json.as_bytes()).expect("write temp file");

        let loaded = SimulationEngine::load_events_from_file(file.path())
            .expect("a recording must load as an EventSequence (metadata is ignored)");
        assert_eq!(loaded.events.len(), events.len());
        assert_eq!(loaded.events[0].key, "A");
        assert_eq!(loaded.events[0].event_type, EventType::Press);
        assert_eq!(loaded.events[1].event_type, EventType::Release);

        // And it replays through the real engine (a pass-through "*" config
        // with no mappings, so A stays A).
        let krx = write_passthrough_krx();
        let mut engine = SimulationEngine::new(krx.path()).expect("load test config");
        let output = engine.replay(&loaded).expect("replay a recorded sequence");
        assert_eq!(output.len(), 2);
        assert_eq!(output[0].key, "A");
    }

    fn write_passthrough_krx() -> tempfile::NamedTempFile {
        use keyrx_core::config::{ConfigRoot, DeviceConfig, DeviceIdentifier, Metadata, Version};
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".to_string(),
                },
                mappings: vec![],
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".to_string(),
                source_hash: "test".to_string(),
            },
        };
        let bytes = keyrx_compiler::serialize::serialize(&config).expect("serialize test config");
        let mut file = tempfile::NamedTempFile::new().expect("create temp krx file");
        file.write_all(&bytes).expect("write temp krx file");
        file
    }
}
