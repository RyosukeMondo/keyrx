# KeyRx Scripting Security Model

## Overview

KeyRx implements a multi-layered security model for Rhai script execution, combining capability-based access control, resource limits, and input validation to prevent malicious or buggy scripts from compromising system security or stability.

## Architecture

The security model operates in three complementary layers:

```
┌─────────────────────────────────────────────────┐
│             User Script Code                    │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Layer 1: Capability-Based Access Control       │
│  - Function tier validation                     │
│  - Execution mode enforcement                   │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Layer 2: Input Validation                      │
│  - Parameter type checking                      │
│  - Range validation                             │
│  - Format validation                            │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│  Layer 3: Resource Limits                       │
│  - Instruction count limit                      │
│  - Recursion depth limit                        │
│  - Memory allocation limit                      │
│  - Execution timeout                            │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         KeyRx Engine / System Resources         │
└─────────────────────────────────────────────────┘
```

## Layer 1: Capability-Based Access Control

### Capability Tiers

Functions are categorized into four security tiers:

#### Safe Tier (Level 0)
- **Purpose**: Pure functions with no side effects
- **Characteristics**:
  - No state modification
  - No I/O operations
  - Deterministic, bounded execution
  - Cannot call unsafe functions
- **Examples**: String operations, arithmetic, key code constants
- **Current Status**: No functions currently assigned (all KeyRx functions modify state)

#### Standard Tier (Level 1)
- **Purpose**: Core keyboard operations
- **Characteristics**:
  - May modify keyboard remapping state
  - May send key events
  - May access keyboard state
  - Cannot access system resources
- **Examples**: `remap()`, `layer_push()`, `modifier_on()`, `print_debug()`
- **Current Status**: 24 functions (all current KeyRx functions)

#### Advanced Tier (Level 2)
- **Purpose**: System interaction requiring explicit trust
- **Characteristics**:
  - May interact with system (clipboard, notifications)
  - May have side effects outside keyboard
  - Require explicit user trust
  - May access external resources
- **Examples**: Clipboard operations, system commands (future)
- **Current Status**: No functions currently assigned

#### Internal Tier (Level 3)
- **Purpose**: Engine internals only
- **Characteristics**:
  - Used only by engine internals
  - Can bypass safety checks
  - Never callable from user scripts
  - Not exposed in any mode
- **Examples**: Debug hooks, engine control (future)
- **Current Status**: No functions currently assigned

### Execution Modes

The execution mode determines which capability tiers are accessible:

#### Safe Mode
```
Allowed Tiers: Safe only
Use Case:      Untrusted scripts from unknown sources
Risk Level:    Minimal - no side effects possible
Current:       No functions available (waiting for Safe tier functions)
```

**Security guarantees:**
- No state modification
- No I/O operations
- Deterministic execution
- Cannot affect keyboard behavior

#### Standard Mode (Default)
```
Allowed Tiers: Safe + Standard
Use Case:      User's personal keyboard configuration
Risk Level:    Low - keyboard operations only
Current:       All 24 functions available
```

**Security guarantees:**
- Keyboard operations only
- No system resource access
- No external side effects
- Bounded execution time

**Recommended for:**
- Personal keyboard configurations
- Shared configs from trusted sources
- Default user settings

#### Full Mode
```
Allowed Tiers: Safe + Standard + Advanced
Use Case:      Advanced users needing system integration
Risk Level:    Medium - system interaction permitted
Current:       Same as Standard (no Advanced functions yet)
```

**Security considerations:**
- Clipboard access possible
- System commands possible
- External side effects possible
- Requires explicit user trust

**Recommended for:**
- Advanced power users
- Automation workflows
- System integration scenarios

### Enforcement Mechanism

Capability checks occur at multiple points:

1. **Registration Time** (core/src/scripting/sandbox/function_capabilities.rs:69)
   - Functions tagged with capability tier
   - Registry built with O(1) HashMap lookup

2. **Call Time** (core/src/scripting/sandbox/mod.rs:142)
   - Function name looked up in registry
   - Capability tier checked against current mode
   - Call rejected if tier insufficient

