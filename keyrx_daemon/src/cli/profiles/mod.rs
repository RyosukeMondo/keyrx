//! Profile management CLI commands.
//!
//! This module implements the `keyrx profiles` command and all its subcommands
//! for managing Rhai configuration profiles, including creation, activation,
//! deletion, duplication, import, and export.

mod args;
mod handlers_activate;
mod handlers_crud;
mod handlers_io;
mod output;

#[cfg(test)]
mod tests;

use crate::error::DaemonResult;
use crate::services::ProfileService;

pub use args::ProfilesArgs;
use args::ProfilesCommands;

/// Execute the profiles command.
pub async fn execute(args: ProfilesArgs, service: &ProfileService) -> DaemonResult<()> {
    match args.command {
        ProfilesCommands::List => handlers_io::handle_list(service, args.json).await,
        ProfilesCommands::Create { name, template } => {
            handlers_crud::handle_create(service, &name, template, args.json).await
        }
        ProfilesCommands::Activate { name } => {
            handlers_activate::handle_activate(service, &name, args.json).await
        }
        ProfilesCommands::Delete { name, confirm } => {
            handlers_crud::handle_delete(service, &name, confirm, args.json).await
        }
        ProfilesCommands::Duplicate { src, dest } => {
            handlers_crud::handle_duplicate(service, &src, &dest, args.json).await
        }
        ProfilesCommands::Export { name, output } => {
            handlers_io::handle_export(service, &name, &output, args.json).await
        }
        ProfilesCommands::Import { input, name } => {
            handlers_io::handle_import(service, &input, &name, args.json).await
        }
    }
}
