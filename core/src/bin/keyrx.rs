//! KeyRx CLI entry point.

use clap::{Parser, Subcommand};
use keyrx_core::cli::{
    commands::{
        AnalyzeCommand, BenchCommand, CheckCommand, CiCheckCommand, DevicesCommand,
        DiscoverCommand, DoctorCommand, GoldenCommand, GoldenSubcommand, RegressionCommand,
        ReplCommand, ReplayCommand, RunCommand, SimulateCommand, StateCommand, TestCommand,
        UatCommand,
    },
    CommandContext, CommandResult, HasExitCode, OutputFormat, Verbosity,
};
use keyrx_core::config::{load_config, merge_cli_overrides, Config};
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::error;

#[derive(Parser)]
#[command(name = "keyrx")]
#[command(about = "KeyRx - The Ultimate Input Remapping Engine")]
#[command(version)]
struct Cli {
    /// Output format (human or json)
    #[arg(long, default_value = "human", conflicts_with = "json")]
    format: String,

    /// Shortcut for JSON output (equivalent to --format json)
    #[arg(long)]
    json: bool,

    /// Path to configuration file (default: ~/.config/keyrx/config.toml)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and lint a Rhai script
    Check {
        /// Path to the script file
        script: PathBuf,
    },

    /// List available keyboard devices
    Devices,

    /// Run the engine in headless mode
    Run {
        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Use mock input instead of real keyboard driver
        #[arg(short, long)]
        mock: bool,

        /// Path to keyboard device (e.g., /dev/input/event3). Auto-detects if not specified.
        #[arg(long)]
        device: Option<PathBuf>,

        /// Record session to a .krx file for replay/analysis
        #[arg(long)]
        record: Option<PathBuf>,

        /// Export OpenTelemetry traces to file (requires otel-tracing feature)
        #[arg(long)]
        trace: Option<PathBuf>,

        /// Override tap timeout in milliseconds (valid: 50-1000)
        #[arg(long)]
        tap_timeout: Option<u32>,

        /// Override combo timeout in milliseconds (valid: 10-200)
        #[arg(long)]
        combo_timeout: Option<u32>,

        /// Override hold delay in milliseconds (valid: 0-500)
        #[arg(long)]
        hold_delay: Option<u32>,
    },

