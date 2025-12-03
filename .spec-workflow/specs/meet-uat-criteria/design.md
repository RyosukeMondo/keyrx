# Design Document

## Overview

This design implements an **Automated UAT System** that shifts traditional User Acceptance Testing left into the development and CI/CD pipeline. By leveraging KeyRx's unique capabilities—deterministic input/output, session recording/replay, CLI-first architecture—we can automate the vast majority of UAT scenarios, reducing manual testing to smoke verification only.

The system extends the existing Rhai test harness with UAT-specific primitives, introduces golden session management for regression detection, and provides configurable quality gates for release readiness.

## Steering Document Alignment

### Technical Standards (tech.md)

- **CLI First**: All UAT commands available via `keyrx uat`, `keyrx regression`, `keyrx ci-check`
- **Rhai Scripting**: Uses existing test harness infrastructure (`simulate_tap()`, `assert_output()`)
- **Session Recording**: Extends existing `.krx` session format for golden sessions
- **Performance Targets**: Enforces <1ms latency requirement, CI fails on >100µs regression
- **Fuzz Testing**: Extends existing proptest/fuzz infrastructure

### Project Structure (structure.md)

```
keyrx/
├── core/
│   └── src/
│       ├── uat/                    # NEW: UAT system module
│       │   ├── mod.rs              # Module exports
│       │   ├── runner.rs           # UAT test discovery & execution
│       │   ├── golden.rs           # Golden session management
│       │   ├── gates.rs            # Quality gate enforcement
│       │   ├── coverage.rs         # Requirements coverage mapping
│       │   ├── perf.rs             # Performance UAT
│       │   ├── fuzz.rs             # Fuzz-based chaos testing
│       │   └── report.rs           # Report generation
│       └── cli/
│           └── commands/
│               ├── uat.rs          # NEW: keyrx uat command
│               ├── regression.rs   # NEW: keyrx regression command
│               ├── golden.rs       # NEW: keyrx record-golden, verify-golden
│               └── ci_check.rs     # NEW: keyrx ci-check command
├── tests/
│   ├── uat/                        # NEW: UAT test scripts
│   │   ├── core/                   # Core functionality UAT
│   │   ├── layers/                 # Layer switching UAT
│   │   └── combos/                 # Combo execution UAT
│   └── golden/                     # NEW: Golden session recordings
│       ├── basic_typing.krx
│       ├── layer_switch.krx
│       └── combo_execution.krx
├── .keyrx/
│   └── quality-gates.toml          # NEW: Quality gate definitions
└── target/
    └── uat-report/                 # Generated reports
```

## Code Reuse Analysis

### Existing Components to Leverage

- **`core/src/test_harness/`**: Existing Rhai test infrastructure
  - `simulate_tap()`, `simulate_hold()`, `assert_output()` functions
  - Script discovery and execution engine
  - Will extend with `uat_*` prefix handling and metadata parsing

- **`core/src/session/`**: Session recording/replay
  - `.krx` format for event serialization
  - Replay engine for deterministic reproduction
  - Will extend with semantic comparison (ignoring non-deterministic fields)

- **`core/src/cli/`**: Existing CLI framework
  - Subcommand structure and argument parsing
  - Will add new subcommands: `uat`, `regression`, `record-golden`, etc.

- **`core/benches/latency.rs`**: Existing benchmark infrastructure
  - Criterion-based latency measurement
  - Will integrate with performance UAT for threshold enforcement

### Integration Points

- **Test Runner**: UAT runner wraps existing test harness
- **Session Format**: Golden sessions use existing `.krx` format with metadata extension
- **CI Pipeline**: `ci-check` command integrates with existing CI workflow
- **Metrics Collection**: Uses existing timing infrastructure for latency capture

## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            UAT Command Layer                                 │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  ┌─────────────────────┐ │
│  │ keyrx uat   │  │keyrx         │  │keyrx       │  │ keyrx ci-check     │ │
│  │             │  │regression    │  │*-golden    │  │                     │ │
│  └──────┬──────┘  └──────┬───────┘  └─────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────┼────────────────────┼────────────┘
          │                │                │                    │
          ▼                ▼                ▼                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UAT Core Engine                                    │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │  Test Runner    │  │ Golden Session  │  │     Quality Gate            │  │
