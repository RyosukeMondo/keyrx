//! Single-instance enforcement (PID file) and web port selection.

use std::path::Path;

/// Ensure only one instance of the daemon is running.
/// Kills any existing instance before starting.
/// Returns true if an old instance was killed.
pub(super) fn ensure_single_instance(config_dir: &Path) -> bool {
    let pid_file = config_dir.join("daemon.pid");
    let mut killed = false;

    if pid_file.exists() {
        if let Some(old_pid) = std::fs::read_to_string(&pid_file)
            .ok()
            .and_then(|contents| contents.trim().parse::<u32>().ok())
        {
            log::info!("Found existing daemon (PID {}), terminating...", old_pid);
            killed = terminate_process(old_pid);
        }
        // Remove old PID file
        let _ = std::fs::remove_file(&pid_file);
    }

    // Write current PID
    let current_pid = std::process::id();
    if let Err(e) = std::fs::write(&pid_file, current_pid.to_string()) {
        log::warn!("Failed to write PID file: {}", e);
    } else {
        log::debug!("Wrote PID {} to {:?}", current_pid, pid_file);
    }

    killed
}

/// Terminates process `pid`. Returns true if it was terminated.
fn terminate_process(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    // SAFETY: the handle is checked for null and closed exactly once.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let terminated = TerminateProcess(handle, 0) != 0;
        if terminated {
            log::info!("Terminated previous daemon instance (PID {})", pid);
            // Give it a moment to clean up
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        CloseHandle(handle);
        terminated
    }
}

/// Clean up PID file on exit
pub(super) fn cleanup_pid_file(config_dir: &Path) {
    let pid_file = config_dir.join("daemon.pid");
    let _ = std::fs::remove_file(&pid_file);
}

/// Find an available port starting from the given port.
/// Tries ports in sequence: port, port+1, port+2, ... up to 10 attempts.
pub(super) fn find_available_port(start_port: u16) -> u16 {
    use std::net::TcpListener;

    (0..10)
        .map(|offset| start_port.saturating_add(offset))
        .filter(|&port| port != 0)
        // The probe listener is dropped immediately, releasing the port
        .find(|port| TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok())
        // Fallback: the original port (will fail with a clear error later)
        .unwrap_or(start_port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_file_is_written_and_cleaned_up() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!ensure_single_instance(dir.path()));
        let pid = std::fs::read_to_string(dir.path().join("daemon.pid")).unwrap();
        assert_eq!(pid, std::process::id().to_string());

        cleanup_pid_file(dir.path());
        assert!(!dir.path().join("daemon.pid").exists());
    }

    #[test]
    fn busy_port_is_skipped() {
        let busy = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = busy.local_addr().unwrap().port();
        assert_ne!(find_available_port(port), port);
    }
}
