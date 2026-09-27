//! Shared state for Windows daemon-to-web-server communication.
//!
//! This module provides thread-safe shared state that enables communication
//! between the daemon's main thread (keyboard event processing) and the web
//! server thread (REST API) on Windows, where Unix domain sockets are not
//! available.
//!
//! # Architecture
//!
//! On Windows, the daemon runs in a single process with two threads:
//! - **Main thread**: Processes keyboard events via Windows hooks
//! - **Web server thread**: Serves REST API on port 9867
//!
//! Instead of IPC (Unix sockets), both threads share a [`DaemonSharedState`]
//! instance via `Arc`, allowing the web server to query daemon status directly.
//!
//! # Thread Safety
//!
//! All fields use lock-free atomics or read-write locks for safe concurrent access:
//! - `running`: `AtomicBool` - lock-free read/write
//! - `device_count`: `AtomicUsize` - lock-free read/write
//! - `active_profile`, `config_path`: `RwLock` - multiple readers, single writer
//! - `start_time`: `Instant` - immutable after creation
//!
//! # Who writes what
//!
//! `active_profile` / `config_path` describe what the daemon has **actually
//! loaded**; only the daemon writes them, after swapping its remapping state.
//! Transports that activate a profile call [`DaemonSharedState::request_activation`],
//! which records the request and raises the reload flag.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Thread-safe shared state for daemon-to-web-server communication on Windows.
///
/// This struct provides a snapshot of daemon state that can be safely shared
/// across threads without IPC. It is created once at daemon startup and passed
/// to the web server via `AppState`.
///
/// # Fields
///
/// - **running**: Whether the daemon is running (shutdown signal not received)
/// - **active_profile**: Name of the currently active profile, if any
/// - **config_path**: Path to the active .krx configuration file
/// - **device_count**: Number of keyboard devices currently captured
/// - **start_time**: Daemon start time (for uptime calculation)
///
/// # Thread Safety
///
/// All fields are protected by either atomics (lock-free) or read-write locks
/// (multiple concurrent readers). Methods use `SeqCst` ordering for atomics to
/// ensure visibility across threads.
///
/// # Example
///
/// ```no_run
/// # use keyrx_daemon::daemon::Daemon;
/// # fn example(daemon: &Daemon) {
/// let shared = daemon.shared_state();
///
/// // Query from web server thread
/// if shared.is_running() {
///     println!("Daemon has been running for {} seconds", shared.uptime_secs());
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct DaemonSharedState {
    /// Running flag shared from Daemon (shutdown signal detection).
    running: Arc<AtomicBool>,

    /// Name of the currently active profile.
    ///
    /// This is `Some(name)` when a profile is active, `None` for pass-through mode.
    /// Updated when the web API activates/deactivates profiles.
    active_profile: Arc<RwLock<Option<String>>>,

    /// Path to the active .krx configuration file.
    ///
    /// This is the path passed to `Daemon::new()` or updated via reload.
    /// Used by the status API to report the config location.
    config_path: Arc<RwLock<PathBuf>>,

    /// Number of keyboard devices currently captured by the daemon.
    ///
    /// This is queried from the platform on creation and can be updated
    /// if devices are hotplugged/unplugged (future enhancement).
    device_count: Arc<AtomicUsize>,

    /// Daemon start time (for uptime calculation).
    ///
    /// This is set once at daemon startup and never changes. The status
    /// API uses this to calculate and report daemon uptime.
    start_time: Instant,

    /// Reload request flag. For a real daemon this is the SAME flag SIGHUP sets
    /// (the signal handler's `ReloadState`), so `request_reload()` reaches the
    /// Linux event loop and the Windows message loop through one mechanism.
    reload_requested: Arc<AtomicBool>,

    /// Profile activation requested by a transport, not yet applied by the
    /// daemon. Taken by the daemon when it services the reload flag.
    pending_activation: Arc<Mutex<Option<String>>>,

    /// Suspended flag — when true, the daemon passes all keys through unchanged.
    ///
    /// This is toggled via the system tray menu ("Suspend / Resume") or
    /// via `POST /api/debug/suspend`. The keyboard hook checks this flag
    /// and skips blocking when suspended.
    suspended: Arc<AtomicBool>,
}