3. **Error Reporting** (core/src/scripting/sandbox/mod.rs:241)
   - Clear error messages
   - Shows required vs. current capability
   - Helps users understand restrictions

## Layer 2: Input Validation

### Validation Architecture

Input validation prevents malformed or malicious input from reaching function implementations.

#### Validator Trait (core/src/scripting/sandbox/validation.rs:135)

```rust
pub trait InputValidator<T> {
    fn validate(&self, value: &T) -> ValidationResult;
}
```

**Design principles:**
- Composable validators (AND/OR combinators)
- Reusable across functions
- Clear error messages
- Type-safe validation

#### Common Validators

**RangeValidator** (core/src/scripting/sandbox/validators/range.rs)
- Validates numeric ranges
- Used for: timeout values (0-5000ms), modifier IDs (0-255)
- Prevents: overflow, underflow, unreasonable values

**TypeValidator** (core/src/scripting/sandbox/validators/type_validator.rs)
- Validates Rhai dynamic types
- Used for: ensuring correct parameter types
- Prevents: type confusion attacks

**KeyCodeValidator** (core/src/scripting/sandbox/validators/keycode.rs)
- Validates key codes against known set
- Used for: all key code parameters
- Prevents: invalid key codes, injection attacks

**LengthValidator** (core/src/scripting/sandbox/validators/length.rs)
- Validates collection lengths
- Used for: combo keys (2-4 keys), layer names
- Prevents: unbounded allocations, DoS

**PatternValidator** (core/src/scripting/sandbox/validators/pattern.rs)
- Validates string patterns
- Used for: layer names (no colons, non-empty)
- Prevents: injection, parsing attacks

### Validation Error Handling

Validation errors are strongly typed and contextual (core/src/scripting/sandbox/validation.rs:16):

```rust
pub enum ValidationError {
    OutOfRange { value, min, max },
    WrongType { expected, actual },
    InvalidValue { context, reason },
    MissingParameter { parameter },
    InvalidLength { actual, constraint },
    PatternMismatch { pattern },
    Custom { message },
}
```

**Benefits:**
- Clear error messages for users
- Structured logging
- No information leakage
- Debugging support

## Layer 3: Resource Limits

Resource limits prevent denial-of-service attacks and accidental infinite loops.

### Tracked Resources

#### 1. Instruction Count (core/src/scripting/sandbox/budget.rs:63)

**Limit:** 100,000 instructions (default)
**Purpose:** Prevent CPU exhaustion
**Implementation:** Atomic counter, O(1) increment
**Enforcement:** Rhai engine built-in + manual checks

**Example attack prevented:**
```rhai
// Infinite loop - blocked at 100k instructions
loop { }
```

#### 2. Recursion Depth (core/src/scripting/sandbox/budget.rs:89)

**Limit:** 64 levels (default)
**Purpose:** Prevent stack overflow
**Implementation:** Atomic counter with RAII guard
**Enforcement:** Rhai call stack limit

**Example attack prevented:**
```rhai
fn recurse(n) { recurse(n + 1); }
recurse(0);  // Blocked at depth 64
```

#### 3. Memory Usage (core/src/scripting/sandbox/budget.rs:106)

**Limit:** 1 MB (default)
**Purpose:** Prevent memory exhaustion
**Implementation:** Manual tracking, atomic counter
**Enforcement:** Allocation tracking in bindings

**Example attack prevented:**
```rhai
// Large allocation - blocked at 1MB
let huge = [];
for i in 0..1000000 { huge.push(i); }
```

#### 4. Execution Timeout (core/src/scripting/sandbox/budget.rs:125)

**Limit:** 100ms (default)
**Purpose:** Prevent hanging scripts
**Implementation:** Instant-based elapsed time check
**Enforcement:** Manual timeout checks

**Example attack prevented:**
```rhai
// Long-running computation - timeout at 100ms
let sum = 0;
for i in 0..99999999 { sum += i; }
```

### Resource Configuration

Limits are user-configurable via ResourceConfig (core/src/scripting/sandbox/budget.rs:8):

