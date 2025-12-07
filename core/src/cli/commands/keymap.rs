//! Keymap CLI commands for listing, inspecting, and updating logical mappings.
use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::models::{ActionBinding, Keymap, KeymapLayer};
use crate::config::{ConfigManager, StorageError};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Actions supported by the keymap command.
#[derive(Debug, Clone)]
pub enum KeymapAction {
    List,
    Show { id: String },
    Map { request: MapRequest },
}

/// Request to map or clear a binding in a keymap layer.
#[derive(Debug, Clone)]
pub struct MapRequest {
    pub keymap_id: String,
    pub layer: String,
    pub virtual_key: String,
    pub action: Option<String>,
    pub clear: bool,
}

/// Keymap command entry point.
pub struct KeymapCommand {
    output: OutputWriter,
    action: KeymapAction,
    config_root: Option<PathBuf>,
}

#[derive(Serialize)]
struct KeymapSummary {
    id: String,
    name: String,
    virtual_layout_id: String,
    layer_count: usize,
    binding_count: usize,
}

#[derive(Serialize)]
struct KeymapListOutput {
    keymaps: Vec<KeymapSummary>,
}

#[derive(Serialize)]
struct KeymapDetailOutput {
    keymap: Keymap,
}

#[derive(Serialize)]
struct KeymapMapOutput {
    saved_path: String,
    keymap: KeymapSummary,
    layer: String,
    virtual_key: String,
    binding: Option<ActionBinding>,
    action: String,
}

impl From<&Keymap> for KeymapSummary {
    fn from(keymap: &Keymap) -> Self {
        let binding_count: usize = keymap.layers.iter().map(|layer| layer.bindings.len()).sum();
        Self {
            id: keymap.id.clone(),
            name: keymap.name.clone(),
            virtual_layout_id: keymap.virtual_layout_id.clone(),
            layer_count: keymap.layers.len(),
            binding_count,
        }
    }
}

impl KeymapCommand {
    pub fn new(format: OutputFormat, action: KeymapAction) -> Self {
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
            KeymapAction::List => self.list(),
            KeymapAction::Show { id } => self.show(id),
            KeymapAction::Map { request } => self.map(request),
        }
    }

    fn list(&self) -> CommandResult<()> {
        let manager = self.manager();
        let keymaps = match manager.load_keymaps() {
            Ok(map) => map.into_values().collect::<Vec<_>>(),
            Err(err) => return self.storage_failure("load keymaps", err),
        };

        let mut summaries: Vec<KeymapSummary> = keymaps.iter().map(KeymapSummary::from).collect();
        summaries.sort_by(|a, b| a.id.cmp(&b.id));

        if let Err(err) = self.output.data(&KeymapListOutput { keymaps: summaries }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render keymap list: {err}"),
            );
        }

        CommandResult::success(())
    }

    fn show(&self, id: &str) -> CommandResult<()> {
        let manager = self.manager();
        let keymaps = match manager.load_keymaps() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load keymaps", err),
        };

        let Some(keymap) = keymaps.get(id) else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Keymap '{id}' not found"),
            );
        };

        if let Err(err) = self.output.data(&KeymapDetailOutput {
            keymap: keymap.clone(),
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render keymap: {err}"),
            );
        }

        CommandResult::success(())
    }

    fn map(&self, request: &MapRequest) -> CommandResult<()> {
        if request.clear && request.action.is_some() {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Use either --action to set or --clear to remove a mapping, not both",
            );
        }

        if request.action.is_none() && !request.clear {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                "Provide --action to set a mapping or --clear to remove one",
            );
        }

        let binding = if request.clear {
            None
        } else {
            let Some(raw_action) = request.action.as_ref() else {
                return CommandResult::failure(
                    ExitCode::ValidationFailed,
                    "Provide --action to set a mapping or --clear to remove one",
                );
            };
            Some(match parse_action_binding(raw_action) {
                Ok(binding) => binding,
                Err(msg) => return CommandResult::failure(ExitCode::ValidationFailed, msg),
            })
        };

        let manager = self.manager();
        let mut keymaps = match manager.load_keymaps() {
            Ok(map) => map,
            Err(err) => return self.storage_failure("load keymaps", err),
        };

        let Some(mut keymap) = keymaps.remove(&request.keymap_id) else {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!("Keymap '{}' not found", request.keymap_id),
            );
        };

        let layer = ensure_layer(&mut keymap.layers, &request.layer);

        if let Some(binding) = binding.clone() {
            layer
                .bindings
                .insert(request.virtual_key.clone(), binding.clone());
        } else {
            layer.bindings.remove(&request.virtual_key);
        }

        let saved_path = match manager.save_keymap(&keymap) {
            Ok(path) => path,
            Err(err) => return self.storage_failure("save keymap", err),
        };

        let summary = KeymapSummary::from(&keymap);
        let action_label = if request.clear {
            "cleared".to_string()
        } else {
            format!(
                "{}",
                binding
                    .as_ref()
                    .map(|b| format!("{b:?}"))
                    .unwrap_or_else(|| "cleared".to_string())
            )
        };

        if let Err(err) = self.output.data(&KeymapMapOutput {
            saved_path: saved_path.display().to_string(),
            keymap: summary,
            layer: request.layer.clone(),
            virtual_key: request.virtual_key.clone(),
            binding: binding.clone(),
            action: action_label.clone(),
        }) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to render keymap mapping output: {err}"),
            );
        }

        if matches!(
            self.output.format(),
            OutputFormat::Human | OutputFormat::Table
        ) {
            if request.clear {
                self.output.success(&format!(
                    "Cleared binding for {} in layer '{}' of keymap '{}'",
                    request.virtual_key, request.layer, keymap.id
                ));
            } else if let Some(binding) = binding {
                self.output.success(&format!(
                    "Mapped {} -> {binding:?} in layer '{}' of keymap '{}'",
                    request.virtual_key, request.layer, keymap.id
                ));
            }
        }

        CommandResult::success(())
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

impl Command for KeymapCommand {
    fn name(&self) -> &str {
        "keymap"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}

fn ensure_layer<'a>(layers: &'a mut Vec<KeymapLayer>, name: &str) -> &'a mut KeymapLayer {
    if let Some(index) = layers.iter().position(|layer| layer.name == name) {
        return layers
            .get_mut(index)
            .expect("layer index should be valid after position lookup");
    }

    layers.push(KeymapLayer {
        name: name.to_string(),
        bindings: HashMap::new(),
    });
    layers.last_mut().expect("new layer present")
}

fn parse_action_binding(input: &str) -> Result<ActionBinding, String> {
    let normalized = input.trim();
    if let Some(value) = normalized.strip_prefix("key:") {
        return Ok(ActionBinding::StandardKey(value.trim().to_string()));
    }
    if let Some(value) = normalized.strip_prefix("macro:") {
        return Ok(ActionBinding::Macro(value.trim().to_string()));
    }
    if let Some(value) = normalized.strip_prefix("layer-toggle:") {
        return Ok(ActionBinding::LayerToggle(value.trim().to_string()));
    }
    if normalized.eq_ignore_ascii_case("transparent") {
        return Ok(ActionBinding::Transparent);
    }

    Err(
        "Invalid action; use key:<code>, macro:<text>, layer-toggle:<layer>, or transparent"
            .to_string(),
    )
}
