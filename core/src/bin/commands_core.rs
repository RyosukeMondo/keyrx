//! Core commands: run, simulate, check, discover
//!
//! This module contains command handlers for the core engine operations:
//! - `check`: Validate and lint Rhai scripts
//! - `run`: Run the engine in headless mode
//! - `simulate`: Simulate key events without real keyboard
//! - `discover`: Discover keyboard layouts and create device profiles
//! - `docs`: Generate API documentation
//! - `state`: Inspect current engine state
//! - `bench`: Run latency benchmarks
//! - `exit-codes`: Display exit code documentation

use keyrx_core::cli::{
    commands::{
        BenchCommand, CheckCommand, DiscoverCommand, DocFormat, DocsCommand, ExitCodesCommand,
        RunCommand, SimulateCommand, StateCommand,
    },
    Command, CommandContext, CommandResult, HasExitCode,
};
use keyrx_core::config::{merge_cli_overrides, Config};
use std::path::PathBuf;
use std::str::FromStr;

/// Execute the `check` command
pub fn execute_check(script: PathBuf, ctx: &CommandContext) -> CommandResult<()> {
    let mut cmd = CheckCommand::new(script, ctx.output_format());
    cmd.execute(ctx)
}

/// Execute the `run` command
#[allow(clippy::too_many_arguments)]
pub fn execute_run(
    script: Option<PathBuf>,
    debug: bool,
    no_capture: bool,
    validate_only: bool,
    device: Option<PathBuf>,
    record: Option<PathBuf>,
    trace: Option<PathBuf>,
    tap_timeout: Option<u32>,
    combo_timeout: Option<u32>,
    hold_delay: Option<u32>,
    no_cache: bool,
    clear_cache: bool,
    config: Config,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let mut config = config;
    merge_cli_overrides(&mut config, tap_timeout, combo_timeout, hold_delay);
    let mut cmd = RunCommand::new(script, debug, no_capture, device, ctx.output_format())
        .with_record_path(record)
        .with_trace_path(trace)
        .with_config(config)
        .with_validate_only(validate_only)
        .with_cache_options(no_cache, clear_cache);
    cmd.execute(ctx)
}

/// Execute the `simulate` command
pub async fn execute_simulate(
    input: Option<String>,
    script: Option<PathBuf>,
    hold_ms: Option<u64>,
    combo: bool,
    interactive: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let result = if interactive {
        SimulateCommand::run_interactive(script, ctx.output_format())
    } else if let Some(input) = input {
        SimulateCommand::new(input, script, ctx.output_format())
            .with_hold_ms(hold_ms)
            .with_combo(combo)
            .run()
            .await
    } else {
        Err(anyhow::anyhow!(
            "--input is required when not using --interactive"
        ))
    };
    convert_result(result)
}

/// Execute the `discover` command
pub async fn execute_discover(
    device: Option<String>,
    force: bool,
    yes: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let result = DiscoverCommand::new(device, force, yes, ctx.output_format())
        .run()
        .await;
    convert_result(result)
}

/// Execute the `docs` command
pub fn execute_docs(format: String, output: PathBuf, ctx: &CommandContext) -> CommandResult<()> {
    let doc_format = DocFormat::from_str(&format).unwrap_or_else(|_| {
        eprintln!("Warning: Unknown format '{}', using markdown", format);
        DocFormat::Markdown
    });
    let mut cmd = DocsCommand::new(doc_format, output, ctx.output_format());
    cmd.execute(ctx)
}

/// Execute the `state` command
pub fn execute_state(
    layers: bool,
    modifiers: bool,
    pending: bool,
    script: Option<PathBuf>,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let mut cmd = StateCommand::new(layers, modifiers, pending, script, ctx.output_format());
    cmd.execute(ctx)
}

/// Execute the `bench` command
pub fn execute_bench(
    iterations: usize,
    script: Option<PathBuf>,
    flamegraph: bool,
    allocations: bool,
    ctx: &CommandContext,
) -> CommandResult<()> {
    let mut cmd = BenchCommand::new(iterations, script, ctx.output_format());
    if flamegraph {
        cmd = cmd.with_flamegraph_output(None);
    }
    if allocations {
        cmd = cmd.with_allocation_report_output(None);
    }
    Command::execute(&mut cmd, ctx)
}

/// Execute the `exit-codes` command
pub fn execute_exit_codes(ctx: &CommandContext) -> CommandResult<()> {
    let mut cmd = ExitCodesCommand::new();
    cmd.execute(ctx)
}

/// Helper to convert anyhow::Result to CommandResult
fn convert_result(res: anyhow::Result<()>) -> CommandResult<()> {
    match res {
        Ok(()) => CommandResult::success(()),
        Err(err) => {
            let exit_code = err.exit_code();
            CommandResult::failure(exit_code, format!("{err:#}"))
        }
    }
}