impl DaemonSharedState {
    /// Creates a new DaemonSharedState for testing or when no Daemon is available.
    ///
    /// A real daemon creates its own instance (see `Daemon::shared_state`);
    /// this constructor serves test mode and unit tests.
    ///
    /// # Arguments
    ///
    /// * `running` - Running flag (typically Arc::new(AtomicBool::new(false)) for test mode)
    /// * `active_profile` - Name of the active profile, if any
    /// * `config_path` - Path to the configuration file
    /// * `device_count` - Number of devices
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    /// use std::sync::atomic::AtomicBool;
    /// use keyrx_daemon::daemon::DaemonSharedState;
    ///
    /// let running = Arc::new(AtomicBool::new(false));
    /// let state = DaemonSharedState::new(
    ///     running,
    ///     None, // No active profile
    ///     PathBuf::from("/test/config.krx"),
    ///     0, // No devices
    /// );
    /// ```
    pub fn new(
        running: Arc<AtomicBool>,
        active_profile: Option<String>,
        config_path: PathBuf,
        device_count: usize,
    ) -> Self {
        Self {
            running,
            active_profile: Arc::new(RwLock::new(active_profile)),
            config_path: Arc::new(RwLock::new(config_path)),
            device_count: Arc::new(AtomicUsize::new(device_count)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        }
    }

    /// Uses `flag` as the reload-request flag, so [`request_reload`](Self::request_reload)
    /// and SIGHUP set the same flag the event loop consumes.
    #[must_use]
    pub fn sharing_reload_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.reload_requested = flag;
        self
    }

    /// Returns whether the daemon is currently running.
    ///
    /// This returns `false` if a shutdown signal (SIGTERM, SIGINT) has been
    /// received. The web server can use this to report daemon status.
    ///
    /// # Thread Safety
    ///
    /// Lock-free atomic read with `SeqCst` ordering for cross-thread visibility.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// if !shared.is_running() {
    ///     println!("Daemon is shutting down!");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns the name of the currently active profile, if any.
    ///
    /// Returns `Some(name)` when a profile is active, `None` for pass-through mode
    /// (no remapping). This is set during daemon startup or when the web API
    /// activates a profile.
    ///
    /// # Thread Safety
    ///
    /// Acquires a read lock. Multiple threads can read concurrently, but writes
    /// (via `set_active_profile`) will block readers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// match shared.get_active_profile() {
    ///     Some(name) => println!("Active profile: {}", name),
    ///     None => println!("Pass-through mode (no remapping)"),
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_active_profile(&self) -> Option<String> {
        self.active_profile.read().expect("RwLock poisoned").clone()
    }

    /// Returns the path to the active .krx configuration file.
    ///
    /// This is the path that was passed to `Daemon::new()` or set via reload.
    /// The web server status API reports this to show which config is loaded.
    ///
    /// # Thread Safety
    ///
    /// Acquires a read lock. Multiple threads can read concurrently.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// let config = shared.get_config_path();
    /// println!("Config loaded from: {}", config.display());
    /// # }
    /// ```
    #[must_use]
    pub fn get_config_path(&self) -> PathBuf {
        self.config_path.read().expect("RwLock poisoned").clone()
    }

    /// Returns the number of keyboard devices currently captured.
    ///
    /// This is the count of devices that the daemon is monitoring for events.
    /// The value is set during initialization and can be updated if devices
    /// are hotplugged/unplugged (future enhancement).
    ///
    /// # Thread Safety
    ///
    /// Lock-free atomic read with `SeqCst` ordering.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// let count = shared.get_device_count();
    /// println!("Monitoring {} keyboard(s)", count);
    /// # }
    /// ```
    #[must_use]
    pub fn get_device_count(&self) -> usize {
        self.device_count.load(Ordering::SeqCst)
    }

