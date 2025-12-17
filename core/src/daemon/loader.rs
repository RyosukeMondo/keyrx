//! Loads configuration and scripts for the daemon.

use crate::config::{ConfigManager, RuntimeConfig};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tracing::info;

/// Load configuration and script content from the given paths.
///
/// If paths are not provided, default values will be used.
pub async fn load_config_and_script(
    config_path: Option<&Path>,
    script_path: Option<&Path>,
) -> Result<(RuntimeConfig, Option<String>)> {
    // 1. Load RuntimeConfig
    let config_manager = match config_path {
        Some(path) => ConfigManager::new(path),
        None => ConfigManager::default(),
    };
    let config = config_manager.load_runtime_config()?;

    // 2. Load Rhai script content
    let script_content = if let Some(path) = script_path {
        info!(service = "keyrx", event = "script_load.custom", path = %path.display());
        Some(
            fs::read_to_string(path)
                .await
                .with_context(|| format!("Failed to read script file: {}", path.display()))?,
        )
    } else {
        info!(service = "keyrx", event = "script_load.none");
        None
    };

    Ok((config, script_content))
}

/// Create a `RhaiRuntime` instance from optional script content.
///
/// If no script is provided, a default, empty runtime is created.
pub fn create_rhai_runtime(script_content: Option<String>) -> Result<RhaiRuntime> {
    let mut runtime = RhaiRuntime::new()?;

    if let Some(script) = script_content {
        info!(service = "keyrx", event = "rhai_runtime.compile_script");
        runtime
            .execute(&script)
            .context("Script compilation or initial execution failed")?;
    } else {
        info!(service = "keyrx", event = "rhai_runtime.default");
    }

    Ok(runtime)
}