    /// Inspect current engine state
    State {
        /// Show layer information
        #[arg(long)]
        layers: bool,

        /// Show modifier information
        #[arg(long)]
        modifiers: bool,

        /// Show pending tap-hold and combo decisions
        #[arg(long)]
        pending: bool,

        /// Path to the script file to hydrate engine state
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Run self-diagnostics
    Doctor {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start interactive REPL
    Repl,

    /// Run latency benchmark
    Bench {
        /// Number of iterations
        #[arg(long, default_value = "10000")]
        iterations: usize,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Simulate key events without real keyboard
    Simulate {
        /// Comma-separated list of keys to simulate (e.g., "A,B,CapsLock")
        #[arg(short, long, required_unless_present = "interactive")]
        input: Option<String>,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Hold duration in milliseconds for each key (overrides default)
        #[arg(long)]
        hold_ms: Option<u64>,

        /// Treat input keys as a simultaneous combo
        #[arg(long)]
        combo: bool,

        /// Start interactive REPL-style simulation mode
        #[arg(long, short = 'I')]
        interactive: bool,
    },

    /// Discover a keyboard layout and write a device profile
    Discover {
        /// Target device vendor:product (hex or decimal). If omitted, auto-detects the first keyboard.
        #[arg(long)]
        device: Option<String>,

        /// Force re-discovery even if a profile already exists.
        #[arg(long)]
        force: bool,

        /// Assume yes for confirmations.
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Run tests in a Rhai script
    Test {
        /// Path to the script file containing test functions
        #[arg(short, long)]
        script: PathBuf,

        /// Filter tests by name pattern (supports * wildcard, e.g., "test_capslock*")
        #[arg(short, long)]
        filter: Option<String>,

        /// Watch the script file and re-run tests on change
        #[arg(short, long)]
        watch: bool,
    },

    /// Replay a recorded session from a .krx file
    Replay {
        /// Path to the .krx session file
        session: PathBuf,

        /// Verify that outputs match the recorded outputs
        #[arg(long)]
        verify: bool,

        /// Replay speed multiplier (0 = instant, 1 = realtime, 2 = 2x speed)
        #[arg(long, default_value = "0")]
        speed: f64,
    },

    /// Analyze a recorded session and generate timing diagrams
    Analyze {
        /// Path to the .krx session file
        session: PathBuf,

        /// Generate ASCII timing diagram
        #[arg(long)]
        diagram: bool,
    },

    /// Run User Acceptance Tests (UAT)
    Uat {
        /// Filter by category (can be specified multiple times)
        #[arg(short, long, value_delimiter = ',')]
        category: Vec<String>,

        /// Filter by priority (P0, P1, P2)
        #[arg(short, long, value_delimiter = ',')]
        priority: Vec<String>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,

        /// Run performance tests
        #[arg(long)]
        perf: bool,

        /// Run fuzz tests
        #[arg(long)]
        fuzz: bool,

        /// Fuzz test duration in seconds
        #[arg(long, default_value = "60")]
        fuzz_duration: u64,

        /// Fuzz test sequence count (overrides duration)
        #[arg(long)]
        fuzz_count: Option<u64>,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Generate full report
        #[arg(long)]
        report: bool,

        /// Report format (html, md, json)
        #[arg(long, default_value = "html")]
        report_format: String,

        /// Report output path
        #[arg(long)]
        report_output: Option<PathBuf>,

        /// Quality gate to enforce (alpha, beta, ga)
        #[arg(long)]
        gate: Option<String>,
    },

    /// Golden session management for regression testing
    Golden {
        #[command(subcommand)]
        command: GoldenCommands,
    },

    /// Verify all golden sessions for regressions
    Regression {
        /// Custom golden sessions directory
        #[arg(long)]
        golden_dir: Option<PathBuf>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Run complete CI check (all tests + quality gates)
    CiCheck {
        /// Quality gate to enforce (default, alpha, beta, rc, ga)
        #[arg(long)]
        gate: Option<String>,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,

        /// Skip unit tests
        #[arg(long)]
        skip_unit: bool,

        /// Skip integration tests
        #[arg(long)]
        skip_integration: bool,

        /// Skip UAT tests
        #[arg(long)]
        skip_uat: bool,

        /// Skip regression tests
        #[arg(long)]
        skip_regression: bool,

        /// Skip performance tests
        #[arg(long)]
        skip_perf: bool,
    },
}

/// Golden session subcommands.
#[derive(Subcommand)]
enum GoldenCommands {
    /// Record a new golden session from a script
    Record {
        /// Name of the golden session (alphanumeric, underscores, hyphens)
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,
    },

    /// Verify an existing golden session
    Verify {
        /// Name of the golden session to verify
        name: String,

        /// Path to the script to run for verification (optional)
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Update an existing golden session
    Update {
        /// Name of the golden session to update
        name: String,

        /// Path to the script that generates events
        #[arg(short, long)]
        script: PathBuf,

        /// Confirm the update (required to overwrite existing session)
        #[arg(long)]
        confirm: bool,
    },

    /// List all golden sessions
    List,
}

fn parse_format(s: &str, json_flag: bool) -> OutputFormat {
    if json_flag {
        return OutputFormat::Json;
    }

    match s.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Human,
    }
}

/// Install a panic handler that logs panic info and ensures proper exit code.
///
/// When a panic occurs, this handler:
/// 1. Logs the panic information at error level using tracing
/// 2. Ensures the process exits with code 101 (Rust panic convention)
///
/// This provides graceful panic handling and consistent exit codes even
/// when unrecoverable errors occur.
fn install_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Extract panic location
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // Extract panic message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic message".to_string()
        };

        // Log panic at error level
        error!(
            location = %location,
            message = %message,
            "Panic occurred"
        );

        // Print to stderr as well for visibility when tracing isn't initialized
        eprintln!("Error: Panic at {}: {}", location, message);
        eprintln!("This is a bug. Please report it at: https://github.com/keyrx/keyrx/issues");

        // Exit with code 101 (Rust panic convention)
        std::process::exit(101);
    }));
}

#[tokio::main]
async fn main() -> ExitCode {
    // Install panic handler to catch panics and return exit code 101
    install_panic_handler();

    let cli = Cli::parse();
    let format = parse_format(&cli.format, cli.json);

    // Load configuration from file (or use defaults)
    let config = load_config(cli.config.as_deref());

    // Create command context
    let ctx = CommandContext::with_config(format, Verbosity::Normal, cli.config);

    // Execute command and get result
    let result = run_command(cli.command, &ctx, config).await;

    // Extract exit code and handle errors
    if result.is_success() {
        ExitCode::SUCCESS
    } else {
        // Print error messages
        for msg in result.messages() {
            eprintln!("Error: {msg}");
        }
        result.exit_code().into()
    }
}