    /// Returns the daemon uptime in seconds.
    ///
    /// This calculates the time elapsed since daemon startup using the stored
    /// `start_time`. The status API uses this to report how long the daemon
    /// has been running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// let uptime = shared.uptime_secs();
    /// println!("Daemon has been running for {} seconds", uptime);
    /// # }
    /// ```
    #[must_use]
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Sets the active profile name.
    ///
    /// This is called by the web API when a profile is activated or deactivated.
    /// Pass `Some(name)` to activate a profile, `None` to enter pass-through mode.
    ///
    /// # Arguments
    ///
    /// * `name` - The profile name to activate, or `None` for pass-through mode
    ///
    /// # Thread Safety
    ///
    /// Acquires a write lock, blocking any concurrent readers until the write
    /// completes. This ensures consistent state across threads.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// // Activate a profile
    /// shared.set_active_profile(Some("gaming".to_string()));
    ///
    /// // Deactivate (pass-through mode)
    /// shared.set_active_profile(None);
    /// # }
    /// ```
    pub fn set_active_profile(&self, name: Option<String>) {
        *self.active_profile.write().expect("RwLock poisoned") = name;
    }

    /// Sets the configuration file path.
    ///
    /// This is called when the daemon reloads its configuration or when a
    /// profile is activated with a new .krx file. The web API can use this
    /// to track which configuration is currently active.
    ///
    /// # Arguments
    ///
    /// * `path` - The new configuration file path
    ///
    /// # Thread Safety
    ///
    /// Acquires a write lock, blocking any concurrent readers until the write
    /// completes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// // Update config path after reload
    /// shared.set_config_path(PathBuf::from("/path/to/new-config.krx"));
    /// # }
    /// ```
    pub fn set_config_path(&self, path: PathBuf) {
        *self.config_path.write().expect("RwLock poisoned") = path;
    }

    /// Atomically sets both the active profile and config path together.
    ///
    /// This prevents readers from seeing an inconsistent state where the profile
    /// name has been updated but the config path still points to the old profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile name to activate, or `None` for pass-through mode
    /// * `config_path` - The new configuration file path
    pub fn set_active_config(&self, profile: Option<String>, config_path: PathBuf) {
        // Acquire both write locks to update atomically.
        // Always acquire in the same order (profile, then config) to prevent deadlocks.
        let mut profile_guard = self.active_profile.write().expect("RwLock poisoned");
        let mut config_guard = self.config_path.write().expect("RwLock poisoned");
        *profile_guard = profile;
        *config_guard = config_path;
    }

    /// Updates the device count.
    ///
    /// This is called when devices are hotplugged or unplugged (future enhancement).
    /// The current implementation sets this once during initialization.
    ///
    /// # Arguments
    ///
    /// * `count` - The new device count
    ///
    /// # Thread Safety
    ///
    /// Lock-free atomic write with `SeqCst` ordering.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use keyrx_daemon::daemon::DaemonSharedState;
    /// # fn example(shared: Arc<DaemonSharedState>) {
    /// // Update after hotplug event
    /// shared.set_device_count(2);
    /// # }
    /// ```
    pub fn set_device_count(&self, count: usize) {
        self.device_count.store(count, Ordering::SeqCst);
    }

    /// Request a daemon reload (e.g., after active profile config is modified).
    pub fn request_reload(&self) {
        self.reload_requested.store(true, Ordering::SeqCst);
    }

    /// Check and clear the reload request flag. Returns true if reload was requested.
    pub fn take_reload_request(&self) -> bool {
        self.reload_requested.swap(false, Ordering::SeqCst)
    }

    /// Asks the daemon to switch to profile `name` (already compiled).
    ///
    /// Status keeps reporting the previously loaded profile until the daemon
    /// has loaded `name`; if loading fails, status never changes.
    pub fn request_activation(&self, name: &str) {
        *self
            .pending_activation
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(name.to_string());
        self.request_reload();
    }

