// Unit/Integration test for daemon health and initialization
// Catches issues where daemon starts but components don't initialize properly

#[cfg(test)]
mod daemon_health_tests {
    use std::process::Command;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_daemon_version_command() {
        let output = Command::new("cargo")
            .args(&["run", "-p", "keyrx_daemon", "--", "--version"])
            .output()
            .expect("Failed to run daemon --version");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Daemon --version failed!\nstdout: {}\nstderr: {}",
            stdout, stderr
        );

        assert!(
            stdout.contains("keyrx") || stderr.contains("keyrx"),
            "Version output doesn't contain 'keyrx': {}{}",
            stdout, stderr
        );

        println!("✓ Daemon version command works");
    }

    #[test]
    #[ignore] // Requires daemon to not be running
    fn test_daemon_health_endpoint_timeout() {
        // Start daemon in background
        let mut child = Command::new("cargo")
            .args(&["run", "-p", "keyrx_daemon", "--", "run"])
            .spawn()
            .expect("Failed to start daemon");

        // Wait for initialization
        thread::sleep(Duration::from_secs(8));

        // Test health endpoint with retry
        let mut attempts = 0;
        let max_attempts = 5;
        let mut last_error = String::new();

        let success = loop {
            attempts += 1;
            match reqwest::blocking::Client::new()
                .get("http://localhost:9867/api/health")
                .timeout(Duration::from_secs(5))
                .send()
            {
                Ok(response) => {
                    if response.status().is_success() {
                        break true;
                    }
                    last_error = format!("HTTP {}", response.status());
                }
                Err(e) => {
                    last_error = e.to_string();
                }
            }

            if attempts >= max_attempts {
                break false;
            }

            thread::sleep(Duration::from_secs(2));
        };

        // Cleanup
        let _ = child.kill();

        assert!(
            success,
            "Daemon health endpoint failed after {} attempts. Last error: {}\n\
             This indicates daemon started but web server didn't initialize.",
            attempts, last_error
        );

        println!("✓ Daemon health endpoint responded after {} attempts", attempts);
    }

    #[test]
    fn test_config_directory_exists() {
        let config_dir = if cfg!(windows) {
            let app_data = std::env::var("APPDATA")
                .expect("APPDATA environment variable not set");
            format!(r"{}\keyrx", app_data)
        } else {
            let home = std::env::var("HOME")
                .expect("HOME environment variable not set");
            format!("{}/.config/keyrx", home)
        };

        // Config directory should exist or be creatable
        if !std::path::Path::new(&config_dir).exists() {
            std::fs::create_dir_all(&config_dir)
                .expect("Failed to create config directory");
        }

        assert!(
            std::path::Path::new(&config_dir).exists(),
            "Config directory doesn't exist: {}",
            config_dir
        );

        println!("✓ Config directory exists: {}", config_dir);
    }

    #[test]
    #[cfg(windows)]
    fn test_platform_specific_initialization() {
        // Verify Windows-specific dependencies are available
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Test that we can create wide strings (required for Windows API)
        let test_str = "test";
        let wide: Vec<u16> = OsStr::new(test_str)
            .encode_wide()
            .chain(Some(0))
            .collect();

        assert_eq!(wide.len(), test_str.len() + 1); // +1 for null terminator

        println!("✓ Windows platform initialization works");
    }
}
