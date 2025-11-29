//! State inspection command.

use crate::cli::{OutputFormat, OutputWriter};
use anyhow::Result;
use serde::Serialize;

/// Inspect current engine state.
pub struct StateCommand {
    pub show_layers: bool,
    pub show_modifiers: bool,
    pub output: OutputWriter,
}

#[derive(Serialize)]
struct StateOutput {
    layers: Vec<LayerInfo>,
    modifiers: Vec<u8>,
}

#[derive(Serialize)]
struct LayerInfo {
    name: String,
    active: bool,
    priority: i32,
}

impl StateCommand {
    pub fn new(show_layers: bool, show_modifiers: bool, format: OutputFormat) -> Self {
        Self {
            show_layers,
            show_modifiers,
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        // TODO: Connect to running engine or load state from file
        let state = StateOutput {
            layers: vec![LayerInfo {
                name: "base".to_string(),
                active: true,
                priority: 0,
            }],
            modifiers: vec![],
        };

        self.output.data(&state)?;
        Ok(())
    }
}