│  │  - Discovery    │  │   Manager       │  │       Enforcer              │  │
│  │  - Execution    │  │  - Record       │  │  - Config loading           │  │
│  │  - Metrics      │  │  - Replay       │  │  - Threshold evaluation     │  │
│  │  - Categories   │  │  - Compare      │  │  - Gate selection           │  │
│  └────────┬────────┘  └────────┬────────┘  └─────────────┬───────────────┘  │
│           │                    │                          │                  │
│           ▼                    ▼                          ▼                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                    Shared Infrastructure                                 ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ ││
│  │  │ Coverage    │  │ Performance │  │ Fuzz        │  │ Report          │ ││
│  │  │ Mapper      │  │ UAT         │  │ Engine      │  │ Generator       │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
          │                    │                          │
          ▼                    ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Existing Infrastructure                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │ Rhai Test       │  │ Session         │  │ Benchmark                   │  │
│  │ Harness         │  │ Recording       │  │ Infrastructure              │  │
│  │ (test_harness/) │  │ (session/)      │  │ (benches/)                  │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         UAT Execution Flow                                │
│                                                                           │
│  1. Discovery          2. Execution          3. Evaluation                │
│  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐             │
│  │ Scan tests/ │──────▶│ Run tests   │──────▶│ Apply       │             │
│  │ for uat_*   │       │ with timing │       │ quality     │             │
│  │ functions   │       │ metrics     │       │ gates       │             │
│  └─────────────┘       └─────────────┘       └──────┬──────┘             │
│        │                     │                      │                     │
│        ▼                     ▼                      ▼                     │
│  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐             │
│  │ Parse       │       │ Collect:    │       │ Determine   │             │
│  │ metadata:   │       │ - Results   │       │ pass/fail   │             │
│  │ @category   │       │ - Timing    │       │ per gate    │             │
│  │ @priority   │       │ - Coverage  │       │ criteria    │             │
│  │ @requirement│       │             │       │             │             │
│  └─────────────┘       └─────────────┘       └──────┬──────┘             │
│                                                      │                    │
│  4. Reporting                                        │                    │
│  ┌────────────────────────────────────────────┐      │                    │
│  │ Generate report with:                       │◀────┘                    │
│  │ - Summary (pass/fail counts)               │                           │
│  │ - Test results by category                 │                           │
│  │ - Coverage matrix                          │                           │
│  │ - Performance metrics                      │                           │
│  │ - Quality gate status                      │                           │
│  └────────────────────────────────────────────┘                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Modular Design Principles

- **Single Responsibility**: Each module handles one concern (runner, golden, gates, etc.)
- **Component Isolation**: UAT system is a separate module from existing test harness
- **Extensibility**: New UAT primitives can be added without modifying core
- **Fail Fast**: Quality gates evaluated immediately after test execution

## Components and Interfaces

### Component 1: UAT Test Runner (`uat/runner.rs`)

**Purpose:** Discover, categorize, and execute UAT tests with metrics collection

**Interfaces:**
```rust
pub struct UatRunner {
    test_dir: PathBuf,
    harness: TestHarness,
}

impl UatRunner {
    /// Discover all uat_* functions in test files
    pub fn discover(&self) -> Result<Vec<UatTest>>;

    /// Run tests matching filter criteria
    pub fn run(&self, filter: UatFilter) -> Result<UatResults>;

    /// Run with fail-fast behavior
    pub fn run_fail_fast(&self, filter: UatFilter) -> Result<UatResults>;
}

pub struct UatFilter {
    pub categories: Option<Vec<String>>,
    pub priorities: Option<Vec<Priority>>,
    pub pattern: Option<String>,
}

pub struct UatTest {
    pub name: String,
    pub file: PathBuf,
    pub category: Option<String>,
    pub priority: Priority,
    pub requirements: Vec<String>,
    pub latency_threshold: Option<Duration>,
}

pub struct UatResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub tests: Vec<UatTestResult>,
}
```

**Dependencies:** TestHarness, Rhai runtime
**Reuses:** `core/src/test_harness/` discovery and execution logic

