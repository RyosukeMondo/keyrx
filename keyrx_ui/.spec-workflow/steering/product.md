# Product Overview

## Product Purpose

keyrx is an ultra-low latency, deterministic keyboard remapping system that bridges the gap between firmware-level solutions (like QMK) and OS-level software remappers (like Karabiner-Elements, AutoHotKey). Built in Rust, keyrx delivers firmware-class performance (<1ms latency) with software-level flexibility, enabling professional users and competitive gamers to customize their keyboard input without hardware constraints or compilation barriers.

The system solves the fundamental trade-off in keyboard customization: **firmware solutions are fast but inflexible**, requiring hardware-specific implementations and recompilation for changes, while **OS-level solutions are flexible but slow**, suffering from garbage collection, interpreter overhead, and OS scheduler unpredictability.

## Target Users

### Primary Users

1. **Professional Power Users**
   - Need sub-millisecond response times for productivity workflows
   - Require complex multi-layered keyboard configurations
   - Want instant configuration changes without firmware flashing

2. **Competitive Gamers**
   - Demand deterministic, zero-lag input processing
   - Need device-specific configurations (e.g., different mappings per keyboard)
   - Require guaranteed performance under system load

3. **AI Coding Agents**
   - Unique design philosophy: "AI Coding Agent First"
   - Need machine-verifiable configuration without human UAT
   - Require deterministic, reproducible behavior for automated testing
   - Must validate complex configurations (255 modifiers × 255 lock keys) programmatically

### Pain Points Addressed

- **Firmware Latency**: Traditional software remappers introduce 5-50ms latency (unacceptable for gaming/professional use)
- **Configuration Complexity**: Managing 255 modifiers + 255 lock keys requires automated validation (impossible via manual testing)
- **Hardware Lock-in**: Firmware solutions require specific keyboard hardware
- **Deployment Friction**: Firmware changes require compilation and flashing; OS software changes are instant but slow

## Key Features

1. **Sub-Millisecond Latency Processing**
   - Target: <100μs processing time (100x faster than typical OS remappers)
   - Zero-copy deserialization with rkyv
   - Lock-free ring buffers for event handling
   - No heap allocation in hot path

2. **Extreme Configuration Flexibility**
   - Support for 255 custom modifier keys (vs. standard 8)
   - Support for 255 custom lock keys
   - Rhai scripting language for configuration DSL
   - Compile-time evaluation: flexibility of scripting + performance of static tables

3. **Cross-Platform OS Integration**
   - Windows: Low-Level Hooks (WH_KEYBOARD_LL) + Raw Input for device identification
   - Linux: evdev/uinput with EVIOCGRAB for kernel-level interception
   - Device-specific configuration via serial number matching

4. **Advanced Input Logic**
   - Deterministic Finite Automaton (DFA) for Tap/Hold behavior
   - Retroactive state correction (QMK-style Permissive Hold)
   - O(1) key lookup via Minimal Perfect Hash Functions (MPHF)
   - Combo keys and layer switching

5. **AI-First Verification**
   - Browser-based WASM simulation with cycle-accurate execution
   - Deterministic Simulation Testing (DST) with virtual clock
   - Property-based testing (PBT) with proptest
   - Fuzz testing with cargo-fuzz
   - 100% configuration verification without human UAT

6. **Real-Time Simulation & Preview**
   - React + WASM frontend runs identical core logic in browser
   - Live visualization of state transitions (Pending → Held → Tapped)
   - Edit-and-preview workflow: instant feedback on configuration changes

7. **Multi-Device Support with Cross-Device State** (QMK-Inspired)
   - **N:M device-to-configuration mapping**: Multiple keyboards, modular configs
   - **Serial number-based identification**: True per-device configs (not USB port-dependent)
   - **Cross-device modifier sharing**: Hold Shift on Keyboard A, press A on Keyboard B → outputs 'A'
   - **QMK-inspired global state**: Proven architecture from split keyboard firmware
   - **Industry-first serial number support**: Fills gap left by Karabiner-Elements
   - **Modular configuration**: Single entry point with Rhai imports for code reuse

## Business Objectives

1. **Eliminate the Firmware vs. Software Trade-off**
   - Achieve firmware-level latency (<1ms) in software
   - Maintain software-level flexibility (no recompilation required)

2. **Enable AI-Driven Configuration Management**
   - Make keyboard remapping fully automatable by AI agents
   - Eliminate manual testing as a validation bottleneck
   - Support complex configurations beyond human testing capability

3. **Create a Platform for Input Innovation**
   - Enable experiments with novel input paradigms (255 modifiers, conditional layers)
   - Provide foundation for input research and competitive optimization

## Success Metrics

### Performance Metrics
- **Latency Budget**: <100μs processing time (target), <1ms maximum (hard requirement)
- **Lookup Performance**: O(1) constant-time key lookup (verified via benchmarks)
- **Determinism**: 100% reproducible behavior (same input → same output, verified via DST)

### Quality Metrics
- **Test Coverage**: 80% minimum, 90% for critical paths (enforced via CI)
- **Zero Manual Testing**: All validation automated (no UAT phase)
- **Fuzz Testing**: No crashes/panics under 1M+ random input sequences

### User Experience Metrics
- **Configuration Change Time**: <5 seconds from script edit to live deployment
- **Simulation Accuracy**: WASM simulation matches daemon behavior byte-for-byte
- **Error Detection**: Configuration errors caught at compile-time (before deployment)

## Product Principles