    /// Takes the pending activation request, if any (daemon side).
    pub fn take_pending_activation(&self) -> Option<String> {
        self.pending_activation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }

    /// Returns whether the daemon is currently suspended.
    ///
    /// When suspended, all keys pass through unchanged (no remapping/blocking).
    ///
    /// # Thread Safety
    ///
    /// Lock-free atomic read with `SeqCst` ordering.
    #[must_use]
    pub fn is_suspended(&self) -> bool {
        self.suspended.load(Ordering::SeqCst)
    }

    /// Sets the suspended state.
    ///
    /// Pass `true` to suspend (pass all keys through), `false` to resume.
    pub fn set_suspended(&self, suspended: bool) {
        self.suspended.store(suspended, Ordering::SeqCst);
    }

    /// Returns an `Arc<AtomicBool>` clone of the suspended flag.
    ///
    /// This is used to share the flag with the keyboard hook callback,
    /// which needs lock-free access from the hook thread.
    #[must_use]
    pub fn suspended_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.suspended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_shared_state_creation() {
        // We can't create a full Daemon in tests without platform setup,
        // so we test the field behavior directly
        let running = Arc::new(AtomicBool::new(true));
        let active_profile = Arc::new(RwLock::new(Some("test".to_string())));
        let config_path = Arc::new(RwLock::new(PathBuf::from("/test/config.krx")));
        let device_count = Arc::new(AtomicUsize::new(2));

        let state = DaemonSharedState {
            running,
            active_profile,
            config_path,
            device_count,
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        assert!(state.is_running());
        assert_eq!(state.get_active_profile(), Some("test".to_string()));
        assert_eq!(state.get_config_path(), PathBuf::from("/test/config.krx"));
        assert_eq!(state.get_device_count(), 2);
        assert_eq!(state.uptime_secs(), 0); // Just created
        assert!(!state.is_suspended());
    }

    #[test]
    fn test_is_running() {
        let running = Arc::new(AtomicBool::new(true));
        let state = DaemonSharedState {
            running: Arc::clone(&running),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        assert!(state.is_running());

        // Simulate shutdown signal
        running.store(false, Ordering::SeqCst);
        assert!(!state.is_running());
    }

    #[test]
    fn test_active_profile_access() {
        let state = DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(Some("default".to_string()))),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        // Initial profile
        assert_eq!(state.get_active_profile(), Some("default".to_string()));

        // Change profile
        state.set_active_profile(Some("gaming".to_string()));
        assert_eq!(state.get_active_profile(), Some("gaming".to_string()));

        // Deactivate (pass-through mode)
        state.set_active_profile(None);
        assert_eq!(state.get_active_profile(), None);
    }

    #[test]
    fn test_config_path_access() {
        let state = DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/initial/config.krx"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        assert_eq!(
            state.get_config_path(),
            PathBuf::from("/initial/config.krx")
        );

        // Update config path
        state.set_config_path(PathBuf::from("/new/config.krx"));
        assert_eq!(state.get_config_path(), PathBuf::from("/new/config.krx"));
    }

    #[test]
    fn test_device_count_access() {
        let state = DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(2)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        assert_eq!(state.get_device_count(), 2);

        // Update device count
        state.set_device_count(3);
        assert_eq!(state.get_device_count(), 3);
    }

    #[test]
    fn test_uptime_calculation() {
        let state = DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        };

        // Just created, uptime should be 0
        assert_eq!(state.uptime_secs(), 0);

        // Wait a bit and check again
        thread::sleep(Duration::from_millis(100));
        assert!(state.uptime_secs() == 0); // Still less than 1 second

