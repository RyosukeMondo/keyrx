# Tasks Document

## Phase 1: Core Types

- [x] 1. Create ScriptCapability enum
  - File: `core/src/scripting/sandbox/capability.rs`
  - Define Safe, Standard, Advanced, Internal tiers
  - Add is_allowed_in method
  - Purpose: Function categorization
  - _Leverage: Enum patterns_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create ScriptCapability enum with tiers | Restrictions: Clear tier semantics, ordered | _Leverage: Enum patterns | Success: Capabilities defined clearly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create ResourceBudget
  - File: `core/src/scripting/sandbox/budget.rs`
  - Track instructions, recursion, memory, timeout
  - Atomic counters for thread safety
  - Purpose: Resource tracking
  - _Leverage: Atomic operations_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating resource tracker | Task: Create ResourceBudget with limits | Restrictions: Thread-safe, low overhead, configurable | _Leverage: Atomics | Success: Resources tracked and limited | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create ResourceConfig
  - File: `core/src/scripting/sandbox/budget.rs`
  - Configurable limits with sensible defaults
  - Serializable for config file
  - Purpose: Limit configuration
  - _Leverage: serde_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating config | Task: Create ResourceConfig with defaults | Restrictions: Sensible defaults, configurable | _Leverage: serde | Success: Limits configurable | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Capability Registry

- [x] 4. Create CapabilityRegistry
  - File: `core/src/scripting/sandbox/registry.rs`
  - HashMap-based O(1) lookup
  - KeyCode to function mapping
  - Purpose: Fast capability lookup
  - _Leverage: HashMap_
  - _Requirements: 2.3, 4.1, 4.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating registry | Task: Create CapabilityRegistry with O(1) lookup | Restrictions: HashMap-based, KeyCode mapping | _Leverage: HashMap | Success: Lookup is O(1) | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Categorize existing functions
  - Files: `core/src/scripting/bindings.rs`, `builtins.rs`
  - Assign capability tier to each function
  - Document tier rationale
  - Purpose: Function categorization
  - _Leverage: ScriptCapability_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer categorizing | Task: Assign capability tiers to all functions | Restrictions: Conservative tiers, document rationale | _Leverage: ScriptCapability | Success: All functions categorized | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Migrate registry to HashMap
  - File: `core/src/scripting/registry.rs`
  - Replace O(n) lookup with HashMap
  - Add KeyCode indexing
  - Purpose: Performance optimization
  - _Leverage: CapabilityRegistry_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing | Task: Migrate registry to HashMap for O(1) lookup | Restrictions: Same API, faster lookup | _Leverage: HashMap | Success: Registry lookup is O(1) | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Input Validation

- [x] 7. Create InputValidator trait
  - File: `core/src/scripting/sandbox/validation.rs`
  - Define validation interface
  - Add ValidationError type
  - Purpose: Input validation abstraction
  - _Leverage: Trait patterns_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating traits | Task: Create InputValidator trait | Restrictions: Pluggable, clear errors | _Leverage: Trait patterns | Success: Validation interface defined | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 8. Implement common validators
  - File: `core/src/scripting/sandbox/validators/`
  - RangeValidator, TypeValidator, KeyCodeValidator
  - Purpose: Reusable validation
  - _Leverage: InputValidator trait_
  - _Requirements: 3.1, 3.3, 3.4_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing validators | Task: Implement common validators | Restrictions: Reusable, composable | _Leverage: InputValidator | Success: Common validations available | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 9. Add validation to existing functions
  - Files: `core/src/scripting/bindings.rs`, `builtins.rs`
  - Add validators to function registrations
  - Purpose: Input validation
  - _Leverage: Validators_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding validation | Task: Add input validators to all functions | Restrictions: Appropriate validation per function | _Leverage: Validators | Success: All functions validate input | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Sandbox Integration

- [x] 10. Create ScriptSandbox
  - File: `core/src/scripting/sandbox/mod.rs`
  - Combine capability checks, validation, resources
  - Configure Rhai engine limits
  - Purpose: Unified sandbox
  - _Leverage: All sandbox components_
  - _Requirements: 1.1, 2.2, 3.1_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating sandbox | Task: Create ScriptSandbox combining all safety | Restrictions: All checks, configurable | _Leverage: All components | Success: Unified sandbox works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 11. Configure Rhai engine limits
  - File: `core/src/scripting/runtime.rs`
  - Set max_operations, max_call_stack_depth
  - Configure memory limits
  - Purpose: Engine-level limits
  - _Leverage: Rhai configuration_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer configuring Rhai | Task: Configure Rhai engine resource limits | Restrictions: Use Rhai built-in limits | _Leverage: Rhai config | Success: Engine limits configured | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 12. Integrate sandbox into engine
  - File: `core/src/engine/scripting.rs`
  - Use ScriptSandbox for all script execution
  - Handle sandbox errors
  - Purpose: Engine integration
  - _Leverage: ScriptSandbox_
  - _Requirements: 1.1, 2.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating | Task: Integrate ScriptSandbox into engine | Restrictions: All script calls through sandbox | _Leverage: ScriptSandbox | Success: Engine uses sandbox | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Safe Mode Enforcement

- [x] 13. Implement ScriptMode switching
  - File: `core/src/scripting/sandbox/mod.rs`
  - Safe/Standard/Full mode switching
  - Enforce tier restrictions
  - Purpose: Mode enforcement
  - _Leverage: ScriptCapability_
  - _Requirements: 2.2, 2.4_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing modes | Task: Implement ScriptMode enforcement | Restrictions: Strict enforcement, clear errors | _Leverage: ScriptCapability | Success: Modes enforced correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 14. Add safe_mode to config
  - File: `core/src/config/scripting.rs`
  - User-configurable script mode
  - Default to Standard
  - Purpose: User control
  - _Leverage: Config patterns_
  - _Requirements: 2.2_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding config | Task: Add script mode to configuration | Restrictions: User-configurable, sensible default | _Leverage: Config | Success: Users can set script mode | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Testing and Documentation

- [-] 15. Add sandbox fuzz tests
  - File: `core/tests/fuzz/sandbox_fuzz.rs`
  - Fuzz script inputs
  - Test resource exhaustion
  - Purpose: Security testing
  - _Leverage: Fuzzing frameworks_
  - _Requirements: Non-functional (security)_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Security Developer | Task: Create sandbox fuzz tests | Restrictions: Test all limit types, input validation | _Leverage: Fuzzing | Success: No bypasses found | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Add sandbox benchmarks
  - File: `core/benches/sandbox_bench.rs`
  - Benchmark validation overhead
  - Benchmark capability checks
  - Purpose: Performance verification
  - _Leverage: criterion_
  - _Requirements: 2.3, 4.1_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Benchmark Developer | Task: Create sandbox benchmarks | Restrictions: Test overhead, compare to baseline | _Leverage: criterion | Success: Overhead targets met | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Document sandbox security model
  - File: `docs/scripting-security.md`
  - Explain capability tiers
  - Document resource limits
  - Purpose: User documentation
  - _Leverage: Implementation knowledge_
  - _Requirements: Non-functional (documentation)_
  - _Prompt: Implement the task for spec rhai-sandbox-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Document sandbox security model | Restrictions: Clear tiers, limit explanations | _Leverage: Implementation | Success: Security model documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