async fn run_command(command: Commands, ctx: &CommandContext, config: Config) -> CommandResult<()> {
    use keyrx_core::cli::ExitCode;

    // Helper to convert anyhow::Result to CommandResult
    let convert_result = |res: anyhow::Result<()>| -> CommandResult<()> {
        match res {
            Ok(()) => CommandResult::success(()),
            Err(err) => {
                let exit_code = err.exit_code();
                CommandResult::failure(exit_code, format!("{err:#}"))
            }
        }
    };

    match command {
        Commands::Check { script } => match CheckCommand::new(script, ctx.output_format()).run() {
            Ok(_) => CommandResult::success(()),
            Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
        },
        Commands::Devices => convert_result(DevicesCommand::new(ctx.output_format()).run()),
        Commands::Run {
            script,
            debug,
            mock,
            device,
            record,
            trace,
            tap_timeout,
            combo_timeout,
            hold_delay,
        } => {
            use keyrx_core::cli::Command;
            let mut config = config;
            merge_cli_overrides(&mut config, tap_timeout, combo_timeout, hold_delay);
            let mut cmd = RunCommand::new(script, debug, mock, device, ctx.output_format())
                .with_record_path(record)
                .with_trace_path(trace)
                .with_config(config);
            cmd.execute(ctx)
        }
        Commands::State {
            layers,
            modifiers,
            pending,
            script,
        } => convert_result(
            StateCommand::new(layers, modifiers, pending, script, ctx.output_format()).run(),
        ),
        Commands::Doctor { verbose } => {
            convert_result(DoctorCommand::new(verbose, ctx.output_format()).run())
        }
        Commands::Repl => convert_result(ReplCommand::new(ctx.output_format()).run()),
        Commands::Bench { iterations, script } => {
            let result = BenchCommand::new(iterations, script, ctx.output_format())
                .run()
                .await;
            convert_result(result)
        }
        Commands::Simulate {
            input,
            script,
            hold_ms,
            combo,
            interactive,
        } => {
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
        Commands::Discover { device, force, yes } => {
            let result = DiscoverCommand::new(device, force, yes, ctx.output_format())
                .run()
                .await;
            convert_result(result)
        }
        Commands::Test {
            script,
            filter,
            watch,
        } => match TestCommand::new(script, ctx.output_format())
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
        },
        Commands::Replay {
            session,
            verify,
            speed,
        } => {
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
        Commands::Analyze { session, diagram } => {
            match AnalyzeCommand::new(session, ctx.output_format())
                .with_diagram(diagram)
                .run()
            {
                Ok(_) => CommandResult::success(()),
                Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
            }
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
        } => match UatCommand::new(ctx.output_format())
            .with_categories(category)
            .with_priorities(priority)
            .with_json(json)
            .with_fail_fast(fail_fast)
            .with_perf(perf)
            .with_fuzz(fuzz)
            .with_fuzz_duration(fuzz_duration)
            .with_fuzz_count(fuzz_count)
            .with_coverage_report(coverage)
            .with_report(report)
            .with_report_format(report_format)
            .with_report_output(report_output)
            .with_gate(gate)
            .run()
        {
            Ok(0) => CommandResult::success(()),
            Ok(code) => CommandResult::failure(
                ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
                format!("UAT failed with exit code {code}"),
            ),
            Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
        },
        Commands::Golden { command } => {
            let subcommand = match command {
                GoldenCommands::Record { name, script } => {
                    GoldenSubcommand::Record { name, script }
                }
                GoldenCommands::Verify { name, script } => {
                    GoldenSubcommand::Verify { name, script }
                }
                GoldenCommands::Update {
                    name,
                    script,
                    confirm,
                } => GoldenSubcommand::Update {
                    name,
                    script,
                    confirm,
                },
                GoldenCommands::List => GoldenSubcommand::List,
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
        Commands::Regression { golden_dir, json } => {
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
        Commands::CiCheck {
            gate,
            json,
            skip_unit,
            skip_integration,
            skip_uat,
            skip_regression,
            skip_perf,
        } => match CiCheckCommand::new(ctx.output_format())
            .with_gate(gate)
            .with_json(json)
            .with_skip_unit(skip_unit)
            .with_skip_integration(skip_integration)
            .with_skip_uat(skip_uat)
            .with_skip_regression(skip_regression)
            .with_skip_perf(skip_perf)
            .run()
        {
            Ok(0) => CommandResult::success(()),
            Ok(code) => CommandResult::failure(
                ExitCode::from_u8(code as u8).unwrap_or(ExitCode::GeneralError),
                format!("CI check failed with exit code {code}"),
            ),
            Err(err) => CommandResult::failure(err.exit_code(), format!("{err:#}")),
        },
    }
}
