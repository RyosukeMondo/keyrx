//! Test mode: IPC + web API without keyboard capture.

use std::sync::Arc;

use super::message_loop::pump_messages;
use super::shell::{open_browser, show_about_dialog};
use crate::daemon::ExitCode;
use crate::daemon_config::DaemonConfig;

/// Run the daemon in test mode (no keyboard capture).
pub(super) fn run(config: DaemonConfig) -> Result<(), (i32, String)> {
    use crate::config::ProfileManager;
    use crate::ipc::commands::IpcCommandHandler;

    log::info!("Starting daemon in test mode (no keyboard capture)");

    let config_dir = crate::cli::config_dir::get_config_dir().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Cannot determine config directory: {e}"),
        )
    })?;

    // ProfileManager has internal mutability; no RwLock needed
    let profile_manager = ProfileManager::new(config_dir.clone())
        .map(Arc::new)
        .map_err(|e| {
            (
                ExitCode::ConfigError as i32,
                format!("Failed to initialize ProfileManager: {}", e),
            )
        })?;

    // One read model shared by the IPC handler and the web API. Test mode has
    // no keyboard daemon, so it reports "not running" consistently on both.
    let daemon_query = Arc::new(crate::services::DaemonQueryService::without_daemon());
    let ipc_handler = Arc::new(IpcCommandHandler::new(
        Arc::clone(&profile_manager),
        Arc::clone(&daemon_query),
    ));

    let test_endpoint = crate::ipc::IpcEndpoint::test_for_process(std::process::id());
    crate::ipc::server::spawn(test_endpoint.clone(), ipc_handler).map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to start IPC server: {}", e),
        )
    })?;
    log::info!("IPC server started on {test_endpoint}");

    let web_url = config.web_url();
    spawn_web_server(&config, config_dir, profile_manager, daemon_query)?;
    log::info!("Test mode running. Web UI at {}", web_url);
    run_tray_loop(&web_url);
    Ok(())
}

/// Builds the test-mode services and serves the web API on its own thread.
fn spawn_web_server(
    config: &DaemonConfig,
    config_dir: std::path::PathBuf,
    profile_manager: Arc<crate::config::ProfileManager>,
    daemon_query: Arc<crate::services::DaemonQueryService>,
) -> Result<(), (i32, String)> {
    use crate::services::{
        ConfigService, DeviceService, ProfileService, SettingsService, SimulationService,
    };

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create tokio runtime: {}", e),
        )
    })?;
    let addr = config.socket_addr().map_err(|e| {
        (
            ExitCode::ConfigError as i32,
            format!("Failed to create socket address: {}", e),
        )
    })?;

    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(1000);
    // Simulator-to-macro-recorder event bus
    let (macro_event_tx, macro_event_rx) =
        tokio::sync::mpsc::channel::<keyrx_core::runtime::KeyEvent>(1000);

    // Test mode: no daemon running, no reload needed
    let macro_recorder = Arc::new(crate::macro_recorder::MacroRecorder::new());
    let profile_service = Arc::new(ProfileService::new(profile_manager));
    let (rpc_event_tx, _) = tokio::sync::broadcast::channel(1000);
    let app_state = Arc::new(crate::web::AppState::new_with_test_mode(
        macro_recorder.clone(),
        Arc::clone(&profile_service),
        Arc::new(DeviceService::new(config_dir.clone())),
        Arc::new(ConfigService::new(profile_service)),
        Arc::new(SettingsService::new(config_dir.clone())),
        Arc::new(SimulationService::new(config_dir, Some(macro_event_tx))),
        Arc::new(crate::web::subscriptions::SubscriptionManager::new()),
        rpc_event_tx,
        daemon_query,
    ));

    log::info!("Starting web server on {}", config.web_url());
    // Main thread runs the message loop for the tray
    std::thread::spawn(move || {
        rt.block_on(async {
            let recorder_for_loop = (*macro_recorder).clone();
            tokio::spawn(async move {
                recorder_for_loop.run_event_loop(macro_event_rx).await;
            });

            match crate::web::serve(addr, event_tx, app_state).await {
                Ok(()) => log::info!("Web server stopped"),
                Err(e) => log::error!("Web server error: {}", e),
            }
        });
    });
    Ok(())
}

/// Pumps messages (the tray needs them) until `WM_QUIT` or tray Exit.
fn run_tray_loop(web_url: &str) {
    use crate::platform::windows::tray::TrayIconController;
    use crate::platform::{SystemTray, TrayControlEvent};

    let tray = match TrayIconController::new() {
        Ok(tray) => {
            log::info!("System tray icon created successfully (test mode)");
            Some(tray)
        }
        Err(e) => {
            log::warn!("Failed to create system tray icon: {}", e);
            None
        }
    };

    while pump_messages() {
        match tray.as_ref().and_then(|tray| tray.poll_event()) {
            Some(TrayControlEvent::OpenWebUI) => {
                log::info!("Opening web UI at {}...", web_url);
                if let Err(e) = open_browser(web_url) {
                    log::error!("Failed to open web UI: {}", e);
                }
            }
            Some(TrayControlEvent::About) => show_about_dialog(),
            Some(TrayControlEvent::Exit) => {
                log::info!("Exiting test mode...");
                return;
            }
            _ => {}
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
