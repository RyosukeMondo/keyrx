//! Top-level CLI and command definitions.

use super::config::{
    DeviceCommands, GoldenCommands, HardwareCommands, KeymapCommands, LayoutCommands,
    RuntimeCommands,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Main CLI struct parsed by clap.
#[derive(Parser)]
#[command(name = "keyrx")]
#[command(about = "KeyRx - The Ultimate Input Remapping Engine")]
#[command(version)]
#[command(after_help = "\
EXIT CODES:
  0 - Success
  1 - General error
  2 - Assertion failed (tests/validation)
  3 - Device not found
  4 - Permission denied
  5 - Timeout
  6 - Invalid argument
  7 - Configuration error
  101 - Panic (internal error)

LOGGING:
  Control logging with environment variables:
    RUST_LOG=<level>           Set log level (trace, debug, info, warn, error)
    KEYRX_LOG_FORMAT=<format>  Set log format (pretty, json)

  Examples:
    RUST_LOG=debug keyrx run --script my_config.rhai
    RUST_LOG=keyrx_core=trace keyrx doctor

For detailed exit code information, run:
  keyrx exit-codes
  keyrx exit-codes --json
")]
pub struct Cli {
    /// Output format (human, json, or yaml)
    #[arg(
        long = "output-format",
        visible_alias = "format",
        default_value = "human",
        value_parser = ["human", "json", "yaml"],
        global = true,
        conflicts_with = "json"
    )]
    pub output_format: String,

    /// Shortcut for JSON output (equivalent to --output-format json)
    #[arg(long, global = true, conflicts_with = "output_format")]
    pub json: bool,

    /// Path to configuration file (default: ~/.config/keyrx/config.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level commands available in the CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Validate and lint a Rhai script
    Check {
        /// Path to the script file
        script: PathBuf,
    },

    /// Manage devices and bindings
    Devices {
        #[command(subcommand)]
        command: Option<DeviceCommands>,
    },

    /// Hardware detection, profiles, and calibration
    Hardware {
        #[command(subcommand)]
        command: HardwareCommands,
    },

    /// Manage virtual layouts
    Layout {
        #[command(subcommand)]
        command: Option<LayoutCommands>,
    },

    /// Manage logical keymaps
    Keymap {
        #[command(subcommand)]
        command: Option<KeymapCommands>,
    },

    /// Inspect and modify runtime profile slots
    Runtime {
        #[command(subcommand)]
        command: Option<RuntimeCommands>,
    },

    /// Show all exit codes with descriptions
    ExitCodes,

    /// Generate API documentation
    Docs {
        /// Output format (markdown, html, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output directory (default: ./docs)
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
    },

    /// Run the engine in headless mode
    Run {
        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Run without keyboard capture (for CI/daemon mode). Use 'keyrx simulate' for interactive testing.
        #[arg(long, alias = "mock")]
        no_capture: bool,

        /// Load and validate script, then exit immediately
        #[arg(long)]
        validate_only: bool,

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

        /// Disable compiled script caching for this run
        #[arg(long)]
        no_cache: bool,

        /// Clear the compiled script cache before running
        #[arg(long)]
        clear_cache: bool,
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

        /// Generate a flame graph SVG (writes bench-flamegraph.svg)
        #[arg(long)]
        flamegraph: bool,

        /// Generate an allocation report JSON (writes bench-allocations.json)
        #[arg(long)]
        allocations: bool,
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

    /// Migrate profiles from old version to new version
    Migrate {
        /// Source version to migrate from (only 'v1' is supported)
        #[arg(long, default_value = "v1")]
        from: String,

        /// Create backup of old profiles before migration
        #[arg(long)]
        backup: bool,
    },
}
