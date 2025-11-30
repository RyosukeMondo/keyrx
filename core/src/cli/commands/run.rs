//! Engine run command.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::Engine;
use crate::mocks::{MockInput, MockState};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

#[cfg(target_os = "linux")]
use crate::drivers::LinuxInput;

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
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_target(false)
                .init();
            debug!("Debug mode enabled");
        }

        self.output.success("Starting KeyRx engine...");

        // Create script runtime
        let mut runtime = RhaiRuntime::new()?;

        // Load script if provided
        if let Some(path) = &self.script_path {
            self.output
                .success(&format!("Loading script: {}", path.display()));
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))?;
            runtime.load_file(path_str)?;

            // Run top-level statements (e.g., remap/block/pass calls)
            debug!("Running script top-level statements");
            runtime.run_script()?;

            // Call on_init() hook if defined
            if runtime.has_hook("on_init") {
                debug!("Calling on_init() hook");
                runtime.call_hook("on_init")?;
                self.output.success("Script initialized (on_init called)");
            }
        }

        // Create state store
        let state = MockState::new();

        // Run with appropriate input source
        if self.use_mock {
            self.run_with_mock(runtime, state).await
        } else {
            self.run_with_platform_driver(runtime, state).await
        }
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

        // Show which device we're using
        if let Some(ref device) = self.device_path {
            self.output
                .success(&format!("Using device: {}", device.display()));
        } else {
            self.output.success("Auto-detecting keyboard device...");
        }

        let input = match LinuxInput::new(self.device_path.clone()) {
            Ok(input) => {
                self.output.success(&format!(
                    "Opened keyboard: {}",
                    input.device_path().display()
                ));
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
        info!("Engine running with Linux input driver");

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
