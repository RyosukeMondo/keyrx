//! Command dispatch logic for the keyrx binary.
//!
//! This module handles routing CLI commands to their respective implementations
//! in the commands_core, commands_config, and commands_test modules.

use keyrx_core::cli::{CommandContext, CommandResult};
use keyrx_core::config::Config;
use tracing::debug;

use crate::args::{
    Commands, DeviceCommands, GoldenCommands, HardwareCommands, KeymapCommands, LayoutCommands,
    RuntimeCommands,
};
use crate::commands_config::{
    DeviceCommandAction, HardwareCommandAction, KeymapCommandAction, LayoutCommandAction,
    RuntimeCommandAction,
};
use crate::commands_test::{CiCheckOptions, GoldenCommandAction, UatOptions};
use crate::{commands_config, commands_core, commands_test};

/// Execute the given command and return a result.
pub async fn run_command(
    command: Commands,
    ctx: &CommandContext,
    config: Config,
) -> CommandResult<()> {
    let command_name = get_command_name(&command);
    debug!(command = command_name, "Executing command");

    match command {
        // Core commands
        Commands::Check { script } => commands_core::execute_check(script, ctx),
        Commands::Run {
            script,
            debug,
            no_capture,
            validate_only,
            device,
            record,
            trace,
            tap_timeout,
            combo_timeout,
            hold_delay,
            no_cache,
            clear_cache,
        } => commands_core::execute_run(
            script,
            debug,
            no_capture,
            validate_only,
            device,
            record,
            trace,
            tap_timeout,
            combo_timeout,
            hold_delay,
            no_cache,
            clear_cache,
            config,
            ctx,
        ),
        Commands::Simulate {
            input,
            script,
            hold_ms,
            combo,
            interactive,
        } => commands_core::execute_simulate(input, script, hold_ms, combo, interactive, ctx).await,
        Commands::Discover { device, force, yes } => {
            commands_core::execute_discover(device, force, yes, ctx).await
        }
        Commands::Docs { format, output } => commands_core::execute_docs(format, output, ctx),
        Commands::State {
            layers,
            modifiers,
            pending,
            script,
        } => commands_core::execute_state(layers, modifiers, pending, script, ctx),
        Commands::Bench {
            iterations,
            script,
            flamegraph,
            allocations,
        } => commands_core::execute_bench(iterations, script, flamegraph, allocations, ctx),
        Commands::ExitCodes => commands_core::execute_exit_codes(ctx),

        // Config commands
        Commands::Devices { command } => dispatch_devices(command, ctx),
        Commands::Hardware { command } => dispatch_hardware(command, ctx),
        Commands::Layout { command } => dispatch_layout(command, ctx),
        Commands::Keymap { command } => dispatch_keymap(command, ctx),
        Commands::Runtime { command } => dispatch_runtime(command, ctx),
        Commands::Migrate { from, backup } => {
            commands_config::execute_migrate(from, backup, ctx).await
        }

        // Test commands
        Commands::Doctor { verbose } => commands_test::execute_doctor(verbose, ctx),
        Commands::Repl => commands_test::execute_repl(ctx),
        Commands::Test {
            script,
            filter,
            watch,
        } => commands_test::execute_test(script, filter, watch, ctx),
        Commands::Replay {
            session,
            verify,
            speed,
        } => commands_test::execute_replay(session, verify, speed, ctx).await,
        Commands::Analyze { session, diagram } => {
            commands_test::execute_analyze(session, diagram, ctx)
        }
        Commands::Uat {
            category,
            priority,
            json,
            fail_fast,
            perf,
            fuzz,
            fuzz_duration,
            fuzz_count,
            coverage,
            report,
            report_format,
            report_output,
            gate,
        } => {
            let options = UatOptions {
                category,
                priority,
                json,
                fail_fast,
                perf,
                fuzz,
                fuzz_duration,
                fuzz_count,
                coverage,
                report,
                report_format,
                report_output,
                gate,
            };
            commands_test::execute_uat(options, ctx)
        }
        Commands::Golden { command } => dispatch_golden(command, ctx),
        Commands::Regression { golden_dir, json } => {
            commands_test::execute_regression(golden_dir, json, ctx)
        }
        Commands::CiCheck {
            gate,
            json,
            skip_unit,
            skip_integration,
            skip_uat,
            skip_regression,
            skip_perf,
        } => {
            let options = CiCheckOptions {
                gate,
                json,
                skip_unit,
                skip_integration,
                skip_uat,
                skip_regression,
                skip_perf,
            };
            commands_test::execute_ci_check(options, ctx)
        }
    }
}

fn dispatch_devices(command: Option<DeviceCommands>, ctx: &CommandContext) -> CommandResult<()> {
    let device_cmd = command.unwrap_or_default();
    let action = match device_cmd {
        DeviceCommands::List => DeviceCommandAction::List,
        DeviceCommands::Show { device } => DeviceCommandAction::Show { device },
        DeviceCommands::Label {
            device,
            label,
            clear,
        } => DeviceCommandAction::Label {
            device,
            label,
            clear,
        },
        DeviceCommands::Remap { device, state } => DeviceCommandAction::Remap {
            device,
            enabled: state.enabled(),
        },
        DeviceCommands::Assign { device, profile } => {
            DeviceCommandAction::Assign { device, profile }
        }
        DeviceCommands::Unassign { device } => DeviceCommandAction::Unassign { device },
    };
    commands_config::execute_devices(action, ctx)
}