```rust
pub struct ResourceConfig {
    pub max_instructions: u64,      // Default: 100,000
    pub max_recursion: u32,          // Default: 64
    pub max_memory: usize,           // Default: 1 MB
    pub timeout: Duration,           // Default: 100ms
}
```

**Configuration guidelines:**
- Default values suitable for 99% of use cases
- Increase for complex configs or slow systems
- Decrease for paranoid security
- Test configurations with representative scripts

### Performance Impact

Resource tracking is designed for minimal overhead:

- **Atomic counters**: Lock-free, thread-safe
- **O(1) operations**: Constant-time checks
- **RAII guards**: Zero-cost abstractions
- **Measured overhead**: <1% in benchmarks (core/benches/sandbox_bench.rs)

## Configuration

### User Configuration

Users control script mode via configuration file (core/src/config/scripting.rs:22):

```toml
[scripting]
mode = "Standard"  # Options: "Safe", "Standard", "Full"
```

**Default:** Standard mode (balances security and functionality)

**Configuration changes:**
- Take effect on next script reload
- Apply to all scripts
- Can be changed without restart
- Validated at load time

### Engine Configuration

The sandbox configures Rhai engine limits (core/src/scripting/sandbox/mod.rs:183):

```rust
pub fn configure_engine(&self, engine: &mut Engine) {
    engine.set_max_operations(100_000);      // Instruction limit
    engine.set_max_call_levels(64);          // Recursion limit
    engine.set_max_expr_depths(64, 64);      // Expression nesting
    engine.set_max_modules(16);              // Module limit
    engine.set_max_functions(256);           // Function limit
}
```

**Limits enforced at engine level:**
- Cannot be bypassed by scripts
- Apply to all script execution
- Independent of capability system
- Defense in depth

## Security Testing

### Property-Based Fuzzing (core/tests/sandbox_fuzz_test.rs)

Comprehensive fuzzing tests validate security properties:

#### Test Coverage

1. **Resource Exhaustion Tests**
   - Instruction count violations
   - Recursion depth violations
   - Memory limit violations
   - Timeout violations

2. **Input Validation Tests**
   - Range boundary testing
   - Type confusion testing
   - Malformed input testing
   - Injection attempt testing

3. **Capability Bypass Tests**
   - Mode enforcement testing
   - Tier escalation attempts
   - Function access testing

#### Fuzzing Strategy

**Tool:** PropTest (property-based testing framework)
**Approach:** Generate random inputs, verify invariants hold
**Properties tested:**
- Scripts cannot exceed resource limits
- Invalid inputs are rejected
- Capability restrictions are enforced
- No panics or crashes occur

### Performance Benchmarks (core/benches/sandbox_bench.rs)

Benchmarks verify security overhead is acceptable:

**Metrics:**
- Validation overhead: <10 µs per function call
- Capability check overhead: <100 ns (O(1) HashMap lookup)
- Resource tracking overhead: <50 ns (atomic operations)
- Total overhead: <1% for typical scripts

## Threat Model

### In-Scope Threats

The sandbox protects against:

1. **Malicious Scripts**
   - DoS via resource exhaustion → Blocked by resource limits
   - System access attempts → Blocked by capability system
   - Input injection attacks → Blocked by validation

2. **Buggy Scripts**
   - Infinite loops → Blocked by instruction limit & timeout
   - Stack overflow → Blocked by recursion limit
   - Memory leaks → Blocked by memory limit

3. **Untrusted Sources**
   - Scripts from unknown authors → Run in Safe mode
   - Scripts from internet → Validate before execution
   - Modified configs → Re-validate on change

### Out-of-Scope Threats

The sandbox does NOT protect against:

1. **Side-Channel Attacks**
   - Timing attacks to infer key presses
   - Power analysis
   - Cache timing

2. **Physical Attacks**
   - Keyboard hardware tampering
   - USB sniffing
   - DMA attacks

3. **OS-Level Attacks**
   - Kernel exploits
   - Driver vulnerabilities
   - Process memory inspection

4. **Logic Bugs**
   - Intentional malicious remapping (e.g., remapping delete key)
   - User shooting themselves in the foot
   - Confusing configurations