### Component 2: Golden Session Manager (`uat/golden.rs`)

**Purpose:** Record, store, and compare golden session recordings

**Interfaces:**
```rust
pub struct GoldenSessionManager {
    golden_dir: PathBuf,
}

impl GoldenSessionManager {
    /// Record a new golden session from script execution
    pub fn record(&self, name: &str, script: &Path) -> Result<GoldenSession>;

    /// Verify current output matches golden session
    pub fn verify(&self, name: &str) -> Result<GoldenVerifyResult>;

    /// Update existing golden session
    pub fn update(&self, name: &str, confirm: bool) -> Result<()>;

    /// List all golden sessions
    pub fn list(&self) -> Result<Vec<GoldenSessionInfo>>;
}

pub struct GoldenSession {
    pub name: String,
    pub created: DateTime<Utc>,
    pub events: Vec<RecordedEvent>,
    pub outputs: Vec<ExpectedOutput>,
}

pub struct GoldenVerifyResult {
    pub matches: bool,
    pub differences: Vec<Difference>,
}

pub struct Difference {
    pub event_index: usize,
    pub expected: String,
    pub actual: String,
    pub diff_type: DiffType,
}
```

**Dependencies:** Session recording module
**Reuses:** `core/src/session/` format and replay logic

### Component 3: Quality Gate Enforcer (`uat/gates.rs`)

**Purpose:** Load and enforce quality gate configurations

**Interfaces:**
```rust
pub struct QualityGateEnforcer {
    config_path: PathBuf,
}

impl QualityGateEnforcer {
    /// Load quality gate configuration
    pub fn load(&self, gate_name: Option<&str>) -> Result<QualityGate>;

    /// Evaluate results against quality gate
    pub fn evaluate(&self, gate: &QualityGate, results: &UatResults) -> GateResult;
}

pub struct QualityGate {
    pub name: String,
    pub pass_rate: Option<f64>,         // e.g., 95.0
    pub p0_open: Option<u32>,            // e.g., 0
    pub p1_open: Option<u32>,            // e.g., 2
    pub max_latency_us: Option<u64>,     // e.g., 1000
    pub coverage_min: Option<f64>,       // e.g., 80.0
}

pub struct GateResult {
    pub passed: bool,
    pub violations: Vec<GateViolation>,
}

pub struct GateViolation {
    pub criterion: String,
    pub expected: String,
    pub actual: String,
}
```

**Dependencies:** None
**Reuses:** TOML parsing patterns from existing config loading

### Component 4: Coverage Mapper (`uat/coverage.rs`)

**Purpose:** Track requirement-to-test traceability

**Interfaces:**
```rust
pub struct CoverageMapper {
    requirements_path: PathBuf,
}

impl CoverageMapper {
    /// Build coverage map from test metadata
    pub fn build(&self, tests: &[UatTest], results: &UatResults) -> CoverageMap;

    /// Generate coverage report
    pub fn report(&self, map: &CoverageMap) -> CoverageReport;
}

pub struct CoverageMap {
    pub requirements: HashMap<String, RequirementCoverage>,
}

pub struct RequirementCoverage {
    pub id: String,
    pub linked_tests: Vec<String>,
    pub status: CoverageStatus,
    pub last_verified: Option<DateTime<Utc>>,
}

pub enum CoverageStatus {
    Verified,    // All linked tests pass
    AtRisk,      // Some linked tests fail
    Uncovered,   // No linked tests
}
```

**Dependencies:** UatRunner
**Reuses:** Requirement parsing from spec-workflow

### Component 5: Performance UAT (`uat/perf.rs`)

**Purpose:** Verify latency requirements are met

**Interfaces:**
```rust
pub struct PerformanceUat {
    baseline_path: Option<PathBuf>,
}

impl PerformanceUat {
    /// Run performance UAT tests
    pub fn run(&self, filter: UatFilter) -> Result<PerfResults>;

    /// Compare against baseline
    pub fn compare_baseline(&self, branch: &str) -> Result<PerfComparison>;
}

pub struct PerfResults {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub violations: Vec<LatencyViolation>,
}

pub struct LatencyViolation {
    pub test_name: String,
    pub event_index: usize,
    pub threshold_us: u64,
    pub actual_us: u64,
}
```

