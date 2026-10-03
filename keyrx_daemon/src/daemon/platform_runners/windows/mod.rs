//! Windows daemon runner.
//!
//! The daemon runs as one process:
//! - **Main thread**: the Windows message loop, which also drives keyboard
//!   event processing (low-level hooks need it) and the tray ([`message_loop`])
//! - **Web server thread**: Tokio runtime serving the REST API and WebSocket
//! - **IPC server thread**: the `\\.\pipe\keyrx-daemon` named pipe for the
//!   `keyrx_daemon status|state|metrics` CLI and `profiles activate`
//!
//! Web and IPC answer from the one [`DaemonQueryService`] over
//! [`DaemonSharedState`], so every transport reports the same thing.
//!
//! [`DaemonQueryService`]: crate::services::DaemonQueryService

mod instance;
mod message_loop;
mod shell;
mod test_mode;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::daemon::{DaemonSharedState, ExitCode};
use crate::daemon_config::DaemonConfig;

/// Run the daemon on Windows.
///
/// # Arguments
///
/// * `source` - Where the startup configuration comes from (see `daemon::live_config`)
/// * `config_dir` - The keyrx config directory (profiles, `.active`)
/// * `debug` - Enable debug logging
/// * `test_mode` - Enable test mode (no keyboard capture)
/// * `container` - Service container with all dependencies wired
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn run_daemon(
    source: crate::daemon::ConfigSource,
    config_dir: PathBuf,
    log: crate::daemon::platform_setup::LogOptions,
    watch: bool,
    test_mode: bool,
    container: Arc<crate::container::ServiceContainer>,
    options: crate::daemon::options::RuntimeOptions,
) -> Result<(), (i32, String)> {
    use crate::daemon::platform_setup::{init_logging, log_startup_version_info};

    init_logging(log);
    log_startup_version_info();
    let config = load_config()?;

    if test_mode {
        log::info!("Test mode enabled - running with IPC infrastructure without keyboard capture");
        return test_mode::run(config);
    }

    // Ensure single instance - kill any existing daemon before starting
    let killed_old = instance::ensure_single_instance(&config_dir);
    let configured_port = configured_port(&config_dir, killed_old);

    let mut daemon = create_daemon(source, &config_dir, &options)?;
    let daemon_state = daemon.shared_state();

    // Real-time event streaming to WebSocket clients
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    let event_broadcaster = crate::daemon::EventBroadcaster::new(event_tx.clone());
    daemon.set_event_broadcaster(event_broadcaster.clone());

    if watch {
        crate::daemon::config_watch::spawn(
            Arc::clone(container.profile_service().profile_manager()),
            Arc::clone(&daemon_state),
            daemon.running_flag(),
            crate::daemon::config_watch::POLL_INTERVAL,
        );
    }
    // The single read model for web API and IPC (see DaemonQueryService docs).
    let daemon_query = Arc::new(crate::services::DaemonQueryService::new(
        Arc::clone(&daemon_state),
        daemon.telemetry(),
    ));
    let app_state = Arc::new(crate::web::AppState::from_container_with_daemon(
        (*container).clone(),
        Arc::clone(&daemon_query),
    ));
    // Best-effort; the daemon runs regardless.
    let _ipc = super::start_production_ipc_server(&container, daemon_query);

    let actual_port = instance::find_available_port(configured_port);
    let port_note = port_change_note(configured_port, actual_port);
    spawn_web_server(
        actual_port,
        event_tx,
        app_state,
        WebTelemetry {
            broadcaster: event_broadcaster,
            running: daemon.running_flag(),
            telemetry: daemon.telemetry(),
        },
    );

    let web_ui_url = format!("http://127.0.0.1:{}", actual_port);
    let tray = message_loop::create_tray(port_note, &web_ui_url);

    finish_hook_setup(&daemon_state);
    log::info!("Daemon initialized. Running message loop...");
    message_loop::run(&mut daemon, &daemon_state, tray.as_ref(), &web_ui_url);

    instance::cleanup_pid_file(&config_dir);
    Ok(())
}

/// Creates the platform and the daemon (installs the keyboard hook).
fn create_daemon(
    source: crate::daemon::ConfigSource,
    config_dir: &Path,
    options: &crate::daemon::options::RuntimeOptions,
) -> Result<crate::daemon::Daemon, (i32, String)> {
    log::info!("Starting keyrx daemon (Windows) from {source:?}");
    let platform = crate::platform::create_platform().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create platform: {}", e),
        )
    })?;
    crate::daemon::Daemon::with_options(platform, source, config_dir.to_path_buf(), options)
        .map_err(daemon_error_to_exit)
}