### Trust Boundaries

**Trusted:**
- KeyRx engine code (Rust, memory-safe)
- Configuration files owned by user
- User's own scripts in Standard mode

**Untrusted:**
- Scripts from unknown sources
- Scripts in Safe mode
- User input during runtime
- External data sources (future)

## Best Practices

### For Users

1. **Use Standard Mode by default**
   - Provides best balance of security and functionality
   - All keyboard operations work
   - No system access

2. **Review scripts before running**
   - Check for suspicious patterns
   - Verify source is trustworthy
   - Test in Safe mode first (when available)

3. **Keep resource limits reasonable**
   - Default limits work for 99% of cases
   - Only increase if scripts timeout
   - Monitor resource usage

4. **Update KeyRx regularly**
   - Security fixes in new versions
   - Improved validation
   - Performance improvements

### For Script Authors

1. **Design for Standard mode**
   - Use only keyboard operations
   - Avoid system interaction
   - Keep scripts simple

2. **Respect resource limits**
   - Avoid loops when possible
   - Use bounded iterations
   - Clean up resources

3. **Validate your own inputs**
   - Don't assume valid data
   - Handle errors gracefully
   - Test edge cases

4. **Document capability requirements**
   - State minimum required mode
   - Explain why Advanced needed (if applicable)
   - Provide alternatives for lower modes

### For Developers

1. **Conservative tier assignment**
   - When in doubt, use higher tier
   - Document rationale
   - Test thoroughly

2. **Always validate inputs**
   - Use existing validators
   - Create new validators for new patterns
   - Never trust user input

3. **Check resource usage**
   - Instrument expensive operations
   - Monitor budget usage
   - Fail early on limits

4. **Write security tests**
   - Add fuzz tests for new functions
   - Test boundary conditions
   - Verify error handling

## Future Enhancements

### Planned Improvements

1. **Safe Tier Functions**
   - Pure utility functions
   - String manipulation
   - Arithmetic helpers
   - Enable true Safe mode

2. **Advanced Tier Functions**
   - Clipboard integration
   - System notifications
   - External command execution
   - Require user approval

3. **Per-Script Modes**
   - Different modes for different scripts
   - Whitelist trusted scripts for Full mode
   - Per-function capability overrides

4. **Sandboxing Improvements**
   - Separate script contexts
   - Script-level resource quotas
   - Priority-based scheduling

5. **Auditing and Logging**
   - Capability check logging
   - Resource usage telemetry
   - Security event alerting

### Research Areas

1. **Formal Verification**
   - Prove capability enforcement properties
   - Verify resource limit enforcement
   - Validate state machine correctness

2. **Advanced Fuzzing**
   - Symbolic execution
   - Concolic testing
   - Grammar-based fuzzing

3. **Runtime Monitoring**
   - Anomaly detection
   - Behavior analysis
   - Adaptive limits

## References

### Implementation Files

- **Capability System**: `core/src/scripting/sandbox/capability.rs`
- **Resource Budget**: `core/src/scripting/sandbox/budget.rs`
- **Sandbox Core**: `core/src/scripting/sandbox/mod.rs`
- **Input Validation**: `core/src/scripting/sandbox/validation.rs`
- **Validators**: `core/src/scripting/sandbox/validators/`
- **Function Registry**: `core/src/scripting/sandbox/function_capabilities.rs`
- **Configuration**: `core/src/config/scripting.rs`

### Testing

- **Fuzz Tests**: `core/tests/sandbox_fuzz_test.rs`
- **Benchmarks**: `core/benches/sandbox_bench.rs`
- **Unit Tests**: Throughout sandbox module

### Documentation

- **Capability Tiers**: `docs/scripting-capability-tiers.md`
- **Architecture**: `docs/ARCHITECTURE.md`
- **Specification**: `.spec-workflow/specs/rhai-sandbox-hardening/`

## Appendix A: Resource Limit Tuning

### Performance vs. Security Tradeoffs

