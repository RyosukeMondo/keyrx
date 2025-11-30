//! Engine run command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::Engine;
use crate::mocks::{MockInput, MockState};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

#[cfg(target_os = "linux")]
use crate::drivers::LinuxInput;

#[cfg(target_os = "linux")]
use signal_hook::consts::{SIGINT, SIGTERM};
#[cfg(target_os = "linux")]
use signal_hook::flag;

/// Run the engine in headless mode.
pub struct RunCommand {
    pub script_path: Option<PathBuf>,
    pub debug: bool,
    pub use_mock: bool,
    pub device_path: Option<PathBuf>,
    pub output: OutputWriter,
}

impl RunCommand {
    pub fn new(
        script_path: Option<PathBuf>,
        debug: bool,
        use_mock: bool,
        device_path: Option<PathBuf>,
        format: OutputFormat,
    ) -> Self {
        Self {
            script_path,
            debug,
            use_mock,
            device_path,
            output: OutputWriter::new(format),
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Initialize tracing if debug mode
        if self.debug {
            self.init_debug_logging()?;
            debug!(
                event = "debug_mode_enabled",
                context = "cli_run",
                format = "json"
            );
        }

        self.output.success("Starting KeyRx engine...");

        let runtime = self.prepare_runtime()?;

        // Create state store
        let state = MockState::new();

        // Run with appropriate input source
        if self.use_mock {
            self.run_with_mock(runtime, state).await
        } else {
            self.run_with_platform_driver(runtime, state).await
        }
    }

    fn prepare_runtime(&self) -> Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;

        if let Some(path) = &self.script_path {
            self.output
                .success(&format!("Loading script: {}", path.display()));
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime.load_file(path_str)?;

            debug!("Running script top-level statements");
            runtime.run_script()?;

            if runtime.has_hook("on_init") {
                debug!("Calling on_init() hook");
                runtime.call_hook("on_init")?;
                self.output.success("Script initialized (on_init called)");
            }
        }

        Ok(runtime)
    }

    fn init_debug_logging(&self) -> Result<()> {
        let env_filter =
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("debug"))?;

        let fmt_layer = fmt::layer()
            .json()
            .flatten_event(true)
            .with_target(true)
            .with_level(true)
            .with_timer(fmt::time::SystemTime)
            .with_current_span(true)
            .with_span_list(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| anyhow!("failed to initialize tracing subscriber: {e}"))?;

        Ok(())
    }

    async fn run_with_mock(&self, runtime: RhaiRuntime, state: MockState) -> Result<()> {
        self.output
            .success("Using mock input (no real keyboard interception)");

        let input = MockInput::new();
        let mut engine = Engine::new(input, runtime, state);
        engine.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!("Engine running with mock input");

        // Set up Ctrl+C handler
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            r.store(false, Ordering::SeqCst);
        });

        // Run event loop until interrupted
        while running.load(Ordering::SeqCst) {
            // With mock input, just sleep since there are no events
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        engine.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime, state: MockState) -> Result<()> {
        self.output.success("Using Linux input driver");

        // Show which device we're using and initialize input
        let input = self.initialize_linux_input()?;

        let mut engine = Engine::new(input, runtime, state);
        engine.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!("Engine running with Linux input driver");

        // Set up graceful shutdown with signal handlers
        let running = self.setup_signal_handlers()?;

        // Run event loop until interrupted
        while running.load(Ordering::SeqCst) && engine.is_running() {
            engine.run_loop().await?;
            // Small delay to prevent busy-waiting when no events
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        self.output.success("Signal received, stopping...");
        engine.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    /// Initialize the Linux input driver with the configured device path.
    #[cfg(target_os = "linux")]
    fn initialize_linux_input(&self) -> Result<LinuxInput> {
        if let Some(ref device) = self.device_path {
            self.output
                .success(&format!("Using device: {}", device.display()));
        } else {
            self.output.success("Auto-detecting keyboard device...");
        }

        match LinuxInput::new(self.device_path.clone()) {
            Ok(input) => {
                self.output.success(&format!(
                    "Opened keyboard: {}",
                    input.device_path().display()
                ));
                Ok(input)
            }
            Err(e) => {
                self.output
                    .error(&format!("Failed to initialize driver: {e:#}"));
                Err(e)
            }
        }
    }

    /// Set up signal handlers for graceful shutdown on SIGINT and SIGTERM.
    #[cfg(target_os = "linux")]
    fn setup_signal_handlers(&self) -> Result<Arc<AtomicBool>> {
        // Set up graceful shutdown using signal-hook for SIGINT and SIGTERM
        // This ensures clean keyboard release even when killed by systemd/init
        let running = Arc::new(AtomicBool::new(true));

        // Register signal handlers for both SIGINT (Ctrl+C) and SIGTERM (kill/systemd)
        // signal-hook uses a single Arc<AtomicBool> and sets it to false on signal
        flag::register(SIGINT, running.clone())
            .map_err(|e| anyhow::anyhow!("Failed to register SIGINT handler: {e}"))?;
        flag::register(SIGTERM, running.clone())
            .map_err(|e| anyhow::anyhow!("Failed to register SIGTERM handler: {e}"))?;

        debug!("Signal handlers registered for SIGINT and SIGTERM");

        Ok(running)
    }

    #[cfg(target_os = "windows")]
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime, state: MockState) -> Result<()> {
        use crate::drivers::WindowsInput;

        self.output.success("Using Windows input driver");

        // Note: Windows uses a global keyboard hook, so --device is ignored
        if self.device_path.is_some() {
            self.output.warning(
                "Note: --device is ignored on Windows (uses global keyboard hook for all keyboards)",
            );
        }

        let input = match WindowsInput::new() {
            Ok(input) => {
                self.output
                    .success("Initialized Windows low-level keyboard hook");
                input
            }
            Err(e) => {
                self.output
                    .error(&format!("Failed to initialize driver: {e:#}"));
                return Err(e);
            }
        };

        let mut engine = Engine::new(input, runtime, state);
        engine.start().await?;

        self.output.success("Engine started. Press Ctrl+C to stop.");
        info!("Engine running with Windows input driver");

        // Set up graceful shutdown
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            r.store(false, Ordering::SeqCst);
        });

        // Run event loop until interrupted
        while running.load(Ordering::SeqCst) && engine.is_running() {
            engine.run_loop().await?;
            // Small delay to prevent busy-waiting when no events
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        engine.stop().await?;
        self.output.success("Engine stopped.");
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    async fn run_with_platform_driver(&self, runtime: RhaiRuntime, state: MockState) -> Result<()> {
        self.output
            .warning("No platform driver available for this OS, falling back to mock input");
        self.run_with_mock(runtime, state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{KeyCode, RemapAction};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn prepare_runtime_loads_script_and_on_init() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("script.rhai");

        fs::write(
            &script_path,
            r#"
remap("A", "B");

fn on_init() {
    block("CapsLock");
}
"#,
        )
        .unwrap();

        let cmd = RunCommand::new(Some(script_path), false, true, None, OutputFormat::Human);
        let runtime = cmd.prepare_runtime().expect("runtime should load script");

        assert_eq!(
            runtime.lookup_remap(KeyCode::A),
            RemapAction::Remap(KeyCode::B)
        );
        assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
    }

    #[test]
    fn prepare_runtime_errors_on_invalid_path() {
        let cmd = RunCommand::new(
            Some(PathBuf::from("/not/a/real/script.rhai")),
            false,
            true,
            None,
            OutputFormat::Human,
        );

        let result = cmd.prepare_runtime();
        assert!(result.is_err());
    }
}
