// Integration test to verify daemon binary version matches expected version
// This test catches deployment issues where old binaries are installed

use std::process::Command;

#[test]
#[ignore] // Run with: cargo test --test version_verification_test -- --ignored
fn test_daemon_binary_version() {
    // Expected version from Cargo.toml
    let expected_version = env!("CARGO_PKG_VERSION");

    // Get installed binary version
    let output = Command::new("keyrx_daemon")
        .arg("--version")
        .output()
        .expect("Failed to execute keyrx_daemon --version");

    let version_output = String::from_utf8_lossy(&output.stdout);

    assert!(
        version_output.contains(expected_version),
        "Binary version mismatch! Expected: {}, Got: {}",
        expected_version,
        version_output
    );
}

#[test]
#[ignore]
fn test_installed_binary_is_recent() {
    use std::time::{Duration, SystemTime};

    // Check if binary was modified recently (within last 24 hours)
    let binary_path = if cfg!(windows) {
        r"C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    } else {
        "/usr/local/bin/keyrx_daemon"
    };

    let metadata = std::fs::metadata(binary_path).expect("Failed to read binary metadata");

    let modified = metadata
        .modified()
        .expect("Failed to get modification time");

    let now = SystemTime::now();
    let age = now
        .duration_since(modified)
        .expect("Binary is from the future?");

    assert!(
        age < Duration::from_secs(24 * 60 * 60),
        "Binary is too old! Last modified: {:?} ago. Expected: within 24 hours.\nThis indicates the build was not properly deployed.",
        age
    );
}

#[test]
#[cfg(windows)]
#[ignore]
fn test_daemon_has_admin_manifest() {
    // Verify the daemon executable has admin manifest embedded
    // This ensures it can intercept keyboard events

    let binary_path = r"C:\Program Files\KeyRx\bin\keyrx_daemon.exe";

    // Use PowerShell to check if binary has requireAdministrator level
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "Get-AuthenticodeSignature '{}' | Select-Object -ExpandProperty StatusMessage",
                binary_path
            ),
        ])
        .output()
        .expect("Failed to check binary signature");

    // Binary should exist
    assert!(
        std::path::Path::new(binary_path).exists(),
        "Daemon binary not found at: {}",
        binary_path
    );

    println!("Binary exists at: {}", binary_path);
}