| Limit | Conservative | Balanced (Default) | Permissive | Use Case |
|-------|--------------|-------------------|------------|----------|
| Instructions | 10,000 | 100,000 | 1,000,000 | Complex scripts |
| Recursion | 16 | 64 | 256 | Deep call stacks |
| Memory | 256 KB | 1 MB | 10 MB | Large data structures |
| Timeout | 10 ms | 100 ms | 1000 ms | Slow systems |

### Tuning Guidelines

**Decrease limits if:**
- Running on embedded systems
- Security is paramount
- Scripts are simple
- Fast response required

**Increase limits if:**
- Scripts timing out
- Complex configurations
- Trusted environment
- Performance not critical

### Monitoring

Check resource usage via sandbox API:

```rust
let usage = sandbox.resource_usage();
println!("Instructions: {}/{}", usage.instructions, usage.max_instructions);
println!("Recursion: {}/{}", usage.recursion_depth, usage.max_recursion);
println!("Memory: {}/{} bytes", usage.memory_used, usage.max_memory);
println!("Time: {:?}/{:?}", usage.elapsed, usage.timeout);
```

## Appendix B: Security Checklist

### For New Functions

- [ ] Assigned capability tier with documented rationale
- [ ] All inputs validated with appropriate validators
- [ ] Resource usage tracked (if significant)
- [ ] Error handling tested
- [ ] Fuzzing tests added
- [ ] Unit tests cover edge cases
- [ ] Documentation includes security notes
- [ ] Code reviewed by security-aware developer

### For Configuration Changes

- [ ] New limits have sensible defaults
- [ ] Configuration validated at load time
- [ ] Invalid configs rejected with clear errors
- [ ] Changes logged for audit trail
- [ ] Documentation updated
- [ ] Migration path for existing configs

### For Security Incidents

- [ ] Vulnerability reported responsibly
- [ ] Patch developed and tested
- [ ] Security advisory drafted
- [ ] Users notified via release notes
- [ ] Regression test added to prevent recurrence
- [ ] Lessons learned documented
- [ ] Similar patterns audited

## Appendix C: FAQ

**Q: Why are there no Safe tier functions?**
A: All current KeyRx functions modify keyboard state. Safe tier requires pure functions with no side effects. Future utility functions will populate this tier.

**Q: Can I disable the sandbox for performance?**
A: No. The sandbox is always active. Performance overhead is <1% and the security benefits are essential.

**Q: What happens if my script exceeds a resource limit?**
A: Script execution stops immediately with a clear error message indicating which limit was exceeded and the current usage.

**Q: Can I increase resource limits in the config?**
A: Yes, ResourceConfig can be customized. However, defaults are suitable for 99% of use cases. Only increase if experiencing timeouts.

**Q: Is the sandbox thread-safe?**
A: Yes. Resource tracking uses atomic operations and is safe for concurrent access. The Rhai engine itself is not thread-safe, but each script execution has its own budget.

**Q: How do I know what capability tier my script needs?**
A: Start with Standard mode (default). If a function is rejected, the error message will tell you which tier is required. Consider if you really need that function.

**Q: What's the difference between Safe and Standard mode?**
A: Safe mode only allows pure functions with no side effects (none currently exist). Standard mode allows keyboard operations but no system access. Standard is recommended for all users.

**Q: Can malicious scripts steal my clipboard or files?**
A: No. Standard mode (default) blocks all system access. Advanced mode would allow clipboard access, but that requires explicit user configuration.

**Q: Will the sandbox be ported to other scripting languages?**
A: The architecture is script-engine agnostic. However, Rhai's built-in safety features make it ideal. Other languages would require similar sandboxing capabilities.

## Conclusion

KeyRx's multi-layered security model provides defense-in-depth protection against malicious and buggy scripts while maintaining low overhead and high usability. The capability system ensures scripts only access necessary functions, input validation prevents malformed data from causing issues, and resource limits prevent denial-of-service attacks.

The default Standard mode provides an excellent balance of security and functionality for the vast majority of users. Advanced users can opt into Full mode when needed, while maintaining strong security boundaries.

Regular security testing, including property-based fuzzing and benchmarking, ensures the sandbox remains robust and performant as KeyRx evolves.