        // Note: Testing uptime > 0 would require sleeping for 1+ seconds,
        // which is too slow for unit tests. Integration tests can verify this.
    }

    #[test]
    fn test_concurrent_reads() {
        let state = Arc::new(DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(Some("test".to_string()))),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(5)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        });

        // Spawn multiple reader threads
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    // Each thread reads all fields
                    assert!(state.is_running());
                    assert_eq!(state.get_active_profile(), Some("test".to_string()));
                    assert_eq!(state.get_device_count(), 5);
                })
            })
            .collect();

        // All threads should complete without deadlock
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_concurrent_writes() {
        let state = Arc::new(DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        });

        // Spawn multiple writer threads
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    // Each thread writes a different profile name
                    state.set_active_profile(Some(format!("profile-{}", i)));
                    state.set_device_count(i);
                })
            })
            .collect();

        // All threads should complete without deadlock
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Final state should be one of the written values
        let final_profile = state.get_active_profile();
        assert!(final_profile.is_some());
        let profile_name = final_profile.unwrap();
        assert!(profile_name.starts_with("profile-"));
    }

    #[test]
    fn test_set_active_config_atomic() {
        let state = Arc::new(DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(Some("old".to_string()))),
            config_path: Arc::new(RwLock::new(PathBuf::from("/old/config.krx"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        });

        // Atomic update of both fields
        state.set_active_config(
            Some("new-profile".to_string()),
            PathBuf::from("/new/config.krx"),
        );

        assert_eq!(state.get_active_profile(), Some("new-profile".to_string()));
        assert_eq!(state.get_config_path(), PathBuf::from("/new/config.krx"));

        // Set to None (pass-through)
        state.set_active_config(None, PathBuf::from("/default.krx"));
        assert_eq!(state.get_active_profile(), None);
        assert_eq!(state.get_config_path(), PathBuf::from("/default.krx"));
    }

    #[test]
    fn test_set_active_config_concurrent() {
        let state = Arc::new(DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(None)),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        });

        // Concurrent atomic updates should not deadlock
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    state.set_active_config(
                        Some(format!("profile-{}", i)),
                        PathBuf::from(format!("/config-{}.krx", i)),
                    );
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Final state should be consistent (profile and path from same write)
        let profile = state.get_active_profile().unwrap();
        let config = state.get_config_path();
        let idx = profile.strip_prefix("profile-").unwrap();
        assert_eq!(config, PathBuf::from(format!("/config-{}.krx", idx)));
    }

    #[test]
    fn test_mixed_concurrent_access() {
        let state = Arc::new(DaemonSharedState {
            running: Arc::new(AtomicBool::new(true)),
            active_profile: Arc::new(RwLock::new(Some("initial".to_string()))),
            config_path: Arc::new(RwLock::new(PathBuf::from("/test"))),
            device_count: Arc::new(AtomicUsize::new(1)),
            start_time: Instant::now(),
            reload_requested: Arc::new(AtomicBool::new(false)),
            suspended: Arc::new(AtomicBool::new(false)),
            pending_activation: Arc::default(),
        });

        // Mix of readers and writers
        let mut handles = vec![];

        // 5 reader threads
        for _ in 0..5 {
            let state = Arc::clone(&state);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let _ = state.get_active_profile();
                    let _ = state.get_device_count();
                }
            }));
        }

        // 2 writer threads
        for i in 0..2 {
            let state = Arc::clone(&state);
            handles.push(thread::spawn(move || {
                for j in 0..50 {
                    state.set_active_profile(Some(format!("writer-{}-{}", i, j)));
                    state.set_device_count(i * 50 + j);
                }
            }));
        }

        // All threads should complete without deadlock
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    /// Regression: `request_reload()` used to set a private flag the Linux
    /// event loop never read, so activation relied on SIGHUP-ing its own
    /// process (killing any process without a SIGHUP handler).
    #[test]
    fn test_request_reload_reaches_signal_reload_state() {
        let reload_state = crate::daemon::state::ReloadState::new();
        let state = DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            None,
            PathBuf::from("/test.krx"),
            0,
        )
        .sharing_reload_flag(reload_state.flag());

        state.request_reload();
        assert!(reload_state.check_and_clear());
        assert!(!state.take_reload_request());
    }
}
