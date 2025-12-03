//! KeyRx CLI entry point.

use clap::{Parser, Subcommand};
use keyrx_core::cli::{
    commands::{
        golden_exit_codes, replay_exit_codes, test_exit_codes, uat_exit_codes, AnalyzeCommand,
        BenchCommand, CheckCommand, DevicesCommand, DiscoverCommand, DiscoverExit, DoctorCommand,
        GoldenCommand, GoldenSubcommand, ReplCommand, ReplayCommand, RunCommand, SimulateCommand,
        StateCommand, TestCommand, UatCommand,
    },
    OutputFormat,
};
use keyrx_core::KeyRxError;
use std::path::PathBuf;
use std::process::ExitCode;

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
        #[arg(short, long)]
        input: String,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Hold duration in milliseconds for each key (overrides default)
        #[arg(long)]
        hold_ms: Option<u64>,

        /// Treat input keys as a simultaneous combo
        #[arg(long)]
        combo: bool,
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

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = parse_format(&cli.format, cli.json);

    let result = run_command(cli.command, format).await;

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let exit_code = determine_exit_code(&err);
            eprintln!("Error: {err:#}");
            exit_code
        }
    }
}

async fn run_command(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Check { script } => {
            CheckCommand::new(script, format).run()?;
        }
        Commands::Devices => {
            DevicesCommand::new(format).run()?;
        }
        Commands::Run {
            script,
            debug,
            mock,
            device,
            record,
            trace,
        } => {
            RunCommand::new(script, debug, mock, device, format)
                .with_record_path(record)
                .with_trace_path(trace)
                .run()
                .await?;
        }
        Commands::State {
            layers,
            modifiers,
            pending,
            script,
        } => {
            StateCommand::new(layers, modifiers, pending, script, format).run()?;
        }
        Commands::Doctor { verbose } => {
            DoctorCommand::new(verbose, format).run()?;
        }
        Commands::Repl => {
            ReplCommand::new(format).run()?;
        }
        Commands::Bench { iterations, script } => {
            BenchCommand::new(iterations, script, format).run().await?;
        }
        Commands::Simulate {
            input,
            script,
            hold_ms,
            combo,
        } => {
            SimulateCommand::new(input, script, format)
                .with_hold_ms(hold_ms)
                .with_combo(combo)
                .run()
                .await?;
        }
        Commands::Discover { device, force, yes } => {
            DiscoverCommand::new(device, force, yes, format)
                .run()
                .await?;
        }
        Commands::Test {
            script,
            filter,
            watch,
        } => {
            let exit_code = TestCommand::new(script, format)
                .with_filter(filter)
                .with_watch(watch)
                .run()?;

            // Return early with specific exit code for test failures
            if exit_code != test_exit_codes::PASS {
                return Err(anyhow::anyhow!("Tests failed with exit code {}", exit_code));
            }
        }
        Commands::Replay {
            session,
            verify,
            speed,
        } => {
            let result = ReplayCommand::new(session, format)
                .with_verify(verify)
                .with_speed(speed)
                .run()
                .await?;

            // Return early with specific exit code for verification failures
            if verify && !result.all_matched() {
                return Err(anyhow::anyhow!(
                    "Replay verification failed with exit code {}",
                    replay_exit_codes::VERIFICATION_FAILED
                ));
            }
        }
        Commands::Analyze { session, diagram } => {
            AnalyzeCommand::new(session, format)
                .with_diagram(diagram)
                .run()?;
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
            let exit_code = UatCommand::new(format)
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
                .run()?;

            // Return early with specific exit code for UAT failures
            if exit_code != uat_exit_codes::PASS {
                return Err(anyhow::anyhow!("UAT failed with exit code {}", exit_code));
            }
        }
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

            let exit_code = GoldenCommand::new(subcommand, format).run()?;

            // Return early with specific exit code for golden failures
            if exit_code != golden_exit_codes::SUCCESS {
                return Err(anyhow::anyhow!(
                    "Golden command failed with exit code {}",
                    exit_code
                ));
            }
        }
    }
    Ok(())
}

/// Determine the exit code based on the error type.
///
/// - Exit code 0: Success (pass)
/// - Exit code 1: General runtime errors
/// - Exit code 2: Validation/compilation errors (script syntax issues), test assertion failures, replay verification failures
/// - Exit code 3: Discovery cancelled by user/emergency-exit, test timeout
fn determine_exit_code(err: &anyhow::Error) -> ExitCode {
    // Check for test command exit codes in the error message
    let err_str = err.to_string();
    if err_str.contains("Tests failed with exit code") {
        if err_str.contains(&format!("{}", test_exit_codes::ASSERTION_FAIL)) {
            return ExitCode::from(test_exit_codes::ASSERTION_FAIL as u8);
        }
        if err_str.contains(&format!("{}", test_exit_codes::TIMEOUT)) {
            return ExitCode::from(test_exit_codes::TIMEOUT as u8);
        }
        return ExitCode::from(test_exit_codes::ERROR as u8);
    }

    // Check for replay verification failures
    if err_str.contains("Replay verification failed with exit code") {
        return ExitCode::from(replay_exit_codes::VERIFICATION_FAILED);
    }

    // Check for UAT failures
    if err_str.contains("UAT failed with exit code") {
        if err_str.contains(&format!("{}", uat_exit_codes::TEST_FAIL)) {
            return ExitCode::from(uat_exit_codes::TEST_FAIL as u8);
        }
        if err_str.contains(&format!("{}", uat_exit_codes::GATE_FAIL)) {
            return ExitCode::from(uat_exit_codes::GATE_FAIL as u8);
        }
        if err_str.contains(&format!("{}", uat_exit_codes::CRASH)) {
            return ExitCode::from(uat_exit_codes::CRASH as u8);
        }
        return ExitCode::from(uat_exit_codes::TEST_FAIL as u8);
    }

    // Check for golden session failures
    if err_str.contains("Golden command failed with exit code") {
        if err_str.contains(&format!("{}", golden_exit_codes::VERIFICATION_FAILED)) {
            return ExitCode::from(golden_exit_codes::VERIFICATION_FAILED as u8);
        }
        if err_str.contains(&format!("{}", golden_exit_codes::CONFIRMATION_REQUIRED)) {
            return ExitCode::from(golden_exit_codes::CONFIRMATION_REQUIRED as u8);
        }
        return ExitCode::from(golden_exit_codes::ERROR as u8);
    }

    // Check if the root cause is a KeyRxError
    if let Some(discover) = err.downcast_ref::<DiscoverExit>() {
        return match discover {
            DiscoverExit::Cancelled => ExitCode::from(3),
            DiscoverExit::Validation(_) => ExitCode::from(2),
        };
    }

    if let Some(keyrx_err) = err.downcast_ref::<KeyRxError>() {
        return match keyrx_err {
            KeyRxError::ScriptCompileError { .. } => ExitCode::from(2),
            _ => ExitCode::from(1),
        };
    }

    // Walk the error chain for wrapped errors
    for cause in err.chain() {
        if let Some(keyrx_err) = cause.downcast_ref::<KeyRxError>() {
            return match keyrx_err {
                KeyRxError::ScriptCompileError { .. } => ExitCode::from(2),
                _ => ExitCode::from(1),
            };
        }
        if let Some(discover) = cause.downcast_ref::<DiscoverExit>() {
            return match discover {
                DiscoverExit::Cancelled => ExitCode::from(3),
                DiscoverExit::Validation(_) => ExitCode::from(2),
            };
        }
    }

    ExitCode::from(1)
}
