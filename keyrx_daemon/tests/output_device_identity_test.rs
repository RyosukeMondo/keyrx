//! A daemon's output keyboard is uniquely named, reported in status, and never
//! grabbed as input by any instance (loop protection by name).
//!
//! Requires /dev/uinput and the `input` group; skipped otherwise.

#![cfg(target_os = "linux")]

use std::time::Duration;

use keyrx_core::config::{DeviceConfig, DeviceIdentifier};
use keyrx_daemon::platform::linux::LinuxPlatform;
use keyrx_daemon::platform::output_device::{default_output_name, OUTPUT_NAME_PREFIX};
use keyrx_daemon::platform::Platform;
use keyrx_daemon::test_utils::VirtualKeyboard;

fn all_devices(pattern: &str) -> Vec<DeviceConfig> {
    vec![DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: pattern.to_string(),
        },
        mappings: Vec::new(),
    }]
}

#[test]
fn default_output_name_is_per_instance() {
    let name = default_output_name();
    assert!(name.starts_with(OUTPUT_NAME_PREFIX));
    assert!(name.ends_with(&std::process::id().to_string()));
}

#[test]
fn another_instances_output_keyboard_is_never_grabbed() {
    keyrx_daemon::skip_if_no_uinput!();
    let tag = std::process::id();
    // Names share a pid-unique stem so the scope below matches only these.
    let scope = format!("keyrx-*-{tag}-*");
    // Looks exactly like another keyrx daemon's output device.
    let _foreign_output =
        VirtualKeyboard::create(&format!("{OUTPUT_NAME_PREFIX}id-{tag}")).expect("foreign output");
    // A genuine test keyboard in the same scope, so the scope is not empty.
    let real = VirtualKeyboard::create(&format!("keyrx-id-kbd-{tag}")).expect("keyboard");
    std::thread::sleep(Duration::from_millis(250));

    let mut platform = LinuxPlatform::scoped(&scope, &format!("keyrx-id-out-{tag}"));
    platform
        .reconfigure(&all_devices(&scope))
        .expect("reconfigure");

    let captured: Vec<String> = platform
        .list_devices()
        .expect("list")
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert!(
        captured
            .iter()
            .all(|n| !n.starts_with(OUTPUT_NAME_PREFIX) && n != "keyrx"),
        "grabbed a keyrx output device: {captured:?}"
    );
    assert!(
        captured.iter().any(|n| n == real.name()),
        "the real test keyboard must still be grabbed: {captured:?}"
    );
}

#[test]
fn platform_reports_its_output_device_name_and_node() {
    keyrx_daemon::skip_if_no_uinput!();
    let name = format!("keyrx-out-test-{}", std::process::id());
    let mut platform = LinuxPlatform::scoped("keyrx-no-such-keyboard*", &name);
    platform
        .reconfigure(&[])
        .expect("reconfigure creates output");
    let info = platform.output_device().expect("output device reported");
    assert_eq!(info.name, name);
    let path = info.path.expect("event node resolved");
    assert!(path.starts_with("/dev/input/event"), "{path}");
}