### 1. AI Coding Agent First
keyrx is designed to be verified, modified, and deployed by AI agents without human intervention. This architectural philosophy is enabled by two foundational mechanisms:

#### Single Source of Truth (SSOT)
- **Unified Configuration Store**: All system state, configuration, and runtime parameters exist in a single, authoritative location (`.krx` binary format)
- **Hash-Based Verification**: Configuration changes are verified via deterministic binary serialization—AI agents can confirm "configuration A == configuration B" with a simple hash comparison
- **No Configuration Drift**: Daemon, UI, and tests all consume the same compiled artifact; impossible for frontend to show one behavior while daemon executes another
- **Atomic Updates**: Configuration changes are all-or-nothing; no partial/inconsistent states during deployment

#### Structured Logging for Machine Observability
- **JSON-formatted logs**: Every event emitted in parseable JSON with strict schema (timestamp, level, service, event_type, context)
- **AI-Readable Diagnostics**: AI agents parse logs to verify behavior, diagnose issues, and validate test outcomes without human interpretation
- **Correlation IDs**: Request/event tracing through the entire pipeline (OS hook → remapping → injection)
- **Never log secrets/PII**: Logs are safe for AI agents to consume and analyze without data leakage concerns

These mechanisms enable:
- **CLI-first design**: Every GUI operation has a machine-readable CLI/API equivalent
- **Deterministic behavior**: Same input always produces same output (no randomness, no time-dependent behavior in tests)
- **Contract-based architecture**: Configuration (Rhai) and execution (Rust) separated by compile-time-verified IR
- **Automated validation**: AI agents verify correctness by comparing hashes, parsing logs, and executing deterministic tests

### 2. Complete Determinism
- Same input sequence + same configuration → identical output (bit-for-bit)
- Time is virtualized in test environments (no wall-clock dependencies)
- No undefined behavior, no race conditions, no non-deterministic optimizations

### 3. Observability & Controllability
- All internal state is inspectable (via debug mode, structured logging)
- All operations are reversible or testable in isolation
- Configuration is serialized to deterministic binary format (rkyv) for hash-based verification

### 4. Zero-Cost Abstractions
- High-level Rhai scripting for configuration
- Low-level Rust execution with no runtime overhead
- Compile-time transformation eliminates abstraction penalties

### 5. Module Isolation
- Core logic (keyrx_core) is `no_std` and OS-agnostic
- Platform-specific code isolated in keyrx_daemon
- WASM compatibility enables browser-based testing

## Monitoring & Visibility

### Development/Debug Mode
- **Dashboard Type**: Browser-based React UI (also available as Electron/Tauri desktop app)
- **Real-Time Updates**: WebSocket for live event streaming from daemon
- **Key Metrics Displayed**:
  - Current layer and active modifiers/locks (255-bit state vector)
  - Event latency histogram (per-key processing time)
  - State transition visualization (DFA states for Tap/Hold)
  - Input device list with serial numbers
- **Sharing Capabilities**: Configuration export as .krx binary or Rhai source

### Production Mode
- **Structured Logging (AI-First Design)**:
  - **Format**: JSON logs with strict schema: `{timestamp, level, service, event_type, context, correlation_id}`
  - **Machine-Parseable**: AI agents can programmatically query, filter, and analyze logs without regex/text parsing fragility
  - **Fail-Fast Validation**: Log schema violations are caught at compile-time (via typed logging macros)
  - **Zero PII/Secrets**: Safe for automated analysis and storage without data privacy concerns
  - **Example**: `{"timestamp":"2025-12-21T03:40:15.234Z","level":"INFO","service":"keyrx_daemon","event_type":"key_remap","context":{"input":"KeyA","output":"KeyB","latency_us":42},"correlation_id":"evt_abc123"}`
- **Performance Counters**: Latency percentiles (p50, p95, p99, max) exposed via `/metrics` endpoint (Prometheus-compatible)
- **Health Checks**: API endpoint for daemon status, active configuration hash (SSOT verification)
- **Configuration Audit Trail**: All configuration changes logged with hash, timestamp, and source (CLI/API/UI)

## Future Vision

keyrx is positioned to become the standard platform for advanced input customization, enabling use cases currently impossible with existing tools.

### Potential Enhancements

- **Remote Configuration Management**
  - Cloud sync for configurations across machines
  - Version control integration (Git-based config history)
  - Team/organization-level configuration sharing

- **Analytics & Telemetry**
  - Heatmaps of key usage patterns
  - Latency trend analysis over time
  - Configuration optimization recommendations (AI-suggested improvements)

- **Extended Platform Support**
  - macOS support (IOKit-based implementation)
  - Mobile platforms (Android/iOS with accessibility APIs)
  - Embedded systems (firmware alternative for custom keyboards)

- **Advanced Input Features**
  - Mouse remapping with same architecture
  - Gesture recognition (chord detection, sequence patterns)
  - Context-aware remapping (per-application configurations)
  - Macro recording and playback with timing precision

- **Ecosystem Development**
  - Community configuration marketplace
  - Plugin system for custom actions (shell commands, IPC triggers)
  - Integration with productivity tools (IDE-aware layers, app-specific profiles)

### Long-Term Vision

keyrx aims to redefine input systems as **programmable, verifiable, and AI-manageable infrastructure**. Just as compilers transformed high-level languages into optimized machine code, keyrx transforms high-level input intentions (Rhai scripts) into deterministic, hardware-speed execution.

The ultimate goal: **make input customization as powerful and accessible as code itself**, with the same rigor, testability, and automation that modern software development demands.
