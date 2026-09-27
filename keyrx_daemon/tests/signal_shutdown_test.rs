//! SIGTERM must request a graceful stop, not kill the daemon.
//!
//! Its own test binary: it raises a real signal, which reaches every handler
//! registered in the process.

#![cfg(target_os = "linux")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use keyrx_daemon::daemon::install_signal_handlers;

/// Regression: handlers were `register_conditional_default(SIGTERM, running)`,
/// which runs the default action (terminate) while `running` is true, so the
/// event loop never saw the stop and Drop never released devices or the socket.
#[test]
fn sigterm_clears_running_flag_and_process_survives() {
    let running = Arc::new(AtomicBool::new(true));
    let _handler = install_signal_handlers(Arc::clone(&running)).expect("install");

    signal_hook::low_level::raise(signal_hook::consts::SIGTERM).expect("raise");

    let deadline = Instant::now() + Duration::from_secs(2);
    while running.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "SIGTERM did not clear running");
        std::thread::sleep(Duration::from_millis(5));
    }
}
