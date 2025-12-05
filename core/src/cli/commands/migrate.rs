//! Migration command for converting V1 profiles to V2.

use crate::cli::{CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::{config_dir, device_profiles_dir};
use crate::migration::{MigrationReport, MigrationV1ToV2};
use crate::registry::profile::ProfileRegistry;

/// Migrate old V1 profiles to new V2 profile system.
pub struct MigrateCommand {
    pub output: OutputWriter,
    from_version: String,
    create_backup: bool,
}

impl MigrateCommand {
    pub fn new(format: OutputFormat, from_version: String, create_backup: bool) -> Self {
        Self {
            output: OutputWriter::new(format),
            from_version,
            create_backup,
        }
    }

    pub async fn run(&self) -> CommandResult<()> {
        // Validate version parameter
        if self.from_version != "v1" {
            return CommandResult::failure(
                ExitCode::ValidationFailed,
                format!(
                    "Invalid migration version '{}'. Only 'v1' is supported.",
                    self.from_version
                ),
            );
        }

        // Get paths for old and new profiles
        let old_profiles_dir = device_profiles_dir();
        let new_profiles_dir = config_dir().join("profiles");

        // Print migration start info
        if matches!(self.output.format(), OutputFormat::Human) {
            println!("Starting migration from {} to V2...", self.from_version);
            println!("Old profiles directory: {}", old_profiles_dir.display());
            println!("New profiles directory: {}", new_profiles_dir.display());
            if self.create_backup {
                println!("Backup: enabled");
            } else {
                println!("Backup: disabled");
            }
            println!();
        }

        // Create profile registry
        let profile_registry = ProfileRegistry::with_directory(new_profiles_dir);

        // Create migrator
        let migrator = MigrationV1ToV2::new(old_profiles_dir, profile_registry, self.create_backup);

        // Run migration
        let report = match migrator.migrate().await {
            Ok(r) => r,
            Err(e) => {
                return CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Migration failed: {}", e),
                )
            }
        };

        // Display results
        self.render_report(&report)
    }

    fn render_report(&self, report: &MigrationReport) -> CommandResult<()> {
        match self.output.format() {
            OutputFormat::Json | OutputFormat::Yaml => {
                // Output structured data
                let data = serde_json::json!({
                    "total": report.total_count,
                    "migrated": report.migrated_count,
                    "failed": report.failed_count,
                    "success_rate": report.success_rate(),
                    "backup_path": report.backup_path,
                    "failures": report.failures.iter().map(|f| {
                        serde_json::json!({
                            "path": f.path,
                            "error": f.error,
                        })
                    }).collect::<Vec<_>>(),
                });

                if let Err(e) = self.output.data(&data) {
                    return CommandResult::failure(
                        ExitCode::GeneralError,
                        format!("Failed to output migration report: {}", e),
                    );
                }
            }
            _ => {
                // Human-readable output
                println!("{}", report.summary());
            }
        }

        // Determine exit code based on results
        if report.total_count == 0 {
            // No profiles to migrate - this is okay
            CommandResult::success(())
        } else if report.is_success() {
            // All profiles migrated successfully
            CommandResult::success(())
        } else if report.is_partial() {
            // Some succeeded, some failed - still exit with error
            CommandResult::failure(
                ExitCode::GeneralError,
                "Migration completed with some failures",
            )
        } else {
            // All failed
            CommandResult::failure(ExitCode::GeneralError, "Migration failed completely")
        }
    }
}

// Note: We don't implement the Command trait for MigrateCommand
// because the execute method is sync but migration needs async.
// Instead, we handle it specially in the main run_command function.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_command_creates_with_human_format() {
        let cmd = MigrateCommand::new(OutputFormat::Human, "v1".to_string(), true);
        assert!(matches!(cmd.output.format(), OutputFormat::Human));
    }

    #[test]
    fn migrate_command_creates_with_json_format() {
        let cmd = MigrateCommand::new(OutputFormat::Json, "v1".to_string(), false);
        assert!(matches!(cmd.output.format(), OutputFormat::Json));
    }

    #[test]
    fn migrate_command_validates_version() {
        let cmd = MigrateCommand::new(OutputFormat::Human, "v2".to_string(), false);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(cmd.run());
        assert!(result.is_failure());
        assert!(result
            .messages()
            .iter()
            .any(|msg| msg.contains("Only 'v1' is supported")));
    }
}