/// Wires suspend (pass-through) into the hook and logs hook health.
fn finish_hook_setup(daemon_state: &Arc<DaemonSharedState>) {
    wire_suspended_flag(daemon_state);
    if !shell::is_admin() {
        log::warn!("Daemon is not running with administrative privileges. Key remapping may not work for elevated applications.");
    }
    crate::daemon::platform_setup::log_post_init_hook_status();
}

/// Loads and validates the daemon configuration from the environment.
fn load_config() -> Result<DaemonConfig, (i32, String)> {
    let config = DaemonConfig::from_env().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Configuration error: {}", e),
        )
    })?;
    config.validate().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Invalid configuration: {}", e),
        )
    })?;
    Ok(config)
}

/// The web port from settings, reset to the default when an old instance was
/// just killed (its port is free again).
fn configured_port(config_dir: &Path, killed_old: bool) -> u16 {
    let settings = crate::services::SettingsService::new(config_dir.to_path_buf());
    let port = if killed_old {
        let default_port = crate::services::DEFAULT_PORT;
        if let Err(e) = settings.set_port(default_port) {
            log::warn!("Failed to reset port to default: {}", e);
        }
        default_port
    } else {
        settings.get_port()
    };
    log::info!("Configured web server port: {}", port);
    port
}

/// Tray note when the configured port was busy. Not persisted: persisting
/// causes port drift (9867→9868→9869... on each restart).
fn port_change_note(configured: u16, actual: u16) -> Option<String> {
    if configured == actual {
        return None;
    }
    log::warn!(
        "Configured port {} is in use. Using port {} instead (not saved to settings).",
        configured,
        actual
    );
    Some(format!(
        "Port {} was in use. Now running on port {}.",
        configured, actual
    ))
}

/// What the latency broadcast task needs from the daemon.
struct WebTelemetry {
    broadcaster: crate::daemon::EventBroadcaster,
    running: Arc<std::sync::atomic::AtomicBool>,
    telemetry: Arc<crate::daemon::DaemonTelemetry>,
}

/// Serves the web API and latency broadcasts on their own Tokio thread.
fn spawn_web_server(
    port: u16,
    event_tx: tokio::sync::broadcast::Sender<crate::web::events::DaemonEvent>,
    app_state: Arc<crate::web::AppState>,
    web: WebTelemetry,
) {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => {
                log::error!("Failed to create tokio runtime for web server: {}", e);
                log::error!("Web server will not start. Ensure your system has sufficient resources (threads, memory)");
                return;
            }
        };
        rt.block_on(async {
            tokio::spawn(crate::daemon::start_latency_broadcast_task(
                web.broadcaster,
                web.running,
                Some(web.telemetry),
            ));

            let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
            log::info!("Starting web server on http://{}", addr);
            match crate::web::serve(addr, event_tx, app_state).await {
                Ok(()) => log::info!("Web server stopped"),
                Err(e) => log::error!("Web server error: {}", e),
            }
        });
    });
}

/// Wire the suspended flag from DaemonSharedState to the keyboard hook.
///
/// The hook callback reads this flag on every keypress to decide whether
/// to pass the key through unchanged (suspended) or process it normally.
fn wire_suspended_flag(daemon_state: &Arc<DaemonSharedState>) {
    use crate::platform::windows::platform_state::PlatformState;

    if let Some(state_arc) = PlatformState::get() {
        if let Ok(state) = state_arc.lock() {
            if let Some(ref blocker) = state.key_blocker {
                blocker.set_suspended_flag(daemon_state.suspended_flag());
                log::info!("Suspended flag wired to keyboard hook");
            }
        }
    }
}

/// Converts a DaemonError to an exit code and message.
fn daemon_error_to_exit(error: crate::daemon::DaemonError) -> (i32, String) {
    (error.exit_code().into(), error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_change_note_only_when_port_moved() {
        assert_eq!(port_change_note(9867, 9867), None);
        let note = port_change_note(9867, 9868).unwrap();
        assert!(note.contains("9867") && note.contains("9868"), "{note}");
    }
}