fn dispatch_hardware(command: HardwareCommands, ctx: &CommandContext) -> CommandResult<()> {
    let action = match command {
        HardwareCommands::List => HardwareCommandAction::List,
        HardwareCommands::Define { source } => HardwareCommandAction::Define { source },
        HardwareCommands::Wire {
            profile,
            scancode,
            virtual_key,
            clear,
        } => HardwareCommandAction::Wire {
            profile,
            scancode,
            virtual_key,
            clear,
        },
        HardwareCommands::Detect => HardwareCommandAction::Detect,
        HardwareCommands::Profile {
            vendor_id,
            product_id,
        } => HardwareCommandAction::Profile {
            vendor_id,
            product_id,
        },
        HardwareCommands::Calibrate {
            vendor_id,
            product_id,
            warmup_samples,
            sample_count,
            max_duration_secs,
            latencies,
            samples_file,
        } => HardwareCommandAction::Calibrate {
            vendor_id,
            product_id,
            warmup_samples,
            sample_count,
            max_duration_secs,
            latencies,
            samples_file,
        },
    };
    commands_config::execute_hardware(action, ctx)
}

fn dispatch_layout(command: Option<LayoutCommands>, ctx: &CommandContext) -> CommandResult<()> {
    let layout_cmd = command.unwrap_or_default();
    let action = match layout_cmd {
        LayoutCommands::List => LayoutCommandAction::List,
        LayoutCommands::Show { id } => LayoutCommandAction::Show { id },
        LayoutCommands::Create { source } => LayoutCommandAction::Create { source },
    };
    commands_config::execute_layout(action, ctx)
}

fn dispatch_keymap(command: Option<KeymapCommands>, ctx: &CommandContext) -> CommandResult<()> {
    let keymap_cmd = command.unwrap_or_default();
    let action = match keymap_cmd {
        KeymapCommands::List => KeymapCommandAction::List,
        KeymapCommands::Show { id } => KeymapCommandAction::Show { id },
        KeymapCommands::Map {
            keymap,
            layer,
            virtual_key,
            action,
            clear,
        } => KeymapCommandAction::Map {
            keymap,
            layer,
            virtual_key,
            action,
            clear,
        },
    };
    commands_config::execute_keymap(action, ctx)
}

fn dispatch_runtime(command: Option<RuntimeCommands>, ctx: &CommandContext) -> CommandResult<()> {
    let runtime_cmd = command.unwrap_or_default();
    let action = match runtime_cmd {
        RuntimeCommands::Devices => RuntimeCommandAction::Devices,
        RuntimeCommands::SlotAdd {
            vendor_id,
            product_id,
            serial,
            slot,
            hardware_profile,
            keymap,
            priority,
            active,
        } => RuntimeCommandAction::SlotAdd {
            vendor_id,
            product_id,
            serial,
            slot,
            hardware_profile,
            keymap,
            priority,
            active,
        },
        RuntimeCommands::SlotRemove {
            vendor_id,
            product_id,
            serial,
            slot,
        } => RuntimeCommandAction::SlotRemove {
            vendor_id,
            product_id,
            serial,
            slot,
        },
        RuntimeCommands::SlotActive {
            vendor_id,
            product_id,
            serial,
            slot,
            active,
        } => RuntimeCommandAction::SlotActive {
            vendor_id,
            product_id,
            serial,
            slot,
            active,
        },
    };
    commands_config::execute_runtime(action, ctx)
}

fn dispatch_golden(command: GoldenCommands, ctx: &CommandContext) -> CommandResult<()> {
    let action = match command {
        GoldenCommands::Record { name, script } => GoldenCommandAction::Record { name, script },
        GoldenCommands::Verify { name, script } => GoldenCommandAction::Verify { name, script },
        GoldenCommands::Update {
            name,
            script,
            confirm,
        } => GoldenCommandAction::Update {
            name,
            script,
            confirm,
        },
        GoldenCommands::List => GoldenCommandAction::List,
    };
    commands_test::execute_golden(action, ctx)
}

/// Get the command name for logging purposes.
pub fn get_command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Check { .. } => "check",
        Commands::Devices { .. } => "devices",
        Commands::Hardware { .. } => "hardware",
        Commands::Layout { .. } => "layout",
        Commands::Keymap { .. } => "keymap",
        Commands::Runtime { .. } => "runtime",
        Commands::ExitCodes => "exit-codes",
        Commands::Docs { .. } => "docs",
        Commands::Run { .. } => "run",
        Commands::State { .. } => "state",
        Commands::Doctor { .. } => "doctor",
        Commands::Repl => "repl",
        Commands::Bench { .. } => "bench",
        Commands::Simulate { .. } => "simulate",
        Commands::Discover { .. } => "discover",
        Commands::Test { .. } => "test",
        Commands::Replay { .. } => "replay",
        Commands::Analyze { .. } => "analyze",
        Commands::Uat { .. } => "uat",
        Commands::Golden { .. } => "golden",
        Commands::Regression { .. } => "regression",
        Commands::CiCheck { .. } => "ci-check",
        Commands::Migrate { .. } => "migrate",
    }
}