**Dependencies:** Benchmark infrastructure
**Reuses:** `core/benches/latency.rs` timing measurement

### Component 6: Fuzz Engine (`uat/fuzz.rs`)

**Purpose:** Generate random inputs for chaos testing

**Interfaces:**
```rust
pub struct FuzzEngine {
    crash_dir: PathBuf,
}

impl FuzzEngine {
    /// Run fuzz testing for specified duration
    pub fn run(&self, duration: Duration, count: Option<u64>) -> Result<FuzzResults>;

    /// Replay a crash sequence
    pub fn replay_crash(&self, crash_file: &Path) -> Result<()>;
}

pub struct FuzzResults {
    pub sequences_tested: u64,
    pub duration: Duration,
    pub unique_paths: u64,
    pub crashes: Vec<CrashInfo>,
}

pub struct CrashInfo {
    pub sequence_file: PathBuf,
    pub error: String,
    pub timestamp: DateTime<Utc>,
}
```

**Dependencies:** KeyEvent generation, Engine
**Reuses:** Existing fuzz testing infrastructure

### Component 7: Report Generator (`uat/report.rs`)

**Purpose:** Generate comprehensive UAT reports

**Interfaces:**
```rust
pub struct ReportGenerator {
    output_dir: PathBuf,
}

impl ReportGenerator {
    /// Generate HTML report
    pub fn generate_html(&self, data: &ReportData) -> Result<PathBuf>;

    /// Generate Markdown report (for PR comments)
    pub fn generate_markdown(&self, data: &ReportData) -> Result<String>;

    /// Generate JSON report (machine-readable)
    pub fn generate_json(&self, data: &ReportData) -> Result<String>;
}

pub struct ReportData {
    pub summary: UatSummary,
    pub test_results: Vec<UatTestResult>,
    pub coverage: CoverageMap,
    pub performance: PerfResults,
    pub gate_status: GateResult,
    pub trend: Option<TrendData>,
}
```

**Dependencies:** All UAT components
**Reuses:** Template patterns from existing report generation

### Component 8: CI Check Command (`cli/commands/ci_check.rs`)

**Purpose:** Unified CI command that runs all checks

**Interfaces:**
```rust
pub struct CiCheckCommand {
    gate: Option<String>,
    json_output: bool,
}

impl CiCheckCommand {
    /// Run all CI checks
    pub fn run(&self) -> Result<CiCheckResults>;
}

pub struct CiCheckResults {
    pub unit_tests: TestSuiteResult,
    pub integration_tests: TestSuiteResult,
    pub uat_tests: UatResults,
    pub regression_tests: RegressionResults,
    pub performance_tests: PerfResults,
    pub gate_result: GateResult,
}

/// Exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_TEST_FAIL: i32 = 2;
pub const EXIT_GATE_FAIL: i32 = 3;
```

**Dependencies:** All test runners, quality gates
**Reuses:** Existing CLI framework

## Data Models

### Quality Gate Configuration (TOML)

```toml
# .keyrx/quality-gates.toml

[default]
pass_rate = 95.0
p0_open = 0
p1_open = 2
max_latency_us = 1000
coverage_min = 80.0

[alpha]
pass_rate = 80.0
p0_open = 0
p1_open = 5
coverage_min = 60.0

[beta]
pass_rate = 90.0
p0_open = 0
p1_open = 2
coverage_min = 75.0

[rc]
pass_rate = 98.0
p0_open = 0
p1_open = 0
max_latency_us = 500
coverage_min = 85.0

[ga]
pass_rate = 100.0
p0_open = 0
p1_open = 0
max_latency_us = 500
coverage_min = 90.0
```

### UAT Test Metadata (Rhai Comments)

```rhai
// @category: core
// @priority: P0
// @requirement: 1.1, 1.2
// @latency: 1000

fn uat_basic_key_mapping() {
    simulate_tap(KEY_A);
    assert_output("a");
    assert_timing(0, 1000);  // 0-1000µs
}
```

### Golden Session Format (JSON)

