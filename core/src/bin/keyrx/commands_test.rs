//! Testing and diagnostic commands: test, replay, analyze, uat, regression, doctor, repl, golden, ci-check
//!
//! This module contains command handlers for testing and diagnostics:
//! - `test`: Run tests in a Rhai script
//! - `replay`: Replay recorded sessions from .krx files
//! - `analyze`: Analyze recorded sessions and generate timing diagrams
//! - `uat`: Run User Acceptance Tests
//! - `regression`: Verify golden sessions for regressions
//! - `doctor`: Run self-diagnostics
//! - `repl`: Start interactive REPL
//! - `golden`: Golden session management
//! - `ci-check`: Run complete CI checks

use keyrx_core::cli::{
    commands::{
        AnalyzeCommand, CiCheckCommand, DoctorCommand, GoldenCommand, GoldenSubcommand,
        RegressionCommand, ReplCommand, ReplayCommand, TestCommand, UatCommand,
    },
    Command, CommandContext, CommandResult, ExitCode, HasExitCode,
};
use std::path::PathBuf;

/// Execute the `doctor` command
pub fn execute_doctor(verbose: bool, ctx: &CommandContext) -> CommandResult<()> {
    let mut cmd = DoctorCommand::new(verbose, ctx.output_format());
    cmd.execute(ctx)
}

/// Execute the `repl` command
pub fn execute_repl(ctx: &CommandContext) -> CommandResult<()> {
    let mut cmd = ReplCommand::new(ctx.output_format());
    cmd.execute(ctx)
}

/// Execute the `test` command
pub fn execute_test(
    script: PathBuf,
    filter: Option<String>,
    watch: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    match TestCommand::new(script, ctx.output_format())
        .with_filter(filter)
        .with_watch(watch)
        .run()
    {
        Ok(0) => CommandResult::success(()),
        Ok(code) => CommandResult::failure(
            ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
            format!("Tests failed with exit code {code}"),
        ),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}

/// Execute the `replay` command
pub async fn execute_replay(
    session: PathBuf,
    verify: bool,
    speed: f64,
    ctx: &CommandContext,
) -> CommandResult<()> {
    match ReplayCommand::new(session, ctx.output_format())
        .with_verify(verify)
        .with_speed(speed)
        .run()
        .await
    {
        Ok(result) if verify && !result.all_matched() => {
            CommandResult::failure(ExitCode::AssertionFailed, "Replay verification failed")
        }
        Ok(_) => CommandResult::success(()),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}

/// Execute the `analyze` command
pub fn execute_analyze(session: PathBuf, diagram: bool, ctx: &CommandContext) -> CommandResult<()> {
    let mut cmd = AnalyzeCommand::new(session, ctx.output_format()).with_diagram(diagram);
    cmd.execute(ctx)
}

/// UAT command options
pub struct UatOptions {
    pub category: Vec<String>,
    pub priority: Vec<String>,
    pub json: bool,
    pub fail_fast: bool,
    pub perf: bool,
    pub fuzz: bool,
    pub fuzz_duration: u64,
    pub fuzz_count: Option<u64>,
    pub coverage: bool,
    pub report: bool,
    pub report_format: String,
    pub report_output: Option<PathBuf>,
    pub gate: Option<String>,
}

/// Execute the `uat` command
pub fn execute_uat(options: UatOptions, ctx: &CommandContext) -> CommandResult<()> {
    match UatCommand::new(ctx.output_format())
        .with_categories(options.category)
        .with_priorities(options.priority)
        .with_json(options.json)
        .with_fail_fast(options.fail_fast)
        .with_perf(options.perf)
        .with_fuzz(options.fuzz)
        .with_fuzz_duration(options.fuzz_duration)
        .with_fuzz_count(options.fuzz_count)
        .with_coverage_report(options.coverage)
        .with_report(options.report)
        .with_report_format(options.report_format)
        .with_report_output(options.report_output)
        .with_gate(options.gate)
        .run()
    {
        Ok(0) => CommandResult::success(()),
        Ok(code) => CommandResult::failure(
            ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
            format!("UAT failed with exit code {code}"),
        ),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}

/// Golden subcommand actions parsed from CLI
pub enum GoldenCommandAction {
    Record {
        name: String,
        script: PathBuf,
    },
    Verify {
        name: String,
        script: Option<PathBuf>,
    },
    Update {
        name: String,
        script: PathBuf,
        confirm: bool,
    },
    List,
}

/// Execute the `golden` command
pub fn execute_golden(action: GoldenCommandAction, ctx: &CommandContext) -> CommandResult<()> {
    let subcommand = match action {
        GoldenCommandAction::Record { name, script } => GoldenSubcommand::Record { name, script },
        GoldenCommandAction::Verify { name, script } => GoldenSubcommand::Verify { name, script },
        GoldenCommandAction::Update {
            name,
            script,
            confirm,
        } => GoldenSubcommand::Update {
            name,
            script,
            confirm,
        },
        GoldenCommandAction::List => GoldenSubcommand::List,
    };
    match GoldenCommand::new(subcommand, ctx.output_format()).run() {
        Ok(0) => CommandResult::success(()),
        Ok(code) => CommandResult::failure(
            ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
            format!("Golden command failed with exit code {code}"),
        ),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}

/// Execute the `regression` command
pub fn execute_regression(
    golden_dir: Option<PathBuf>,
    json: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    match RegressionCommand::new(ctx.output_format())
        .with_golden_dir(golden_dir)
        .with_json(json)
        .run()
    {
        Ok(0) => CommandResult::success(()),
        Ok(code) => CommandResult::failure(
            ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
            format!("Regression tests failed with exit code {code}"),
        ),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}

/// CI check command options
pub struct CiCheckOptions {
    pub gate: Option<String>,
    pub json: bool,
    pub skip_unit: bool,
    pub skip_integration: bool,
    pub skip_uat: bool,
    pub skip_regression: bool,
    pub skip_perf: bool,
}

/// Execute the `ci-check` command
pub fn execute_ci_check(options: CiCheckOptions, ctx: &CommandContext) -> CommandResult<()> {
    match CiCheckCommand::new(ctx.output_format())
        .with_gate(options.gate)
        .with_json(options.json)
        .with_skip_unit(options.skip_unit)
        .with_skip_integration(options.skip_integration)
        .with_skip_uat(options.skip_uat)
        .with_skip_regression(options.skip_regression)
        .with_skip_perf(options.skip_perf)
        .run()
    {
        Ok(0) => CommandResult::success(()),
        Ok(code) => CommandResult::failure(
            ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
            format!("CI check failed with exit code {code}"),
        ),
        Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
    }
}
