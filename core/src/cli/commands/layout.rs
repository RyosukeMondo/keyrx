//! Virtual layout management CLI commands.
//!
//! Provides `keyrx layout list|show|create` for working with stored
//! `VirtualLayout` definitions in the KeyRx config directory.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::models::{LayoutType, VirtualLayout};
use crate::config::{ConfigManager, StorageError};
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Actions supported by the layout command.
#[derive(Debug, Clone)]
pub enum LayoutAction {
    List,
    Show { id: String },
    Create { source: LayoutSource },
}

/// Source for layout creation (file path or stdin).
#[derive(Debug, Clone)]
pub enum LayoutSource {
    File(PathBuf),
    Stdin,
}

/// Layout command entry point.
pub struct LayoutCommand {
    output: OutputWriter,
    action: LayoutAction,
    config_root: Option<PathBuf>,
}

#[derive(Serialize)]
struct LayoutSummary {
    id: String,
    name: String,
    layout_type: LayoutType,
    key_count: usize,
}

#[derive(Serialize)]
struct LayoutListOutput {
    layouts: Vec<LayoutSummary>,
}

#[derive(Serialize)]
struct LayoutDetailOutput {
    layout: VirtualLayout,
}

#[derive(Serialize)]
struct LayoutCreateOutput {
    saved_path: String,
    layout: LayoutSummary,
}

impl LayoutCommand {
    pub fn new(format: OutputFormat, action: LayoutAction) -> Self {
        Self {
            output: OutputWriter::new(format),
            action,
            config_root: None,
        }
    }

    /// Override the config root (useful for tests).
    pub fn with_config_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.config_root = Some(root.into());
        self
    }

    fn run(&self) -> CommandResult<()> {
        match &self.action {
            LayoutAction::List => self.list(),
            LayoutAction::Show { id } => self.show(id),
            LayoutAction::Create { source } => self.create(source),
        }
    }

    fn list(&self) -> CommandResult<()> {
        let manager = self.manager();
        let layouts = match manager.load_virtual_layouts() {
            Ok(map) => map.into_values().collect::<Vec<_>>(),
            Err(err) => return self.storage_failure("load layouts", err),
        };

        let mut summaries: Vec<LayoutSummary> = layouts
            .into_iter()
            .map(|layout| LayoutSummary {
                id: layout.id.clone(),
                name: layout.name,
                layout_type: layout.layout_type,
                key_count: layout.keys.len(),
            })
            .collect();
        summaries.sort_by(|a, b| a.id.cmp(&b.id));

        if let Err(err) = self.output.data(&LayoutListOutput { layouts: summaries }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render layout list: {err}"),
            );
        }

        CommandResult::success(())
    }

    fn show(&self, id: &str) -> CommandResult<()> {
        let manager = self.manager();
        let layouts = match manager.load_virtual_layouts() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load layouts", err),
        };

        let Some(layout) = layouts.get(id) else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Layout '{id}' not found"),
            );
        };

        if let Err(err) = self.output.data(&LayoutDetailOutput {
            layout: layout.clone(),
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render layout: {err}"),
            );
        }

        CommandResult::success(())
    }

    fn create(&self, source: &LayoutSource) -> CommandResult<()> {
        let json = match self.read_source(source) {
            Ok(content) => content,
            Err(result) => return result,
        };

        let layout: VirtualLayout = match serde_json::from_str(&json) {
            Ok(parsed) => parsed,
            Err(err) => {
                return CommandResult::failure(
                    ExitCode::ValidationFailed,
                    format!("Invalid layout JSON: {err}"),
                )
            }
        };

        let manager = self.manager();
        let saved_path = match manager.save_virtual_layout(&layout) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save layout", err),
        };

        let summary = LayoutSummary {
            id: layout.id.clone(),
            name: layout.name.clone(),
            layout_type: layout.layout_type,
            key_count: layout.keys.len(),
        };

        if let Err(err) = self.output.data(&LayoutCreateOutput {
            saved_path: saved_path.display().to_string(),
            layout: summary,
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render layout output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            self.output.success(&format!(
                "Saved layout '{}' to {}",
                layout.id,
                saved_path.display()
            ));
        }
        CommandResult::success(())
    }

    fn read_source(&self, source: &LayoutSource) -> Result<String, CommandResult<()>> {
        match source {
            LayoutSource::Stdin => {
                let mut buffer = String::new();
                if let Err(err) = io::stdin().read_to_string(&mut buffer) {
                    return Err(CommandResult::failure(
                        ExitCode::GeneralError,
                        format!("Failed to read layout from stdin: {err}"),
                    ));
                }
                Ok(buffer)
            }
            LayoutSource::File(path) => fs::read_to_string(path).map_err(|err| {
                let code = if err.kind() == io::ErrorKind::PermissionDenied {
                    ExitCode::PermissionDenied
                } else {
                    ExitCode::GeneralError
                };
                CommandResult::failure(
                    code,
                    format!("Failed to read layout file '{}': {err}", path.display()),
                )
            }),
        }
    }

    fn storage_failure(&self, action: &str, err: StorageError) -> CommandResult<()> {
        let code = match &err {
            StorageError::CreateDir(_, e)
            | StorageError::ReadDir(_, e)
            | StorageError::ReadFile(_, e)
            | StorageError::WriteFile(_, e)
                if e.kind() == io::ErrorKind::PermissionDenied =>
            {
                ExitCode::PermissionDenied
            }
            StorageError::Parse(_, _) => ExitCode::ValidationFailed,
            _ => ExitCode::GeneralError,
        };
        CommandResult::failure(code, format!("Failed to {action}: {err}"))
    }

    fn manager(&self) -> ConfigManager {
        match &self.config_root {
            Some(root) => ConfigManager::new(root.clone()),
            None => ConfigManager::default(),
        }
    }
}

impl Command for LayoutCommand {
    fn name(&self) -> &str {
        "layout"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}