```json
{
  "name": "basic_typing",
  "version": "1.0",
  "created": "2024-12-03T10:00:00Z",
  "metadata": {
    "description": "Basic key-to-character mapping",
    "requirements": ["1.1"]
  },
  "events": [
    {"type": "key_press", "code": 30, "time_us": 0},
    {"type": "key_release", "code": 30, "time_us": 50000}
  ],
  "expected_outputs": [
    {"index": 0, "output": "a", "timing_range_us": [0, 1000]}
  ]
}
```

### UAT Results (JSON)

```json
{
  "total": 100,
  "passed": 95,
  "failed": 3,
  "skipped": 2,
  "duration_ms": 5000,
  "by_category": {
    "core": {"total": 50, "passed": 49, "failed": 1},
    "layers": {"total": 30, "passed": 28, "failed": 2},
    "combos": {"total": 20, "passed": 18, "failed": 0, "skipped": 2}
  },
  "by_priority": {
    "P0": {"total": 20, "passed": 20, "failed": 0},
    "P1": {"total": 40, "passed": 38, "failed": 2},
    "P2": {"total": 40, "passed": 37, "failed": 1, "skipped": 2}
  },
  "failed_tests": [
    {"name": "uat_layer_rapid_switch", "error": "Assertion failed: expected layer 2, got layer 1"}
  ]
}
```

## Error Handling

### Error Scenarios

1. **UAT test discovery fails (no tests found)**
   - **Handling:** Return warning, exit with success (not an error)
   - **User Impact:** Message: "No UAT tests found in tests/uat/"

2. **Golden session not found**
   - **Handling:** Return error with helpful message
   - **User Impact:** "Golden session 'foo' not found. Run 'keyrx record-golden foo' first."

3. **Quality gate config not found**
   - **Handling:** Use default quality gate
   - **User Impact:** Warning: "Using default quality gate. Create .keyrx/quality-gates.toml to customize."

4. **Latency threshold exceeded**
   - **Handling:** Mark test as failed, include in report
   - **User Impact:** Report shows exact event and latency: "Event 5 took 1500µs (threshold: 1000µs)"

5. **Fuzz testing crash**
   - **Handling:** Save sequence to tests/crashes/, continue testing
   - **User Impact:** "Crash detected! Saved to tests/crashes/2024-12-03T10-00-00.krx"

6. **Report generation fails**
   - **Handling:** Fall back to console output
   - **User Impact:** Warning: "Could not write report to file. Displaying in console."

### Exit Codes

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | All tests pass, all gates pass |
| 1 | Error | System error (config parse, file not found) |
| 2 | Test Failure | One or more tests failed |
| 3 | Gate Failure | Tests passed but gate criteria not met |

## Testing Strategy

### Unit Testing

- **Runner tests:** Test metadata parsing, filter matching, result aggregation
- **Golden tests:** Test comparison logic, diff generation
- **Gate tests:** Test threshold evaluation, violation detection
- **Coverage tests:** Test requirement linking, status calculation

### Integration Testing

- **Full UAT flow:** Discovery → Execution → Gate evaluation → Report
- **Golden session lifecycle:** Record → Verify → Update
- **CI check flow:** All test types → Consolidated report

### End-to-End Testing

- **New developer scenario:** Clone → Setup → Run UAT → All pass
- **Regression scenario:** Make breaking change → Regression detected → Fix → Pass
- **Release scenario:** Run ci-check with RC gate → Generate release report

## Implementation Sequence

1. **Core UAT runner** (runner.rs) - Foundation for test discovery and execution
2. **Metadata parsing** - @category, @priority, @requirement parsing
3. **Quality gates** (gates.rs) - Gate configuration and evaluation
4. **CLI commands** - keyrx uat command
5. **Golden sessions** (golden.rs) - Record/verify/update functionality
6. **Regression command** - keyrx regression command
7. **Coverage mapping** (coverage.rs) - Requirements traceability
8. **Performance UAT** (perf.rs) - Latency verification
9. **Fuzz engine** (fuzz.rs) - Chaos testing
10. **Report generator** (report.rs) - HTML/Markdown/JSON output
11. **CI check command** - Unified CI command
12. **Integration tests** - End-to-end verification
